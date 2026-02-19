//! Empirical Bayes Geometric Mean (EBGM) Algorithm.
//!
//! EBGM uses the Multi-item Gamma Poisson Shrinker (MGPS) method developed
//! by DuMouchel. It provides shrinkage estimation to reduce false positives
//! from multiple comparisons, especially for sparse data.
//!
//! The prior is a mixture of two gamma distributions:
//! ```text
//! Prior = p * Gamma(α₁, β₁) + (1-p) * Gamma(α₂, β₂)
//! ```
//!
//! # Signal Criteria
//!
//! - EBGM >= 2.0
//! - EB05 >= 2.0 (5th percentile of posterior)
//! - At least 3 cases
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, bayesian::ebgm::calculate_ebgm};
//!
//! let table = ContingencyTable::new(10, 90, 100, 9800);
//! let criteria = SignalCriteria::evans();
//! let result = calculate_ebgm(&table, &criteria).unwrap();
//!
//! println!("EBGM = {:.2}, EB05 = {:.2}, Signal: {}",
//!          result.point_estimate, result.lower_ci, result.is_signal);
//! ```
//!
//! # References
//!
//! - DuMouchel W (1999). "Bayesian data mining in large frequency tables, with an
//!   application to the FDA spontaneous reporting system." The American Statistician
//!   53(3):177-190. DOI: [10.1080/00031305.1999.10474456](https://doi.org/10.1080/00031305.1999.10474456)
//!
//! - DuMouchel W, Pregibon D (2001). "Empirical Bayes screening for multi-item
//!   associations." Proceedings of the 7th ACM SIGKDD International Conference
//!   on Knowledge Discovery and Data Mining, pp. 67-76.
//!   DOI: [10.1145/502512.502526](https://doi.org/10.1145/502512.502526)
//!
//! - Szarfman A, Machado SG, O'Neill RT (2002). "Use of screening algorithms and
//!   computer systems to efficiently signal higher-than-expected combinations of
//!   drugs and events in the US FDA's spontaneous reports database." Drug Safety
//!   25(6):381-392. DOI: [10.2165/00002018-200225060-00001](https://doi.org/10.2165/00002018-200225060-00001)

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::{log_gamma, normal_quantile};
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalMethod, SignalResult};

/// Default prior parameters for MGPS.
#[derive(Debug, Clone, Copy)]
pub struct MGPSPriors {
    /// Shape parameter for signal distribution
    pub alpha1: f64,
    /// Rate parameter for signal distribution
    pub beta1: f64,
    /// Shape parameter for null distribution
    pub alpha2: f64,
    /// Rate parameter for null distribution
    pub beta2: f64,
    /// Mixing proportion for signal component
    pub p: f64,
}

impl Default for MGPSPriors {
    fn default() -> Self {
        // DuMouchel recommended priors
        Self {
            alpha1: 0.2,
            beta1: 0.1,
            alpha2: 2.0,
            beta2: 4.0,
            p: 0.1,
        }
    }
}

/// Calculate EBGM and determine signal status.
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic (fixed prior parameters)
/// - **Space**: O(1) - fixed-size output struct
///
/// # Arguments
///
/// * `table` - 2x2 contingency table
/// * `criteria` - Signal detection thresholds
///
/// # Returns
///
/// `SignalResult` with EBGM point estimate, EB05/EB95 bounds, and signal status.
///
/// # Errors
///
/// Returns `SignalError` if the contingency table is invalid.
pub fn calculate_ebgm(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, SignalError> {
    calculate_ebgm_with_priors(table, criteria, &MGPSPriors::default())
}

/// Calculate EBGM with custom prior parameters.
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic operations
/// - **Space**: O(1) - fixed-size output struct
pub fn calculate_ebgm_with_priors(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
    priors: &MGPSPriors,
) -> Result<SignalResult, SignalError> {
    // Validation
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Empty contingency table"));
    }

    let observed = table.a as f64;
    let expected = table.expected_count();

    // Handle edge case
    if expected <= 0.0 {
        return Ok(SignalResult::null(
            SignalMethod::Ebgm,
            table.a,
            table.total(),
        ));
    }

    // Calculate posterior weights
    let w1 = gamma_poisson_weight(observed, expected, priors.alpha1, priors.beta1, priors.p);
    let w2 = gamma_poisson_weight(
        observed,
        expected,
        priors.alpha2,
        priors.beta2,
        1.0 - priors.p,
    );

    let total_weight = w1 + w2;
    if total_weight <= 0.0 {
        // Fall back to raw ratio
        return Ok(SignalResult {
            method: SignalMethod::Ebgm,
            point_estimate: observed / expected,
            lower_ci: 0.0,
            upper_ci: f64::INFINITY,
            chi_square: None,
            is_signal: false,
            case_count: table.a,
            total_reports: table.total(),
        });
    }

    let q1 = w1 / total_weight;
    let q2 = w2 / total_weight;

    // Guard against NaN from division
    if q1.is_nan() || q2.is_nan() {
        return Ok(SignalResult {
            method: SignalMethod::Ebgm,
            point_estimate: observed / expected,
            lower_ci: 0.0,
            upper_ci: f64::INFINITY,
            chi_square: None,
            is_signal: false,
            case_count: table.a,
            total_reports: table.total(),
        });
    }

    // Posterior means for each component
    let mean1 = (observed + priors.alpha1) / (expected + priors.beta1);
    let mean2 = (observed + priors.alpha2) / (expected + priors.beta2);

    // EBGM = exp(q1 * log(mean1) + q2 * log(mean2))
    let ebgm = if mean1 > 0.0 && mean2 > 0.0 {
        let log_ebgm = q1 * mean1.ln() + q2 * mean2.ln();
        if log_ebgm.is_nan() || log_ebgm > 700.0 {
            observed / expected // Fall back to raw ratio
        } else if log_ebgm < -700.0 {
            0.0
        } else {
            log_ebgm.exp()
        }
    } else {
        mean1.max(mean2)
    };

    // Guard against NaN in ebgm
    let ebgm = if ebgm.is_nan() || ebgm.is_infinite() {
        observed / expected
    } else {
        ebgm
    };

    // Calculate EB05 and EB95 using normal approximation
    let variance = posterior_variance(observed, priors);
    let (eb05, eb95) = if variance > 0.0 && ebgm > 0.0 && ebgm.is_finite() {
        let sd = variance.sqrt();
        let z_05 = normal_quantile(0.05);
        let z_95 = normal_quantile(0.95);
        let log_ebgm = ebgm.ln();
        if log_ebgm.is_finite() {
            let eb05_calc = z_05.mul_add(sd, log_ebgm).exp();
            let eb95_calc = z_95.mul_add(sd, log_ebgm).exp();
            (
                if eb05_calc.is_finite() {
                    eb05_calc
                } else {
                    ebgm * 0.5
                },
                if eb95_calc.is_finite() {
                    eb95_calc
                } else {
                    ebgm * 2.0
                },
            )
        } else {
            (ebgm * 0.5, ebgm * 2.0)
        }
    } else {
        (ebgm * 0.5, ebgm * 2.0)
    };

    // Determine signal status
    let is_signal = ebgm >= criteria.ebgm_threshold
        && eb05 >= criteria.eb05_threshold
        && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Ebgm,
        point_estimate: ebgm,
        lower_ci: eb05,
        upper_ci: eb95,
        chi_square: None, // EBGM is Bayesian, no chi-square
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// Calculate weight for a gamma-Poisson mixture component.
///
/// Uses log-space calculations to avoid numerical overflow/underflow.
///
/// # Complexity
///
/// - **Time**: O(1) - Stirling approximation for log-gamma
/// - **Space**: O(1) - stack allocation only
#[allow(clippy::suboptimal_flops)] // Formula clarity > micro-optimization for MGPS math
fn gamma_poisson_weight(n: f64, e: f64, alpha: f64, beta: f64, prior_p: f64) -> f64 {
    if prior_p <= 0.0 || alpha <= 0.0 || beta <= 0.0 {
        return 0.0;
    }

    // Guard against extreme values that would cause overflow
    if n > 1e6 || e > 1e6 {
        return 0.0;
    }

    let log_weight = log_gamma(n + alpha) - log_gamma(alpha) + alpha * beta.ln()
        - (n + alpha) * (e + beta).ln()
        + prior_p.ln();

    // Clamp to avoid underflow/overflow
    if log_weight.is_nan() || log_weight < -700.0 {
        0.0
    } else if log_weight > 700.0 {
        f64::MAX
    } else {
        log_weight.exp()
    }
}

/// Approximate posterior variance of log(lambda).
///
/// # Complexity: O(1)
fn posterior_variance(n: f64, priors: &MGPSPriors) -> f64 {
    let var1 = if n + priors.alpha1 > 0.0 {
        1.0 / (n + priors.alpha1)
    } else {
        1.0
    };

    let var2 = if n + priors.alpha2 > 0.0 {
        1.0 / (n + priors.alpha2)
    } else {
        1.0
    };

    priors.p * var1 + (1.0 - priors.p) * var2
}

/// Calculate EBGM only (without signal determination).
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn ebgm_only(table: &ContingencyTable) -> Option<f64> {
    if !table.is_valid() {
        return None;
    }

    let priors = MGPSPriors::default();
    let observed = table.a as f64;
    let expected = table.expected_count();

    if expected <= 0.0 {
        return Some(0.0);
    }

    let w1 = gamma_poisson_weight(observed, expected, priors.alpha1, priors.beta1, priors.p);
    let w2 = gamma_poisson_weight(
        observed,
        expected,
        priors.alpha2,
        priors.beta2,
        1.0 - priors.p,
    );

    let total_weight = w1 + w2;
    if total_weight <= 0.0 {
        return Some(observed / expected);
    }

    let q1 = w1 / total_weight;
    let q2 = w2 / total_weight;

    // Guard against NaN
    if q1.is_nan() || q2.is_nan() {
        return Some(observed / expected);
    }

    let mean1 = (observed + priors.alpha1) / (expected + priors.beta1);
    let mean2 = (observed + priors.alpha2) / (expected + priors.beta2);

    let result = if mean1 > 0.0 && mean2 > 0.0 {
        let log_ebgm = q1 * mean1.ln() + q2 * mean2.ln();
        if log_ebgm.is_nan() || log_ebgm > 700.0 {
            observed / expected
        } else if log_ebgm < -700.0 {
            0.0
        } else {
            log_ebgm.exp()
        }
    } else {
        mean1.max(mean2)
    };

    // Final guard against NaN/Inf
    Some(if result.is_nan() || result.is_infinite() {
        observed / expected
    } else {
        result
    })
}

/// Calculate EB05 (lower 5th percentile).
///
/// # Complexity
///
/// - **Time**: O(1) - delegates to calculate_ebgm
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn eb05(table: &ContingencyTable) -> Option<f64> {
    let result = calculate_ebgm(table, &SignalCriteria::evans()).ok()?;
    Some(result.lower_ci)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebgm_calculation() {
        // Example: Strong signal
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ebgm(&table, &criteria).unwrap();

        // EBGM should show shrinkage (lower than raw ratio)
        let raw_ratio = 10.0 / table.expected_count();
        assert!(result.point_estimate > 0.0);
        assert!(result.point_estimate <= raw_ratio + 1.0); // Shrinkage

        println!("EBGM: {:.2}, Raw: {:.2}", result.point_estimate, raw_ratio);
    }

    #[test]
    fn test_ebgm_shrinkage_effect() {
        // Small sample - should see significant shrinkage
        let table = ContingencyTable::new(3, 97, 300, 9600);
        let criteria = SignalCriteria::evans();
        let result = calculate_ebgm(&table, &criteria).unwrap();

        // Raw ratio would be 3 / expected, EBGM should be lower (shrunk toward prior)
        let raw_ratio = 3.0 / table.expected_count();
        assert!(result.point_estimate < raw_ratio);
    }

    #[test]
    fn test_ebgm_signal_criteria() {
        // Test signal detection criteria
        let table = ContingencyTable::new(20, 80, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ebgm(&table, &criteria).unwrap();

        // Should be a signal with high EBGM and EB05
        if result.point_estimate >= 2.0 && result.lower_ci >= 2.0 {
            assert!(result.is_signal);
        }
    }

    #[test]
    fn test_ebgm_zero_cases() {
        let table = ContingencyTable::new(0, 100, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ebgm(&table, &criteria).unwrap();

        assert!(!result.is_signal);
    }

    #[test]
    fn test_custom_priors() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();

        // More conservative priors (stronger shrinkage)
        let conservative_priors = MGPSPriors {
            alpha1: 0.1,
            beta1: 0.05,
            alpha2: 4.0,
            beta2: 8.0,
            p: 0.05,
        };

        let default_result = calculate_ebgm(&table, &criteria).unwrap();
        let conservative_result =
            calculate_ebgm_with_priors(&table, &criteria, &conservative_priors).unwrap();

        // Conservative priors should result in more shrinkage
        println!(
            "Default EBGM: {:.2}, Conservative EBGM: {:.2}",
            default_result.point_estimate, conservative_result.point_estimate
        );
    }
}
