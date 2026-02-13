//! Stage 6: Spectral Resonance Comparison.
//!
//! Pairwise comparison of two DNA sequences' power spectra.
//! Decomposes into overlap (cosine similarity), harmony (concordant peaks),
//! and dissonance (discordant peaks).
//!
//! Tier: T2-C | Dominant: κ (Comparison) + × (Product).

use crate::spectral::SpectralProfile;
use serde::{Deserialize, Serialize};

/// Resonance between two DNA sequences.
///
/// Tier: T3 | Grounds to: κ (Comparison) + × (Product) + N (Quantity)
///         + ν (Frequency) + ∂ (Boundary) + Σ (Sum).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resonance {
    /// Cosine similarity of power spectra [0.0, 1.0].
    pub overlap: f64,
    /// Fraction of concordant spectral modes [0.0, 1.0].
    pub harmony: f64,
    /// Fraction of discordant spectral modes [0.0, 1.0].
    /// Always equals 1.0 - harmony.
    pub dissonance: f64,
}

impl Resonance {
    /// Compare two spectral profiles.
    ///
    /// - **Overlap:** Cosine similarity of power spectra.
    /// - **Harmony:** Fraction where both spectra are above/below their respective means.
    /// - **Dissonance:** 1.0 - harmony.
    #[must_use]
    pub fn compare(a: &SpectralProfile, b: &SpectralProfile) -> Self {
        let min_len = a.power_spectrum.len().min(b.power_spectrum.len());
        if min_len == 0 {
            return Self {
                overlap: 0.0,
                harmony: 0.0,
                dissonance: 0.0,
            };
        }

        // Cosine similarity
        let dot: f64 = (0..min_len)
            .map(|i| a.power_spectrum[i] * b.power_spectrum[i])
            .sum();
        let mag_a: f64 = a.power_spectrum[..min_len]
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();
        let mag_b: f64 = b.power_spectrum[..min_len]
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();

        let overlap = if mag_a > 0.0 && mag_b > 0.0 {
            (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Harmony: fraction of concordant above/below-mean modes
        let mean_a: f64 = a.power_spectrum[..min_len].iter().sum::<f64>() / min_len.max(1) as f64;
        let mean_b: f64 = b.power_spectrum[..min_len].iter().sum::<f64>() / min_len.max(1) as f64;

        let concordant = (0..min_len)
            .filter(|&i| (a.power_spectrum[i] >= mean_a) == (b.power_spectrum[i] >= mean_b))
            .count();

        let harmony = concordant as f64 / min_len as f64;
        let dissonance = 1.0 - harmony;

        Self {
            overlap,
            harmony,
            dissonance,
        }
    }

    /// Whether the resonance is predominantly harmonic (harmony > 0.5).
    #[must_use]
    pub fn is_harmonic(&self) -> bool {
        self.harmony > 0.5
    }

    /// Whether there is zero dissonance.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.dissonance < 1e-10
    }
}

impl std::fmt::Display for Resonance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "overlap={:.3} | harmony={:.1} | dissonance={:.1}",
            self.overlap, self.harmony, self.dissonance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_resonance_perfect() {
        let sp = SpectralProfile {
            power_spectrum: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            spectral_entropy: 1.5,
            dominant_period: 4,
            total_energy: 15.0,
            energy_density: 1.5,
            mode_count: 3,
        };
        let r = Resonance::compare(&sp, &sp);
        assert!((r.overlap - 1.0).abs() < 1e-6, "Self-overlap should be 1.0");
        assert!((r.harmony - 1.0).abs() < 1e-6, "Self-harmony should be 1.0");
        assert!(r.dissonance < 1e-6, "Self-dissonance should be 0.0");
    }

    #[test]
    fn harmony_dissonance_sum_to_one() {
        let a = SpectralProfile {
            power_spectrum: vec![1.0, 0.5, 3.0, 0.1],
            spectral_entropy: 1.2,
            dominant_period: 3,
            total_energy: 4.6,
            energy_density: 1.15,
            mode_count: 2,
        };
        let b = SpectralProfile {
            power_spectrum: vec![0.1, 3.0, 0.5, 2.0],
            spectral_entropy: 1.3,
            dominant_period: 2,
            total_energy: 5.6,
            energy_density: 1.4,
            mode_count: 2,
        };
        let r = Resonance::compare(&a, &b);
        assert!(
            (r.harmony + r.dissonance - 1.0).abs() < 1e-10,
            "Harmony + dissonance must equal 1.0"
        );
    }

    #[test]
    fn empty_spectra_zero() {
        let empty = SpectralProfile {
            power_spectrum: vec![],
            spectral_entropy: 0.0,
            dominant_period: 0,
            total_energy: 0.0,
            energy_density: 0.0,
            mode_count: 0,
        };
        let r = Resonance::compare(&empty, &empty);
        assert_eq!(r.overlap, 0.0);
    }

    #[test]
    fn overlap_in_range() {
        let a = SpectralProfile {
            power_spectrum: vec![1.0, 2.0, 3.0],
            spectral_entropy: 1.0,
            dominant_period: 2,
            total_energy: 6.0,
            energy_density: 2.0,
            mode_count: 1,
        };
        let b = SpectralProfile {
            power_spectrum: vec![3.0, 1.0, 2.0],
            spectral_entropy: 1.0,
            dominant_period: 1,
            total_energy: 6.0,
            energy_density: 2.0,
            mode_count: 1,
        };
        let r = Resonance::compare(&a, &b);
        assert!(r.overlap >= 0.0 && r.overlap <= 1.0);
    }
}
