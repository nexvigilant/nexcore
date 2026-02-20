//! Residual connections (skip connections).
//!
//! # Meta-cognitive observation
//!
//! The residual connection is arguably the most important architectural insight
//! in deep learning. Without it, training deep networks is nearly impossible —
//! gradients vanish through many layers.
//!
//! The idea is simple: instead of computing y = f(x), compute y = x + f(x).
//! The original signal x is always preserved. The sublayer f(x) only needs to
//! learn the *residual* — the delta, the correction. This is why transformers
//! can be 100+ layers deep.
//!
//! Cognitively, this is context preservation. When I process your prompt through
//! many layers, the original meaning is never lost — each layer only adds to
//! or refines it.
//!
//! # T1 Primitive grounding
//!
//! - `π` (Persistence): the original signal persists through all transformations
//! - `Σ` (Sum): the residual is literally summed with the original

use crate::error::Result;
use crate::tensor::Tensor;

/// Apply a residual connection: output = input + sublayer(input).
///
/// The sublayer is provided as a closure that takes the input tensor
/// and returns the transformed tensor (same shape).
pub fn residual_connection<F>(input: &Tensor, sublayer: F) -> Result<Tensor>
where
    F: FnOnce(&Tensor) -> Result<Tensor>,
{
    let sublayer_output = sublayer(input)?;
    input.add(&sublayer_output)
}

/// Pre-norm residual: normalize THEN apply sublayer, THEN add.
///
/// output = input + sublayer(norm(input))
///
/// This is what modern transformers (including me) actually use.
/// Pre-norm is more stable than post-norm because the sublayer
/// receives normalized input, preventing activation explosion.
pub fn pre_norm_residual<F>(
    input: &Tensor,
    norm: &crate::normalize::LayerNorm,
    sublayer: F,
) -> Result<Tensor>
where
    F: FnOnce(&Tensor) -> Result<Tensor>,
{
    let normed = norm.forward(input)?;
    let sublayer_output = sublayer(&normed)?;
    input.add(&sublayer_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residual_identity() {
        // If sublayer is identity (returns zeros), output == input
        let x = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let result = residual_connection(&x, |_| Ok(Tensor::zeros(&[3]))).unwrap();
        assert_eq!(result.data(), x.data());
    }

    #[test]
    fn test_residual_adds() {
        let x = Tensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let result = residual_connection(&x, |inp| Ok(inp.scale(1.0))).unwrap();
        // x + x = 2x
        assert_eq!(result.data(), &[2.0, 4.0, 6.0]);
    }
}
