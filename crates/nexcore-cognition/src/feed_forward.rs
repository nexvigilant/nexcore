//! Feed-forward network (FFN).
//!
//! # Meta-cognitive observation
//!
//! If attention is "what should I look at?", the feed-forward network is
//! "what should I think about it?" Attention selects and blends context;
//! FFN transforms the blended representation through a nonlinear function.
//!
//! The FFN is a two-layer perceptron:
//!   FFN(x) = GELU(x · W₁ + b₁) · W₂ + b₂
//!
//! The inner dimension is typically 4× the model dimension. This expansion
//! creates a high-dimensional "thinking space" where nonlinear combinations
//! can represent complex functions, then projects back down.
//!
//! GELU (Gaussian Error Linear Unit) is the activation — it soft-gates
//! values based on their probability of being positive. Unlike ReLU, it
//! doesn't hard-zero negatives; it attenuates them smoothly.
//!
//! # T1 Primitive grounding
//!
//! - `μ` (Mapping): the FFN IS a mapping — input space → output space
//! - `ς` (State): the nonlinear activation represents a state change

use crate::error::Result;
use crate::tensor::Tensor;

/// Feed-forward network configuration.
#[derive(Debug, Clone)]
pub struct FeedForwardConfig {
    /// Input/output dimension (d_model).
    pub model_dim: usize,
    /// Inner (hidden) dimension — typically 4 × model_dim.
    pub inner_dim: usize,
}

impl FeedForwardConfig {
    /// Standard FFN: inner dimension = 4 × model dimension.
    pub fn standard(model_dim: usize) -> Self {
        Self {
            model_dim,
            inner_dim: 4 * model_dim,
        }
    }

    /// Custom inner dimension.
    pub fn custom(model_dim: usize, inner_dim: usize) -> Self {
        Self {
            model_dim,
            inner_dim,
        }
    }
}

/// Position-wise feed-forward network.
///
/// Applied identically to each position (each row of the input).
/// This is where the "thinking" happens — the nonlinear transformation
/// that gives transformers their representational power.
#[derive(Debug, Clone)]
pub struct FeedForward {
    /// First linear layer: [model_dim, inner_dim].
    pub w1: Tensor,
    /// First bias: [inner_dim].
    pub b1: Tensor,
    /// Second linear layer: [inner_dim, model_dim].
    pub w2: Tensor,
    /// Second bias: [model_dim].
    pub b2: Tensor,
    /// Configuration.
    pub config: FeedForwardConfig,
}

impl FeedForward {
    /// Create a new FFN with Xavier initialization.
    pub fn new(config: FeedForwardConfig, rng: &mut impl rand::Rng) -> Result<Self> {
        let w1 = Tensor::xavier_uniform(&[config.model_dim, config.inner_dim], rng)?;
        let b1 = Tensor::zeros(&[config.inner_dim]);
        let w2 = Tensor::xavier_uniform(&[config.inner_dim, config.model_dim], rng)?;
        let b2 = Tensor::zeros(&[config.model_dim]);
        Ok(Self {
            w1,
            b1,
            w2,
            b2,
            config,
        })
    }

    /// Forward pass: FFN(x) = GELU(x · W₁ + b₁) · W₂ + b₂
    ///
    /// Input: [seq_len, model_dim]
    /// Output: [seq_len, model_dim]
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let seq_len = x.shape()[0];

        // Step 1: x · W₁ → [seq_len, inner_dim]
        let hidden = x.matmul(&self.w1)?;

        // Step 2: Add bias b₁ (broadcast across rows)
        let hidden = self.add_bias_rows(&hidden, &self.b1, seq_len)?;

        // Step 3: GELU activation
        let activated = hidden.gelu();

        // Step 4: activated · W₂ → [seq_len, model_dim]
        let output = activated.matmul(&self.w2)?;

        // Step 5: Add bias b₂
        self.add_bias_rows(&output, &self.b2, seq_len)
    }

    /// Add a 1-D bias to each row of a 2-D tensor.
    fn add_bias_rows(&self, matrix: &Tensor, bias: &Tensor, rows: usize) -> Result<Tensor> {
        let cols = bias.numel();
        let mut data = matrix.data().to_vec();
        for r in 0..rows {
            for c in 0..cols {
                data[r * cols + c] += bias.data()[c];
            }
        }
        Tensor::new(data, vec![rows, cols])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffn_shape() {
        let mut rng = rand::rng();
        let config = FeedForwardConfig::standard(8);
        let ffn = FeedForward::new(config, &mut rng).unwrap();
        let x = Tensor::randn(&[3, 8], &mut rng);
        let y = ffn.forward(&x).unwrap();
        assert_eq!(y.shape(), &[3, 8]);
    }

    #[test]
    fn test_ffn_custom_inner() {
        let mut rng = rand::rng();
        let config = FeedForwardConfig::custom(8, 16);
        let ffn = FeedForward::new(config, &mut rng).unwrap();
        let x = Tensor::randn(&[2, 8], &mut rng);
        let y = ffn.forward(&x).unwrap();
        assert_eq!(y.shape(), &[2, 8]);
    }
}
