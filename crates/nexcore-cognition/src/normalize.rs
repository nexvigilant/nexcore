//! Layer normalization.
//!
//! # Meta-cognitive observation
//!
//! Without normalization, signals drift as they pass through layers. Activations
//! grow or shrink exponentially, and the network becomes unstable. Layer norm
//! re-centers and re-scales activations at each layer boundary, keeping the
//! signal in a stable range.
//!
//! I use this after every attention block and every feed-forward block. It's
//! the homeostasis of neural networks — maintaining internal stability despite
//! wildly varying inputs.
//!
//! # T1 Primitive grounding
//!
//! - `∂` (Boundary): normalization enforces boundaries on activation magnitude
//! - `N` (Quantity): operates on the statistics (mean, variance) of quantities

use crate::error::Result;
use crate::tensor::Tensor;

/// Layer normalization: normalize across the feature dimension.
///
/// LayerNorm(x) = γ · (x - μ) / √(σ² + ε) + β
///
/// - γ (gain) and β (bias) are learnable parameters
/// - ε is a small constant for numerical stability
/// - μ, σ² are computed per-sample across the feature dimension
///
/// Unlike batch norm, layer norm doesn't depend on batch statistics —
/// each sample is normalized independently. This is critical for
/// autoregressive generation where batch size is 1.
#[derive(Debug, Clone)]
pub struct LayerNorm {
    /// Learnable scale parameter (γ): [feature_dim].
    pub gain: Tensor,
    /// Learnable shift parameter (β): [feature_dim].
    pub bias: Tensor,
    /// Numerical stability constant.
    pub eps: f64,
    /// Feature dimension.
    pub feature_dim: usize,
}

impl LayerNorm {
    /// Create a new LayerNorm with gain=1, bias=0 (identity initialization).
    pub fn new(feature_dim: usize) -> Self {
        Self {
            gain: Tensor::ones(&[feature_dim]),
            bias: Tensor::zeros(&[feature_dim]),
            eps: 1e-5,
            feature_dim,
        }
    }

    /// Create with custom epsilon.
    pub fn with_eps(feature_dim: usize, eps: f64) -> Self {
        Self {
            eps,
            ..Self::new(feature_dim)
        }
    }

    /// Normalize a 1-D vector (single sample).
    pub fn forward_1d(&self, x: &Tensor) -> Result<Tensor> {
        let mean = x.mean()?;
        let var = x.variance()?;
        let inv_std = 1.0 / (var + self.eps).sqrt();

        // (x - mean) * inv_std * gain + bias
        let centered = x.map(|v| (v - mean) * inv_std);
        let scaled = centered.mul(&self.gain)?;
        scaled.add(&self.bias)
    }

    /// Normalize a 2-D tensor row-wise: [seq_len, feature_dim] → [seq_len, feature_dim].
    /// Each row is normalized independently.
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let shape = x.shape();
        if shape.len() == 1 {
            return self.forward_1d(x);
        }

        let rows = shape[0];
        let cols = shape[1];
        let mut result_data = Vec::with_capacity(rows * cols);

        for r in 0..rows {
            let row = x.row(r)?;
            let normed = self.forward_1d(&row)?;
            result_data.extend_from_slice(normed.data());
        }

        Tensor::new(result_data, vec![rows, cols])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_norm_1d() {
        let ln = LayerNorm::new(4);
        let x = Tensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        let y = ln.forward_1d(&x).unwrap();
        // After normalization, mean should be ~0 and std ~1
        let mean = y.mean().unwrap();
        assert!(mean.abs() < 1e-6, "mean={mean}");
    }

    #[test]
    fn test_layer_norm_2d() {
        let ln = LayerNorm::new(4);
        let x = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], vec![2, 4]).unwrap();
        let y = ln.forward(&x).unwrap();
        assert_eq!(y.shape(), &[2, 4]);
    }
}
