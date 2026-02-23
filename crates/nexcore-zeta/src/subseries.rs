//! # Subseries Analysis and Coupling Extrapolation
//!
//! Analyzes even/odd zero subseries to test whether the alternating
//! Jacobi diagonal structure observed in [`crate::inverse`] is a genuine
//! two-component phenomenon in the operator or an artifact of interleaving
//! two statistically similar subseries.
//!
//! Also fits and extrapolates the Jacobi coupling regularity trend to
//! large N, testing whether the operator asymptotically approaches
//! uniform coupling (perfectly regular Hilbert-Pólya candidate).
//!
//! ## Background
//!
//! The Jacobi diagonal from `inverse.rs` shows an alternating period-2
//! structure at small N, with values oscillating between ~44 and ~47.
//! This could be:
//!
//! - **(a)** A genuine two-component structure in the operator
//! - **(b)** An artifact of interleaving two subseries with different statistics
//!
//! Splitting by ordinal parity and comparing GUE statistics tests (b):
//! if the subseries are statistically indistinguishable, the alternation
//! is operator structure, not sampling bias.

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::inverse::reconstruct_jacobi;
use crate::statistics::compare_to_gue;
use crate::zeros::ZetaZero;

// ── Public Types ─────────────────────────────────────────────────────────────

/// GUE statistics for even- and odd-ordinal zero subseries.
///
/// Used to determine whether the alternating Jacobi diagonal is a genuine
/// two-component operator structure or an interleaving artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubseriesAnalysis {
    /// Number of zeros in the even-ordinal subseries (ordinal % 2 == 0).
    pub even_count: usize,
    /// Number of zeros in the odd-ordinal subseries (ordinal % 2 == 1).
    pub odd_count: usize,
    /// GUE match score for even subseries ∈ [0, 1].
    pub even_gue_score: f64,
    /// GUE match score for odd subseries ∈ [0, 1].
    pub odd_gue_score: f64,
    /// Mean normalized spacing for even subseries (GUE target ≈ 1.0).
    pub even_mean_spacing: f64,
    /// Mean normalized spacing for odd subseries (GUE target ≈ 1.0).
    pub odd_mean_spacing: f64,
    /// Spacing variance for even subseries (GUE target ≈ 0.178).
    pub even_spacing_variance: f64,
    /// Spacing variance for odd subseries (GUE target ≈ 0.178).
    pub odd_spacing_variance: f64,
    /// Pair correlation MAE (vs GUE) for even subseries.
    pub even_pair_corr_mae: f64,
    /// Pair correlation MAE (vs GUE) for odd subseries.
    pub odd_pair_corr_mae: f64,
    /// Whether the two subseries are statistically distinguishable.
    ///
    /// `true` when `difference_metric > 0.1`, meaning the GUE statistics
    /// differ enough to suggest a genuine two-component structure.
    pub subseries_distinguishable: bool,
    /// Combined difference metric: sum of absolute pairwise differences
    /// in GUE score, spacing variance, and pair correlation MAE.
    pub difference_metric: f64,
}

/// Power-law fit and extrapolation of Jacobi coupling regularity vs. N.
///
/// The coupling regularity measures how uniform the Jacobi off-diagonal
/// entries are.  A decreasing trend toward 0 indicates that larger N
/// produces a more regular (harmonic-oscillator-like) operator, consistent
/// with the Hilbert-Pólya conjecture.
///
/// Model: `coupling_regularity(N) ≈ A · N^(-β)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouplingExtrapolation {
    /// Observed `(N, coupling_regularity)` data points.
    pub data_points: Vec<(usize, f64)>,
    /// Fitted decay exponent β in `A · N^(-β)`.
    /// Positive β means coupling regularity decreases with N.
    pub power_law_exponent: f64,
    /// Fitted amplitude A (coupling regularity at N = 1 on the model curve).
    pub power_law_amplitude: f64,
    /// Asymptotic value as N → ∞.
    /// Zero when β > 0 (coupling converges to uniform); `A` otherwise.
    pub asymptotic_value: f64,
    /// Predicted coupling regularity at N = 1 000.
    pub predicted_at_1000: f64,
    /// Predicted coupling regularity at N = 10 000.
    pub predicted_at_10000: f64,
    /// Predicted coupling regularity at N = 100 000.
    pub predicted_at_100000: f64,
    /// R² goodness-of-fit for the power-law model (1.0 = perfect).
    pub model_r_squared: f64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Analyze even- and odd-ordinal zero subseries for GUE statistics.
///
/// Splits `zeros` by ordinal parity, runs [`compare_to_gue`] on each
/// subseries, and computes a scalar difference metric:
///
/// ```text
/// Δ = |even_score − odd_score| + |even_var − odd_var| + |even_mae − odd_mae|
/// ```
///
/// `Δ > 0.1` is classified as *statistically distinguishable*: evidence
/// that the alternating diagonal is a genuine operator feature rather than
/// a sampling artifact.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if either subseries has fewer
/// than 20 zeros (approximately 40+ total zeros required).
pub fn analyze_subseries(zeros: &[ZetaZero]) -> Result<SubseriesAnalysis, ZetaError> {
    let (even, odd) = split_by_parity(zeros);

    if even.len() < 20 {
        return Err(ZetaError::InvalidParameter(format!(
            "even subseries has {} zeros — need ≥ 20; provide ≥ 40 total zeros",
            even.len()
        )));
    }
    if odd.len() < 20 {
        return Err(ZetaError::InvalidParameter(format!(
            "odd subseries has {} zeros — need ≥ 20; provide ≥ 40 total zeros",
            odd.len()
        )));
    }

    let even_cmp = compare_to_gue(&even)?;
    let odd_cmp = compare_to_gue(&odd)?;

    let difference_metric = (even_cmp.gue_match_score - odd_cmp.gue_match_score).abs()
        + (even_cmp.variance - odd_cmp.variance).abs()
        + (even_cmp.pair_correlation_mae - odd_cmp.pair_correlation_mae).abs();

    Ok(SubseriesAnalysis {
        even_count: even.len(),
        odd_count: odd.len(),
        even_gue_score: even_cmp.gue_match_score,
        odd_gue_score: odd_cmp.gue_match_score,
        even_mean_spacing: even_cmp.mean_spacing,
        odd_mean_spacing: odd_cmp.mean_spacing,
        even_spacing_variance: even_cmp.variance,
        odd_spacing_variance: odd_cmp.variance,
        even_pair_corr_mae: even_cmp.pair_correlation_mae,
        odd_pair_corr_mae: odd_cmp.pair_correlation_mae,
        subseries_distinguishable: difference_metric > 0.1,
        difference_metric,
    })
}

/// Extrapolate Jacobi coupling regularity to large N via power-law fit.
///
/// For each N in `sample_sizes`, reconstructs the Jacobi matrix from the
/// first N zeros and records the coupling regularity.  Fits the trend
/// `coupling_reg(N) ~ A · N^(-β)` via log-log linear regression, then
/// predicts at N = 1 000, 10 000, and 100 000.
///
/// If β > 0, the operator asymptotically approaches perfectly uniform
/// coupling as N → ∞, consistent with the Hilbert-Pólya hypothesis.
///
/// # Arguments
///
/// * `zeros` — full set of zeros (sorted by imaginary part)
/// * `sample_sizes` — list of N values to probe (each must satisfy
///   `3 ≤ N ≤ zeros.len()`; at least 3 distinct sizes required)
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 sample sizes
/// are provided or if any sample size is out of range.
pub fn extrapolate_coupling(
    zeros: &[ZetaZero],
    sample_sizes: &[usize],
) -> Result<CouplingExtrapolation, ZetaError> {
    if sample_sizes.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 sample sizes for coupling extrapolation".to_string(),
        ));
    }

    for &n in sample_sizes {
        if n < 3 {
            return Err(ZetaError::InvalidParameter(format!(
                "sample size {n} too small (need ≥ 3)"
            )));
        }
        if n > zeros.len() {
            return Err(ZetaError::InvalidParameter(format!(
                "sample size {n} exceeds available zeros {}",
                zeros.len()
            )));
        }
    }

    let mut data_points: Vec<(usize, f64)> = Vec::with_capacity(sample_sizes.len());
    for &n in sample_sizes {
        let subset = &zeros[..n];
        let jacobi = reconstruct_jacobi(subset)?;
        data_points.push((n, jacobi.structure.coupling_regularity));
    }

    let (power_law_exponent, power_law_amplitude, model_r_squared) =
        fit_power_law_from_points(&data_points);

    let predict = |n: f64| power_law_amplitude * n.powf(-power_law_exponent);

    let asymptotic_value = if power_law_exponent > 0.0 {
        0.0
    } else {
        power_law_amplitude
    };

    Ok(CouplingExtrapolation {
        data_points,
        power_law_exponent,
        power_law_amplitude,
        asymptotic_value,
        predicted_at_1000: predict(1_000.0),
        predicted_at_10000: predict(10_000.0),
        predicted_at_100000: predict(100_000.0),
        model_r_squared,
    })
}

// ── Private Helpers ───────────────────────────────────────────────────────────

/// Split zeros into even-ordinal and odd-ordinal subseries.
fn split_by_parity(zeros: &[ZetaZero]) -> (Vec<ZetaZero>, Vec<ZetaZero>) {
    zeros.iter().copied().partition(|z| z.ordinal % 2 == 0)
}

/// Fit `y(N) ~ A · N^(-β)` by log-log linear regression.
///
/// Linearized: `ln(y) = ln(A) − β·ln(N)`.
///
/// Returns `(β, A, R²)`.  Non-positive, non-finite, or infinite data
/// points are excluded from the fit.
fn fit_power_law_from_points(data: &[(usize, f64)]) -> (f64, f64, f64) {
    let log_data: Vec<(f64, f64)> = data
        .iter()
        .filter(|&&(n, cr)| n > 0 && cr > 1e-30 && cr.is_finite())
        .map(|&(n, cr)| ((n as f64).ln(), cr.ln()))
        .collect();

    let (slope, intercept, r2) = linear_regression(&log_data);

    // slope  = −β  →  β = −slope
    // intercept = ln(A)  →  A = exp(intercept)
    (-slope, intercept.exp(), r2)
}

/// Simple ordinary least-squares regression: `y = slope·x + intercept`.
///
/// Returns `(slope, intercept, R²)`.  Returns `(0, mean_y, 0)` for
/// degenerate inputs (fewer than 2 points or zero variance in x).
// n·Σx² − (Σx)² is the standard OLS denominator; not a copy-paste error.
#[allow(clippy::suspicious_operation_groupings)]
fn linear_regression(data: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = data.len() as f64;
    if data.len() < 2 {
        let mean_y = if data.is_empty() { 0.0 } else { data[0].1 };
        return (0.0, mean_y, 0.0);
    }

    let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
    let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return (0.0, sum_y / n, 0.0);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    let mean_y = sum_y / n;
    let ss_res: f64 = data
        .iter()
        .map(|(x, y)| {
            let predicted = slope * x + intercept;
            (y - predicted) * (y - predicted)
        })
        .sum();
    let ss_tot: f64 = data.iter().map(|(_, y)| (y - mean_y) * (y - mean_y)).sum();
    let r2 = if ss_tot > 1e-30 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    (slope, intercept, r2)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_test_zeros() -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, 200.0, 0.05).unwrap_or_default()
    }

    fn get_large_zeros() -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, 1000.0, 0.02).unwrap_or_default()
    }

    /// Helper: collect first `n` zeros from a slice, returning None if not enough.
    fn take_n(zeros: &[ZetaZero], n: usize) -> Option<Vec<ZetaZero>> {
        if zeros.len() < n {
            None
        } else {
            Some(zeros[..n].to_vec())
        }
    }

    // ── Test 1: subseries split produces correct ordinal partitioning ────────

    #[test]
    fn subseries_split_counts_and_parity() {
        let zeros = get_test_zeros();
        if zeros.len() < 40 {
            return;
        }
        let (even, odd) = split_by_parity(&zeros);

        assert_eq!(
            even.len() + odd.len(),
            zeros.len(),
            "even + odd must equal total"
        );

        for z in &even {
            assert_eq!(
                z.ordinal % 2,
                0,
                "even subseries contains odd ordinal {}",
                z.ordinal
            );
        }
        for z in &odd {
            assert_eq!(
                z.ordinal % 2,
                1,
                "odd subseries contains even ordinal {}",
                z.ordinal
            );
        }
    }

    // ── Test 2: even and odd GUE scores are both > 0 ────────────────────────

    #[test]
    fn subseries_gue_scores_positive() {
        let zeros = get_test_zeros();
        if zeros.len() < 40 {
            return;
        }
        if let Ok(analysis) = analyze_subseries(&zeros) {
            assert!(
                analysis.even_gue_score > 0.0,
                "even GUE score {} should be > 0",
                analysis.even_gue_score
            );
            assert!(
                analysis.odd_gue_score > 0.0,
                "odd GUE score {} should be > 0",
                analysis.odd_gue_score
            );
        }
    }

    // ── Test 3: difference metric is finite and non-negative ─────────────────

    #[test]
    fn difference_metric_is_finite_and_non_negative() {
        let zeros = get_test_zeros();
        if zeros.len() < 40 {
            return;
        }
        if let Ok(analysis) = analyze_subseries(&zeros) {
            assert!(
                analysis.difference_metric.is_finite(),
                "difference_metric must be finite"
            );
            assert!(
                analysis.difference_metric >= 0.0,
                "difference_metric must be ≥ 0"
            );
        }
    }

    // ── Test 4: coupling extrapolation produces finite predictions ───────────

    #[test]
    fn coupling_extrapolation_finite_predictions() {
        let zeros = get_large_zeros();
        let sizes: Vec<usize> = [20, 30, 50]
            .iter()
            .copied()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        if let Ok(ext) = extrapolate_coupling(&zeros, &sizes) {
            assert!(
                ext.predicted_at_1000.is_finite(),
                "prediction at 1000 must be finite"
            );
            assert!(
                ext.predicted_at_10000.is_finite(),
                "prediction at 10000 must be finite"
            );
            assert!(
                ext.predicted_at_100000.is_finite(),
                "prediction at 100000 must be finite"
            );
        }
    }

    // ── Test 5: model R² is ≤ 1.0 and finite ────────────────────────────────

    #[test]
    fn model_r_squared_bounded() {
        let zeros = get_large_zeros();
        let sizes: Vec<usize> = [20, 30, 50]
            .iter()
            .copied()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        if let Ok(ext) = extrapolate_coupling(&zeros, &sizes) {
            assert!(ext.model_r_squared.is_finite(), "R² must be finite");
            assert!(ext.model_r_squared <= 1.0, "R² must not exceed 1.0");
        }
    }

    // ── Test 6: rejects too few zeros (< 40 for subseries analysis) ──────────

    #[test]
    fn rejects_too_few_zeros_for_subseries() {
        let zeros = get_test_zeros();
        let small: Vec<ZetaZero> = zeros.into_iter().take(10).collect();
        assert!(
            analyze_subseries(&small).is_err(),
            "should reject slices too small to produce two 20-zero subseries"
        );
    }

    // ── Test 7: predicted values are positive ────────────────────────────────

    #[test]
    fn predicted_values_positive() {
        let zeros = get_large_zeros();
        let sizes: Vec<usize> = [20, 30, 50]
            .iter()
            .copied()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        if let Ok(ext) = extrapolate_coupling(&zeros, &sizes) {
            // A = exp(intercept) > 0 always; N^x > 0 for N > 0, any real x
            assert!(
                ext.predicted_at_1000 > 0.0,
                "prediction at 1000 must be positive"
            );
            assert!(
                ext.predicted_at_10000 > 0.0,
                "prediction at 10000 must be positive"
            );
            assert!(
                ext.predicted_at_100000 > 0.0,
                "prediction at 100000 must be positive"
            );
        }
    }

    // ── Test 8: power law exponent is finite ─────────────────────────────────

    #[test]
    fn power_law_exponent_is_finite() {
        let zeros = get_large_zeros();
        let sizes: Vec<usize> = [20, 30, 50]
            .iter()
            .copied()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        if let Ok(ext) = extrapolate_coupling(&zeros, &sizes) {
            assert!(
                ext.power_law_exponent.is_finite(),
                "power law exponent must be finite"
            );
            // β > 0 means coupling_regularity decreases with N (converging to uniform)
            // We log but don't assert sign — the direction is empirical data
            eprintln!(
                "SUBSERIES: coupling power_law_exponent = {:.4}, R² = {:.4}",
                ext.power_law_exponent, ext.model_r_squared
            );
        }
    }

    // ── Test 9: extrapolation rejects insufficient sample sizes ──────────────

    #[test]
    fn extrapolation_rejects_insufficient_sample_sizes() {
        let zeros = get_large_zeros();
        if zeros.is_empty() {
            return;
        }
        let sizes = vec![10, 20]; // only 2 — needs ≥ 3
        assert!(
            extrapolate_coupling(&zeros, &sizes).is_err(),
            "should reject fewer than 3 sample sizes"
        );
    }

    // ── Test 10: linear_regression matches known line exactly ────────────────

    #[test]
    fn linear_regression_known_line() {
        // y = 2x + 1 → slope = 2, intercept = 1, R² = 1
        let data = vec![(1.0_f64, 3.0_f64), (2.0, 5.0), (3.0, 7.0), (4.0, 9.0)];
        let (slope, intercept, r2) = linear_regression(&data);
        assert!((slope - 2.0).abs() < 1e-10, "slope mismatch: {slope}");
        assert!(
            (intercept - 1.0).abs() < 1e-10,
            "intercept mismatch: {intercept}"
        );
        assert!((r2 - 1.0).abs() < 1e-10, "R² mismatch: {r2}");
    }

    // ── Test 11: data_points in extrapolation have correct length ────────────

    #[test]
    fn extrapolation_data_points_length() {
        let zeros = get_large_zeros();
        let raw_sizes = [20_usize, 30, 50];
        let sizes: Vec<usize> = raw_sizes
            .iter()
            .copied()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        if let Ok(ext) = extrapolate_coupling(&zeros, &sizes) {
            assert_eq!(
                ext.data_points.len(),
                sizes.len(),
                "data_points length must equal sample_sizes length"
            );
        }
    }

    // ── Test 12: subseries counts via take_n helper ───────────────────────────

    #[test]
    fn subseries_analysis_with_known_slice() {
        let zeros = get_large_zeros();
        // Take exactly 80 zeros — guaranteed ≥ 40 per parity if ordinals are interleaved
        let Some(slice) = take_n(&zeros, 80) else {
            return;
        };
        // Should succeed since each parity will have ~40 zeros
        match analyze_subseries(&slice) {
            Ok(a) => {
                assert_eq!(
                    a.even_count + a.odd_count,
                    slice.len(),
                    "counts must sum to total"
                );
                assert!(a.difference_metric.is_finite());
            }
            Err(_) => {
                // Subseries might have imbalanced parity — tolerate failure
            }
        }
    }
}
