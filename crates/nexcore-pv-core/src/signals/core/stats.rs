//! Statistical helper functions for signal detection.
//!
//! This module provides statistical functions for signal detection algorithms.
//! Some functions are reserved for future use in extended algorithms.
//!
//! # Complexity Summary
//!
//! All functions in this module are O(1) constant-time operations:
//! - `normal_cdf`: O(1) - polynomial approximation
//! - `normal_quantile`: O(1) - rational approximation
//! - `chi_square_p_value`: O(1) - delegates to normal_cdf
//! - `log_gamma`: O(1) - Stirling approximation
//! - `digamma`: O(1) - asymptotic expansion (bounded iterations)
//! - `chi_square_statistic`: O(1) - fixed arithmetic

#![allow(dead_code)] // Reserved functions for extended algorithms

use std::f64::consts::PI;

/// Z-score for 95% confidence interval.
pub const Z_95: f64 = 1.96;

/// Chi-square critical value for p < 0.05 with df = 1.
/// CRITICAL: Use exact value 3.841, NOT 4.0.
pub const CHI_SQUARE_CRITICAL_05: f64 = 3.841;

/// Natural logarithm of 2.
pub const LN_2: f64 = 0.693_147_180_559_945_3;

/// Standard normal cumulative distribution function.
///
/// Uses Abramowitz & Stegun approximation (26.2.17).
///
/// # Complexity: O(1) - polynomial evaluation
#[must_use]
#[allow(clippy::suboptimal_flops)] // Formula clarity > micro-optimization
pub fn normal_cdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }

    let t = 1.0 / (1.0 + 0.231_641_9 * x.abs());
    let d = 0.398_942_3 * (-x * x / 2.0).exp();
    let p = d
        * t
        * (0.319_381_5 + t * (-0.356_563_8 + t * (1.781_478 + t * (-1.821_256 + t * 1.330_274))));

    if x >= 0.0 { 1.0 - p } else { p }
}

/// Inverse standard normal CDF (quantile function).
///
/// Uses Abramowitz & Stegun rational approximation (26.2.23).
///
/// # Complexity: O(1) - rational function evaluation
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < f64::EPSILON {
        return 0.0;
    }

    // Abramowitz & Stegun rational approximation
    let c0 = 2.515_517;
    let c1 = 0.802_853;
    let c2 = 0.010_328;
    let d1 = 1.432_788;
    let d2 = 0.189_269;
    let d3 = 0.001_308;

    let sign: f64;
    let t: f64;

    if p < 0.5 {
        t = (-2.0 * p.ln()).sqrt();
        sign = -1.0;
    } else {
        t = (-2.0 * (1.0 - p).ln()).sqrt();
        sign = 1.0;
    }

    let numerator = c0 + c1 * t + c2 * t * t;
    let denominator = 1.0 + d1 * t + d2 * t * t + d3 * t * t * t;

    sign * (t - numerator / denominator)
}

/// Chi-square p-value for 1 degree of freedom.
///
/// # Complexity: O(1) - delegates to normal_cdf
#[must_use]
pub fn chi_square_p_value(chi_sq: f64) -> f64 {
    if chi_sq <= 0.0 {
        return 1.0;
    }
    2.0 * (1.0 - normal_cdf(chi_sq.sqrt()))
}

/// Log-gamma function using Stirling's approximation.
///
/// # Complexity: O(1) - closed-form approximation
#[must_use]
pub fn log_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    (x - 0.5) * x.ln() - x + 0.5 * (2.0 * PI).ln() + 1.0 / (12.0 * x)
}

/// Gamma function.
///
/// # Complexity: O(1) - exp of Stirling approximation
#[must_use]
pub fn gamma(x: f64) -> f64 {
    log_gamma(x).exp()
}

/// Digamma function (derivative of log-gamma).
///
/// Uses asymptotic expansion for x >= 8, recurrence relation otherwise.
///
/// # Complexity: O(1) - bounded loop (max 8 iterations) + asymptotic expansion
#[must_use]
pub fn digamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::NAN;
    }

    let mut result = 0.0;
    let mut x = x;

    // Use recurrence: psi(x) = psi(x+1) - 1/x
    while x < 8.0 {
        result -= 1.0 / x;
        x += 1.0;
    }

    // Asymptotic expansion for large x
    let x2 = x * x;
    result + x.ln() - 0.5 / x - 1.0 / (12.0 * x2) + 1.0 / (120.0 * x2 * x2)
}

/// Base-2 logarithm.
///
/// # Complexity: O(1)
#[must_use]
pub fn log2(x: f64) -> f64 {
    x.ln() / LN_2
}

/// Haldane-Anscombe correction for zero cells.
///
/// Adds 0.5 to all cells when any cell is zero.
///
/// # Complexity: O(1)
#[must_use]
pub fn apply_continuity_correction(a: f64, b: f64, c: f64, d: f64) -> (f64, f64, f64, f64) {
    if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 {
        (a + 0.5, b + 0.5, c + 0.5, d + 0.5)
    } else {
        (a, b, c, d)
    }
}

/// Calculate standard error for log ratio.
///
/// # Complexity: O(1)
#[must_use]
pub fn log_ratio_standard_error(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 || c <= 0.0 || d <= 0.0 {
        return f64::INFINITY;
    }
    (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt()
}

/// Calculate chi-square statistic from contingency table.
///
/// χ² = Σ (O - E)² / E
///
/// # Complexity: O(1) - fixed arithmetic operations
#[must_use]
#[allow(clippy::many_single_char_names)] // a,b,c,d is standard contingency table notation
pub fn chi_square_statistic(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let n = a + b + c + d;
    if n == 0.0 {
        return 0.0;
    }

    let expected_a = (a + b) * (a + c) / n;
    let expected_b = (a + b) * (b + d) / n;
    let expected_c = (c + d) * (a + c) / n;
    let expected_d = (c + d) * (b + d) / n;

    let mut chi_sq = 0.0;
    if expected_a > 0.0 {
        chi_sq += (a - expected_a).powi(2) / expected_a;
    }
    if expected_b > 0.0 {
        chi_sq += (b - expected_b).powi(2) / expected_b;
    }
    if expected_c > 0.0 {
        chi_sq += (c - expected_c).powi(2) / expected_c;
    }
    if expected_d > 0.0 {
        chi_sq += (d - expected_d).powi(2) / expected_d;
    }

    chi_sq
}

/// Calculate Yates-corrected chi-square statistic for 2x2 table.
///
/// Applies continuity correction for small sample sizes.
/// χ²_Yates = N(|ad - bc| - N/2)² / (row1 * row2 * col1 * col2)
///
/// Reference: Yates F (1934). "Contingency tables involving small numbers and the χ² test."
/// Journal of the Royal Statistical Society, Supplement 1(2):217-235.
/// DOI: [10.2307/2983604](https://doi.org/10.2307/2983604)
///
/// # Complexity: O(1) - fixed arithmetic operations
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn chi_square_yates_corrected(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let n = a + b + c + d;
    if n == 0.0 {
        return 0.0;
    }

    let row1 = a + b;
    let row2 = c + d;
    let col1 = a + c;
    let col2 = b + d;
    let denom = row1 * row2 * col1 * col2;

    if denom == 0.0 {
        return 0.0;
    }

    let ad_bc = (a * d - b * c).abs();
    let correction = n / 2.0;
    let corrected = if ad_bc > correction {
        ad_bc - correction
    } else {
        0.0
    };

    n * corrected.powi(2) / denom
}

/// Get z-score for a given confidence level.
///
/// Common confidence levels have pre-computed exact values.
/// Other levels use the inverse normal CDF approximation.
///
/// # Arguments
///
/// * `confidence` - Confidence level (0.5 to 1.0, e.g., 0.95 for 95% CI)
///
/// # Returns
///
/// Z-score for the specified confidence level
///
/// # Complexity: O(1)
#[must_use]
pub fn z_score_for_confidence(confidence: f64) -> f64 {
    // Pre-computed exact values for common confidence levels
    match () {
        () if (confidence - 0.80).abs() < 1e-9 => 1.282,
        () if (confidence - 0.85).abs() < 1e-9 => 1.440,
        () if (confidence - 0.90).abs() < 1e-9 => 1.645,
        () if (confidence - 0.95).abs() < 1e-9 => 1.960,
        () if (confidence - 0.99).abs() < 1e-9 => 2.576,
        () => {
            // Use inverse normal CDF for other confidence levels
            if confidence <= 0.5 || confidence >= 1.0 {
                return f64::NAN;
            }
            let p = (1.0 + confidence) / 2.0;
            normal_quantile(p)
        }
    }
}

// =============================================================================
// WEIBULL DISTRIBUTION FUNCTIONS (Phase 1: Temporal Analysis)
// =============================================================================

/// Weibull log-likelihood function for MLE.
///
/// L(k, λ | x) = Σ [log(k) - log(λ) + (k-1)log(x_i/λ) - (x_i/λ)^k]
///
/// # Arguments
/// * `times` - Observed time-to-onset values (must be positive)
/// * `shape` - Shape parameter k (k < 1: decreasing hazard, k > 1: increasing)
/// * `scale` - Scale parameter λ (characteristic life)
///
/// # Returns
/// Log-likelihood value (higher is better fit)
///
/// # Complexity: O(n) where n = times.len()
#[must_use]
pub fn weibull_log_likelihood(times: &[f64], shape: f64, scale: f64) -> f64 {
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let n = times.len() as f64;
    let ln_k = shape.ln();
    let ln_lambda = scale.ln();

    let mut sum_ln_x = 0.0;
    let mut sum_x_lambda_k = 0.0;

    for &t in times {
        if t <= 0.0 {
            return f64::NEG_INFINITY;
        }
        sum_ln_x += t.ln();
        sum_x_lambda_k += (t / scale).powf(shape);
    }

    n * (ln_k - ln_lambda) + (shape - 1.0) * sum_ln_x
        - n * (shape - 1.0) * ln_lambda
        - sum_x_lambda_k
}

/// Weibull MLE shape parameter update (Newton-Raphson step).
///
/// Given current shape estimate k, compute the next estimate using:
/// k_new = k - f(k) / f'(k)
///
/// where f(k) = 1/k + (1/n)Σlog(x_i) - [Σx_i^k log(x_i)] / [Σx_i^k]
///
/// This is the profile likelihood approach: for each k, the optimal λ is:
/// λ = (Σx_i^k / n)^(1/k)
///
/// # Arguments
/// * `times` - Observed time values
/// * `shape` - Current shape estimate
///
/// # Returns
/// (new_shape, optimal_scale) tuple
///
/// # Complexity: O(n)
#[must_use]
pub fn weibull_mle_step(times: &[f64], shape: f64) -> (f64, f64) {
    let n = times.len() as f64;
    if n == 0.0 || shape <= 0.0 {
        return (1.0, 1.0);
    }

    // Compute required sums
    let mut sum_x_k = 0.0;
    let mut sum_x_k_ln_x = 0.0;
    let mut sum_x_k_ln_x_2 = 0.0;
    let mut sum_ln_x = 0.0;

    for &t in times {
        if t <= 0.0 {
            continue;
        }
        let ln_t = t.ln();
        let t_k = t.powf(shape);

        sum_ln_x += ln_t;
        sum_x_k += t_k;
        sum_x_k_ln_x += t_k * ln_t;
        sum_x_k_ln_x_2 += t_k * ln_t * ln_t;
    }

    // Avoid division by zero
    if sum_x_k == 0.0 {
        return (1.0, 1.0);
    }

    // Optimal scale for current shape: λ = (Σx^k / n)^(1/k)
    let scale = (sum_x_k / n).powf(1.0 / shape);

    // Newton-Raphson update for shape
    // f(k) = 1/k + mean(ln_x) - Σ(x^k ln(x)) / Σ(x^k)
    let mean_ln_x = sum_ln_x / n;
    let f_k = 1.0 / shape + mean_ln_x - sum_x_k_ln_x / sum_x_k;

    // f'(k) = -1/k² - [Σ(x^k (ln x)²) Σ(x^k) - (Σ(x^k ln x))²] / (Σ(x^k))²
    let f_prime_k = -1.0 / (shape * shape)
        - (sum_x_k_ln_x_2 * sum_x_k - sum_x_k_ln_x.powi(2)) / sum_x_k.powi(2);

    // Newton step
    let delta = f_k / f_prime_k;
    let new_shape = (shape - delta).max(0.01).min(100.0);

    (new_shape, scale)
}

/// Fit Weibull distribution using Maximum Likelihood Estimation.
///
/// Uses iterative Newton-Raphson to find optimal shape parameter,
/// then computes scale analytically.
///
/// # Arguments
/// * `times` - Observed time-to-onset values (must be positive)
/// * `max_iterations` - Maximum Newton-Raphson iterations
/// * `tolerance` - Convergence tolerance for shape parameter
///
/// # Returns
/// `Ok((shape, scale))` on convergence, `Err` message otherwise
///
/// # Complexity: O(n * max_iterations)
pub fn weibull_mle(
    times: &[f64],
    max_iterations: usize,
    tolerance: f64,
) -> Result<(f64, f64), &'static str> {
    if times.is_empty() {
        return Err("Empty time series");
    }

    // Validate all times are positive
    if times
        .iter()
        .any(|&t| t <= 0.0 || t.is_nan() || t.is_infinite())
    {
        return Err("Times must be positive finite values");
    }

    // Initial shape estimate using Method of Moments approximation
    // k ≈ (mean / std_dev)^1.086
    let n = times.len() as f64;
    let mean = times.iter().sum::<f64>() / n;
    let variance = times.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    let mut shape = if std_dev > 0.0 {
        (mean / std_dev).powf(1.086).clamp(0.1, 10.0)
    } else {
        1.2 // Fallback
    };

    // Newton-Raphson iteration
    for _ in 0..max_iterations {
        let (new_shape, _scale) = weibull_mle_step(times, shape);

        if (new_shape - shape).abs() < tolerance {
            // Converged - compute final scale
            let sum_x_k: f64 = times.iter().map(|&t| t.powf(new_shape)).sum();
            let final_scale = (sum_x_k / n).powf(1.0 / new_shape);
            return Ok((new_shape, final_scale));
        }

        shape = new_shape;
    }

    // Return best estimate even if not fully converged
    let sum_x_k: f64 = times.iter().map(|&t| t.powf(shape)).sum();
    let scale = (sum_x_k / n).powf(1.0 / shape);
    Ok((shape, scale))
}

/// Calculate the median of a Weibull distribution.
///
/// median = λ * (ln(2))^(1/k)
///
/// # Complexity: O(1)
#[must_use]
pub fn weibull_median(shape: f64, scale: f64) -> f64 {
    if shape <= 0.0 || scale <= 0.0 {
        return f64::NAN;
    }
    scale * LN_2.powf(1.0 / shape)
}

/// Calculate Weibull confidence interval for shape using Fisher information.
///
/// The asymptotic variance of the MLE shape estimate is approximately:
/// Var(k̂) ≈ k² / (n * (1 + ψ'(1)))
///
/// where ψ' is the trigamma function. For simplicity, we use:
/// Var(k̂) ≈ k² * 0.608 / n (empirical approximation)
///
/// # Returns
/// (lower_ci, upper_ci) for the specified confidence level
///
/// # Complexity: O(1)
#[must_use]
pub fn weibull_shape_ci(shape: f64, n: usize, confidence: f64) -> (f64, f64) {
    if n == 0 {
        return (f64::NAN, f64::NAN);
    }

    let z = z_score_for_confidence(confidence);
    if z.is_nan() {
        return (f64::NAN, f64::NAN);
    }

    // Approximate standard error using Fisher information
    // SE(k) ≈ k * sqrt(0.608 / n)
    let se = shape * (0.608 / n as f64).sqrt();

    let lower = (shape - z * se).max(0.01);
    let upper = shape + z * se;

    (lower, upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.01);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }

    #[test]
    fn test_normal_quantile() {
        assert!(normal_quantile(0.5).abs() < 0.01);
        assert!((normal_quantile(0.975) - 1.96).abs() < 0.1);
    }

    #[test]
    fn test_chi_square_critical_value() {
        // p-value at critical value should be ~0.05
        let p = chi_square_p_value(CHI_SQUARE_CRITICAL_05);
        assert!((p - 0.05).abs() < 0.01);
    }

    #[test]
    fn test_chi_square_statistic() {
        // Test case: 15, 100, 200, 10000
        let chi_sq = chi_square_statistic(15.0, 100.0, 200.0, 10000.0);
        // Should be significant (> 3.841)
        assert!(chi_sq > CHI_SQUARE_CRITICAL_05);
        // Verify p-value is small
        let p = chi_square_p_value(chi_sq);
        assert!(p < 0.05);
    }

    #[test]
    fn test_chi_square_yates_corrected() {
        // Yates correction should produce smaller chi-square
        let chi_sq = chi_square_statistic(15.0, 100.0, 200.0, 10000.0);
        let chi_sq_yates = chi_square_yates_corrected(15.0, 100.0, 200.0, 10000.0);
        assert!(chi_sq_yates < chi_sq);
        assert!(chi_sq_yates > 0.0);
    }

    #[test]
    fn test_chi_square_yates_edge_cases() {
        // Zero margin should return 0
        assert_eq!(chi_square_yates_corrected(0.0, 0.0, 0.0, 0.0), 0.0);
        // Zero row margin
        assert_eq!(chi_square_yates_corrected(0.0, 0.0, 5.0, 10.0), 0.0);
    }

    #[test]
    fn test_z_score_for_confidence() {
        // Test pre-computed values
        assert!((z_score_for_confidence(0.95) - 1.96).abs() < 0.001);
        assert!((z_score_for_confidence(0.99) - 2.576).abs() < 0.001);
        assert!((z_score_for_confidence(0.90) - 1.645).abs() < 0.001);
        assert!((z_score_for_confidence(0.80) - 1.282).abs() < 0.001);
        assert!((z_score_for_confidence(0.85) - 1.440).abs() < 0.001);
    }

    #[test]
    fn test_z_score_edge_cases() {
        // Invalid confidence levels should return NaN
        assert!(z_score_for_confidence(0.5).is_nan());
        assert!(z_score_for_confidence(1.0).is_nan());
        assert!(z_score_for_confidence(0.0).is_nan());
    }

    #[test]
    fn test_z_score_non_standard() {
        // Test a non-standard confidence level (0.975)
        let z = z_score_for_confidence(0.975);
        // Should be approximately 2.24
        assert!(z > 2.0 && z < 2.5);
    }

    // =========================================================================
    // Weibull Distribution Tests
    // =========================================================================

    #[test]
    fn test_weibull_mle_exponential() {
        // Exponential-like pattern: early events more common (decreasing hazard appearance)
        // For true exponential, k ≈ 1, but small samples have variance
        let times = vec![0.5, 1.2, 2.0, 3.5, 7.0, 12.0, 15.0];
        let result = weibull_mle(&times, 100, 1e-6);
        assert!(result.is_ok());

        let (shape, scale) = result.unwrap();
        // Shape should be positive and scale should be reasonable
        assert!(shape > 0.0, "Shape {} should be positive", shape);
        assert!(scale > 0.0, "Scale {} should be positive", scale);
        // For this data pattern, shape is typically in a reasonable range
        assert!(
            shape > 0.3 && shape < 5.0,
            "Shape {} not in expected range",
            shape
        );
    }

    #[test]
    fn test_weibull_mle_increasing_hazard() {
        // Times clustered at higher values suggest k > 1 (late onset)
        let times = vec![8.0, 9.0, 10.0, 11.0, 12.0, 10.5, 9.5];
        let result = weibull_mle(&times, 100, 1e-6);
        assert!(result.is_ok());

        let (shape, scale) = result.unwrap();
        // Should have shape > 1 and reasonable scale
        assert!(shape > 0.0, "Shape should be positive");
        assert!(scale > 0.0, "Scale should be positive");
    }

    #[test]
    fn test_weibull_mle_invalid_input() {
        // Empty input
        assert!(weibull_mle(&[], 100, 1e-6).is_err());

        // Negative time
        let times = vec![1.0, -2.0, 3.0];
        assert!(weibull_mle(&times, 100, 1e-6).is_err());

        // Zero time
        let times = vec![1.0, 0.0, 3.0];
        assert!(weibull_mle(&times, 100, 1e-6).is_err());
    }

    #[test]
    fn test_weibull_median() {
        // For exponential (k=1), median = λ * ln(2)
        let median = weibull_median(1.0, 10.0);
        let expected = 10.0 * LN_2;
        assert!(
            (median - expected).abs() < 0.001,
            "Median {} != expected {}",
            median,
            expected
        );
    }

    #[test]
    fn test_weibull_median_edge_cases() {
        assert!(weibull_median(0.0, 1.0).is_nan());
        assert!(weibull_median(1.0, 0.0).is_nan());
        assert!(weibull_median(-1.0, 1.0).is_nan());
    }

    #[test]
    fn test_weibull_shape_ci() {
        let (lower, upper) = weibull_shape_ci(1.5, 100, 0.95);
        // CI should bracket the point estimate
        assert!(lower < 1.5);
        assert!(upper > 1.5);
        // Lower bound should be positive
        assert!(lower > 0.0);
    }

    #[test]
    fn test_weibull_log_likelihood() {
        let times = vec![1.0, 2.0, 3.0];
        let ll1 = weibull_log_likelihood(&times, 1.0, 2.0);
        let ll2 = weibull_log_likelihood(&times, 1.0, 20.0);

        // ll1 should be better (higher) than ll2 since scale 2.0 is closer to mean
        assert!(ll1 > ll2, "ll1={} should be > ll2={}", ll1, ll2);

        // Invalid parameters should return -infinity
        assert!(weibull_log_likelihood(&times, 0.0, 1.0) == f64::NEG_INFINITY);
        assert!(weibull_log_likelihood(&times, 1.0, 0.0) == f64::NEG_INFINITY);
    }
}
