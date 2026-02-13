//! Stage 4: Nearest-Neighbor Thermodynamics.
//!
//! SantaLucia (1998) unified parameters for DNA stacking energies.
//! Computes ΔH, ΔS, ΔG at 37°C, and normalized tension.
//!
//! Tier: T2-C | Dominant: → (Causality) — thermodynamic consequence of adjacency.

use crate::nucleotide::{DnaSequence, Nucleotide};
use serde::{Deserialize, Serialize};

/// Body temperature in Kelvin (37°C).
const BODY_TEMP_K: f64 = 310.15;

/// Maximum step ΔG magnitude for normalization (~CG step at 37°C).
const MAX_STEP_DG: f64 = 3.5;

/// Thermodynamic tension analysis.
///
/// Tier: T3 | Grounds to: → (Causality) + N (Quantity) + κ (Comparison)
///         + σ (Sequence) + ∂ (Boundary) + ν (Frequency).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    /// Total enthalpy ΔH (kcal/mol).
    pub total_delta_h: f64,
    /// Total entropy ΔS (cal/mol·K).
    pub total_delta_s: f64,
    /// Gibbs free energy at 37°C: ΔG = ΔH - TΔS (kcal/mol).
    pub delta_g_37: f64,
    /// Mean |ΔG| per dinucleotide step.
    pub mean_step_energy: f64,
    /// Peak |ΔG| across all steps.
    pub peak_step_energy: f64,
    /// Normalized tension [0.0, 1.0] — mean_step / MAX_STEP_DG.
    pub normalized: f64,
    /// Number of dinucleotide steps analyzed.
    pub step_count: usize,
}

/// SantaLucia (1998) unified nearest-neighbor stacking parameters.
///
/// Returns (ΔH in kcal/mol, ΔS in cal/(mol·K)) for a dinucleotide step.
/// All 16 dinucleotide combinations mapped via Watson-Crick symmetry
/// to 10 unique nearest-neighbor parameters.
#[must_use]
fn stacking_params(first: Nucleotide, second: Nucleotide) -> (f64, f64) {
    use Nucleotide::*;
    match (first, second) {
        (A, A) | (T, T) => (-7.9, -22.2),
        (A, T) => (-7.2, -20.4),
        (T, A) => (-7.2, -21.3),
        (C, A) | (T, G) => (-8.5, -22.7),
        (G, T) | (A, C) => (-8.4, -22.4),
        (C, T) | (A, G) => (-7.8, -21.0),
        (G, A) | (T, C) => (-8.2, -22.2),
        (C, G) => (-10.6, -27.2),
        (G, C) => (-9.8, -24.4),
        (G, G) | (C, C) => (-8.0, -19.9),
    }
}

impl Tension {
    /// Analyze thermodynamic tension of a DNA sequence.
    ///
    /// Sums nearest-neighbor stacking energies along the sequence.
    /// ΔG = ΔH - TΔS at 37°C (310.15 K).
    #[must_use]
    pub fn analyze(seq: &DnaSequence) -> Self {
        let bases = seq.bases();
        if bases.len() < 2 {
            return Self {
                total_delta_h: 0.0,
                total_delta_s: 0.0,
                delta_g_37: 0.0,
                mean_step_energy: 0.0,
                peak_step_energy: 0.0,
                normalized: 0.0,
                step_count: 0,
            };
        }

        let mut total_h = 0.0;
        let mut total_s = 0.0;
        let mut step_energies = Vec::with_capacity(bases.len() - 1);

        for pair in bases.windows(2) {
            let (dh, ds) = stacking_params(pair[0], pair[1]);
            total_h += dh;
            total_s += ds;
            // ΔG = ΔH - TΔS (convert ΔS from cal to kcal)
            let dg = dh - BODY_TEMP_K * ds / 1000.0;
            step_energies.push(dg.abs());
        }

        let n_steps = step_energies.len();
        let total_dg = total_h - BODY_TEMP_K * total_s / 1000.0;
        let mean = step_energies.iter().sum::<f64>() / n_steps.max(1) as f64;
        let peak = step_energies.iter().cloned().fold(0.0_f64, f64::max);

        let normalized = (mean / MAX_STEP_DG).min(1.0);

        Self {
            total_delta_h: total_h,
            total_delta_s: total_s,
            delta_g_37: total_dg,
            mean_step_energy: mean,
            peak_step_energy: peak,
            normalized,
            step_count: n_steps,
        }
    }

    /// Whether the sequence is thermodynamically stable (ΔG < 0).
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.delta_g_37 < 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleotide::{DnaSequence, Nucleotide, encode};

    #[test]
    fn cg_step_strongest() {
        // CG has the most negative ΔH (-10.6) and should produce highest step energy
        let (dh_cg, _) = stacking_params(Nucleotide::C, Nucleotide::G);
        let (dh_at, _) = stacking_params(Nucleotide::A, Nucleotide::T);
        assert!(dh_cg < dh_at, "CG should have more negative ΔH than AT");
    }

    #[test]
    fn all_sequences_stable() {
        // DNA stacking is always thermodynamically favorable
        let seq = encode("NexVigilant");
        let t = Tension::analyze(&seq);
        assert!(t.is_stable(), "DNA stacking should be stable (ΔG < 0)");
    }

    #[test]
    fn tension_normalized_range() {
        let seq = encode("NexVigilant");
        let t = Tension::analyze(&seq);
        assert!(t.normalized >= 0.0 && t.normalized <= 1.0);
    }

    #[test]
    fn step_count_correct() {
        let seq = encode("NexVigilant");
        let t = Tension::analyze(&seq);
        assert_eq!(t.step_count, seq.len() - 1);
    }

    #[test]
    fn empty_sequence_safe() {
        let seq = DnaSequence::new(vec![]);
        let t = Tension::analyze(&seq);
        assert_eq!(t.step_count, 0);
        assert_eq!(t.normalized, 0.0);
    }

    #[test]
    fn peak_gte_mean() {
        let seq = encode("NexVigilant");
        let t = Tension::analyze(&seq);
        assert!(
            t.peak_step_energy >= t.mean_step_energy,
            "Peak must be >= mean"
        );
    }
}
