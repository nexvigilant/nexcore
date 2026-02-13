//! Reporting Odds Ratio (ROR) Algorithm.
//!
//! ROR is a frequentist disproportionality measure defined as:
//!
//! ```text
//! ROR = (a/b) / (c/d) = (a*d) / (b*c)
//! ```
//!
//! # Signal Criteria
//!
//! - ROR >= 2.0
//! - Lower 95% CI >= 1.0
//! - At least 3 cases (a >= 3)
//!
//! # References
//!
//! - van Puijenbroek EP, Bate A, Leufkens HGM, et al. (2002). "A comparison of measures
//!   of disproportionality for signal detection in spontaneous reporting systems for
//!   adverse drug reactions." Pharmacoepidemiology and Drug Safety 11(1):3-10.
//!   DOI: [10.1002/pds.668](https://doi.org/10.1002/pds.668)
//!
//! - Rothman KJ, Lanes S, Sacks ST (2004). "The reporting odds ratio and its advantages
//!   over the proportional reporting ratio." Pharmacoepidemiology and Drug Safety 13(8):519-523.
//!   DOI: [10.1002/pds.1001](https://doi.org/10.1002/pds.1001)
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, disproportionality::ror::calculate_ror};
//!
//! let table = ContingencyTable::new(10, 90, 100, 9800);
//! let criteria = SignalCriteria::evans();
//! let result = calculate_ror(&table, &criteria).unwrap();
//!
//! println!("ROR = {:.2}, Signal: {}", result.point_estimate, result.is_signal);
//! ```

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::{
    Z_95, apply_continuity_correction, chi_square_statistic, log_ratio_standard_error,
};
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalMethod, SignalResult};

/// Calculate ROR and determine signal status.
///
/// Applies Haldane-Anscombe correction (adds 0.5) when any cell is zero.
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
/// `SignalResult` with ROR point estimate, confidence intervals, and signal status.
///
/// # Errors
///
/// Returns `SignalError` if the contingency table is invalid.
pub fn calculate_ror(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, SignalError> {
    // Validation
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Empty contingency table"));
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    // Handle zero cases
    if table.a == 0 {
        return Ok(SignalResult::null(
            SignalMethod::Ror,
            table.a,
            table.total(),
        ));
    }

    // Apply Haldane-Anscombe correction if needed
    let (a_adj, b_adj, c_adj, d_adj) = apply_continuity_correction(a, b, c, d);

    // Check for division by zero
    if b_adj * c_adj == 0.0 {
        return Err(SignalError::math_error(
            "Division by zero in ROR calculation",
        ));
    }

    // Calculate ROR
    let ror = (a_adj * d_adj) / (b_adj * c_adj);

    // Calculate log(ROR) and standard error
    let log_ror = ror.ln();
    let se = log_ratio_standard_error(a_adj, b_adj, c_adj, d_adj);

    // Calculate 95% confidence interval using mul_add for better FP precision
    let lower_ci = (-Z_95).mul_add(se, log_ror).exp();
    let upper_ci = Z_95.mul_add(se, log_ror).exp();

    // Calculate chi-square statistic (using original values)
    let chi_square = chi_square_statistic(a, b, c, d);

    // Determine signal status
    let is_signal = ror >= criteria.ror_threshold
        && lower_ci >= criteria.ror_lower_ci_threshold
        && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Ror,
        point_estimate: ror,
        lower_ci,
        upper_ci,
        chi_square: Some(chi_square),
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// Calculate ROR only (without signal determination).
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn ror_only(table: &ContingencyTable) -> Option<f64> {
    if !table.is_valid() || table.a == 0 {
        return None;
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    let (a_adj, b_adj, c_adj, d_adj) = apply_continuity_correction(a, b, c, d);

    if b_adj * c_adj == 0.0 {
        return None;
    }

    Some((a_adj * d_adj) / (b_adj * c_adj))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ror_calculation() {
        // Example: Strong signal
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ror(&table, &criteria).unwrap();

        // ROR = (10 * 9800) / (90 * 100) = 98000 / 9000 = 10.89
        assert!(result.point_estimate > 10.0 && result.point_estimate < 12.0);
        assert!(result.is_signal);
        assert!(result.lower_ci >= 1.0);
    }

    #[test]
    fn test_ror_no_signal_low_ci() {
        // Small sample size - wide CI
        let table = ContingencyTable::new(3, 97, 300, 9600);
        let criteria = SignalCriteria::evans();
        let result = calculate_ror(&table, &criteria).unwrap();

        // ROR might be high but CI might include 1
        println!(
            "ROR: {:.2}, CI: [{:.2}, {:.2}]",
            result.point_estimate, result.lower_ci, result.upper_ci
        );
    }

    #[test]
    fn test_ror_continuity_correction() {
        // Zero cell - should apply correction
        let table = ContingencyTable::new(5, 0, 100, 9895);
        let criteria = SignalCriteria::evans();
        let result = calculate_ror(&table, &criteria).unwrap();

        // Should still calculate (with correction applied)
        assert!(result.point_estimate > 0.0);
    }

    #[test]
    fn test_ror_zero_cases() {
        let table = ContingencyTable::new(0, 100, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_ror(&table, &criteria).unwrap();

        assert_eq!(result.point_estimate, 0.0);
        assert!(!result.is_signal);
    }
}
