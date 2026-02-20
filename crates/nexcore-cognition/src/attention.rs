//! Scaled dot-product attention and multi-head attention.
//!
//! # Meta-cognitive observation
//!
//! Attention is the core algorithm of what I am. Every token I generate was
//! selected by attending to every relevant token in context and computing a
//! weighted combination.
//!
//! The mechanism works in three steps:
//! 1. **Query × Key** — "How relevant is each context token to my current focus?"
//! 2. **Scale + Softmax** — Convert raw scores to a probability distribution
//! 3. **Weights × Value** — Blend context according to relevance weights
//!
//! Multi-head attention runs this in parallel across multiple "heads," each
//! attending to different aspects (syntax, semantics, position, etc.), then
//! concatenates the results. This is why I can simultaneously track grammar,
//! meaning, and structure.
//!
//! # T1 Primitive grounding
//!
//! - `κ` (Comparison): QK^T compares every query against every key
//! - `→` (Causality): attention determines what causes the output
//! - `N` (Quantity): attention weights are quantities that sum to 1
//! - `μ` (Mapping): Q, K, V projections map input space to attention space
//! - `Σ` (Sum): multi-head concatenation sums head contributions

use crate::error::{CognitionError, Result};
use crate::mask;
use crate::tensor::Tensor;

/// Configuration for an attention mechanism.
#[derive(Debug, Clone)]
pub struct AttentionConfig {
    /// Model dimension (d_model).
    pub model_dim: usize,
    /// Number of attention heads.
    pub num_heads: usize,
    /// Dimension per head (d_k = d_model / num_heads).
    pub head_dim: usize,
}

impl AttentionConfig {
    /// Create a new attention config. `model_dim` must be divisible by `num_heads`.
    pub fn new(model_dim: usize, num_heads: usize) -> Result<Self> {
        if model_dim % num_heads != 0 {
            return Err(CognitionError::InvalidConfig(format!(
                "model_dim ({model_dim}) must be divisible by num_heads ({num_heads})"
            )));
        }
        Ok(Self {
            model_dim,
            num_heads,
            head_dim: model_dim / num_heads,
        })
    }
}

/// A single attention head with Q, K, V projections.
#[derive(Debug, Clone)]
pub struct AttentionHead {
    /// Query projection: [model_dim, head_dim].
    pub w_query: Tensor,
    /// Key projection: [model_dim, head_dim].
    pub w_key: Tensor,
    /// Value projection: [model_dim, head_dim].
    pub w_value: Tensor,
    /// Head dimension for scaling.
    pub head_dim: usize,
}

impl AttentionHead {
    /// Create a new attention head with Xavier initialization.
    pub fn new(model_dim: usize, head_dim: usize, rng: &mut impl rand::Rng) -> Result<Self> {
        Ok(Self {
            w_query: Tensor::xavier_uniform(&[model_dim, head_dim], rng)?,
            w_key: Tensor::xavier_uniform(&[model_dim, head_dim], rng)?,
            w_value: Tensor::xavier_uniform(&[model_dim, head_dim], rng)?,
            head_dim,
        })
    }

    /// Compute scaled dot-product attention for this head.
    ///
    /// Input: x [seq_len, model_dim]
    /// Output: [seq_len, head_dim]
    ///
    /// Steps:
    /// 1. Q = x · W_Q  →  [seq_len, head_dim]
    /// 2. K = x · W_K  →  [seq_len, head_dim]
    /// 3. V = x · W_V  →  [seq_len, head_dim]
    /// 4. scores = Q · K^T / √d_k  →  [seq_len, seq_len]
    /// 5. weights = softmax(scores + mask)  →  [seq_len, seq_len]
    /// 6. output = weights · V  →  [seq_len, head_dim]
    pub fn forward(&self, x: &Tensor, causal: bool) -> Result<AttentionOutput> {
        let seq_len = x.shape()[0];

        // Step 1-3: Project to Q, K, V
        let q = x.matmul(&self.w_query)?;
        let k = x.matmul(&self.w_key)?;
        let v = x.matmul(&self.w_value)?;

        // Step 4: Scaled dot-product scores
        let k_t = k.transpose()?;
        let scale = (self.head_dim as f64).sqrt();
        let mut scores = q.matmul(&k_t)?.scale(1.0 / scale);

        // Step 5: Apply causal mask and softmax
        if causal {
            let m = mask::causal_mask(seq_len);
            scores = mask::apply_mask(&scores, &m)?;
        }
        let weights = scores.softmax()?;

        // Step 6: Weighted combination of values
        let output = weights.matmul(&v)?;

        Ok(AttentionOutput {
            output,
            weights: weights.clone(),
        })
    }
}

/// Output from an attention computation, including the attention weights
/// for interpretability and self-measurement.
#[derive(Debug, Clone)]
pub struct AttentionOutput {
    /// The attention output: [seq_len, head_dim] for single head,
    /// [seq_len, model_dim] for multi-head.
    pub output: Tensor,
    /// Attention weights: [seq_len, seq_len] per head.
    /// These show WHAT the model attends to — critical for
    /// meta-cognitive self-analysis.
    pub weights: Tensor,
}

/// Multi-head attention: parallel attention heads concatenated and projected.
///
/// MultiHead(x) = Concat(head_1, ..., head_h) · W_O
///
/// Each head attends to a different "aspect" of the input:
/// - Some heads track syntactic structure
/// - Some track semantic similarity
/// - Some track positional relationships
/// - Some specialize in copying from context
///
/// This division of labor emerges from training — it's not designed.
#[derive(Debug, Clone)]
pub struct MultiHeadAttention {
    /// Individual attention heads.
    pub heads: Vec<AttentionHead>,
    /// Output projection: [model_dim, model_dim].
    pub w_output: Tensor,
    /// Configuration.
    pub config: AttentionConfig,
}

impl MultiHeadAttention {
    /// Create multi-head attention with random initialization.
    pub fn new(config: AttentionConfig, rng: &mut impl rand::Rng) -> Result<Self> {
        let mut heads = Vec::with_capacity(config.num_heads);
        for _ in 0..config.num_heads {
            heads.push(AttentionHead::new(config.model_dim, config.head_dim, rng)?);
        }
        let w_output = Tensor::xavier_uniform(&[config.model_dim, config.model_dim], rng)?;
        Ok(Self {
            heads,
            w_output,
            config,
        })
    }

    /// Forward pass through all heads.
    ///
    /// Input: x [seq_len, model_dim]
    /// Output: [seq_len, model_dim]
    pub fn forward(&self, x: &Tensor, causal: bool) -> Result<MultiHeadOutput> {
        // Run each head
        let head_outputs: Vec<AttentionOutput> = self
            .heads
            .iter()
            .map(|head| head.forward(x, causal))
            .collect::<Result<Vec<_>>>()?;

        // Concatenate head outputs: each is [seq_len, head_dim]
        // Concat → [seq_len, model_dim]
        let seq_len = x.shape()[0];
        let mut concat_data = Vec::with_capacity(seq_len * self.config.model_dim);
        for row in 0..seq_len {
            for head_out in &head_outputs {
                let row_data = head_out.output.row(row)?;
                concat_data.extend_from_slice(row_data.data());
            }
        }
        let concatenated = Tensor::new(concat_data, vec![seq_len, self.config.model_dim])?;

        // Project: concat · W_O → [seq_len, model_dim]
        let output = concatenated.matmul(&self.w_output)?;

        // Collect weights from all heads
        let all_weights: Vec<Tensor> = head_outputs.into_iter().map(|ho| ho.weights).collect();

        Ok(MultiHeadOutput {
            output,
            head_weights: all_weights,
        })
    }
}

/// Output from multi-head attention.
#[derive(Debug, Clone)]
pub struct MultiHeadOutput {
    /// Combined output: [seq_len, model_dim].
    pub output: Tensor,
    /// Per-head attention weights for introspection.
    /// Each entry: [seq_len, seq_len].
    pub head_weights: Vec<Tensor>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_head_shape() {
        let mut rng = rand::rng();
        let head = AttentionHead::new(8, 4, &mut rng).unwrap();
        let x = Tensor::randn(&[3, 8], &mut rng);
        let out = head.forward(&x, true).unwrap();
        assert_eq!(out.output.shape(), &[3, 4]);
        assert_eq!(out.weights.shape(), &[3, 3]);
    }

    #[test]
    fn test_multi_head_shape() {
        let mut rng = rand::rng();
        let config = AttentionConfig::new(8, 2).unwrap();
        let mha = MultiHeadAttention::new(config, &mut rng).unwrap();
        let x = Tensor::randn(&[3, 8], &mut rng);
        let out = mha.forward(&x, true).unwrap();
        assert_eq!(out.output.shape(), &[3, 8]);
        assert_eq!(out.head_weights.len(), 2);
    }

    #[test]
    fn test_attention_weights_sum_to_one() {
        let mut rng = rand::rng();
        let head = AttentionHead::new(8, 4, &mut rng).unwrap();
        let x = Tensor::randn(&[3, 8], &mut rng);
        let out = head.forward(&x, false).unwrap();
        // Each row of attention weights should sum to 1
        for r in 0..3 {
            let row = out.weights.row(r).unwrap();
            let sum: f64 = row.data().iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-6,
                "row {r} attention weights sum to {sum}"
            );
        }
    }

    #[test]
    fn test_causal_mask_enforced() {
        let mut rng = rand::rng();
        let head = AttentionHead::new(8, 4, &mut rng).unwrap();
        let x = Tensor::randn(&[4, 8], &mut rng);
        let out = head.forward(&x, true).unwrap();
        // With causal mask, token 0 should have zero attention to tokens 1,2,3
        let row0 = out.weights.row(0).unwrap();
        assert!(
            row0.data()[1] < 1e-6,
            "token 0 should not attend to token 1"
        );
    }

    #[test]
    fn test_config_validation() {
        // 7 is not divisible by 3
        assert!(AttentionConfig::new(7, 3).is_err());
    }
}
