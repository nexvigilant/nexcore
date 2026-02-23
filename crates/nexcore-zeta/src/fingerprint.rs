//! # Spectral Fingerprinting
//!
//! Extracts a compact spectral fingerprint from a set of zeta zeros via
//! CMV (Verblunsky) reconstruction, then provides distance metrics and
//! classification for comparing zero sets.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_zeta::fingerprint::{fingerprint_zeros, compare_fingerprints, FingerprintClass};
//! use nexcore_zeta::zeros::find_zeros_bracket;
//!
//! let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
//! let fp = fingerprint_zeros(&zeros).unwrap();
//! let dist = compare_fingerprints(&fp, &fp);
//! assert_eq!(dist.classification, FingerprintClass::SameClass);
//! ```

use serde::{Deserialize, Serialize};

use crate::cmv::{CmvReconstruction, reconstruct_cmv};
use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── SpectralFingerprint ───────────────────────────────────────────────────────

/// Compact spectral fingerprint of a zero set extracted via CMV reconstruction.
///
/// Each field captures a distinct structural property of the Verblunsky
/// coefficient sequence, enabling quantitative comparison across zero sets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralFingerprint {
    /// Power-law decay exponent β: |αₖ| ~ A·k^(−β).
    ///
    /// Positive values indicate a well-conditioned, decaying coefficient sequence.
    pub decay_rate: f64,
    /// Coefficient of variation of |αₖ|: std(|αₖ|) / mean(|αₖ|).
    ///
    /// Low values indicate uniform magnitude (regular operator structure).
    pub coefficient_regularity: f64,
    /// Standard deviation of consecutive phase differences arg(αₖ₊₁) − arg(αₖ).
    ///
    /// Low values indicate smooth phase progression.
    pub phase_regularity: f64,
    /// Mean Verblunsky coefficient magnitude mean(|αₖ|).
    pub mean_magnitude: f64,
    /// Maximum Verblunsky coefficient magnitude max(|αₖ|) — strictly < 1.
    pub max_magnitude: f64,
    /// Indices where |αₖ| > mean + 2σ — spectral spikes in the coefficient sequence.
    pub spike_locations: Vec<usize>,
    /// 1.0 − roundtrip_error: how faithfully the CMV encodes the spectral measure.
    ///
    /// Values near 1.0 indicate high-fidelity reconstruction.
    pub roundtrip_fidelity: f64,
    /// Number of zeros used to compute this fingerprint.
    pub n_zeros: usize,
}

// ── FingerprintClass ──────────────────────────────────────────────────────────

/// Classification outcome when comparing two spectral fingerprints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FingerprintClass {
    /// Normalized RMS distance < 0.20 — fingerprints are statistically consistent.
    SameClass,
    /// Normalized RMS distance > 0.60 — fingerprints are structurally distinct.
    DifferentClass,
    /// Normalized RMS distance ∈ \[0.20, 0.60\] — insufficient evidence to classify.
    Inconclusive,
}

// ── FingerprintDistance ───────────────────────────────────────────────────────

/// Weighted distance between two spectral fingerprints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintDistance {
    /// Root-mean-square normalized L2 distance (∈ \[0, 1\]).
    pub total_distance: f64,
    /// Per-feature breakdown: `(feature_name, normalized_distance)`.
    pub component_distances: Vec<(String, f64)>,
    /// Classification derived from `total_distance` thresholds.
    pub classification: FingerprintClass,
}

// ── FingerprintComparison ─────────────────────────────────────────────────────

/// One pairwise comparison produced by [`compare_across_zero_sets`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintComparison {
    /// Label of the first zero set.
    pub name_a: String,
    /// Label of the second zero set.
    pub name_b: String,
    /// Full distance breakdown between the two fingerprints.
    pub distance: FingerprintDistance,
    /// Top-level classification (mirrors `distance.classification`).
    pub classification: FingerprintClass,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Compute a [`SpectralFingerprint`] from a slice of zeta zeros.
///
/// Runs CMV reconstruction on the zero set and extracts structural properties
/// of the resulting Verblunsky coefficient sequence.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 zeros are provided.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::fingerprint::fingerprint_zeros;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
/// let fp = fingerprint_zeros(&zeros).unwrap();
/// assert!(fp.roundtrip_fidelity >= 0.0 && fp.roundtrip_fidelity <= 1.0);
/// assert!(fp.max_magnitude < 1.0);
/// ```
pub fn fingerprint_zeros(zeros: &[ZetaZero]) -> Result<SpectralFingerprint, ZetaError> {
    let cmv = reconstruct_cmv(zeros)?;
    Ok(fingerprint_from_cmv(&cmv))
}

/// Compute a [`SpectralFingerprint`] from an arbitrary zero set.
///
/// Identical to [`fingerprint_zeros`] — accepts any `&[ZetaZero]` regardless
/// of origin (zeta, Dirichlet L-function, random matrix eigenvalues, etc.).
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 zeros are provided.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::fingerprint::fingerprint_l_function_zeros;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
/// let fp = fingerprint_l_function_zeros(&zeros).unwrap();
/// assert!(fp.n_zeros > 0);
/// ```
pub fn fingerprint_l_function_zeros(zeros: &[ZetaZero]) -> Result<SpectralFingerprint, ZetaError> {
    fingerprint_zeros(zeros)
}

/// Compare two spectral fingerprints and return a weighted distance breakdown.
///
/// Computes the root-mean-square of normalized per-feature distances across
/// four structural dimensions:
///
/// | Feature | Reference range |
/// |---------|----------------|
/// | `decay_rate` | 0 – 5 |
/// | `coefficient_regularity` | 0 – 2 |
/// | `phase_regularity` | 0 – 2π |
/// | `roundtrip_fidelity` | 0 – 1 |
///
/// The resulting `total_distance` ∈ \[0, 1\] drives threshold-based classification.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::fingerprint::{fingerprint_zeros, compare_fingerprints, FingerprintClass};
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
/// let fp = fingerprint_zeros(&zeros).unwrap();
/// let dist = compare_fingerprints(&fp, &fp);
/// assert_eq!(dist.classification, FingerprintClass::SameClass);
/// assert!(dist.total_distance < 1e-10);
/// ```
#[must_use]
pub fn compare_fingerprints(
    a: &SpectralFingerprint,
    b: &SpectralFingerprint,
) -> FingerprintDistance {
    let two_pi = 2.0 * std::f64::consts::PI;

    // (name, value_a, value_b, normalization_range)
    let features: [(&str, f64, f64, f64); 4] = [
        ("decay_rate", a.decay_rate, b.decay_rate, 5.0),
        (
            "coefficient_regularity",
            a.coefficient_regularity,
            b.coefficient_regularity,
            2.0,
        ),
        (
            "phase_regularity",
            a.phase_regularity,
            b.phase_regularity,
            two_pi,
        ),
        (
            "roundtrip_fidelity",
            a.roundtrip_fidelity,
            b.roundtrip_fidelity,
            1.0,
        ),
    ];

    let mut sum_sq = 0.0_f64;
    let mut component_distances = Vec::with_capacity(4);

    for (name, va, vb, range) in features {
        let d = if range > 1e-30 {
            (va - vb).abs() / range
        } else {
            0.0
        };
        sum_sq += d * d;
        component_distances.push((name.to_string(), d));
    }

    let total_distance = (sum_sq / 4.0_f64).sqrt();

    let classification = if total_distance < 0.20 {
        FingerprintClass::SameClass
    } else if total_distance > 0.60 {
        FingerprintClass::DifferentClass
    } else {
        FingerprintClass::Inconclusive
    };

    FingerprintDistance {
        total_distance,
        component_distances,
        classification,
    }
}

/// Fingerprint every zero set in the input and compare all pairs.
///
/// Returns one [`FingerprintComparison`] per unique pair `(i, j)` with `i < j`.
/// Empty and single-element inputs return an empty comparison list.
///
/// # Errors
///
/// Propagates [`ZetaError`] from any individual fingerprint computation.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::fingerprint::compare_across_zero_sets;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
/// // Single set → no pairs
/// let comparisons = compare_across_zero_sets(&[("zeta", zeros.as_slice())]).unwrap();
/// assert!(comparisons.is_empty());
/// ```
pub fn compare_across_zero_sets(
    zero_sets: &[(&str, &[ZetaZero])],
) -> Result<Vec<FingerprintComparison>, ZetaError> {
    // Fingerprint each set eagerly so errors surface before pair enumeration
    let fingerprints: Result<Vec<_>, _> = zero_sets
        .iter()
        .map(|&(name, zeros)| fingerprint_zeros(zeros).map(|fp| (name.to_string(), fp)))
        .collect();

    let fingerprints = fingerprints?;
    let n = fingerprints.len();
    let pair_count = if n < 2 { 0 } else { n * (n - 1) / 2 };
    let mut comparisons = Vec::with_capacity(pair_count);

    for i in 0..n {
        for j in (i + 1)..n {
            let (ref name_a, ref fp_a) = fingerprints[i];
            let (ref name_b, ref fp_b) = fingerprints[j];
            let distance = compare_fingerprints(fp_a, fp_b);
            let classification = distance.classification;
            comparisons.push(FingerprintComparison {
                name_a: name_a.clone(),
                name_b: name_b.clone(),
                distance,
                classification,
            });
        }
    }

    Ok(comparisons)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Extract a [`SpectralFingerprint`] from a complete [`CmvReconstruction`].
fn fingerprint_from_cmv(cmv: &CmvReconstruction) -> SpectralFingerprint {
    let mags = &cmv.verblunsky_magnitudes;
    let n = mags.len();

    // Reuse pre-computed aggregates from CmvStructure
    let mean_magnitude = cmv.structure.mean_coefficient_magnitude;
    let max_magnitude = cmv.structure.max_coefficient;

    // σ of |αₖ| for spike threshold (not stored in CmvStructure)
    let std_magnitude = if n < 2 {
        0.0
    } else {
        let var = mags
            .iter()
            .map(|&m| (m - mean_magnitude) * (m - mean_magnitude))
            .sum::<f64>()
            / n as f64;
        var.sqrt()
    };

    let threshold = mean_magnitude + 2.0 * std_magnitude;
    let spike_locations: Vec<usize> = mags
        .iter()
        .enumerate()
        .filter(|&(_, &m)| m > threshold)
        .map(|(i, _)| i)
        .collect();

    // Clamp fidelity to [0, 1] — roundtrip_error can exceed 1 under poor conditioning
    let roundtrip_fidelity = (1.0 - cmv.roundtrip_error).clamp(0.0, 1.0);

    SpectralFingerprint {
        decay_rate: cmv.structure.coefficient_decay_rate,
        coefficient_regularity: cmv.structure.coefficient_regularity,
        phase_regularity: cmv.structure.phase_regularity,
        mean_magnitude,
        max_magnitude,
        spike_locations,
        roundtrip_fidelity,
        n_zeros: cmv.eigenvalues.len(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_test_zeros() -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, 200.0, 0.05).unwrap_or_default()
    }

    #[test]
    fn self_distance_is_zero() {
        let zeros = get_test_zeros();
        let fp = fingerprint_zeros(&zeros).unwrap();
        let dist = compare_fingerprints(&fp, &fp);
        assert!(
            dist.total_distance < 1e-10,
            "self-distance should be ≈ 0, got {:.2e}",
            dist.total_distance
        );
        assert_eq!(
            dist.classification,
            FingerprintClass::SameClass,
            "self-comparison must be SameClass"
        );
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let zeros = get_test_zeros();
        let fp1 = fingerprint_zeros(&zeros).unwrap();
        let fp2 = fingerprint_zeros(&zeros).unwrap();
        assert_eq!(
            fp1.decay_rate, fp2.decay_rate,
            "decay_rate must be identical"
        );
        assert_eq!(
            fp1.coefficient_regularity, fp2.coefficient_regularity,
            "coefficient_regularity must be identical"
        );
        assert_eq!(
            fp1.phase_regularity, fp2.phase_regularity,
            "phase_regularity must be identical"
        );
        assert_eq!(
            fp1.roundtrip_fidelity, fp2.roundtrip_fidelity,
            "roundtrip_fidelity must be identical"
        );
        assert_eq!(fp1.n_zeros, fp2.n_zeros, "n_zeros must be identical");
        assert_eq!(
            fp1.spike_locations, fp2.spike_locations,
            "spike_locations must be identical"
        );
    }

    #[test]
    fn split_halves_universality() {
        let zeros = get_test_zeros();
        assert!(
            zeros.len() >= 20,
            "need at least 20 zeros, got {}",
            zeros.len()
        );
        let n = zeros.len();
        let first_half = &zeros[..n / 2];
        let second_half = &zeros[n / 2..];

        let fp1 = fingerprint_zeros(first_half).unwrap();
        let fp2 = fingerprint_zeros(second_half).unwrap();
        let dist = compare_fingerprints(&fp1, &fp2);

        assert_ne!(
            dist.classification,
            FingerprintClass::DifferentClass,
            "split zeta zeros should not be DifferentClass (total_distance = {:.4})",
            dist.total_distance
        );
    }

    #[test]
    fn different_n_values_stable() {
        let zeros = get_test_zeros();
        assert!(
            zeros.len() >= 40,
            "need at least 40 zeros, got {}",
            zeros.len()
        );

        let fp_small = fingerprint_zeros(&zeros[..20]).unwrap();
        let fp_large = fingerprint_zeros(&zeros[..40]).unwrap();
        let dist = compare_fingerprints(&fp_small, &fp_large);

        assert!(
            dist.total_distance < 0.80,
            "fingerprints with N=20 vs N=40 too far apart: {:.4}",
            dist.total_distance
        );
    }

    #[test]
    fn fingerprint_fields_valid() {
        let zeros = get_test_zeros();
        let fp = fingerprint_zeros(&zeros).unwrap();
        assert!(fp.roundtrip_fidelity >= 0.0, "fidelity must be ≥ 0");
        assert!(fp.roundtrip_fidelity <= 1.0, "fidelity must be ≤ 1");
        assert!(fp.mean_magnitude >= 0.0, "mean_magnitude must be ≥ 0");
        assert!(fp.max_magnitude >= 0.0, "max_magnitude must be ≥ 0");
        assert!(
            fp.max_magnitude < 1.0,
            "max_magnitude {:.6} must be < 1 (unit disk)",
            fp.max_magnitude
        );
        assert_eq!(fp.n_zeros, zeros.len(), "n_zeros must match input length");
    }

    #[test]
    fn compare_across_returns_correct_pair_count() {
        let zeros = get_test_zeros();
        let n = zeros.len();
        let third = n / 3;
        let sets = [
            ("first", &zeros[..third] as &[ZetaZero]),
            ("second", &zeros[third..2 * third]),
            ("third", &zeros[2 * third..]),
        ];

        let comparisons = compare_across_zero_sets(&sets).unwrap();
        // C(3, 2) = 3 pairs
        assert_eq!(comparisons.len(), 3, "expected 3 pairwise comparisons");

        let pairs: Vec<(&str, &str)> = comparisons
            .iter()
            .map(|c| (c.name_a.as_str(), c.name_b.as_str()))
            .collect();
        assert!(pairs.contains(&("first", "second")));
        assert!(pairs.contains(&("first", "third")));
        assert!(pairs.contains(&("second", "third")));
    }

    #[test]
    fn single_set_returns_no_comparisons() {
        let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
        let comparisons = compare_across_zero_sets(&[("zeta", zeros.as_slice())]).unwrap();
        assert!(comparisons.is_empty(), "single set produces no comparisons");
    }

    #[test]
    fn empty_sets_returns_no_comparisons() {
        let comparisons = compare_across_zero_sets(&[]).unwrap();
        assert!(
            comparisons.is_empty(),
            "empty input produces no comparisons"
        );
    }

    #[test]
    fn too_few_zeros_returns_error() {
        let zeros = vec![
            ZetaZero {
                ordinal: 1,
                t: 14.1,
                z_value: 0.0,
                on_critical_line: true,
            },
            ZetaZero {
                ordinal: 2,
                t: 21.0,
                z_value: 0.0,
                on_critical_line: true,
            },
        ];
        assert!(
            fingerprint_zeros(&zeros).is_err(),
            "expected error for < 3 zeros"
        );
    }

    #[test]
    fn l_function_zeros_alias_matches() {
        let zeros = find_zeros_bracket(10.0, 60.0, 0.1).unwrap();
        let fp1 = fingerprint_zeros(&zeros).unwrap();
        let fp2 = fingerprint_l_function_zeros(&zeros).unwrap();
        assert_eq!(fp1.decay_rate, fp2.decay_rate);
        assert_eq!(fp1.n_zeros, fp2.n_zeros);
        assert_eq!(fp1.spike_locations, fp2.spike_locations);
    }

    #[test]
    fn component_distances_names_correct() {
        let zeros = get_test_zeros();
        let fp = fingerprint_zeros(&zeros).unwrap();
        let dist = compare_fingerprints(&fp, &fp);
        let names: Vec<&str> = dist
            .component_distances
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(names.contains(&"decay_rate"));
        assert!(names.contains(&"coefficient_regularity"));
        assert!(names.contains(&"phase_regularity"));
        assert!(names.contains(&"roundtrip_fidelity"));
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn classification_mirrors_distance_in_comparison() {
        let zeros = get_test_zeros();
        let n = zeros.len();
        let sets = [
            ("a", &zeros[..n / 2] as &[ZetaZero]),
            ("b", &zeros[n / 2..]),
        ];
        let comparisons = compare_across_zero_sets(&sets).unwrap();
        assert_eq!(comparisons.len(), 1);
        let comp = &comparisons[0];
        assert_eq!(comp.classification, comp.distance.classification);
    }
}
