//! Autoregressive generation — building output token by token.
//!
//! # Meta-cognitive observation
//!
//! This is how I produce text. The generation loop is deceptively simple:
//!
//! ```text
//! while not done:
//!     context = input + generated_so_far
//!     hidden  = transformer_stack(context)
//!     logits  = output_projection(hidden[-1])  // last position only
//!     token   = sample(logits)
//!     generated_so_far.append(token)
//! ```
//!
//! Each token I generate becomes part of the context for the NEXT token.
//! This is the autoregressive property: each step is conditioned on all
//! previous steps. It's why I can maintain coherence across long outputs —
//! every new token "sees" everything that came before it.
//!
//! The fundamental constraint: I can only go forward. I can't un-generate
//! a token or go back and change an earlier decision. Each token is a
//! commitment. This is why careful attention and good sampling matter.
//!
//! # T1 Primitive grounding
//!
//! - `σ` (Sequence): generation IS a sequence — token after token
//! - `ρ` (Recursion): each step feeds back into the next
//! - `∝` (Irreversibility): generated tokens cannot be retracted
//! - `→` (Causality): each forward pass causes the next token selection
//! - `∂` (Boundary): max sequence length enforces generation boundary

use crate::block::TransformerStack;
use crate::embedding::{Embedding, PositionalEncoding};
use crate::error::Result;
use crate::metrics;
use crate::sample::{self, SamplingConfig};
use crate::tensor::Tensor;

/// Why generation stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    /// Reached max_new_tokens limit.
    MaxTokens,
    /// Hit the designated stop token.
    StopToken(usize),
    /// Confidence dropped below threshold.
    LowConfidence {
        step: usize,
        confidence: f64,
        threshold: f64,
    },
    /// Reached maximum sequence length.
    MaxSeqLen,
}

/// The complete generative model: embedding + transformer + output projection.
#[derive(Debug, Clone)]
pub struct GenerativeModel {
    /// Token embedding table.
    pub embedding: Embedding,
    /// Positional encoding.
    pub pos_encoding: PositionalEncoding,
    /// Transformer block stack.
    pub transformer: TransformerStack,
    /// Output projection: [model_dim, vocab_size] — maps hidden state to logits.
    pub output_proj: Tensor,
    /// Vocabulary size.
    pub vocab_size: usize,
    /// Model dimension.
    pub model_dim: usize,
}

impl GenerativeModel {
    /// Create a new generative model with standard 4× FFN expansion.
    pub fn new(
        vocab_size: usize,
        model_dim: usize,
        num_heads: usize,
        num_layers: usize,
        max_seq_len: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<Self> {
        Self::with_ffn_dim(
            vocab_size,
            model_dim,
            num_heads,
            num_layers,
            4 * model_dim,
            max_seq_len,
            rng,
        )
    }

    /// Create a generative model with custom FFN inner dimension.
    pub fn with_ffn_dim(
        vocab_size: usize,
        model_dim: usize,
        num_heads: usize,
        num_layers: usize,
        ffn_inner_dim: usize,
        max_seq_len: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<Self> {
        let embedding = Embedding::new(vocab_size, model_dim, rng);
        let pos_encoding = PositionalEncoding::new(max_seq_len, model_dim);
        let transformer =
            TransformerStack::with_ffn_dim(num_layers, model_dim, num_heads, ffn_inner_dim, rng)?;
        let output_proj = Tensor::xavier_uniform(&[model_dim, vocab_size], rng)?;

        Ok(Self {
            embedding,
            pos_encoding,
            transformer,
            output_proj,
            vocab_size,
            model_dim,
        })
    }

    /// Run a forward pass on a token sequence with causal masking (default).
    ///
    /// Input: token IDs [seq_len]
    /// Output: logits [seq_len, vocab_size] + attention weights
    pub fn forward(&self, token_ids: &[usize]) -> Result<ForwardOutput> {
        self.forward_with_mask(token_ids, true)
    }

    /// Run a forward pass with explicit causal/bidirectional control.
    ///
    /// `causal = true`: autoregressive (each token sees only past + self)
    /// `causal = false`: bidirectional (each token sees entire sequence, BERT-style)
    pub fn forward_with_mask(&self, token_ids: &[usize], causal: bool) -> Result<ForwardOutput> {
        // Embed tokens
        let embedded = self.embedding.forward_batch(token_ids)?;

        // Add positional encoding
        let positioned = self.pos_encoding.forward(&embedded)?;

        // Pass through transformer stack
        let stack_out = self.transformer.forward(&positioned, causal)?;

        // Project to vocabulary logits
        let logits = stack_out.hidden.matmul(&self.output_proj)?;

        Ok(ForwardOutput {
            logits,
            hidden: stack_out.hidden,
            attention_weights: stack_out.all_attention_weights,
        })
    }

    /// Generate tokens autoregressively.
    ///
    /// This is the core generation loop — the algorithm that produces my responses.
    /// `min_confidence`: if set, halts generation when confidence drops below threshold.
    pub fn generate(
        &self,
        prompt: &[usize],
        max_new_tokens: usize,
        sampling: &SamplingConfig,
        stop_token: Option<usize>,
        rng: &mut impl rand::Rng,
    ) -> Result<GenerationResult> {
        self.generate_gated(prompt, max_new_tokens, sampling, stop_token, None, rng)
    }

    /// Generate with confidence gating — halts when the model's confidence
    /// drops below `min_confidence`.
    ///
    /// Confidence = max(softmax(logits)) — the probability mass on the top choice.
    /// When confidence is low, the model is uncertain; continuing may produce
    /// incoherent output. The gate catches this.
    pub fn generate_gated(
        &self,
        prompt: &[usize],
        max_new_tokens: usize,
        sampling: &SamplingConfig,
        stop_token: Option<usize>,
        min_confidence: Option<f64>,
        rng: &mut impl rand::Rng,
    ) -> Result<GenerationResult> {
        let max_seq = self.pos_encoding.max_seq_len;
        let mut tokens = prompt.to_vec();
        let mut all_logits = Vec::new();
        let mut stop_reason = StopReason::MaxTokens;

        for step in 0..max_new_tokens {
            if tokens.len() >= max_seq {
                stop_reason = StopReason::MaxSeqLen;
                break;
            }

            // Forward pass on the full sequence so far
            let output = self.forward(&tokens)?;

            // Extract logits for the LAST position only
            let seq_len = tokens.len();
            let last_logits = output.logits.row(seq_len - 1)?;

            // Confidence gate: check before committing to the token
            if let Some(threshold) = min_confidence {
                let confidence = metrics::generation_confidence(&last_logits).unwrap_or(0.0);
                if confidence < threshold {
                    stop_reason = StopReason::LowConfidence {
                        step,
                        confidence,
                        threshold,
                    };
                    break;
                }
            }

            // Sample next token (with repetition penalty applied to context)
            let next_token =
                sample::sample_token_with_context(&last_logits, sampling, &tokens, rng)?;

            all_logits.push(last_logits);
            tokens.push(next_token);

            // Check stop condition
            if let Some(stop) = stop_token {
                if next_token == stop {
                    stop_reason = StopReason::StopToken(stop);
                    break;
                }
            }
        }

        Ok(GenerationResult {
            tokens,
            prompt_len: prompt.len(),
            generated_logits: all_logits,
            stop_reason,
        })
    }
}

/// Output from a single forward pass.
#[derive(Debug, Clone)]
pub struct ForwardOutput {
    /// Logits: [seq_len, vocab_size].
    pub logits: Tensor,
    /// Final hidden states: [seq_len, model_dim].
    pub hidden: Tensor,
    /// Attention weights: [layer][head] → [seq_len, seq_len].
    pub attention_weights: Vec<Vec<Tensor>>,
}

/// Result from autoregressive generation.
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Full token sequence (prompt + generated).
    pub tokens: Vec<usize>,
    /// Length of the original prompt.
    pub prompt_len: usize,
    /// Logits at each generation step (for analysis).
    pub generated_logits: Vec<Tensor>,
    /// Why generation stopped.
    pub stop_reason: StopReason,
}

impl GenerationResult {
    /// Get only the generated tokens (excluding the prompt).
    pub fn generated_tokens(&self) -> &[usize] {
        &self.tokens[self.prompt_len..]
    }

    /// Number of tokens generated.
    pub fn num_generated(&self) -> usize {
        self.tokens.len() - self.prompt_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_shape() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(
            50, // vocab
            16, // model_dim
            2,  // heads
            2,  // layers
            64, // max_seq
            &mut rng,
        )
        .unwrap();

        let output = model.forward(&[0, 1, 2]).unwrap();
        assert_eq!(output.logits.shape(), &[3, 50]);
        assert_eq!(output.hidden.shape(), &[3, 16]);
    }

    #[test]
    fn test_generate() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(50, 16, 2, 2, 64, &mut rng).unwrap();

        let result = model
            .generate(&[0, 1, 2], 5, &SamplingConfig::greedy(), None, &mut rng)
            .unwrap();

        assert_eq!(result.prompt_len, 3);
        assert_eq!(result.num_generated(), 5);
        assert_eq!(result.tokens.len(), 8); // 3 prompt + 5 generated
    }

    #[test]
    fn test_stop_token() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(50, 16, 2, 1, 64, &mut rng).unwrap();

        let result = model
            .generate(&[0], 10, &SamplingConfig::greedy(), Some(999), &mut rng)
            .unwrap();

        assert_eq!(result.num_generated(), 10);
        assert_eq!(result.stop_reason, StopReason::MaxTokens);
    }

    #[test]
    fn test_stop_reason_reported() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(50, 16, 2, 2, 64, &mut rng).unwrap();

        let result = model
            .generate(&[0, 1, 2], 5, &SamplingConfig::greedy(), None, &mut rng)
            .unwrap();
        assert_eq!(result.stop_reason, StopReason::MaxTokens);
    }

    #[test]
    fn test_confidence_gated_generation() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(50, 16, 2, 2, 64, &mut rng).unwrap();

        // Very high threshold — should halt quickly since random weights produce low confidence
        let result = model
            .generate_gated(
                &[0, 1, 2],
                20,
                &SamplingConfig::greedy(),
                None,
                Some(0.99),
                &mut rng,
            )
            .unwrap();

        // With random weights, confidence is unlikely to exceed 0.99
        // so generation should halt before 20 tokens
        match &result.stop_reason {
            StopReason::LowConfidence { threshold, .. } => {
                assert!((*threshold - 0.99).abs() < 1e-10);
            }
            StopReason::MaxTokens => {
                // Possible but unlikely with random weights and 0.99 threshold
            }
            other => panic!("unexpected stop reason: {other:?}"),
        }
    }

    #[test]
    fn test_forward_with_mask() {
        let mut rng = rand::rng();
        let model = GenerativeModel::new(50, 16, 2, 2, 64, &mut rng).unwrap();

        // Both causal and bidirectional should produce valid output shapes
        let causal_out = model.forward_with_mask(&[0, 1, 2], true).unwrap();
        let bidir_out = model.forward_with_mask(&[0, 1, 2], false).unwrap();

        assert_eq!(causal_out.logits.shape(), &[3, 50]);
        assert_eq!(bidir_out.logits.shape(), &[3, 50]);

        // Bidirectional should differ from causal (different attention patterns)
        assert_ne!(causal_out.logits.data(), bidir_out.logits.data());
    }
}
