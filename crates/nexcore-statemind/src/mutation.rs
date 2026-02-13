//! Stage 7: Mutation Stability Analysis.
//!
//! Generates all Hamming-1 neighbors in nucleotide space,
//! re-projects each, and measures drift from the original position.
//!
//! Tier: T2-C | Dominant: ς (State) — perturbation of base identity.

use crate::composition::Composition;
use crate::nucleotide::{DnaSequence, Nucleotide};
use crate::projection::Point3D;
use crate::spectral::SpectralProfile;
use serde::{Deserialize, Serialize};

/// Mutation stability profile.
///
/// Tier: T3 | Grounds to: ς (State) + ∂ (Boundary) + κ (Comparison)
///         + N (Quantity) + λ (Location) + Σ (Sum).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStability {
    /// Mean Euclidean drift across all single-base mutations.
    pub avg_drift: f64,
    /// Maximum drift from any single mutation.
    pub max_drift: f64,
    /// Number of mutations producing zero measurable drift.
    pub zero_drift_count: usize,
    /// Total mutations tested (3 × sequence length).
    pub total_mutations: usize,
}

/// Threshold below which drift is considered zero.
const ZERO_DRIFT_THRESHOLD: f64 = 1e-6;

impl MutationStability {
    /// Analyze mutation stability of a DNA sequence.
    ///
    /// For each position, substitutes the 3 alternative bases,
    /// re-analyzes composition + spectrum, re-projects to 3D,
    /// and measures Euclidean drift from the original projection.
    #[must_use]
    pub fn analyze(seq: &DnaSequence, original_point: &Point3D) -> Self {
        let bases = seq.bases();
        let n = bases.len();
        if n == 0 {
            return Self {
                avg_drift: 0.0,
                max_drift: 0.0,
                zero_drift_count: 0,
                total_mutations: 0,
            };
        }

        let mut drifts = Vec::with_capacity(n * 3);

        for i in 0..n {
            for &alt in &Nucleotide::ALL {
                if alt == bases[i] {
                    continue;
                }

                // Create mutant
                let mut mutant_bases = bases.to_vec();
                mutant_bases[i] = alt;
                let mutant = DnaSequence::new(mutant_bases);

                // Re-analyze and project
                let comp = Composition::analyze(&mutant);
                let spec = SpectralProfile::analyze(&mutant);
                let point = Point3D::from_features(&comp, &spec);

                let drift = original_point.distance(&point);
                drifts.push(drift);
            }
        }

        let total = drifts.len();
        let avg = if total > 0 {
            drifts.iter().sum::<f64>() / total as f64
        } else {
            0.0
        };
        let max = drifts.iter().cloned().fold(0.0_f64, f64::max);
        let zero_count = drifts.iter().filter(|&&d| d < ZERO_DRIFT_THRESHOLD).count();

        Self {
            avg_drift: avg,
            max_drift: max,
            zero_drift_count: zero_count,
            total_mutations: total,
        }
    }

    /// Fraction of mutations that produce zero drift.
    #[must_use]
    pub fn zero_drift_fraction(&self) -> f64 {
        if self.total_mutations == 0 {
            return 0.0;
        }
        self.zero_drift_count as f64 / self.total_mutations as f64
    }

    /// Whether the sequence is robust (>25% zero-drift mutations).
    #[must_use]
    pub fn is_robust(&self) -> bool {
        self.zero_drift_fraction() > 0.25
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleotide::encode;

    #[test]
    fn mutation_count_correct() {
        let seq = encode("AB");
        let comp = Composition::analyze(&seq);
        let spec = SpectralProfile::analyze(&seq);
        let point = Point3D::from_features(&comp, &spec);
        let ms = MutationStability::analyze(&seq, &point);
        // 3 alternative bases per position × sequence length
        assert_eq!(ms.total_mutations, 3 * seq.len());
    }

    #[test]
    fn drift_non_negative() {
        let seq = encode("test");
        let comp = Composition::analyze(&seq);
        let spec = SpectralProfile::analyze(&seq);
        let point = Point3D::from_features(&comp, &spec);
        let ms = MutationStability::analyze(&seq, &point);
        assert!(ms.avg_drift >= 0.0);
        assert!(ms.max_drift >= 0.0);
    }

    #[test]
    fn max_gte_avg() {
        let seq = encode("NexVigilant");
        let comp = Composition::analyze(&seq);
        let spec = SpectralProfile::analyze(&seq);
        let point = Point3D::from_features(&comp, &spec);
        let ms = MutationStability::analyze(&seq, &point);
        assert!(
            ms.max_drift >= ms.avg_drift,
            "Max drift must be >= average drift"
        );
    }

    #[test]
    fn empty_sequence_safe() {
        let seq = DnaSequence::new(vec![]);
        let point = Point3D::origin();
        let ms = MutationStability::analyze(&seq, &point);
        assert_eq!(ms.total_mutations, 0);
        assert_eq!(ms.avg_drift, 0.0);
    }
}
