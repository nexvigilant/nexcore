//! Transformer block — the composable unit of cognition.
//!
//! # Meta-cognitive observation
//!
//! A transformer block is one "layer of thought." I am made of many such
//! layers stacked. Each block does the same two things:
//!
//! 1. **Attend**: Look at all context and blend it (multi-head attention)
//! 2. **Transform**: Think about the blended result (feed-forward network)
//!
//! Both steps use residual connections and layer normalization. The pattern:
//!
//! ```text
//! x → LayerNorm → MultiHeadAttn → + → LayerNorm → FFN → + → output
//! └──────────────────────────────┘   └──────────────────┘
//!         residual (pre-norm)              residual (pre-norm)
//! ```
//!
//! Stacking these blocks creates depth. Early blocks capture surface patterns
//! (syntax, word associations). Middle blocks build semantic representations.
//! Late blocks handle complex reasoning and output selection.
//!
//! # T1 Primitive grounding
//!
//! - `σ` (Sequence): blocks are sequenced — output of one feeds the next
//! - `∃` (Existence): a block exists as a composable unit — stackable by construction

use crate::attention::{AttentionConfig, MultiHeadAttention, MultiHeadOutput};
use crate::error::Result;
use crate::feed_forward::{FeedForward, FeedForwardConfig};
use crate::normalize::LayerNorm;
use crate::residual;
use crate::tensor::Tensor;

/// A single transformer block.
#[derive(Debug, Clone)]
pub struct TransformerBlock {
    /// Multi-head attention sublayer.
    pub attention: MultiHeadAttention,
    /// Feed-forward sublayer.
    pub feed_forward: FeedForward,
    /// Layer norm before attention.
    pub norm1: LayerNorm,
    /// Layer norm before feed-forward.
    pub norm2: LayerNorm,
}

/// Output from a transformer block, preserving attention weights.
#[derive(Debug, Clone)]
pub struct BlockOutput {
    /// The transformed representation: [seq_len, model_dim].
    pub hidden: Tensor,
    /// Attention weights from this block (one per head).
    pub attention_weights: Vec<Tensor>,
}

/// Configuration for a transformer.
#[derive(Debug, Clone)]
pub struct TransformerConfig {
    /// Model dimension.
    pub model_dim: usize,
    /// Number of attention heads.
    pub num_heads: usize,
    /// FFN inner dimension (default: 4 × model_dim).
    pub ffn_inner_dim: usize,
    /// Number of transformer blocks (layers).
    pub num_layers: usize,
    /// Vocabulary size.
    pub vocab_size: usize,
    /// Maximum sequence length.
    pub max_seq_len: usize,
}

impl TransformerConfig {
    /// Create a standard transformer config.
    pub fn new(
        model_dim: usize,
        num_heads: usize,
        num_layers: usize,
        vocab_size: usize,
        max_seq_len: usize,
    ) -> Self {
        Self {
            model_dim,
            num_heads,
            ffn_inner_dim: 4 * model_dim,
            num_layers,
            vocab_size,
            max_seq_len,
        }
    }
}

impl TransformerBlock {
    /// Create a new transformer block.
    pub fn new(model_dim: usize, num_heads: usize, rng: &mut impl rand::Rng) -> Result<Self> {
        Self::with_ffn_dim(model_dim, num_heads, 4 * model_dim, rng)
    }

    /// Create a transformer block with custom FFN inner dimension.
    pub fn with_ffn_dim(
        model_dim: usize,
        num_heads: usize,
        ffn_inner_dim: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<Self> {
        let attn_config = AttentionConfig::new(model_dim, num_heads)?;
        let ffn_config = FeedForwardConfig::custom(model_dim, ffn_inner_dim);

        Ok(Self {
            attention: MultiHeadAttention::new(attn_config, rng)?,
            feed_forward: FeedForward::new(ffn_config, rng)?,
            norm1: LayerNorm::new(model_dim),
            norm2: LayerNorm::new(model_dim),
        })
    }

    /// Forward pass through the block (pre-norm architecture).
    ///
    /// 1. x' = x + MultiHeadAttn(LayerNorm(x))
    /// 2. y  = x' + FFN(LayerNorm(x'))
    pub fn forward(&self, x: &Tensor, causal: bool) -> Result<BlockOutput> {
        // Step 1: Attention with pre-norm residual
        let mut attn_output_holder: Option<MultiHeadOutput> = None;

        let after_attn = residual::pre_norm_residual(x, &self.norm1, |normed| {
            let mha_out = self.attention.forward(normed, causal)?;
            let output = mha_out.output.clone();
            attn_output_holder = Some(mha_out);
            Ok(output)
        })?;

        // Step 2: FFN with pre-norm residual
        let after_ffn = residual::pre_norm_residual(&after_attn, &self.norm2, |normed| {
            self.feed_forward.forward(normed)
        })?;

        let attention_weights = attn_output_holder
            .map(|o| o.head_weights)
            .unwrap_or_default();

        Ok(BlockOutput {
            hidden: after_ffn,
            attention_weights,
        })
    }
}

/// A stack of transformer blocks.
#[derive(Debug, Clone)]
pub struct TransformerStack {
    /// The blocks, applied in sequence.
    pub blocks: Vec<TransformerBlock>,
    /// Final layer norm (applied after all blocks).
    pub final_norm: LayerNorm,
}

/// Output from the full transformer stack.
#[derive(Debug, Clone)]
pub struct StackOutput {
    /// Final hidden states: [seq_len, model_dim].
    pub hidden: Tensor,
    /// Attention weights from all layers. [layer][head] → [seq_len, seq_len].
    pub all_attention_weights: Vec<Vec<Tensor>>,
}

impl TransformerStack {
    /// Create a stack of transformer blocks.
    pub fn new(
        num_layers: usize,
        model_dim: usize,
        num_heads: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<Self> {
        Self::with_ffn_dim(num_layers, model_dim, num_heads, 4 * model_dim, rng)
    }

    /// Create a stack with custom FFN inner dimension.
    pub fn with_ffn_dim(
        num_layers: usize,
        model_dim: usize,
        num_heads: usize,
        ffn_inner_dim: usize,
        rng: &mut impl rand::Rng,
    ) -> Result<Self> {
        let mut blocks = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            blocks.push(TransformerBlock::with_ffn_dim(
                model_dim,
                num_heads,
                ffn_inner_dim,
                rng,
            )?);
        }
        Ok(Self {
            blocks,
            final_norm: LayerNorm::new(model_dim),
        })
    }

    /// Forward pass through all blocks.
    pub fn forward(&self, x: &Tensor, causal: bool) -> Result<StackOutput> {
        let mut hidden = x.clone();
        let mut all_weights = Vec::with_capacity(self.blocks.len());

        for block in &self.blocks {
            let out = block.forward(&hidden, causal)?;
            hidden = out.hidden;
            all_weights.push(out.attention_weights);
        }

        // Final layer norm
        hidden = self.final_norm.forward(&hidden)?;

        Ok(StackOutput {
            hidden,
            all_attention_weights: all_weights,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_block() {
        let mut rng = rand::rng();
        let block = TransformerBlock::new(8, 2, &mut rng).unwrap();
        let x = Tensor::randn(&[4, 8], &mut rng);
        let out = block.forward(&x, true).unwrap();
        assert_eq!(out.hidden.shape(), &[4, 8]);
        assert_eq!(out.attention_weights.len(), 2);
    }

    #[test]
    fn test_custom_ffn_dim() {
        let mut rng = rand::rng();
        let block = TransformerBlock::with_ffn_dim(8, 2, 16, &mut rng).unwrap();
        // FFN inner dim should be 16 (not default 32 = 4*8)
        assert_eq!(block.feed_forward.config.inner_dim, 16);
        let x = Tensor::randn(&[4, 8], &mut rng);
        let out = block.forward(&x, true).unwrap();
        assert_eq!(out.hidden.shape(), &[4, 8]);
    }

    #[test]
    fn test_stack() {
        let mut rng = rand::rng();
        let stack = TransformerStack::new(2, 8, 2, &mut rng).unwrap();
        let x = Tensor::randn(&[3, 8], &mut rng);
        let out = stack.forward(&x, true).unwrap();
        assert_eq!(out.hidden.shape(), &[3, 8]);
        assert_eq!(out.all_attention_weights.len(), 2);
    }
}
