//! # BCPNN (Bayesian Confidence Propagation Neural Network)
//!
//! The BCPNN method was developed by Bate et al. (1998) and is used by
//! WHO-UMC for signal detection. It's closely related to the IC (Information
//! Component) but uses a neural network-inspired shrinkage approach.
//!
//! ## Algorithm
//!
//! BCPNN computes the Information Component (IC) with Bayesian shrinkage:
//!
//! ```text
//! IC = log₂(observed / expected)
//!
//! where:
//!   observed = a (drug-event co-occurrence)
//!   expected = (a+b)(a+c) / N (under independence)
//! ```
//!
//! The key difference from simple IC is the use of a proper Bayesian
//! posterior distribution with informative priors, leading to shrinkage
//! that is especially useful for rare events.
//!
//! ## Signal Criteria (WHO-UMC)
//!
//! A signal is detected when:
//! - IC025 > 0 (lower 2.5% credibility bound is positive)
//! - n >= 3 cases
//!
//! ## Complexity
//!
//! - TIME: O(1) - constant time (fixed number of arithmetic operations)
//! - SPACE: O(1) - constant space (only local variables)
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, bayesian::bcpnn::calculate_bcpnn};
//!
//! let table = ContingencyTable::new(10, 90, 100, 9800);
//! let criteria = SignalCriteria::evans();
//! let result = calculate_bcpnn(&table, &criteria).unwrap();
//!
//! println!("IC = {:.3}, IC025 = {:.3}", result.point_estimate, result.lower_ci);
//! ```
//!
//! ## References
//!
//! - Bate A, Lindquist M, Edwards IR, et al. (1998). "A Bayesian neural network method
//!   for adverse drug reaction signal generation." European Journal of Clinical
//!   Pharmacology 54(4):315-321. DOI: [10.1007/s002280050466](https://doi.org/10.1007/s002280050466)
//!
//! - Norén GN, Bate A, Orre R, Edwards IR (2006). "Extending the methods used to screen
//!   the WHO drug safety database towards analysis of complex associations and improved
//!   accuracy for rare events." Statistics in Medicine 25(21):3740-3757.
//!   DOI: [10.1002/sim.2473](https://doi.org/10.1002/sim.2473)
//!
//! - Norén GN, Hopstadius J, Bate A (2010). "Shrinkage observed-to-expected ratios for
//!   robust and transparent large-scale pattern discovery." Statistical Methods in
//!   Medical Research 22(1):57-69. DOI: [10.1177/0962280211403604](https://doi.org/10.1177/0962280211403604)

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::log2;
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalMethod, SignalResult};

/// Prior strength parameter (k) for BCPNN.
///
/// This controls the amount of shrinkage toward the null hypothesis.
/// WHO-UMC uses k = 0.5 (weak prior), which provides minimal shrinkage
/// for cells with sufficient data while stabilizing estimates for rare events.
const BCPNN_PRIOR_K: f64 = 0.5;

/// Calculate BCPNN (Bayesian Confidence Propagation Neural Network) signal metric.
///
/// # Algorithm
///
/// Uses Bayesian shrinkage estimation with Beta-binomial priors:
///
/// 1. Calculate prior probabilities using marginal frequencies
/// 2. Apply Bayesian shrinkage to observed counts
/// 3. Compute IC = log₂(posterior_observed / posterior_expected)
/// 4. Estimate credibility interval using posterior variance
///
/// # Complexity
///
/// - TIME: O(1)
/// - SPACE: O(1)
///
/// # Arguments
///
/// * `table` - 2x2 contingency table
/// * `criteria` - Signal detection thresholds
///
/// # Returns
///
/// `SignalResult` with IC point estimate and 95% credibility interval.
///
/// # Errors
///
/// Returns `SignalError` if table is invalid (total = 0).
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, bayesian::bcpnn::calculate_bcpnn};
///
/// let table = ContingencyTable::new(10, 90, 100, 9800);
/// let result = calculate_bcpnn(&table, &SignalCriteria::evans()).unwrap();
/// assert!(result.point_estimate > 0.0); // Elevated IC
/// ```
pub fn calculate_bcpnn(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, SignalError> {
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;
    let n = a + b + c + d;

    // Validate input - O(1)
    if n == 0.0 {
        return Err(SignalError::invalid_table("Total count is zero"));
    }

    // Handle zero cases - return null result - O(1)
    if table.a == 0 {
        return Ok(SignalResult::null(
            SignalMethod::Bcpnn,
            table.a,
            table.total(),
        ));
    }

    // Marginal probabilities - O(1)
    let p_drug = (a + b) / n; // P(drug)
    let p_event = (a + c) / n; // P(event)

    // Prior parameters using marginal frequencies - O(1)
    // These create weakly informative priors centered on marginal rates
    let alpha_drug = BCPNN_PRIOR_K * p_drug;
    let _beta_drug = BCPNN_PRIOR_K * (1.0 - p_drug);
    let alpha_event = BCPNN_PRIOR_K * p_event;
    let _beta_event = BCPNN_PRIOR_K * (1.0 - p_event);
    let alpha_joint = BCPNN_PRIOR_K * p_drug * p_event;
    let _beta_joint = BCPNN_PRIOR_K * (1.0 - p_drug * p_event);

    // Posterior estimates with shrinkage - O(1)
    // E[p_ij] = (a + alpha_joint) / (n + alpha_joint + beta_joint)
    let posterior_joint = (a + alpha_joint) / (n + BCPNN_PRIOR_K);
    let posterior_drug = (a + b + alpha_drug) / (n + BCPNN_PRIOR_K);
    let posterior_event = (a + c + alpha_event) / (n + BCPNN_PRIOR_K);

    // Expected probability under independence - O(1)
    let expected_joint = posterior_drug * posterior_event;

    // IC = log₂(observed / expected) - O(1)
    let ic = if expected_joint > 0.0 && posterior_joint > 0.0 {
        log2(posterior_joint / expected_joint)
    } else {
        0.0
    };

    // Variance estimation using delta method - O(1)
    // Var(IC) ≈ 1 / (n * p_observed * ln(2)²) for large n
    let variance = if posterior_joint > 0.0 && n > 0.0 {
        1.0 / (n * posterior_joint * core::f64::consts::LN_2.powi(2))
    } else {
        f64::INFINITY
    };

    // 95% credibility interval - O(1)
    let se = variance.sqrt();
    let ic025 = ic - 1.96 * se; // Lower 2.5% bound
    let ic975 = ic + 1.96 * se; // Upper 97.5% bound

    // Signal detection using WHO-UMC criteria - O(1)
    let is_signal = ic025 > criteria.ic025_threshold && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Bcpnn,
        point_estimate: ic,
        lower_ci: ic025,
        upper_ci: ic975,
        chi_square: None,
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// Calculate IC025 (lower 2.5% credibility bound) using BCPNN.
///
/// This is the primary metric used by WHO-UMC for signal detection.
/// A positive IC025 indicates statistical significance.
///
/// # Complexity
///
/// - TIME: O(1)
/// - SPACE: O(1)
#[must_use]
pub fn calculate_ic025(table: &ContingencyTable, criteria: &SignalCriteria) -> f64 {
    calculate_bcpnn(table, criteria)
        .map(|r| r.lower_ci)
        .unwrap_or(0.0)
}

/// Check if BCPNN detects a signal.
///
/// # Complexity
///
/// - TIME: O(1)
/// - SPACE: O(1)
#[must_use]
pub fn is_bcpnn_signal(table: &ContingencyTable, criteria: &SignalCriteria) -> bool {
    calculate_bcpnn(table, criteria)
        .map(|r| r.is_signal)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcpnn_basic() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();

        let result = calculate_bcpnn(&table, &criteria).unwrap();

        assert!(
            result.point_estimate > 0.0,
            "IC should be positive for elevated rate"
        );
        assert!(result.lower_ci < result.point_estimate);
        assert!(result.upper_ci > result.point_estimate);
    }

    #[test]
    fn test_bcpnn_signal_detection() {
        // Strong signal case
        let table = ContingencyTable::new(50, 50, 100, 9800);
        let criteria = SignalCriteria::evans();

        let result = calculate_bcpnn(&table, &criteria).unwrap();
        assert!(result.is_signal, "Should detect strong signal");
        assert!(result.lower_ci > 0.0, "IC025 should be positive");
    }

    #[test]
    fn test_bcpnn_no_signal() {
        // Balanced case (no elevation)
        let table = ContingencyTable::new(10, 90, 1000, 8900);
        let criteria = SignalCriteria::evans();

        let result = calculate_bcpnn(&table, &criteria).unwrap();
        // IC should be near zero or negative for balanced rates
        assert!(result.point_estimate < 1.0);
    }

    #[test]
    fn test_bcpnn_zero_cases() {
        let table = ContingencyTable::new(0, 100, 100, 9800);
        let criteria = SignalCriteria::evans();

        let result = calculate_bcpnn(&table, &criteria).unwrap();
        assert!(!result.is_signal);
        assert_eq!(result.point_estimate, 0.0);
    }

    #[test]
    fn test_bcpnn_empty_table() {
        let table = ContingencyTable::new(0, 0, 0, 0);
        let criteria = SignalCriteria::evans();

        let result = calculate_bcpnn(&table, &criteria);
        assert!(result.is_err());
    }

    #[test]
    fn test_ic025_helper() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();

        let ic025 = calculate_ic025(&table, &criteria);
        let result = calculate_bcpnn(&table, &criteria).unwrap();

        assert!((ic025 - result.lower_ci).abs() < 0.001);
    }
}
