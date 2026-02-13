//! Stage 3: Spectral Analysis via Discrete Fourier Transform.
//!
//! Voss representation (4 binary indicator sequences) → DFT → power spectrum.
//! Extracts spectral entropy, dominant period, total energy, and mode count.
//!
//! Tier: T2-C | Dominant: ν (Frequency) + ρ (Recursion — DFT butterfly).

use crate::nucleotide::{DnaSequence, Nucleotide};
use serde::Serialize;
use std::f64::consts::PI;

/// Spectral profile of a DNA sequence.
///
/// Tier: T3 | Grounds to: ν (Frequency) + N (Quantity) + ρ (Recursion)
///         + κ (Comparison) + Σ (Sum) + ∂ (Boundary).
#[derive(Debug, Clone, Serialize)]
pub struct SpectralProfile {
    /// Power spectrum (half-spectrum, DC to Nyquist).
    pub power_spectrum: Vec<f64>,
    /// Spectral entropy: H_s = -Σ p_k ln(p_k) over normalized power.
    pub spectral_entropy: f64,
    /// Period of the dominant spectral peak (in nucleotides).
    pub dominant_period: usize,
    /// Total spectral energy: Σ |X[k]|².
    pub total_energy: f64,
    /// Energy per nucleotide.
    pub energy_density: f64,
    /// Number of spectral modes above mean power.
    pub mode_count: usize,
}

/// Voss binary indicator representation.
///
/// For each base type, creates a binary sequence:
/// indicator[base][i] = 1.0 if seq[i] == base, else 0.0.
fn voss_indicators(seq: &DnaSequence) -> [Vec<f64>; 4] {
    let n = seq.len();
    let mut indicators = [vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]];
    for (i, &base) in seq.bases().iter().enumerate() {
        match base {
            Nucleotide::A => indicators[0][i] = 1.0,
            Nucleotide::T => indicators[1][i] = 1.0,
            Nucleotide::G => indicators[2][i] = 1.0,
            Nucleotide::C => indicators[3][i] = 1.0,
        }
    }
    indicators
}

/// Discrete Fourier Transform (direct computation).
///
/// X[k] = Σ_{n=0}^{N-1} x[n] · e^{-2πi·k·n/N}
///
/// Returns (Re, Im) pairs for frequencies k = 0..N/2 (Nyquist).
/// O(N²) — sufficient for sequences < 500 nt.
fn dft(signal: &[f64]) -> Vec<(f64, f64)> {
    let n = signal.len();
    if n == 0 {
        return Vec::new();
    }
    let half = n / 2;
    (0..half)
        .map(|k| {
            let mut re = 0.0;
            let mut im = 0.0;
            for (idx, &x) in signal.iter().enumerate() {
                let angle = -2.0 * PI * k as f64 * idx as f64 / n as f64;
                re += x * angle.cos();
                im += x * angle.sin();
            }
            (re, im)
        })
        .collect()
}

/// Power spectrum: P[k] = Re(X[k])² + Im(X[k])².
fn power_spectrum(coeffs: &[(f64, f64)]) -> Vec<f64> {
    coeffs.iter().map(|(re, im)| re * re + im * im).collect()
}

impl SpectralProfile {
    /// Analyze a DNA sequence's spectral properties.
    ///
    /// Uses Voss representation: 4 binary indicator sequences → DFT each →
    /// sum power spectra → extract features.
    #[must_use]
    pub fn analyze(seq: &DnaSequence) -> Self {
        let n = seq.len();
        if n < 2 {
            return Self {
                power_spectrum: vec![],
                spectral_entropy: 0.0,
                dominant_period: 0,
                total_energy: 0.0,
                energy_density: 0.0,
                mode_count: 0,
            };
        }

        let indicators = voss_indicators(seq);
        let half = n / 2;

        // Sum power spectra across all 4 base indicators
        let mut total_power = vec![0.0; half];
        for indicator in &indicators {
            let coeffs = dft(indicator);
            let ps = power_spectrum(&coeffs);
            for (i, &p) in ps.iter().enumerate() {
                if i < half {
                    total_power[i] += p;
                }
            }
        }

        // Total energy
        let total_energy: f64 = total_power.iter().sum();

        // Spectral entropy: H_s = -Σ (p_k/E) · ln(p_k/E)
        let spectral_entropy = if total_energy > 0.0 {
            -total_power
                .iter()
                .filter(|&&p| p > 0.0)
                .map(|&p| {
                    let norm = p / total_energy;
                    norm * norm.ln()
                })
                .sum::<f64>()
        } else {
            0.0
        };

        // Dominant period: skip DC (k=0), find peak frequency
        let dominant_k = total_power
            .iter()
            .enumerate()
            .skip(1) // skip DC component
            .fold((1_usize, 0.0_f64), |(best_k, best_p), (k, &p)| {
                if p > best_p { (k, p) } else { (best_k, best_p) }
            })
            .0;
        let dominant_period = if dominant_k > 0 { n / dominant_k } else { n };

        // Mode count: frequencies with power above mean
        let mean_power = total_energy / total_power.len().max(1) as f64;
        let mode_count = total_power.iter().filter(|&&p| p > mean_power).count();

        let energy_density = if n > 0 { total_energy / n as f64 } else { 0.0 };

        Self {
            power_spectrum: total_power,
            spectral_entropy,
            dominant_period,
            total_energy,
            energy_density,
            mode_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nucleotide::{DnaSequence, Nucleotide, encode};

    #[test]
    fn spectral_energy_non_negative() {
        let seq = encode("NexVigilant");
        let sp = SpectralProfile::analyze(&seq);
        assert!(sp.total_energy >= 0.0);
        for &p in &sp.power_spectrum {
            assert!(p >= 0.0, "Power must be non-negative");
        }
    }

    #[test]
    fn spectral_entropy_non_negative() {
        let seq = encode("NexVigilant");
        let sp = SpectralProfile::analyze(&seq);
        assert!(sp.spectral_entropy >= 0.0);
    }

    #[test]
    fn dominant_period_positive() {
        let seq = encode("NexVigilant");
        let sp = SpectralProfile::analyze(&seq);
        assert!(sp.dominant_period > 0);
    }

    #[test]
    fn constant_signal_concentrated_at_dc() {
        // All same base → energy concentrated at DC (k=0)
        let seq = DnaSequence::new(vec![Nucleotide::A; 32]);
        let sp = SpectralProfile::analyze(&seq);
        // DC component should dominate
        if !sp.power_spectrum.is_empty() {
            let dc = sp.power_spectrum[0];
            let rest_max = sp
                .power_spectrum
                .iter()
                .skip(1)
                .cloned()
                .fold(0.0_f64, f64::max);
            assert!(
                dc > rest_max,
                "DC should dominate for constant signal: dc={dc}, rest_max={rest_max}"
            );
        }
    }

    #[test]
    fn empty_sequence_safe() {
        let seq = DnaSequence::new(vec![]);
        let sp = SpectralProfile::analyze(&seq);
        assert_eq!(sp.total_energy, 0.0);
        assert_eq!(sp.spectral_entropy, 0.0);
        assert!(sp.power_spectrum.is_empty());
    }

    #[test]
    fn power_spectrum_length() {
        let seq = encode("test");
        let sp = SpectralProfile::analyze(&seq);
        // Half the sequence length
        assert_eq!(sp.power_spectrum.len(), seq.len() / 2);
    }
}
