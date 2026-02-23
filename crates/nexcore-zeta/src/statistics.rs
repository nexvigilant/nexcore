//! # Zero Statistics — GUE Random Matrix Connection
//!
//! Computes statistical properties of zeta zero spacings and compares
//! them to predictions from the Gaussian Unitary Ensemble (GUE).
//!
//! ## Montgomery's Pair Correlation Conjecture (1973)
//!
//! The pair correlation function of normalized zeta zero spacings
//! approaches:
//!
//! ```text
//! R₂(u) = 1 − (sin(πu) / (πu))²
//! ```
//!
//! This is EXACTLY the pair correlation of eigenvalues of large random
//! Hermitian matrices from GUE — connecting number theory to quantum
//! physics.
//!
//! ## What This Proves (Computationally)
//!
//! If the zero spacing statistics match GUE predictions, we have evidence
//! that the Riemann zeros behave as if they are eigenvalues of a
//! self-adjoint operator. This is the **Hilbert-Pólya conjecture** —
//! and it would imply RH if such an operator exists.

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::zeros::{ZetaZero, count_zeros_to_height};

/// Normalized zero spacing: δₙ = (γₙ₊₁ − γₙ) · ln(γₙ/(2π)) / (2π)
///
/// The normalization ensures mean spacing is 1 regardless of height.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NormalizedSpacing {
    /// Index of the zero (1-based).
    pub ordinal: u64,
    /// Raw gap γₙ₊₁ − γₙ.
    pub raw_gap: f64,
    /// Normalized gap (mean ~1 under GUE).
    pub normalized_gap: f64,
}

/// GUE comparison results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GueComparison {
    /// Number of zero spacings analyzed.
    pub n_spacings: usize,
    /// Mean normalized spacing (should be ~1.0).
    pub mean_spacing: f64,
    /// Variance of normalized spacings (GUE predicts ~0.178).
    pub variance: f64,
    /// GUE predicted variance: 1 − 2/π² + ln(2π)/π² ≈ 0.178.
    pub gue_predicted_variance: f64,
    /// Pair correlation computed at sample points.
    pub pair_correlation_samples: Vec<(f64, f64)>,
    /// GUE prediction at the same sample points.
    pub gue_prediction_samples: Vec<(f64, f64)>,
    /// Mean absolute error between computed and GUE pair correlation.
    pub pair_correlation_mae: f64,
    /// Level spacing distribution histogram (bins, counts normalized).
    pub spacing_histogram: Vec<(f64, f64)>,
    /// Overall GUE match score ∈ [0, 1].
    pub gue_match_score: f64,
}

/// Compute normalized spacings from a sorted list of zeros.
///
/// Normalization: δₙ = (γₙ₊₁ − γₙ) · d(γₙ)
/// where d(t) = ln(t / (2π)) / (2π) is the mean density at height t.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 2 zeros provided.
pub fn normalized_spacings(zeros: &[ZetaZero]) -> Result<Vec<NormalizedSpacing>, ZetaError> {
    if zeros.len() < 2 {
        return Err(ZetaError::InvalidParameter(
            "need at least 2 zeros for spacing analysis".to_string(),
        ));
    }

    let mut spacings = Vec::with_capacity(zeros.len() - 1);

    for i in 0..zeros.len() - 1 {
        let t = zeros[i].t;
        let raw_gap = zeros[i + 1].t - t;

        // Mean density at height t: d(t) = ln(t/(2π)) / (2π)
        let two_pi = 2.0 * PI;
        let density = if t > two_pi {
            (t / two_pi).ln() / two_pi
        } else {
            1.0 / two_pi // fallback for very low t
        };

        let normalized_gap = raw_gap * density;

        spacings.push(NormalizedSpacing {
            ordinal: zeros[i].ordinal,
            raw_gap,
            normalized_gap,
        });
    }

    Ok(spacings)
}

/// Montgomery's pair correlation: R₂(u) = 1 − (sin(πu)/(πu))²
///
/// This is the GUE prediction for the pair correlation of eigenvalues.
#[must_use]
pub fn gue_pair_correlation(u: f64) -> f64 {
    if u.abs() < 1e-12 {
        return 0.0; // lim_{u→0} R₂(u) = 0
    }
    let sinc = (PI * u).sin() / (PI * u);
    1.0 - sinc * sinc
}

/// Compute the empirical pair correlation function from zero spacings.
///
/// Counts pairs of zeros (γᵢ, γⱼ) with normalized distance in each bin,
/// then normalizes to match the expected density.
///
/// Returns `(u_center, R₂_empirical)` pairs.
pub fn empirical_pair_correlation(
    zeros: &[ZetaZero],
    n_bins: usize,
    max_u: f64,
) -> Result<Vec<(f64, f64)>, ZetaError> {
    if zeros.len() < 10 {
        return Err(ZetaError::InvalidParameter(
            "need at least 10 zeros for pair correlation".to_string(),
        ));
    }
    if n_bins == 0 {
        return Err(ZetaError::InvalidParameter(
            "n_bins must be > 0".to_string(),
        ));
    }

    let bin_width = max_u / n_bins as f64;
    let mut bins = vec![0_u64; n_bins];
    let n = zeros.len();

    // For each pair (i, j) with i < j, compute normalized distance
    for i in 0..n {
        let two_pi = 2.0 * PI;
        let density_i = if zeros[i].t > two_pi {
            (zeros[i].t / two_pi).ln() / two_pi
        } else {
            1.0 / two_pi
        };

        for j in (i + 1)..n {
            let delta = (zeros[j].t - zeros[i].t) * density_i;
            if delta >= max_u {
                break; // zeros are sorted, so all further j are farther
            }
            let bin_idx = (delta / bin_width) as usize;
            if bin_idx < n_bins {
                bins[bin_idx] += 1;
            }
        }
    }

    // Normalize: expected count per bin = n · bin_width (if uniform)
    let norm = n as f64 * bin_width;
    let result: Vec<(f64, f64)> = bins
        .iter()
        .enumerate()
        .map(|(idx, &count)| {
            let u_center = (idx as f64 + 0.5) * bin_width;
            let r2 = if norm > 0.0 { count as f64 / norm } else { 0.0 };
            (u_center, r2)
        })
        .collect();

    Ok(result)
}

/// Build a spacing histogram (Wigner surmise comparison).
///
/// The GUE nearest-neighbor spacing distribution is approximated by
/// Wigner's surmise: p(s) = (32/π²) s² exp(−4s²/π)
pub fn spacing_histogram(
    spacings: &[NormalizedSpacing],
    n_bins: usize,
    max_s: f64,
) -> Vec<(f64, f64)> {
    if spacings.is_empty() || n_bins == 0 {
        return vec![];
    }

    let bin_width = max_s / n_bins as f64;
    let mut bins = vec![0_u64; n_bins];

    for s in spacings {
        let idx = (s.normalized_gap / bin_width) as usize;
        if idx < n_bins {
            bins[idx] += 1;
        }
    }

    let total = spacings.len() as f64;
    bins.iter()
        .enumerate()
        .map(|(idx, &count)| {
            let s_center = (idx as f64 + 0.5) * bin_width;
            let density = count as f64 / (total * bin_width);
            (s_center, density)
        })
        .collect()
}

/// Wigner surmise for GUE: p(s) = (32/π²) s² exp(−4s²/π)
#[must_use]
pub fn wigner_surmise_gue(s: f64) -> f64 {
    if s < 0.0 {
        return 0.0;
    }
    (32.0 / (PI * PI)) * s * s * (-4.0 * s * s / PI).exp()
}

/// Full GUE comparison: compute all statistics and match score.
///
/// # Errors
///
/// Returns error if fewer than 20 zeros provided.
pub fn compare_to_gue(zeros: &[ZetaZero]) -> Result<GueComparison, ZetaError> {
    if zeros.len() < 20 {
        return Err(ZetaError::InvalidParameter(
            "need at least 20 zeros for GUE comparison".to_string(),
        ));
    }

    let spacings = normalized_spacings(zeros)?;

    // Mean and variance
    let n = spacings.len() as f64;
    let mean: f64 = spacings.iter().map(|s| s.normalized_gap).sum::<f64>() / n;
    let variance: f64 = spacings
        .iter()
        .map(|s| {
            let d = s.normalized_gap - mean;
            d * d
        })
        .sum::<f64>()
        / n;

    // GUE predicted variance ≈ 0.178
    // Exact: 1 - 2/π² + (ln(2π) + γ - 1)/π² but ≈ 0.178 suffices
    let gue_predicted_variance = 0.178;

    // Pair correlation
    let n_bins = 20;
    let max_u = 4.0;
    let empirical = empirical_pair_correlation(zeros, n_bins, max_u)?;
    let gue_pred: Vec<(f64, f64)> = empirical
        .iter()
        .map(|&(u, _)| (u, gue_pair_correlation(u)))
        .collect();

    // MAE between empirical and GUE
    let mae: f64 = empirical
        .iter()
        .zip(gue_pred.iter())
        .map(|(&(_, r_emp), &(_, r_gue))| (r_emp - r_gue).abs())
        .sum::<f64>()
        / empirical.len() as f64;

    // Spacing histogram
    let hist = spacing_histogram(&spacings, 20, 4.0);

    // Match score: combine variance match + pair correlation match
    // Variance component: how close is our variance to GUE prediction
    let var_score =
        1.0 - ((variance - gue_predicted_variance).abs() / gue_predicted_variance).min(1.0);

    // Pair correlation component: 1 - MAE (lower MAE = better match)
    let pc_score = (1.0 - mae).max(0.0);

    // Mean component: how close is mean to 1.0
    let mean_score = 1.0 - (mean - 1.0).abs().min(1.0);

    // Weighted composite
    let gue_match_score = 0.40 * var_score + 0.40 * pc_score + 0.20 * mean_score;

    Ok(GueComparison {
        n_spacings: spacings.len(),
        mean_spacing: mean,
        variance,
        gue_predicted_variance,
        pair_correlation_samples: empirical,
        gue_prediction_samples: gue_pred,
        pair_correlation_mae: mae,
        spacing_histogram: hist,
        gue_match_score,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    #[test]
    fn gue_pair_correlation_at_zero() {
        // R₂(0) = 0 (zero repulsion)
        assert!((gue_pair_correlation(0.0)).abs() < 1e-10);
    }

    #[test]
    fn gue_pair_correlation_at_large_u() {
        // R₂(u) → 1 as u → ∞
        let r = gue_pair_correlation(10.0);
        assert!(r > 0.99, "R₂(10) = {r}, expected > 0.99");
    }

    #[test]
    fn gue_pair_correlation_at_one() {
        // R₂(1) = 1 − (sin(π)/π)² = 1 − 0 = 1
        let r = gue_pair_correlation(1.0);
        assert!((r - 1.0).abs() < 1e-6, "R₂(1) = {r}, expected 1.0");
    }

    #[test]
    fn gue_pair_correlation_at_half() {
        // R₂(0.5) = 1 − (sin(π/2)/(π/2))² = 1 − (1/(π/2))² = 1 − 4/π²
        let expected = 1.0 - 4.0 / (PI * PI);
        let r = gue_pair_correlation(0.5);
        assert!(
            (r - expected).abs() < 1e-10,
            "R₂(0.5) = {r}, expected {expected}"
        );
    }

    #[test]
    fn wigner_surmise_peak_near_unity() {
        // Wigner surmise peaks near s ≈ 0.886
        // p'(s) = 0 when 2s - 8s³/π = 0 → s² = π/4 → s = √(π/4) ≈ 0.886
        let peak_s = (PI / 4.0).sqrt();
        let p_peak = wigner_surmise_gue(peak_s);
        // Check that it's a local maximum
        let p_before = wigner_surmise_gue(peak_s - 0.1);
        let p_after = wigner_surmise_gue(peak_s + 0.1);
        assert!(
            p_peak > p_before,
            "not a peak: p({peak_s})={p_peak} <= p({})={p_before}",
            peak_s - 0.1
        );
        assert!(
            p_peak > p_after,
            "not a peak: p({peak_s})={p_peak} <= p({})={p_after}",
            peak_s + 0.1
        );
    }

    #[test]
    fn wigner_surmise_zero_at_origin() {
        // p(0) = 0 — zero repulsion
        assert!((wigner_surmise_gue(0.0)).abs() < 1e-15);
    }

    #[test]
    fn normalized_spacings_basic() {
        let zeros = find_zeros_bracket(10.0, 50.0, 0.1);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            let spacings = normalized_spacings(&zeros);
            assert!(spacings.is_ok());
            if let Some(spacings) = spacings.ok() {
                assert!(!spacings.is_empty());
                // All spacings should be positive
                for s in &spacings {
                    assert!(s.raw_gap > 0.0, "negative raw gap at ordinal {}", s.ordinal);
                    assert!(
                        s.normalized_gap > 0.0,
                        "negative normalized gap at ordinal {}",
                        s.ordinal
                    );
                }
            }
        }
    }

    #[test]
    fn gue_comparison_with_200_zeros() {
        let zeros = find_zeros_bracket(10.0, 200.0, 0.05);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            if zeros.len() < 20 {
                return;
            }
            let comparison = compare_to_gue(&zeros);
            assert!(
                comparison.is_ok(),
                "GUE comparison failed: {:?}",
                comparison.err()
            );
            if let Some(c) = comparison.ok() {
                // Mean spacing should be roughly 1.0 (within tolerance)
                assert!(
                    c.mean_spacing > 0.5 && c.mean_spacing < 2.0,
                    "mean spacing {:.3} far from 1.0",
                    c.mean_spacing
                );
                // GUE match score should be positive
                assert!(
                    c.gue_match_score > 0.0,
                    "GUE match score {:.3} should be > 0",
                    c.gue_match_score
                );
                // Pair correlation should have samples
                assert!(!c.pair_correlation_samples.is_empty());
                assert!(!c.spacing_histogram.is_empty());
            }
        }
    }

    #[test]
    fn gue_comparison_with_1000_zeros() {
        // The BIG test: 649 zeros, full GUE comparison
        let zeros = find_zeros_bracket(10.0, 1000.0, 0.02);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            if zeros.len() < 100 {
                return; // not enough zeros
            }
            let comparison = compare_to_gue(&zeros);
            assert!(comparison.is_ok());
            if let Some(c) = comparison.ok() {
                // With 649 zeros, statistics should converge well
                // Mean spacing near 1.0
                assert!(
                    (c.mean_spacing - 1.0).abs() < 0.5,
                    "mean spacing {:.3} too far from 1.0 with {} spacings",
                    c.mean_spacing,
                    c.n_spacings
                );
                // GUE match score should be meaningful
                assert!(
                    c.gue_match_score > 0.2,
                    "GUE match {:.3} too low with {} zeros — the connection should be visible",
                    c.gue_match_score,
                    zeros.len()
                );
            }
        }
    }

    #[test]
    fn spacing_histogram_integrates_to_one() {
        let zeros = find_zeros_bracket(10.0, 200.0, 0.05);
        assert!(zeros.is_ok());
        if let Some(zeros) = zeros.ok() {
            let spacings = normalized_spacings(&zeros);
            assert!(spacings.is_ok());
            if let Some(spacings) = spacings.ok() {
                let hist = spacing_histogram(&spacings, 20, 4.0);
                // Integral ≈ Σ density × bin_width ≈ 1.0 (probability distribution)
                let bin_width = 4.0 / 20.0;
                let integral: f64 = hist.iter().map(|&(_, d)| d * bin_width).sum();
                assert!(
                    (integral - 1.0).abs() < 0.2,
                    "histogram integral {integral:.3} not near 1.0"
                );
            }
        }
    }
}
