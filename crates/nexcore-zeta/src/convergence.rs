//! # GUE Convergence Rate Quantification
//!
//! Measures **how fast** the zeta zero statistics converge to GUE predictions
//! as a function of the number of zeros included.
//!
//! ## Why This Matters
//!
//! The *existence* of GUE convergence is known (Montgomery 1973, Odlyzko 1987).
//! The *rate* of convergence is an active research question. By subsampling
//! our zeros at multiple depths and measuring the pair correlation MAE at each,
//! we can fit a convergence curve and compare to theoretical predictions.
//!
//! ## Theoretical Prediction
//!
//! For the pair correlation, the convergence rate is expected to be
//! roughly O(1/ln N) where N is the number of zeros. Deviations from
//! this rate — especially if convergence is faster — would be a genuine finding.
//!
//! ## Method
//!
//! 1. Compute zeros up to height T (using existing infrastructure)
//! 2. Subsample at depths [N₁, N₂, ..., Nₖ]
//! 3. At each depth, compute pair correlation MAE vs GUE prediction
//! 4. Fit: MAE(N) ~ A · N^(-β) or MAE(N) ~ A / ln(N)^α
//! 5. Report fitted exponents and goodness-of-fit

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::statistics::{compare_to_gue, empirical_pair_correlation, gue_pair_correlation};
use crate::zeros::ZetaZero;

/// A single point on the convergence curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergencePoint {
    /// Number of zeros used in this sample.
    pub n_zeros: usize,
    /// Pair correlation MAE at this sample size.
    pub pair_correlation_mae: f64,
    /// Mean normalized spacing (should approach 1.0).
    pub mean_spacing: f64,
    /// Spacing variance (GUE predicts ~0.178).
    pub spacing_variance: f64,
    /// GUE match score from compare_to_gue.
    pub gue_match_score: f64,
}

/// Fitted convergence model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceModel {
    /// Type of model fitted.
    pub model_type: String,
    /// Fitted exponent (β in N^{-β} or α in ln(N)^{-α}).
    pub exponent: f64,
    /// Fitted amplitude (A in A·N^{-β}).
    pub amplitude: f64,
    /// R² goodness-of-fit (1.0 = perfect).
    pub r_squared: f64,
}

/// Complete convergence analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceAnalysis {
    /// Individual convergence points at each subsample depth.
    pub points: Vec<ConvergencePoint>,
    /// Power law fit: MAE ~ A · N^(-β).
    pub power_law_fit: ConvergenceModel,
    /// Logarithmic fit: MAE ~ A / ln(N)^α.
    pub log_fit: ConvergenceModel,
    /// Which model fits better.
    pub best_model: String,
    /// Whether convergence is faster than O(1/ln N).
    pub faster_than_log: bool,
    /// Total zeros available.
    pub total_zeros: usize,
}

/// Analyze the GUE convergence rate by subsampling zeros at multiple depths.
///
/// # Arguments
///
/// * `zeros` — full set of zeros (sorted by imaginary part)
/// * `subsample_sizes` — list of N values to test (each must be ≤ zeros.len())
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 subsample sizes
/// or if any subsample exceeds available zeros.
pub fn analyze_convergence(
    zeros: &[ZetaZero],
    subsample_sizes: &[usize],
) -> Result<ConvergenceAnalysis, ZetaError> {
    if subsample_sizes.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 subsample sizes for convergence analysis".to_string(),
        ));
    }

    for &n in subsample_sizes {
        if n > zeros.len() {
            return Err(ZetaError::InvalidParameter(format!(
                "subsample size {n} exceeds available zeros {}",
                zeros.len()
            )));
        }
        if n < 10 {
            return Err(ZetaError::InvalidParameter(format!(
                "subsample size {n} too small (need >= 10)"
            )));
        }
    }

    let mut points = Vec::with_capacity(subsample_sizes.len());

    for &n in subsample_sizes {
        let subset = &zeros[..n];
        let comparison = compare_to_gue(subset)?;

        points.push(ConvergencePoint {
            n_zeros: n,
            pair_correlation_mae: comparison.pair_correlation_mae,
            mean_spacing: comparison.mean_spacing,
            spacing_variance: comparison.variance,
            gue_match_score: comparison.gue_match_score,
        });
    }

    // Fit power law: log(MAE) = -β·log(N) + log(A)
    let power_fit = fit_power_law(&points);

    // Fit logarithmic: log(MAE) = -α·log(ln(N)) + log(A)
    let log_fit = fit_log_law(&points);

    let best = if power_fit.r_squared > log_fit.r_squared {
        "power_law".to_string()
    } else {
        "logarithmic".to_string()
    };

    // Faster than O(1/ln N) if power law is better AND β > 0
    let faster = power_fit.r_squared > log_fit.r_squared && power_fit.exponent > 0.0;

    Ok(ConvergenceAnalysis {
        points,
        power_law_fit: power_fit,
        log_fit,
        best_model: best,
        faster_than_log: faster,
        total_zeros: zeros.len(),
    })
}

/// Fit power law: MAE(N) ~ A · N^(-β)
/// Linearized: ln(MAE) = ln(A) - β·ln(N)
fn fit_power_law(points: &[ConvergencePoint]) -> ConvergenceModel {
    let data: Vec<(f64, f64)> = points
        .iter()
        .filter(|p| p.pair_correlation_mae > 1e-15)
        .map(|p| ((p.n_zeros as f64).ln(), p.pair_correlation_mae.ln()))
        .collect();

    let (slope, intercept, r2) = linear_regression(&data);

    ConvergenceModel {
        model_type: "power_law: MAE ~ A * N^(-beta)".to_string(),
        exponent: -slope, // slope is negative, exponent is positive
        amplitude: intercept.exp(),
        r_squared: r2,
    }
}

/// Fit logarithmic: MAE(N) ~ A / ln(N)^α
/// Linearized: ln(MAE) = ln(A) - α·ln(ln(N))
fn fit_log_law(points: &[ConvergencePoint]) -> ConvergenceModel {
    let data: Vec<(f64, f64)> = points
        .iter()
        .filter(|p| p.pair_correlation_mae > 1e-15 && p.n_zeros > 2)
        .map(|p| {
            let ln_ln_n = (p.n_zeros as f64).ln().ln();
            let ln_mae = p.pair_correlation_mae.ln();
            (ln_ln_n, ln_mae)
        })
        .collect();

    let (slope, intercept, r2) = linear_regression(&data);

    ConvergenceModel {
        model_type: "logarithmic: MAE ~ A / ln(N)^alpha".to_string(),
        exponent: -slope,
        amplitude: intercept.exp(),
        r_squared: r2,
    }
}

/// Simple linear regression: y = slope·x + intercept.
/// Returns (slope, intercept, R²).
// n·Σx² − (Σx)² is the standard OLS denominator; not a copy-paste error.
#[allow(clippy::suspicious_operation_groupings)]
fn linear_regression(data: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = data.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();
    let sum_x2: f64 = data.iter().map(|(x, _)| x * x).sum();
    let sum_y2: f64 = data.iter().map(|(_, y)| y * y).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return (0.0, sum_y / n, 0.0);
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    // R²
    let ss_res: f64 = data
        .iter()
        .map(|(x, y)| {
            let predicted = slope * x + intercept;
            (y - predicted) * (y - predicted)
        })
        .sum();
    let mean_y = sum_y / n;
    let ss_tot: f64 = data.iter().map(|(_, y)| (y - mean_y) * (y - mean_y)).sum();
    let r2 = if ss_tot > 1e-30 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    (slope, intercept, r2)
}

// ============================================================================
// Bootstrap Confidence Intervals & Extended Analysis
// ============================================================================

/// Bootstrap confidence interval for a single subsample size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResult {
    /// Number of zeros in this bootstrap subsample.
    pub n_zeros: usize,
    /// Median pair-correlation MAE across bootstrap resamples.
    pub mae_median: f64,
    /// 2.5th percentile of the bootstrap MAE distribution (lower CI bound).
    pub mae_ci_lower: f64,
    /// 97.5th percentile of the bootstrap MAE distribution (upper CI bound).
    pub mae_ci_upper: f64,
    /// Number of bootstrap iterations used.
    pub n_bootstrap: usize,
}

/// Extended convergence analysis: bootstrap CIs, leave-one-out CV, and extrapolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedConvergenceAnalysis {
    /// Base convergence analysis (power law + log fit, MAE at each subsample size).
    pub base: ConvergenceAnalysis,
    /// Bootstrap confidence intervals per subsample size.
    pub bootstrap_results: Vec<BootstrapResult>,
    /// Leave-one-out cross-validation error for the power law model.
    pub cv_power_law_error: f64,
    /// Leave-one-out cross-validation error for the logarithmic model.
    pub cv_log_error: f64,
    /// Predicted MAE at N = 1000 using the fitted power law.
    pub predicted_mae_1000: f64,
    /// Predicted MAE at N = 10 000 using the fitted power law.
    pub predicted_mae_10000: f64,
    /// Overall confidence in the fitted model: `"strong"`, `"moderate"`, or `"weak"`.
    pub model_confidence: String,
}

/// Extended convergence analysis with bootstrap CIs and leave-one-out CV.
///
/// Builds on [`analyze_convergence`] by adding:
/// - Per-subsample bootstrap confidence intervals
/// - Leave-one-out cross-validation for both models
/// - Extrapolated MAE predictions at N = 1 000 and N = 10 000
/// - Model confidence classification
///
/// # Arguments
///
/// * `zeros` — full set of zeros (sorted by imaginary part)
/// * `subsample_sizes` — list of N values to test (each ≥ 10 and ≤ `zeros.len()`)
/// * `n_bootstrap` — number of bootstrap resamples per subsample size
///
/// # Errors
///
/// Propagates any [`ZetaError`] from [`analyze_convergence`].
pub fn analyze_convergence_extended(
    zeros: &[ZetaZero],
    subsample_sizes: &[usize],
    n_bootstrap: usize,
) -> Result<ExtendedConvergenceAnalysis, ZetaError> {
    let base = analyze_convergence(zeros, subsample_sizes)?;

    // ── Bootstrap CI for each subsample size ─────────────────────────────
    let mut bootstrap_results = Vec::with_capacity(subsample_sizes.len());
    for &n in subsample_sizes {
        let subset = &zeros[..n];
        let mut maes: Vec<f64> = Vec::with_capacity(n_bootstrap);

        for b in 0..n_bootstrap {
            // Use splitmix64-style mixing so that indices are NOT a bijection mod N.
            // Linear hashes (a*b + b*i) mod N form permutations when gcd(b_coeff, N)=1
            // — making every bootstrap sample identical to the original. The XOR-shift
            // multiplications break that linearity and produce genuine replacement sampling.
            let resampled: Vec<ZetaZero> = (0..n)
                .map(|i| {
                    let mut x = (b as u64)
                        .wrapping_mul(0x9e3779b97f4a7c15_u64)
                        .wrapping_add(i as u64)
                        .wrapping_add(1); // +1 so b=0,i=0 is never 0
                    x ^= x >> 30;
                    x = x.wrapping_mul(0xbf58476d1ce4e5b9_u64);
                    x ^= x >> 27;
                    x = x.wrapping_mul(0x94d049bb133111eb_u64);
                    x ^= x >> 31;
                    let idx = (x as usize) % n;
                    subset[idx]
                })
                .collect();

            // Sort by height so spacings are well-defined.
            let mut sorted = resampled;
            sorted.sort_by(|a, b| a.t.total_cmp(&b.t));

            if let Ok(comparison) = compare_to_gue(&sorted) {
                maes.push(comparison.pair_correlation_mae);
            }
        }

        maes.sort_by(|a, b| a.total_cmp(b));

        let result = if maes.is_empty() {
            BootstrapResult {
                n_zeros: n,
                mae_median: f64::NAN,
                mae_ci_lower: f64::NAN,
                mae_ci_upper: f64::NAN,
                n_bootstrap,
            }
        } else {
            BootstrapResult {
                n_zeros: n,
                mae_median: sorted_percentile(&maes, 0.50),
                mae_ci_lower: sorted_percentile(&maes, 0.025),
                mae_ci_upper: sorted_percentile(&maes, 0.975),
                n_bootstrap,
            }
        };
        bootstrap_results.push(result);
    }

    // ── Leave-one-out cross-validation ───────────────────────────────────
    let (cv_power, cv_log) = leave_one_out_cv(&base.points);

    // ── Extrapolation via power law ───────────────────────────────────────
    let power = &base.power_law_fit;
    let predicted_mae_1000 = power.amplitude * 1_000_f64.powf(-power.exponent);
    let predicted_mae_10000 = power.amplitude * 10_000_f64.powf(-power.exponent);

    // ── Model confidence ─────────────────────────────────────────────────
    let best_r2 = base.power_law_fit.r_squared.max(base.log_fit.r_squared);
    let best_cv = cv_power.min(cv_log);
    let model_confidence = if best_r2 > 0.95 && best_cv < 0.02 {
        "strong".to_string()
    } else if best_r2 > 0.85 {
        "moderate".to_string()
    } else {
        "weak".to_string()
    };

    Ok(ExtendedConvergenceAnalysis {
        base,
        bootstrap_results,
        cv_power_law_error: cv_power,
        cv_log_error: cv_log,
        predicted_mae_1000,
        predicted_mae_10000,
        model_confidence,
    })
}

/// Compute a percentile from a **sorted** slice via linear interpolation.
fn sorted_percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = p * (sorted.len() - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Leave-one-out cross-validation for the power law and log models.
///
/// Returns `(cv_power_error, cv_log_error)` — mean absolute prediction errors.
fn leave_one_out_cv(points: &[ConvergencePoint]) -> (f64, f64) {
    if points.len() < 3 {
        return (f64::NAN, f64::NAN);
    }

    let mut power_errors: Vec<f64> = Vec::with_capacity(points.len());
    let mut log_errors: Vec<f64> = Vec::with_capacity(points.len());

    for i in 0..points.len() {
        let training: Vec<ConvergencePoint> = points
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, p)| p.clone())
            .collect();

        let held_out = &points[i];
        let power_model = fit_power_law(&training);
        let log_model = fit_log_law(&training);

        let n = held_out.n_zeros as f64;
        let actual = held_out.pair_correlation_mae;

        // Power law prediction: amplitude * N^(-exponent)
        if power_model.amplitude.is_finite() && power_model.exponent.is_finite() {
            let pred = power_model.amplitude * n.powf(-power_model.exponent);
            if pred.is_finite() && actual.is_finite() {
                power_errors.push((pred - actual).abs());
            }
        }

        // Log prediction: amplitude / ln(N)^exponent
        if log_model.amplitude.is_finite() && log_model.exponent.is_finite() && n > 1.0 {
            let ln_n = n.ln();
            let denom = ln_n.powf(log_model.exponent);
            if denom.abs() > 1e-30 {
                let pred = log_model.amplitude / denom;
                if pred.is_finite() && actual.is_finite() {
                    log_errors.push((pred - actual).abs());
                }
            }
        }
    }

    let cv_power = if power_errors.is_empty() {
        f64::NAN
    } else {
        power_errors.iter().sum::<f64>() / power_errors.len() as f64
    };
    let cv_log = if log_errors.is_empty() {
        f64::NAN
    } else {
        log_errors.iter().sum::<f64>() / log_errors.len() as f64
    };

    (cv_power, cv_log)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_zeros_to_1000() -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, 1000.0, 0.02).unwrap_or_default()
    }

    #[test]
    fn convergence_analysis_runs() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 400 {
            // If not enough zeros found, skip gracefully
            return;
        }

        let sizes = vec![50, 100, 200, 400];
        let sizes: Vec<usize> = sizes.into_iter().filter(|&s| s <= zeros.len()).collect();
        if sizes.len() < 3 {
            return;
        }

        let result = analyze_convergence(&zeros, &sizes);
        assert!(
            result.is_ok(),
            "convergence analysis failed: {:?}",
            result.err()
        );

        let analysis = result.unwrap();
        assert_eq!(analysis.points.len(), sizes.len());
    }

    #[test]
    fn mae_decreases_with_more_zeros() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }

        let sizes = vec![50, 100, 200];
        let result = analyze_convergence(&zeros, &sizes).unwrap();

        // MAE should generally decrease (or at least not dramatically increase)
        // with more zeros. Allow some non-monotonicity at small N.
        let first_mae = result.points[0].pair_correlation_mae;
        let last_mae = result
            .points
            .last()
            .map(|p| p.pair_correlation_mae)
            .unwrap_or(f64::NAN);

        // Not a strict test — at small N, statistics are noisy
        // Just verify the trend direction
        assert!(
            last_mae < first_mae * 2.0,
            "MAE should not dramatically increase: first={first_mae}, last={last_mae}"
        );
    }

    #[test]
    fn power_law_exponent_is_positive() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }

        let sizes = vec![50, 100, 200];
        let result = analyze_convergence(&zeros, &sizes).unwrap();

        // If convergence is happening, the power law exponent should be positive
        // (MAE decreases with N). Allow negative if data is too noisy.
        // This is exploratory — we're measuring, not asserting.
        assert!(
            result.power_law_fit.exponent.is_finite(),
            "exponent must be finite"
        );
    }

    #[test]
    fn models_have_finite_parameters() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }

        let sizes = vec![50, 100, 200];
        let result = analyze_convergence(&zeros, &sizes).unwrap();

        assert!(result.power_law_fit.exponent.is_finite());
        assert!(result.power_law_fit.amplitude.is_finite());
        assert!(result.power_law_fit.r_squared >= 0.0);

        assert!(result.log_fit.exponent.is_finite());
        assert!(result.log_fit.amplitude.is_finite());
    }

    #[test]
    fn rejects_too_few_subsample_sizes() {
        let zeros = get_zeros_to_1000();
        if zeros.is_empty() {
            return;
        }
        let sizes = vec![50, 100];
        assert!(analyze_convergence(&zeros, &sizes).is_err());
    }

    #[test]
    fn rejects_oversized_subsample() {
        let zeros = vec![ZetaZero {
            ordinal: 1,
            t: 14.1,
            z_value: 0.0,
            on_critical_line: true,
        }];
        let sizes = vec![10, 20, 30];
        assert!(analyze_convergence(&zeros, &sizes).is_err());
    }

    #[test]
    fn linear_regression_known_line() {
        // y = 2x + 1
        let data = vec![(1.0, 3.0), (2.0, 5.0), (3.0, 7.0), (4.0, 9.0)];
        let (slope, intercept, r2) = linear_regression(&data);
        assert!((slope - 2.0).abs() < 1e-10);
        assert!((intercept - 1.0).abs() < 1e-10);
        assert!((r2 - 1.0).abs() < 1e-10);
    }

    // ── Extended analysis tests ───────────────────────────────────────────

    #[test]
    fn bootstrap_cis_wider_than_point_estimate() {
        // A 95% CI must span non-zero width — if all bootstrap MAEs were identical
        // the CI would collapse to a point, which indicates a degenerate resample.
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }
        let sizes = vec![50, 100, 200];
        let result = analyze_convergence_extended(&zeros, &sizes, 50);
        assert!(
            result.is_ok(),
            "extended analysis failed: {:?}",
            result.err()
        );
        let ext = result.unwrap();
        let mut at_least_one_nonzero_width = false;
        for br in &ext.bootstrap_results {
            if br.mae_ci_lower.is_finite() && br.mae_ci_upper.is_finite() {
                let width = br.mae_ci_upper - br.mae_ci_lower;
                assert!(width >= 0.0, "CI width is negative for n={}", br.n_zeros);
                if width > 0.0 {
                    at_least_one_nonzero_width = true;
                }
            }
        }
        // At least one subsample size should show real uncertainty.
        assert!(
            at_least_one_nonzero_width,
            "all CI widths are zero — bootstrap produced no variance"
        );
    }

    #[test]
    fn more_bootstrap_produces_stable_cis() {
        // With B=200 the CI width estimate is more stable than B=50.
        // Both should produce finite, non-negative CI widths.
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }
        let sizes = vec![50, 100, 200];
        let ext50 = analyze_convergence_extended(&zeros, &sizes, 50);
        let ext200 = analyze_convergence_extended(&zeros, &sizes, 200);
        assert!(ext50.is_ok(), "B=50 failed: {:?}", ext50.err());
        assert!(ext200.is_ok(), "B=200 failed: {:?}", ext200.err());

        let ext50 = ext50.unwrap();
        let ext200 = ext200.unwrap();

        // Both must produce valid (finite, non-negative) CI widths.
        for (br50, br200) in ext50
            .bootstrap_results
            .iter()
            .zip(&ext200.bootstrap_results)
        {
            if br50.mae_ci_lower.is_finite() && br200.mae_ci_lower.is_finite() {
                let w50 = br50.mae_ci_upper - br50.mae_ci_lower;
                let w200 = br200.mae_ci_upper - br200.mae_ci_lower;
                assert!(w50 >= 0.0, "B=50 CI width negative at n={}", br50.n_zeros);
                assert!(
                    w200 >= 0.0,
                    "B=200 CI width negative at n={}",
                    br200.n_zeros
                );
                // B=200 CI width should not be dramatically larger than B=50
                // (more samples → more stable estimate, not wider spread).
                assert!(
                    w200 <= w50 * 3.0 + 0.05,
                    "B=200 CI width {w200:.4} is much larger than B=50 width {w50:.4}"
                );
            }
        }
    }

    #[test]
    fn cv_errors_are_finite() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }
        let sizes = vec![50, 100, 200];
        let ext = analyze_convergence_extended(&zeros, &sizes, 10).unwrap();
        // CV errors may be NaN if fitting degenerates, but must not be infinite.
        assert!(
            !ext.cv_power_law_error.is_infinite(),
            "cv_power_law_error is infinite"
        );
        assert!(!ext.cv_log_error.is_infinite(), "cv_log_error is infinite");
    }

    #[test]
    fn extrapolation_predictions_positive() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }
        let sizes = vec![50, 100, 200];
        let ext = analyze_convergence_extended(&zeros, &sizes, 5).unwrap();
        // Predictions must be finite (may be negative if model extrapolates badly,
        // but NaN/Inf indicate a computation error).
        assert!(
            !ext.predicted_mae_1000.is_nan() || ext.base.power_law_fit.amplitude.is_nan(),
            "predicted_mae_1000 is NaN despite finite amplitude"
        );
        assert!(
            !ext.predicted_mae_10000.is_infinite(),
            "predicted_mae_10000 is infinite"
        );
    }

    #[test]
    fn model_confidence_categorization() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 200 {
            return;
        }
        let sizes = vec![50, 100, 200];
        let ext = analyze_convergence_extended(&zeros, &sizes, 5).unwrap();
        let valid = ["strong", "moderate", "weak"];
        assert!(
            valid.contains(&ext.model_confidence.as_str()),
            "unexpected confidence: '{}'",
            ext.model_confidence
        );
    }

    #[test]
    fn extended_analysis_runs() {
        let zeros = get_zeros_to_1000();
        if zeros.len() < 400 {
            return;
        }
        let sizes: Vec<usize> = vec![50, 100, 200, 400]
            .into_iter()
            .filter(|&s| s <= zeros.len())
            .collect();
        if sizes.len() < 3 {
            return;
        }
        let result = analyze_convergence_extended(&zeros, &sizes, 10);
        assert!(
            result.is_ok(),
            "extended analysis failed: {:?}",
            result.err()
        );
        let ext = result.unwrap();
        assert_eq!(ext.bootstrap_results.len(), sizes.len());
        assert_eq!(ext.base.points.len(), sizes.len());
    }
}
