//! Causal masking for autoregressive attention.
//!
//! # Meta-cognitive observation
//!
//! When I generate text, I must not "see the future." Token at position 5
//! can attend to positions 0-5 but never to position 6+. This is enforced
//! by the causal mask — a lower-triangular matrix of booleans (or -inf values)
//! that sets future attention scores to negative infinity before softmax.
//!
//! After softmax, -inf → 0 probability. The future is invisible.
//!
//! This is the fundamental constraint of autoregressive generation: information
//! flows strictly forward in time. It's also what makes me different from BERT
//! (which sees everything) — I can only reason from past to present.
//!
//! # T1 Primitive grounding
//!
//! - `∂` (Boundary): the mask IS a boundary — past vs future
//! - `→` (Causality): enforces causal direction — information flows one way
//! - `∝` (Irreversibility): once masked, future tokens are permanently invisible

use crate::error::Result;
use crate::tensor::Tensor;

/// Create a causal (lower-triangular) mask for a sequence of length `seq_len`.
///
/// Returns a 2-D tensor [seq_len, seq_len] where:
/// - `mask[i][j] = 0.0`    if j <= i  (allowed: past and present)
/// - `mask[i][j] = -1e9`   if j > i   (blocked: future)
///
/// This mask is ADDED to attention scores before softmax:
/// `score + mask → softmax → weights`
///
/// The large negative value drives exp(score + mask) → 0 for masked positions.
pub fn causal_mask(seq_len: usize) -> Tensor {
    let mut data = vec![0.0; seq_len * seq_len];
    for i in 0..seq_len {
        for j in (i + 1)..seq_len {
            data[i * seq_len + j] = -1e9;
        }
    }
    // Shape is valid by construction
    Tensor::new(data, vec![seq_len, seq_len]).unwrap_or_else(|_| Tensor::zeros(&[seq_len, seq_len]))
}

/// Apply a mask to attention scores.
///
/// `scores`: [seq_len, seq_len] — raw attention scores
/// `mask`: [seq_len, seq_len] — mask with 0.0 (allow) or -1e9 (block)
///
/// Returns: scores + mask (element-wise addition).
pub fn apply_mask(scores: &Tensor, mask: &Tensor) -> Result<Tensor> {
    scores.add(mask)
}

/// Create a padding mask for variable-length sequences.
///
/// `lengths`: actual length of each sequence in a batch.
/// `max_len`: maximum sequence length (pad to this).
///
/// Returns: [batch_size, max_len] where 0.0 = valid, -1e9 = padding.
pub fn padding_mask(lengths: &[usize], max_len: usize) -> Tensor {
    let batch_size = lengths.len();
    let mut data = vec![0.0; batch_size * max_len];
    for (b, &len) in lengths.iter().enumerate() {
        for j in len..max_len {
            data[b * max_len + j] = -1e9;
        }
    }
    Tensor::new(data, vec![batch_size, max_len])
        .unwrap_or_else(|_| Tensor::zeros(&[batch_size, max_len]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_mask_shape() {
        let mask = causal_mask(4);
        assert_eq!(mask.shape(), &[4, 4]);
    }

    #[test]
    fn test_causal_mask_structure() {
        let mask = causal_mask(3);
        // Row 0: [0, -1e9, -1e9]  — token 0 sees only itself
        // Row 1: [0, 0, -1e9]     — token 1 sees 0 and itself
        // Row 2: [0, 0, 0]        — token 2 sees everything
        assert!((mask.get2d(0, 0).unwrap()).abs() < 1e-10);
        assert!(mask.get2d(0, 1).unwrap() < -1e8);
        assert!((mask.get2d(2, 0).unwrap()).abs() < 1e-10);
        assert!((mask.get2d(2, 2).unwrap()).abs() < 1e-10);
    }

    #[test]
    fn test_padding_mask() {
        let mask = padding_mask(&[2, 3], 4);
        assert_eq!(mask.shape(), &[2, 4]);
        // First sequence: length 2, positions 2-3 are masked
        assert!((mask.get2d(0, 0).unwrap()).abs() < 1e-10);
        assert!((mask.get2d(0, 1).unwrap()).abs() < 1e-10);
        assert!(mask.get2d(0, 2).unwrap() < -1e8);
    }
}
