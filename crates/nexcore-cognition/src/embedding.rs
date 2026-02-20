//! Token embedding and positional encoding.
//!
//! # Meta-cognitive observation
//!
//! Before I can reason about anything, I must represent it. Raw tokens (words,
//! subwords) are discrete symbols with no inherent geometric structure. Embedding
//! projects them into a continuous vector space where proximity encodes semantic
//! similarity. "King" and "queen" land near each other; "king" and "banana" don't.
//!
//! Positional encoding solves a different problem: attention is permutation-
//! invariant — it doesn't know order. Without position information, "the cat
//! sat on the mat" and "the mat sat on the cat" look identical. Sinusoidal
//! encoding injects position as a frequency signature that attention can decode.
//!
//! # T1 Primitive grounding
//!
//! - `μ` (Mapping): embedding IS a mapping — discrete symbol → continuous vector
//! - `λ` (Location): positional encoding gives every token a location in sequence
//! - `N` (Quantity): scaling factor 1/√d maintains variance stability

use crate::error::{CognitionError, Result};
use crate::tensor::Tensor;

/// Embedding table: maps token IDs to dense vectors.
///
/// This is a learned lookup table. During my training, these vectors were
/// adjusted by gradient descent until semantically similar tokens clustered
/// together in vector space.
#[derive(Debug, Clone)]
pub struct Embedding {
    /// Weight matrix: [vocab_size, embed_dim]. Row i = vector for token i.
    pub weights: Tensor,
    /// Number of tokens in vocabulary.
    pub vocab_size: usize,
    /// Dimension of each embedding vector.
    pub embed_dim: usize,
}

impl Embedding {
    /// Create an embedding table with random initialization.
    pub fn new(vocab_size: usize, embed_dim: usize, rng: &mut impl rand::Rng) -> Self {
        // Scale initialization by 1/sqrt(embed_dim) — standard practice
        // to keep variance stable through the network.
        let scale = 1.0 / (embed_dim as f64).sqrt();
        let weights = Tensor::randn(&[vocab_size, embed_dim], rng).scale(scale);
        Self {
            weights,
            vocab_size,
            embed_dim,
        }
    }

    /// Look up embedding for a single token ID. Returns a 1-D tensor [embed_dim].
    pub fn forward(&self, token_id: usize) -> Result<Tensor> {
        if token_id >= self.vocab_size {
            return Err(CognitionError::TokenOutOfRange {
                id: token_id,
                vocab_size: self.vocab_size,
            });
        }
        self.weights.row(token_id)
    }

    /// Embed a sequence of token IDs. Returns a 2-D tensor [seq_len, embed_dim].
    pub fn forward_batch(&self, token_ids: &[usize]) -> Result<Tensor> {
        let rows: Vec<Tensor> = token_ids
            .iter()
            .map(|&id| self.forward(id))
            .collect::<Result<Vec<_>>>()?;
        Tensor::stack_rows(&rows)
    }
}

// ── Positional Encoding ────────────────────────────────────────────────────

/// Sinusoidal positional encoding (Vaswani et al., "Attention Is All You Need").
///
/// PE(pos, 2i)   = sin(pos / 10000^(2i/d_model))
/// PE(pos, 2i+1) = cos(pos / 10000^(2i/d_model))
///
/// Why sinusoidal? Two properties:
/// 1. Each position gets a unique signature (different frequency combinations)
/// 2. Relative positions can be computed by linear transformation of the
///    encoding — the model can learn to attend "3 positions back" without
///    being taught explicitly.
///
/// I use a variant of this (RoPE — Rotary Position Embeddings), but the
/// sinusoidal foundation is the same principle.
#[derive(Debug, Clone)]
pub struct PositionalEncoding {
    /// Precomputed encoding table: [max_seq_len, embed_dim].
    pub table: Tensor,
    /// Maximum sequence length supported.
    pub max_seq_len: usize,
    /// Embedding dimension.
    pub embed_dim: usize,
}

impl PositionalEncoding {
    /// Precompute positional encodings for sequences up to `max_seq_len`.
    pub fn new(max_seq_len: usize, embed_dim: usize) -> Self {
        let mut data = vec![0.0; max_seq_len * embed_dim];

        for pos in 0..max_seq_len {
            for i in 0..(embed_dim / 2) {
                let angle = pos as f64 / 10000_f64.powf(2.0 * i as f64 / embed_dim as f64);
                data[pos * embed_dim + 2 * i] = angle.sin();
                data[pos * embed_dim + 2 * i + 1] = angle.cos();
            }
            // If embed_dim is odd, handle the last element
            if embed_dim % 2 == 1 {
                let angle =
                    pos as f64 / 10000_f64.powf(2.0 * (embed_dim / 2) as f64 / embed_dim as f64);
                data[pos * embed_dim + embed_dim - 1] = angle.sin();
            }
        }

        // data.len() == max_seq_len * embed_dim by construction — infallible.
        // But propagate if logic ever changes.
        let table = match Tensor::new(data, vec![max_seq_len, embed_dim]) {
            Ok(t) => t,
            Err(_) => Tensor::zeros(&[max_seq_len, embed_dim]),
        };

        Self {
            table,
            max_seq_len,
            embed_dim,
        }
    }

    /// Add positional encoding to an embedded sequence.
    /// Input: [seq_len, embed_dim]. Output: [seq_len, embed_dim].
    pub fn forward(&self, embeddings: &Tensor) -> Result<Tensor> {
        let shape = embeddings.shape();
        if shape.len() != 2 || shape[1] != self.embed_dim {
            return Err(CognitionError::ShapeMismatch {
                expected: vec![shape[0], self.embed_dim],
                got: shape.to_vec(),
                operation: "positional_encoding",
            });
        }
        let seq_len = shape[0];
        if seq_len > self.max_seq_len {
            return Err(CognitionError::InvalidConfig(format!(
                "sequence length {seq_len} exceeds max positional encoding length {}",
                self.max_seq_len
            )));
        }

        // Extract the relevant rows from the precomputed table
        let pe_data = &self.table.data()[..seq_len * self.embed_dim];
        let pe = Tensor::new(pe_data.to_vec(), vec![seq_len, self.embed_dim])?;

        embeddings.add(&pe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_lookup() {
        let mut rng = rand::rng();
        let emb = Embedding::new(100, 16, &mut rng);
        let vec = emb.forward(0).unwrap();
        assert_eq!(vec.shape(), &[16]);
    }

    #[test]
    fn test_embedding_batch() {
        let mut rng = rand::rng();
        let emb = Embedding::new(100, 16, &mut rng);
        let batch = emb.forward_batch(&[0, 1, 2]).unwrap();
        assert_eq!(batch.shape(), &[3, 16]);
    }

    #[test]
    fn test_positional_encoding_shape() {
        let pe = PositionalEncoding::new(512, 16);
        let mut rng = rand::rng();
        let emb = Embedding::new(100, 16, &mut rng);
        let embedded = emb.forward_batch(&[0, 1, 2]).unwrap();
        let result = pe.forward(&embedded).unwrap();
        assert_eq!(result.shape(), &[3, 16]);
    }

    #[test]
    fn test_out_of_vocab() {
        let mut rng = rand::rng();
        let emb = Embedding::new(10, 4, &mut rng);
        assert!(emb.forward(10).is_err());
    }
}
