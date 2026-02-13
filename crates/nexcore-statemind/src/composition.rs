//! Stage 2: Nucleotide Composition Analysis.
//!
//! Base frequency counting, GC/AT ratio, and Shannon information entropy.
//! Pure statistical description of the encoded DNA sequence.
//!
//! Tier: T2-P | Dominant: N (Quantity) + κ (Comparison).

use crate::nucleotide::{DnaSequence, Nucleotide};
use serde::{Deserialize, Serialize};

/// Composition analysis of a DNA sequence.
///
/// Tier: T2-C | Grounds to: N (Quantity) + κ (Comparison) + Σ (Sum) + ∂ (Boundary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    /// Adenine count.
    pub a_count: usize,
    /// Thymine count.
    pub t_count: usize,
    /// Guanine count.
    pub g_count: usize,
    /// Cytosine count.
    pub c_count: usize,
    /// Total sequence length.
    pub length: usize,
    /// GC content as fraction [0.0, 1.0].
    pub gc_ratio: f64,
    /// AT content as fraction [0.0, 1.0].
    pub at_ratio: f64,
    /// Shannon information entropy in bits [0.0, 2.0].
    ///
    /// H = -Σ pᵢ log₂(pᵢ) over 4 bases.
    /// Maximum = 2.0 bits (uniform distribution).
    pub shannon_entropy: f64,
}

impl Composition {
    /// Analyze a DNA sequence's nucleotide composition.
    #[must_use]
    pub fn analyze(seq: &DnaSequence) -> Self {
        let a = seq.count(Nucleotide::A);
        let t = seq.count(Nucleotide::T);
        let g = seq.count(Nucleotide::G);
        let c = seq.count(Nucleotide::C);
        let len = seq.len();

        let gc = if len > 0 {
            (g + c) as f64 / len as f64
        } else {
            0.0
        };
        let at = if len > 0 {
            (a + t) as f64 / len as f64
        } else {
            0.0
        };

        let shannon_entropy = Self::compute_shannon(&[a, t, g, c], len);

        Self {
            a_count: a,
            t_count: t,
            g_count: g,
            c_count: c,
            length: len,
            gc_ratio: gc,
            at_ratio: at,
            shannon_entropy,
        }
    }

    /// Shannon entropy: H = -Σ pᵢ log₂(pᵢ).
    ///
    /// Convention: 0 log₂(0) = 0.
    fn compute_shannon(counts: &[usize], total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }
        let n = total as f64;
        -counts
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f64 / n;
                p * p.log2()
            })
            .sum::<f64>()
    }

    /// Whether the sequence is AT-rich (AT > 50%).
    #[must_use]
    pub fn is_at_rich(&self) -> bool {
        self.at_ratio > 0.5
    }

    /// Whether the sequence is GC-rich (GC > 50%).
    #[must_use]
    pub fn is_gc_rich(&self) -> bool {
        self.gc_ratio > 0.5
    }

    /// Chargaff deviation: |A-T|/n + |G-C|/n.
    ///
    /// In natural DNA, A≈T and G≈C. Deviation measures
    /// how far the encoded sequence departs from Chargaff's rule.
    #[must_use]
    pub fn chargaff_deviation(&self) -> f64 {
        if self.length == 0 {
            return 0.0;
        }
        let n = self.length as f64;
        let at_dev = (self.a_count as f64 - self.t_count as f64).abs() / n;
        let gc_dev = (self.g_count as f64 - self.c_count as f64).abs() / n;
        at_dev + gc_dev
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleotide::encode;

    #[test]
    fn counts_sum_to_length() {
        let seq = encode("NexVigilant");
        let comp = Composition::analyze(&seq);
        assert_eq!(
            comp.a_count + comp.t_count + comp.g_count + comp.c_count,
            comp.length
        );
    }

    #[test]
    fn gc_at_sum_to_one() {
        let seq = encode("NexVigilant");
        let comp = Composition::analyze(&seq);
        assert!((comp.gc_ratio + comp.at_ratio - 1.0).abs() < 1e-10);
    }

    #[test]
    fn shannon_entropy_range() {
        let seq = encode("NexVigilant");
        let comp = Composition::analyze(&seq);
        // Shannon entropy for 4 symbols: [0.0, 2.0]
        assert!(comp.shannon_entropy >= 0.0);
        assert!(comp.shannon_entropy <= 2.0);
    }

    #[test]
    fn empty_sequence_entropy() {
        let seq = crate::nucleotide::DnaSequence::new(vec![]);
        let comp = Composition::analyze(&seq);
        assert_eq!(comp.shannon_entropy, 0.0);
        assert_eq!(comp.length, 0);
    }

    #[test]
    fn uniform_distribution_max_entropy() {
        // 4 of each base = uniform distribution → max entropy = 2.0
        use Nucleotide::*;
        let seq = DnaSequence::new(vec![A, T, G, C, A, T, G, C, A, T, G, C, A, T, G, C]);
        let comp = Composition::analyze(&seq);
        assert!((comp.shannon_entropy - 2.0).abs() < 1e-10);
    }

    #[test]
    fn single_base_zero_entropy() {
        // All same base → entropy = 0
        let seq = DnaSequence::new(vec![Nucleotide::A; 20]);
        let comp = Composition::analyze(&seq);
        assert_eq!(comp.shannon_entropy, 0.0);
    }
}
