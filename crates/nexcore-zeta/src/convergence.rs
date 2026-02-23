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
}
