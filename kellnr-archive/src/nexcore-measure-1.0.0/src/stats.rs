//! Statistics: Poisson CI, Bayesian posterior, Welch t-test, OLS regression.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | observations → statistics |
//! | T1: Comparison (κ) | hypothesis testing, p-values |
//! | T1: Boundary (δ) | confidence intervals |
//!
//! Student-t CDF via regularized incomplete beta function (continued fraction).
//! No external stats library.

use crate::error::{MeasureError, MeasureResult};
use crate::types::{BayesianPosterior, PoissonCi, RegressionResult, WelchResult};

// ---------------------------------------------------------------------------
// Poisson confidence interval
// ---------------------------------------------------------------------------

/// Poisson confidence interval via normal approximation.
///
/// `count`: observed events, `exposure`: in KLOC, `alpha`: e.g. 0.05 for 95%.
pub fn poisson_ci(count: usize, exposure: f64, alpha: f64) -> MeasureResult<PoissonCi> {
    if exposure < f64::EPSILON {
        return Err(MeasureError::OutOfRange {
            value: exposure,
            min: 0.0,
            max: f64::MAX,
            context: "exposure must be > 0".into(),
        });
    }
    let rate = count as f64 / exposure;
    let z = z_score(1.0 - alpha / 2.0);
    let se = (rate / exposure).sqrt();
    let lower = (rate - z * se).max(0.0);
    let upper = rate + z * se;
    Ok(PoissonCi {
        rate,
        lower,
        upper,
        alpha,
    })
}

/// Approximate z-score for common quantiles using Abramowitz & Stegun.
fn z_score(p: f64) -> f64 {
    // Rational approximation for normal quantile function
    if p <= 0.0 || p >= 1.0 {
        return 0.0;
    }
    let t = if p < 0.5 {
        (-2.0 * p.ln()).sqrt()
    } else {
        (-2.0 * (1.0 - p).ln()).sqrt()
    };
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    let z = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);
    if p < 0.5 { -z } else { z }
}

// ---------------------------------------------------------------------------
// Bayesian posterior (Gamma-Poisson conjugate)
// ---------------------------------------------------------------------------

/// Gamma-Poisson conjugate posterior.
///
/// Prior: Gamma(alpha_prior, beta_prior).
/// Observation: `observed` events in `exposure` time.
pub fn bayesian_posterior(
    alpha_prior: f64,
    beta_prior: f64,
    observed: usize,
    exposure: f64,
) -> MeasureResult<BayesianPosterior> {
    if alpha_prior <= 0.0 || beta_prior <= 0.0 {
        return Err(MeasureError::OutOfRange {
            value: alpha_prior.min(beta_prior),
            min: 0.0,
            max: f64::MAX,
            context: "prior parameters must be > 0".into(),
        });
    }
    let alpha_post = alpha_prior + observed as f64;
    let beta_post = beta_prior + exposure;
    let mean = alpha_post / beta_post;
    let variance = alpha_post / (beta_post * beta_post);
    Ok(BayesianPosterior {
        mean,
        variance,
        alpha_post,
        beta_post,
    })
}

// ---------------------------------------------------------------------------
// Welch's t-test
// ---------------------------------------------------------------------------

/// Welch's t-test for unequal variances.
pub fn welch_t_test(a: &[f64], b: &[f64]) -> MeasureResult<WelchResult> {
    let na = a.len();
    let nb = b.len();
    if na < 2 || nb < 2 {
        return Err(MeasureError::InsufficientData {
            need: 2,
            got: na.min(nb),
            context: "welch_t_test requires n >= 2 per group".into(),
        });
    }
    let (mean_a, var_a) = mean_var(a);
    let (mean_b, var_b) = mean_var(b);

    let se2_a = var_a / na as f64;
    let se2_b = var_b / nb as f64;
    let se_sum = se2_a + se2_b;

    if se_sum < f64::EPSILON {
        // Zero variance in both groups
        let means_equal = (mean_a - mean_b).abs() < f64::EPSILON;
        if means_equal {
            return Ok(WelchResult {
                t_statistic: 0.0,
                dof: (na + nb - 2) as f64,
                p_value: 1.0,
            });
        }
        // Different means with zero variance = perfectly significant
        return Ok(WelchResult {
            t_statistic: f64::INFINITY,
            dof: (na + nb - 2) as f64,
            p_value: 0.0,
        });
    }

    let t = (mean_a - mean_b) / se_sum.sqrt();
    let dof = welch_dof(se2_a, se2_b, na, nb);
    let p = student_t_p_value(t.abs(), dof);
    Ok(WelchResult {
        t_statistic: t,
        dof,
        p_value: p,
    })
}

/// Welch-Satterthwaite degrees of freedom.
fn welch_dof(se2_a: f64, se2_b: f64, na: usize, nb: usize) -> f64 {
    let num = (se2_a + se2_b).powi(2);
    let denom = se2_a.powi(2) / (na as f64 - 1.0) + se2_b.powi(2) / (nb as f64 - 1.0);
    if denom < f64::EPSILON {
        return (na + nb - 2) as f64;
    }
    num / denom
}

/// Mean and variance of a slice.
fn mean_var(data: &[f64]) -> (f64, f64) {
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, var)
}

// ---------------------------------------------------------------------------
// OLS Linear Regression
// ---------------------------------------------------------------------------

/// Ordinary least squares regression.
pub fn linear_regression(x: &[f64], y: &[f64]) -> MeasureResult<RegressionResult> {
    let n = x.len();
    if n != y.len() {
        return Err(MeasureError::InsufficientData {
            need: n,
            got: y.len(),
            context: "x and y must have equal length".into(),
        });
    }
    if n < 3 {
        return Err(MeasureError::InsufficientData {
            need: 3,
            got: n,
            context: "regression needs n >= 3".into(),
        });
    }

    let (slope, intercept) = ols_coefficients(x, y, n);
    let r_squared = compute_r_squared(x, y, slope, intercept);
    let p_value = regression_p_value(x, y, slope, intercept, n);

    Ok(RegressionResult {
        slope,
        intercept,
        r_squared,
        p_value,
    })
}

/// Compute OLS slope and intercept.
fn ols_coefficients(x: &[f64], y: &[f64], n: usize) -> (f64, f64) {
    let nf = n as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|&xi| xi * xi).sum();

    let denom = nf * sum_x2 - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return (0.0, sum_y / nf);
    }
    let slope = (nf * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / nf;
    (slope, intercept)
}

/// Compute R-squared.
fn compute_r_squared(x: &[f64], y: &[f64], slope: f64, intercept: f64) -> f64 {
    let y_mean = y.iter().sum::<f64>() / y.len() as f64;
    let ss_tot: f64 = y.iter().map(|&yi| (yi - y_mean).powi(2)).sum();
    let ss_res: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (yi - (slope * xi + intercept)).powi(2))
        .sum();
    if ss_tot < f64::EPSILON {
        return 1.0;
    }
    1.0 - ss_res / ss_tot
}

/// Compute p-value for regression slope via t-test.
fn regression_p_value(x: &[f64], y: &[f64], slope: f64, intercept: f64, n: usize) -> f64 {
    let ss_res: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (yi - (slope * xi + intercept)).powi(2))
        .sum();
    let x_mean = x.iter().sum::<f64>() / n as f64;
    let ss_x: f64 = x.iter().map(|&xi| (xi - x_mean).powi(2)).sum();
    if ss_x < f64::EPSILON {
        return 1.0;
    }
    let mse = ss_res / (n as f64 - 2.0);
    let se_slope = (mse / ss_x).sqrt();
    if se_slope < f64::EPSILON {
        return 0.0;
    }
    let t = slope / se_slope;
    student_t_p_value(t.abs(), (n - 2) as f64)
}

// ---------------------------------------------------------------------------
// Student-t CDF via regularized incomplete beta function
// ---------------------------------------------------------------------------

/// Two-tailed p-value from Student-t distribution.
fn student_t_p_value(t_abs: f64, dof: f64) -> f64 {
    let x = dof / (dof + t_abs * t_abs);
    let p_one_tail = 0.5 * regularized_incomplete_beta(dof / 2.0, 0.5, x);
    2.0 * p_one_tail
}

/// Regularized incomplete beta function I_x(a, b) via continued fraction.
fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    // Use symmetry relation when x > (a+1)/(a+b+2)
    let threshold = (a + 1.0) / (a + b + 2.0);
    if x > threshold {
        return 1.0 - regularized_incomplete_beta(b, a, 1.0 - x);
    }
    let ln_prefix = ln_beta_prefix(a, b, x);
    let cf = beta_continued_fraction(a, b, x);
    if cf <= 0.0 || !cf.is_finite() {
        return 0.0;
    }
    let result = (ln_prefix + cf.ln()).exp() / a;
    if !result.is_finite() {
        return 0.0;
    }
    result.clamp(0.0, 1.0)
}

/// Log of x^a * (1-x)^b / B(a,b) prefix.
fn ln_beta_prefix(a: f64, b: f64, x: f64) -> f64 {
    a * x.ln() + b * (1.0 - x).ln() - ln_beta(a, b)
}

/// Log of Beta function via Stirling's approximation of log-gamma.
fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// Lanczos approximation of log-gamma.
fn ln_gamma(x: f64) -> f64 {
    let coeffs = [
        76.180_091_729_471_46,
        -86.505_320_329_416_77,
        24.014_098_240_830_91,
        -1.231_739_572_450_155,
        0.001_208_650_973_866_179,
        -0.000_005_395_239_384_953,
    ];
    let g = 5.0_f64;
    let xx = x - 1.0;
    let mut sum = 0.999_999_999_999_997_1_f64;
    for (i, &c) in coeffs.iter().enumerate() {
        sum += c / (xx + (i as f64) + 1.0);
    }
    let t = xx + g + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (t.ln() * (xx + 0.5)) - t + sum.ln()
}

/// Continued fraction for incomplete beta (Lentz's method).
fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 1e-14;
    let tiny = 1e-30;

    let mut f = 1.0_f64;
    let mut c = 1.0_f64;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    f = d;

    for m in 1..=max_iter {
        let mf = m as f64;
        // Even step: d_{2m}
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + num_even / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        f *= c * d;

        // Odd step: d_{2m+1}
        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + num_odd / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < eps {
            break;
        }
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisson_ci_basic() {
        let ci = poisson_ci(10, 1.0, 0.05).unwrap_or_else(|_| PoissonCi {
            rate: 0.0,
            lower: 0.0,
            upper: 0.0,
            alpha: 0.05,
        });
        assert!((ci.rate - 10.0).abs() < 1e-10);
        assert!(ci.lower < ci.rate);
        assert!(ci.upper > ci.rate);
    }

    #[test]
    fn poisson_ci_zero_exposure_errors() {
        assert!(poisson_ci(5, 0.0, 0.05).is_err());
    }

    #[test]
    fn bayesian_posterior_basic() {
        // Prior Gamma(1,1), observe 10 events in 2.0 exposure
        let post = bayesian_posterior(1.0, 1.0, 10, 2.0).unwrap_or_else(|_| BayesianPosterior {
            mean: 0.0,
            variance: 0.0,
            alpha_post: 0.0,
            beta_post: 0.0,
        });
        assert!((post.alpha_post - 11.0).abs() < 1e-10);
        assert!((post.beta_post - 3.0).abs() < 1e-10);
        assert!((post.mean - 11.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn bayesian_bad_prior_errors() {
        assert!(bayesian_posterior(0.0, 1.0, 5, 1.0).is_err());
        assert!(bayesian_posterior(1.0, -1.0, 5, 1.0).is_err());
    }

    #[test]
    fn welch_identical_samples() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = welch_t_test(&a, &b).unwrap_or_else(|_| WelchResult {
            t_statistic: 999.0,
            dof: 0.0,
            p_value: 0.0,
        });
        assert!(result.p_value > 0.9, "p={} should be ~1.0", result.p_value);
    }

    #[test]
    fn welch_very_different_samples() {
        let a = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let b = vec![100.0, 100.0, 100.0, 100.0, 100.0];
        let result = welch_t_test(&a, &b).unwrap_or_else(|_| WelchResult {
            t_statistic: 0.0,
            dof: 0.0,
            p_value: 1.0,
        });
        assert!(
            result.p_value < 0.001,
            "p={} should be < 0.001",
            result.p_value
        );
    }

    #[test]
    fn welch_insufficient_data() {
        assert!(welch_t_test(&[1.0], &[2.0, 3.0]).is_err());
    }

    #[test]
    fn regression_perfect_linear() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&xi| 2.0 * xi + 1.0).collect();
        let reg = linear_regression(&x, &y).unwrap_or_else(|_| RegressionResult {
            slope: 0.0,
            intercept: 0.0,
            r_squared: 0.0,
            p_value: 1.0,
        });
        assert!((reg.slope - 2.0).abs() < 1e-10);
        assert!((reg.intercept - 1.0).abs() < 1e-10);
        assert!((reg.r_squared - 1.0).abs() < 1e-10);
    }

    #[test]
    fn regression_insufficient_data() {
        assert!(linear_regression(&[1.0, 2.0], &[3.0, 4.0]).is_err());
    }

    #[test]
    fn z_score_95_percentile() {
        let z = z_score(0.975);
        assert!((z - 1.96).abs() < 0.01, "z={} should be ~1.96", z);
    }

    #[test]
    fn student_t_p_value_zero() {
        let p = student_t_p_value(0.0, 10.0);
        assert!((p - 1.0).abs() < 0.01, "p={} for t=0 should be ~1.0", p);
    }

    #[test]
    fn student_t_p_value_large_t() {
        let p = student_t_p_value(100.0, 10.0);
        assert!(p < 0.001, "p={} for t=100 should be < 0.001", p);
    }
}
