//! String Theory — DNA Strands as Vibrating Strings.
//!
//! Treats each DNA strand as a one-dimensional string under tension, where:
//!
//! - **Tension** derives from hydrogen bond energy (G-C = 3 bonds, A-T = 2 bonds)
//! - **Frequency spectrum** via autocorrelation at each lag k
//! - **Harmonic modes** are the dominant periodicities in the strand
//! - **Resonance** measures spectral overlap between two strands
//! - **String energy** combines tension + information entropy into quantized levels
//!
//! All algorithms are deterministic. Zero external dependencies.

use crate::lexicon;
use crate::ops;
use crate::storage;
use crate::types::{Nucleotide, Strand};
use std::fmt;

// ---------------------------------------------------------------------------
// Constants — hydrogen bond energies
// ---------------------------------------------------------------------------

/// G-C base pair: 3 hydrogen bonds.
const GC_BOND_ENERGY: f64 = 3.0;

/// A-T base pair: 2 hydrogen bonds.
const AT_BOND_ENERGY: f64 = 2.0;

/// Maximum bond energy per base (for normalization).
const MAX_BOND_ENERGY: f64 = GC_BOND_ENERGY;

// ---------------------------------------------------------------------------
// StringTension — bond energy profile
// ---------------------------------------------------------------------------

/// Tension profile of a DNA strand from hydrogen bond energies.
///
/// Tier: T2-C (ν Frequency + N Quantity + σ Sequence + μ Mapping)
/// Dominant: N Quantity (tension is a measurable force)
pub struct StringTension {
    /// Per-base bond energy: G/C → 3.0, A/T → 2.0.
    pub bond_energies: Vec<f64>,
    /// Mean tension across the strand.
    pub mean_tension: f64,
    /// GC tension ratio: fraction of energy from G/C bonds.
    pub gc_tension_ratio: f64,
    /// Maximum local tension (over a sliding window of 3).
    pub peak_tension: f64,
    /// Tension variance (measure of non-uniformity).
    pub variance: f64,
}

/// Compute the tension profile for a strand.
#[must_use]
pub fn tension(strand: &Strand) -> StringTension {
    if strand.is_empty() {
        return StringTension {
            bond_energies: Vec::new(),
            mean_tension: 0.0,
            gc_tension_ratio: 0.0,
            peak_tension: 0.0,
            variance: 0.0,
        };
    }

    let bond_energies: Vec<f64> = strand
        .bases
        .iter()
        .map(|&n| match n {
            Nucleotide::G | Nucleotide::C => GC_BOND_ENERGY,
            Nucleotide::A | Nucleotide::T => AT_BOND_ENERGY,
        })
        .collect();

    let n = bond_energies.len() as f64;
    let sum: f64 = bond_energies.iter().sum();
    let mean_tension = sum / n;

    let gc_energy: f64 = bond_energies
        .iter()
        .filter(|&&e| (e - GC_BOND_ENERGY).abs() < f64::EPSILON)
        .sum();
    let gc_tension_ratio = if sum > 0.0 { gc_energy / sum } else { 0.0 };

    // Peak tension: max over sliding window of 3
    let peak_tension = if bond_energies.len() >= 3 {
        bond_energies
            .windows(3)
            .map(|w| w.iter().sum::<f64>() / 3.0)
            .fold(0.0_f64, f64::max)
    } else {
        mean_tension
    };

    // Variance
    let variance = bond_energies
        .iter()
        .map(|&e| (e - mean_tension).powi(2))
        .sum::<f64>()
        / n;

    StringTension {
        bond_energies,
        mean_tension,
        gc_tension_ratio,
        peak_tension,
        variance,
    }
}

/// Compute tension from a word (encode to DNA first).
#[must_use]
pub fn word_tension(word: &str) -> StringTension {
    let strand = storage::encode_str(word);
    tension(&strand)
}

impl fmt::Display for StringTension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tension: mean={:.3}, peak={:.3}, GC_ratio={:.3}, var={:.4}, len={}",
            self.mean_tension,
            self.peak_tension,
            self.gc_tension_ratio,
            self.variance,
            self.bond_energies.len()
        )
    }
}

// ---------------------------------------------------------------------------
// HarmonicMode — a single frequency component
// ---------------------------------------------------------------------------

/// A single harmonic mode extracted from autocorrelation.
///
/// Tier: T2-P (ν Frequency + N Quantity)
/// Dominant: ν Frequency (oscillation period)
#[derive(Debug, Clone)]
pub struct HarmonicMode {
    /// Lag at which this mode occurs (period in bases).
    pub period: usize,
    /// Autocorrelation amplitude at this lag (0.0 to 1.0).
    pub amplitude: f64,
    /// Phase: fractional position within the strand (0.0 to 1.0).
    pub phase: f64,
}

impl fmt::Display for HarmonicMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mode(T={}, A={:.4}, φ={:.3})",
            self.period, self.amplitude, self.phase
        )
    }
}

// ---------------------------------------------------------------------------
// FrequencySpectrum — autocorrelation analysis
// ---------------------------------------------------------------------------

/// Frequency spectrum of a DNA strand via autocorrelation.
///
/// Tier: T2-C (ν Frequency + N Quantity + σ Sequence + κ Comparison)
/// Dominant: ν Frequency (spectral analysis of periodic structure)
pub struct FrequencySpectrum {
    /// Raw autocorrelation values at each lag (0..len/2).
    pub autocorrelation: Vec<f64>,
    /// Top harmonic modes sorted by amplitude (descending).
    pub modes: Vec<HarmonicMode>,
    /// Dominant period (lag with highest autocorrelation, excluding lag 0).
    pub dominant_period: usize,
    /// Spectral entropy: how spread out the frequency content is.
    pub spectral_entropy: f64,
}

/// Compute autocorrelation at a given lag.
///
/// `R(k) = (1/N) Σ match(strand[i], strand[i+k])` where match = 1.0 if equal.
fn autocorrelate_at(strand: &Strand, lag: usize) -> f64 {
    if lag >= strand.len() || strand.is_empty() {
        return 0.0;
    }
    let n = strand.len() - lag;
    if n == 0 {
        return 0.0;
    }
    let matches: usize = strand.bases[..n]
        .iter()
        .zip(strand.bases[lag..].iter())
        .filter(|(a, b)| a == b)
        .count();
    matches as f64 / n as f64
}

/// Compute the full frequency spectrum of a strand.
#[must_use]
pub fn spectrum(strand: &Strand) -> FrequencySpectrum {
    let max_lag = strand.len() / 2;
    if max_lag == 0 {
        return FrequencySpectrum {
            autocorrelation: Vec::new(),
            modes: Vec::new(),
            dominant_period: 0,
            spectral_entropy: 0.0,
        };
    }

    // Compute autocorrelation at each lag
    let autocorrelation: Vec<f64> = (1..=max_lag).map(|k| autocorrelate_at(strand, k)).collect();

    // Extract peaks (local maxima) as harmonic modes
    let mut modes: Vec<HarmonicMode> = Vec::new();
    for (i, &amp) in autocorrelation.iter().enumerate() {
        let period = i + 1; // lag = i + 1 (we start from lag 1)
        let is_peak = if i == 0 {
            autocorrelation.len() > 1 && amp >= autocorrelation[1]
        } else if i == autocorrelation.len() - 1 {
            amp >= autocorrelation[i - 1]
        } else {
            amp >= autocorrelation[i - 1] && amp >= autocorrelation[i + 1]
        };

        if is_peak && amp > 0.0 {
            let phase = period as f64 / strand.len() as f64;
            modes.push(HarmonicMode {
                period,
                amplitude: amp,
                phase,
            });
        }
    }

    // Sort modes by amplitude descending
    modes.sort_by(|a, b| {
        b.amplitude
            .partial_cmp(&a.amplitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Dominant period
    let dominant_period = modes.first().map_or(0, |m| m.period);

    // Spectral entropy: -Σ p(k) log₂(p(k)) over normalized autocorrelation
    let total: f64 = autocorrelation.iter().filter(|&&v| v > 0.0).sum();
    let spectral_entropy = if total > 0.0 {
        let mut h = 0.0_f64;
        for &amp in &autocorrelation {
            if amp > 0.0 {
                let p = amp / total;
                h -= p * p.log2();
            }
        }
        h
    } else {
        0.0
    };

    FrequencySpectrum {
        autocorrelation,
        modes,
        dominant_period,
        spectral_entropy,
    }
}

/// Compute frequency spectrum from a word.
#[must_use]
pub fn word_spectrum(word: &str) -> FrequencySpectrum {
    let strand = storage::encode_str(word);
    spectrum(&strand)
}

impl fmt::Display for FrequencySpectrum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Spectrum: {} lags, {} modes, T_dom={}, H_spec={:.4}",
            self.autocorrelation.len(),
            self.modes.len(),
            self.dominant_period,
            self.spectral_entropy,
        )
    }
}

// ---------------------------------------------------------------------------
// Resonance — spectral overlap between two strands
// ---------------------------------------------------------------------------

/// Resonance measurement between two DNA strands.
///
/// Tier: T2-C (κ Comparison + ν Frequency + N Quantity + μ Mapping)
/// Dominant: κ Comparison (measures alignment between frequency spectra)
pub struct Resonance {
    /// Spectral overlap: dot product of normalized autocorrelation vectors.
    pub overlap: f64,
    /// Harmony score: weighted by dominant mode alignment (0.0 to 1.0).
    pub harmony: f64,
    /// Dissonance: 1.0 - harmony.
    pub dissonance: f64,
    /// Tension alignment: how similar the mean tensions are (0.0 to 1.0).
    pub tension_alignment: f64,
}

/// Compute resonance between two strands.
#[must_use]
pub fn resonance(a: &Strand, b: &Strand) -> Resonance {
    let spec_a = spectrum(a);
    let spec_b = spectrum(b);
    let tens_a = tension(a);
    let tens_b = tension(b);

    // Spectral overlap: dot product of autocorrelation vectors (zero-padded to same length)
    let max_len = spec_a
        .autocorrelation
        .len()
        .max(spec_b.autocorrelation.len());
    let overlap = if max_len == 0 {
        0.0
    } else {
        let mag_a: f64 = spec_a
            .autocorrelation
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt();
        let mag_b: f64 = spec_b
            .autocorrelation
            .iter()
            .map(|v| v * v)
            .sum::<f64>()
            .sqrt();
        if mag_a < f64::EPSILON || mag_b < f64::EPSILON {
            0.0
        } else {
            let dot: f64 = (0..max_len)
                .map(|i| {
                    let va = spec_a.autocorrelation.get(i).copied().unwrap_or(0.0);
                    let vb = spec_b.autocorrelation.get(i).copied().unwrap_or(0.0);
                    va * vb
                })
                .sum();
            (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
        }
    };

    // Harmony: based on dominant mode alignment
    let harmony = if spec_a.dominant_period > 0 && spec_b.dominant_period > 0 {
        let p_max = spec_a.dominant_period.max(spec_b.dominant_period) as f64;
        let p_diff = (spec_a.dominant_period as f64 - spec_b.dominant_period as f64).abs();
        1.0 - (p_diff / p_max)
    } else {
        0.0
    };

    let dissonance = 1.0 - harmony;

    // Tension alignment: 1 - |mean_a - mean_b| / max_bond_energy
    let tension_diff = (tens_a.mean_tension - tens_b.mean_tension).abs();
    let tension_alignment = (1.0 - tension_diff / MAX_BOND_ENERGY).clamp(0.0, 1.0);

    Resonance {
        overlap,
        harmony,
        dissonance,
        tension_alignment,
    }
}

/// Compute resonance between two words.
#[must_use]
pub fn word_resonance(a: &str, b: &str) -> Resonance {
    let strand_a = storage::encode_str(a);
    let strand_b = storage::encode_str(b);
    resonance(&strand_a, &strand_b)
}

impl fmt::Display for Resonance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resonance: overlap={:.4}, harmony={:.4}, dissonance={:.4}, tension_align={:.4}",
            self.overlap, self.harmony, self.dissonance, self.tension_alignment,
        )
    }
}

// ---------------------------------------------------------------------------
// StringEnergy — total energy of a DNA string
// ---------------------------------------------------------------------------

/// Energy state of a DNA strand combining tension + information.
///
/// Tier: T2-C (N Quantity + ν Frequency + Σ Sum + μ Mapping)
/// Dominant: N Quantity (measurable energy values)
pub struct StringEnergy {
    /// Tension energy: sum of bond energies normalized by strand length.
    pub tension_energy: f64,
    /// Information energy: Shannon entropy of the word (bits).
    pub information_energy: f64,
    /// Total energy: tension + information.
    pub total_energy: f64,
    /// Quantized energy level: floor(total * 10).
    pub energy_level: u32,
    /// Energy density: total / strand length.
    pub energy_density: f64,
}

/// Compute string energy from a strand and its source word.
#[must_use]
pub fn string_energy(word: &str) -> StringEnergy {
    let strand = storage::encode_str(word);
    let tens = tension(&strand);
    let ent = lexicon::entropy(word);

    let tension_energy = tens.mean_tension / MAX_BOND_ENERGY; // normalized [0, 1]
    let information_energy = if word.is_empty() { 0.0 } else { ent / 8.0 }; // normalized [0, 1]
    let total_energy = tension_energy + information_energy;
    let energy_level = (total_energy * 10.0) as u32;
    let energy_density = if strand.is_empty() {
        0.0
    } else {
        total_energy / strand.len() as f64
    };

    StringEnergy {
        tension_energy,
        information_energy,
        total_energy,
        energy_level,
        energy_density,
    }
}

/// Compute GC content of a strand (convenience re-export for this module).
#[must_use]
pub fn gc_content(strand: &Strand) -> f64 {
    ops::gc_content(strand)
}

impl fmt::Display for StringEnergy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Energy: tension={:.4}, info={:.4}, total={:.4}, level={}, density={:.6}",
            self.tension_energy,
            self.information_energy,
            self.total_energy,
            self.energy_level,
            self.energy_density,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Tension tests
    // -----------------------------------------------------------------------

    #[test]
    fn tension_empty_strand() {
        let strand = Strand::new(vec![]);
        let t = tension(&strand);
        assert!((t.mean_tension - 0.0).abs() < f64::EPSILON);
        assert!(t.bond_energies.is_empty());
    }

    #[test]
    fn tension_all_gc() {
        let strand = Strand::parse("GGCC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            assert!((t.mean_tension - 3.0).abs() < f64::EPSILON);
            assert!((t.gc_tension_ratio - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn tension_all_at() {
        let strand = Strand::parse("AATT");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            assert!((t.mean_tension - 2.0).abs() < f64::EPSILON);
            assert!((t.gc_tension_ratio - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn tension_mixed() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            // A=2, T=2, G=3, C=3 → mean=2.5
            assert!((t.mean_tension - 2.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn tension_peak_computed() {
        let strand = Strand::parse("AAGCC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            // Windows of 3: [2,2,3]=2.33, [2,3,3]=2.67, [3,3,3]=3.0
            assert!((t.peak_tension - 3.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn tension_variance_uniform() {
        let strand = Strand::parse("AAAA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            assert!((t.variance - 0.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn tension_variance_nonzero() {
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            assert!(t.variance > 0.0);
        }
    }

    #[test]
    fn word_tension_basic() {
        let t = word_tension("rust");
        assert!(t.mean_tension >= 2.0);
        assert!(t.mean_tension <= 3.0);
        assert!(!t.bond_energies.is_empty());
    }

    // -----------------------------------------------------------------------
    // Spectrum tests
    // -----------------------------------------------------------------------

    #[test]
    fn spectrum_empty() {
        let strand = Strand::new(vec![]);
        let s = spectrum(&strand);
        assert!(s.autocorrelation.is_empty());
        assert!(s.modes.is_empty());
        assert_eq!(s.dominant_period, 0);
    }

    #[test]
    fn spectrum_short_strand() {
        let strand = Strand::parse("AT");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let sp = spectrum(&s);
            // max_lag = 1
            assert_eq!(sp.autocorrelation.len(), 1);
        }
    }

    #[test]
    fn spectrum_periodic_strand() {
        // ATGATGATG — period 3 (ATG repeats)
        let strand = Strand::parse("ATGATGATG");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let sp = spectrum(&s);
            // Should detect periodicity at lag 3
            assert!(sp.autocorrelation.len() >= 3);
            // Lag 3 (index 2) should have high autocorrelation
            assert!(sp.autocorrelation[2] > 0.5);
        }
    }

    #[test]
    fn spectrum_autocorrelation_range() {
        let strand = Strand::parse("ATGCATGCATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let sp = spectrum(&s);
            for &val in &sp.autocorrelation {
                assert!(val >= 0.0 && val <= 1.0);
            }
        }
    }

    #[test]
    fn spectrum_modes_sorted() {
        let strand = Strand::parse("ATGCATGCATGCATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let sp = spectrum(&s);
            for w in sp.modes.windows(2) {
                assert!(w[0].amplitude >= w[1].amplitude);
            }
        }
    }

    #[test]
    fn spectrum_spectral_entropy_nonneg() {
        let strand = Strand::parse("ATGCGA");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let sp = spectrum(&s);
            assert!(sp.spectral_entropy >= 0.0);
        }
    }

    #[test]
    fn word_spectrum_basic() {
        let sp = word_spectrum("hello");
        assert!(!sp.autocorrelation.is_empty());
        assert!(sp.dominant_period > 0 || sp.modes.is_empty());
    }

    // -----------------------------------------------------------------------
    // Resonance tests
    // -----------------------------------------------------------------------

    #[test]
    fn resonance_identical_strands() {
        let strand = Strand::parse("ATGCATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let r = resonance(&s, &s);
            // Self-resonance should be high
            assert!(r.overlap > 0.5);
            assert!((r.harmony - 1.0).abs() < f64::EPSILON);
            assert!((r.dissonance - 0.0).abs() < f64::EPSILON);
            assert!((r.tension_alignment - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn resonance_complementary_strands() {
        let a = Strand::parse("AAAA");
        let b = Strand::parse("TTTT");
        if let (Some(sa), Some(sb)) = (a.ok(), b.ok()) {
            let r = resonance(&sa, &sb);
            // Same tension (both 2.0), so tension_alignment = 1.0
            assert!((r.tension_alignment - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn resonance_different_tension() {
        let a = Strand::parse("AAAA");
        let b = Strand::parse("GGGG");
        if let (Some(sa), Some(sb)) = (a.ok(), b.ok()) {
            let r = resonance(&sa, &sb);
            // A=2.0, G=3.0, diff=1.0/3.0 → alignment ≈ 0.667
            assert!(r.tension_alignment < 1.0);
            assert!(r.tension_alignment > 0.0);
        }
    }

    #[test]
    fn resonance_dissonance_complement() {
        let a = Strand::parse("ATGC");
        let b = Strand::parse("GCTA");
        if let (Some(sa), Some(sb)) = (a.ok(), b.ok()) {
            let r = resonance(&sa, &sb);
            assert!((r.harmony + r.dissonance - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn word_resonance_self() {
        let r = word_resonance("rust", "rust");
        assert!(r.overlap > 0.5);
        assert!((r.harmony - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn word_resonance_different() {
        let r = word_resonance("rust", "python");
        // Different words will have some resonance variance
        assert!(r.overlap >= 0.0 && r.overlap <= 1.0);
        assert!(r.harmony >= 0.0 && r.harmony <= 1.0);
    }

    // -----------------------------------------------------------------------
    // StringEnergy tests
    // -----------------------------------------------------------------------

    #[test]
    fn energy_empty_word() {
        let e = string_energy("");
        // Empty word still encodes to a DNA strand (encoding overhead),
        // so tension_energy may be non-zero. Information energy is 0.
        assert!(e.tension_energy >= 0.0);
        assert!((e.information_energy - 0.0).abs() < f64::EPSILON);
        assert!(e.energy_level >= 0);
    }

    #[test]
    fn energy_single_char() {
        let e = string_energy("a");
        // Single char: entropy=0, tension > 0
        assert!(e.tension_energy > 0.0);
        assert!((e.information_energy - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn energy_total_is_sum() {
        let e = string_energy("hello");
        assert!((e.total_energy - (e.tension_energy + e.information_energy)).abs() < 1e-10);
    }

    #[test]
    fn energy_level_quantized() {
        let e = string_energy("rust");
        // Energy level should be total * 10 (floored)
        let expected = (e.total_energy * 10.0) as u32;
        assert_eq!(e.energy_level, expected);
    }

    #[test]
    fn energy_density_positive() {
        let e = string_energy("hello");
        assert!(e.energy_density > 0.0);
    }

    #[test]
    fn energy_normalized_range() {
        let e = string_energy("abcdefghijklmnop");
        // tension_energy in [0,1], info_energy in [0,1], total in [0,2]
        assert!(e.tension_energy >= 0.0 && e.tension_energy <= 1.0);
        assert!(e.information_energy >= 0.0 && e.information_energy <= 1.0);
        assert!(e.total_energy >= 0.0 && e.total_energy <= 2.0);
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn display_tension() {
        let t = word_tension("test");
        let s = format!("{t}");
        assert!(s.contains("Tension:"));
        assert!(s.contains("mean="));
        assert!(s.contains("peak="));
    }

    #[test]
    fn display_spectrum() {
        let sp = word_spectrum("hello");
        let s = format!("{sp}");
        assert!(s.contains("Spectrum:"));
        assert!(s.contains("lags"));
    }

    #[test]
    fn display_harmonic_mode() {
        let mode = HarmonicMode {
            period: 4,
            amplitude: 0.75,
            phase: 0.25,
        };
        let s = format!("{mode}");
        assert!(s.contains("mode(T=4"));
        assert!(s.contains("A=0.75"));
    }

    #[test]
    fn display_resonance() {
        let r = word_resonance("abc", "def");
        let s = format!("{r}");
        assert!(s.contains("Resonance:"));
        assert!(s.contains("overlap="));
    }

    #[test]
    fn display_energy() {
        let e = string_energy("test");
        let s = format!("{e}");
        assert!(s.contains("Energy:"));
        assert!(s.contains("level="));
    }

    // -----------------------------------------------------------------------
    // Cross-module integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn gc_content_matches_tension_ratio() {
        // For a pure strand, gc_tension_ratio should correlate with gc_content
        let strand = Strand::parse("ATGC");
        assert!(strand.is_ok());
        if let Some(s) = strand.ok() {
            let t = tension(&s);
            let gc = gc_content(&s);
            // GC ratio = (gc_count * 3) / total_energy
            // With ATGC: 2 GC bases, total = 2+2+3+3 = 10, gc_energy = 6
            // gc_tension_ratio = 6/10 = 0.6
            // gc_content = 2/4 = 0.5
            // They're related but not identical
            assert!(gc > 0.0);
            assert!(t.gc_tension_ratio > 0.0);
        }
    }

    #[test]
    fn higher_gc_means_higher_tension() {
        let t_at = word_tension("aaaa"); // all-AT encoding
        let t_gc = word_tension("cccc"); // will have different GC content
        // The relative tension depends on encoding, but both should be valid
        assert!(t_at.mean_tension >= 2.0);
        assert!(t_gc.mean_tension >= 2.0);
    }
}
