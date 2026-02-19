//! Information Component (IC) Algorithm.
//!
//! IC is a Bayesian disproportionality measure developed by the WHO Uppsala
//! Monitoring Centre. It measures the degree of unexpectedness of an
//! observed drug-event combination.
//!
//! ```text
//! IC = log2(observed / expected)
//! ```
//!
//! # Signal Criteria
//!
//! - IC025 > 0 (lower 95% credibility interval above zero)
//! - At least 3 cases
//!
//! # References
//!
//! - Bate A, Lindquist M, Edwards IR, et al. (1998). "A Bayesian neural network method
//!   for adverse drug reaction signal generation." European Journal of Clinical
//!   Pharmacology 54(4):315-321. DOI: [10.1007/s002280050466](https://doi.org/10.1007/s002280050466)
//!
//! - Norén GN, Hopstadius J, Bate A (2010). "Shrinkage observed-to-expected ratios for
//!   robust and transparent large-scale pattern discovery." Statistical Methods in
//!   Medical Research 22(1):57-69. DOI: [10.1177/0962280211403604](https://doi.org/10.1177/0962280211403604)
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, bayesian::ic::calculate_ic};
//!
//! let table = ContingencyTable::new(10, 90, 100, 9800);
//! let criteria = SignalCriteria::evans();
//! let result = calculate_ic(&table, &criteria).unwrap();
//!
//! println!("IC = {:.2}, IC025 = {:.2}, Signal: {}",
//!          result.point_estimate, result.lower_ci, result.is_signal);
//! ```

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::{Z_95, log2};
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalMethod, SignalResult};

/// Calculate IC and determine signal status.
///
/// Uses a shrinkage estimator: IC = log2((a + 0.5) / (E + 0.5))
/// where E is the expected count under independence.
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic operations
/// - **Space**: O(1) - fixed-size output struct
///
/// # Arguments
///
/// * `table` - 2x2 contingency table
/// * `criteria` - Signal detection thresholds
///
/// # Returns
///
/// `SignalResult` with IC point estimate, credibility intervals, and signal status.
///
/// # Errors
///
/// Returns `SignalError` if the contingency table is invalid.
pub fn calculate_ic(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, SignalError> {
    // Validation
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Empty contingency table"));
    }

    let a = table.a as f64;
    let expected = table.expected_count();

    // Calculate IC with shrinkage (add 0.5 for regularization)
    let ic = log2((a + 0.5) / (expected + 0.5));

    // Calculate variance using approximation
    // Var(IC) ≈ 1 / ((a + 0.5) * ln(2)^2)
    let ln2_squared = std::f64::consts::LN_2.powi(2);
    let variance = 1.0 / ((a + 0.5) * ln2_squared);
    let sd = variance.sqrt();

    // Calculate 95% credibility interval using mul_add for better FP precision
    // IC025 and IC975 (2.5th and 97.5th percentiles)
    let ic025 = (-Z_95).mul_add(sd, ic);
    let ic975 = Z_95.mul_add(sd, ic);

    // Determine signal status
    // Signal if IC025 > 0 (lower bound excludes zero) and minimum cases met
    let is_signal = ic025 > criteria.ic025_threshold && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Ic,
        point_estimate: ic,
        lower_ci: ic025,
        upper_ci: ic975,
        chi_square: None, // IC is Bayesian, no chi-square
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// Calculate IC only (without signal determination).
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn ic_only(table: &ContingencyTable) -> Option<f64> {
    if !table.is_valid() {
        return None;
    }

    let a = table.a as f64;
    let expected = table.expected_count();

    Some(log2((a + 0.5) / (expected + 0.5)))
}

/// Calculate IC025 (lower 95% credibility bound).
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn ic025(table: &ContingencyTable) -> Option<f64> {
    if !table.is_valid() {
        return None;
    }

    let a = table.a as f64;
    let expected = table.expected_count();

    let ic = log2((a + 0.5) / (expected + 0.5));
    let ln2_squared = std::f64::consts::LN_2.powi(2);
    let variance = 1.0 / ((a + 0.5) * ln2_squared);
    let sd = variance.sqrt();

    Some(ic - Z_95 * sd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ic_calculation() {
        // Example: Strong signal
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ic(&table, &criteria).unwrap();

        // Expected count = (100 * 110) / 10000 = 1.1
        // IC = log2((10.5) / (1.6)) = log2(6.56) ≈ 2.71
        assert!(result.point_estimate > 2.0);
        assert!(result.is_signal);
        assert!(result.lower_ci > 0.0); // IC025 > 0
    }

    #[test]
    fn test_ic_no_signal_low_ic025() {
        // Few cases - IC025 might be below 0
        let table = ContingencyTable::new(2, 98, 200, 9700);
        let criteria = SignalCriteria::evans();
        let result = calculate_ic(&table, &criteria).unwrap();

        // With only 2 cases, credibility interval is wide
        // And min_cases = 3 not met
        assert!(!result.is_signal);
    }

    #[test]
    fn test_ic_zero_cases() {
        let table = ContingencyTable::new(0, 100, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ic(&table, &criteria).unwrap();

        // IC should be negative (observed << expected)
        assert!(result.point_estimate < 0.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn test_ic_independence() {
        // When observed ≈ expected, IC ≈ 0
        let table = ContingencyTable::new(100, 900, 1000, 8000);
        let result = calculate_ic(&table, &SignalCriteria::evans()).unwrap();

        // IC should be close to 0
        assert!(result.point_estimate.abs() < 0.5);
    }
}
