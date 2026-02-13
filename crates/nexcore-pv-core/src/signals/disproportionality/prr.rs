//! Proportional Reporting Ratio (PRR) Algorithm.
//!
//! PRR is a frequentist disproportionality measure defined as:
//!
//! ```text
//! PRR = [a/(a+b)] / [c/(c+d)]
//!     = P(Event|Drug) / P(Event|No Drug)
//! ```
//!
//! # Signal Criteria (Evans et al. 2001)
//!
//! - PRR >= 2.0
//! - Chi-square >= 3.841 (p < 0.05, df=1) - CRITICAL: NOT 4.0
//! - At least 3 cases (a >= 3)
//!
//! # References
//!
//! - Evans SJW, Waller PC, Davis S (2001). "Use of proportional reporting ratios (PRRs)
//!   for signal generation from spontaneous adverse drug reaction reports."
//!   Pharmacoepidemiology and Drug Safety 10(6):483-486.
//!   DOI: [10.1002/pds.677](https://doi.org/10.1002/pds.677)
//!
//! - Rothman KJ, Lanes S, Sacks ST (2004). "The reporting odds ratio and its advantages
//!   over the proportional reporting ratio." Pharmacoepidemiology and Drug Safety 13(8):519-523.
//!   DOI: [10.1002/pds.1001](https://doi.org/10.1002/pds.1001)
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, disproportionality::prr::calculate_prr};
//!
//! let table = ContingencyTable::new(10, 90, 100, 9800);
//! let criteria = SignalCriteria::evans();
//! let result = calculate_prr(&table, &criteria).unwrap();
//!
//! println!("PRR = {:.2}, Signal: {}", result.point_estimate, result.is_signal);
//! ```

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::Z_95;
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalMethod, SignalResult};

/// Calculate PRR and determine signal status.
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
/// `SignalResult` with PRR point estimate, confidence intervals, and signal status.
///
/// # Errors
///
/// Returns `SignalError` if the contingency table is invalid.
#[allow(clippy::many_single_char_names)] // a,b,c,d is standard contingency table notation
pub fn calculate_prr(
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
    let n = table.total() as f64;

    // Handle zero cases
    if table.a == 0 {
        return Ok(SignalResult::null(
            SignalMethod::Prr,
            table.a,
            table.total(),
        ));
    }

    // Calculate PRR
    let drug_event_rate = a / (a + b);
    let non_drug_event_rate = c / (c + d);

    if non_drug_event_rate == 0.0 {
        return Err(SignalError::math_error(
            "Division by zero: no events in non-drug group",
        ));
    }

    let prr = drug_event_rate / non_drug_event_rate;

    // Calculate standard error of log(PRR)
    let se = if a > 0.0 && (a + b) > 0.0 && c > 0.0 && (c + d) > 0.0 {
        (1.0 / a - 1.0 / (a + b) + 1.0 / c - 1.0 / (c + d)).sqrt()
    } else {
        f64::INFINITY
    };

    // Calculate 95% confidence interval using mul_add for better FP precision
    let log_prr = prr.ln();
    let lower_ci = (-Z_95).mul_add(se, log_prr).exp();
    let upper_ci = Z_95.mul_add(se, log_prr).exp();

    // Calculate chi-square statistic
    let expected_a = (a + b) * (a + c) / n;
    let chi_square = if expected_a > 0.0 {
        (a - expected_a).powi(2) / expected_a
    } else {
        0.0
    };

    // Determine signal status using Evans criteria
    // CRITICAL: Chi-square threshold is 3.841, NOT 4.0
    let is_signal = prr >= criteria.prr_threshold
        && chi_square >= criteria.chi_square_threshold
        && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Prr,
        point_estimate: prr,
        lower_ci,
        upper_ci,
        chi_square: Some(chi_square),
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// Calculate PRR only (without signal determination).
///
/// # Complexity
///
/// - **Time**: O(1) - constant-time arithmetic
/// - **Space**: O(1) - returns single f64
#[must_use]
pub fn prr_only(table: &ContingencyTable) -> Option<f64> {
    if !table.is_valid() || table.a == 0 {
        return None;
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    let drug_event_rate = a / (a + b);
    let non_drug_event_rate = c / (c + d);

    if non_drug_event_rate == 0.0 {
        return None;
    }

    Some(drug_event_rate / non_drug_event_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prr_calculation() {
        // Example: Strong signal
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_prr(&table, &criteria).unwrap();

        // PRR = (10/100) / (100/9900) = 0.1 / 0.0101 = 9.9
        assert!(result.point_estimate > 9.0 && result.point_estimate < 10.0);
        assert!(result.is_signal);
    }

    #[test]
    fn test_prr_no_signal_low_prr() {
        // Low PRR (close to 1)
        let table = ContingencyTable::new(100, 900, 1000, 8000);
        let criteria = SignalCriteria::evans();
        let result = calculate_prr(&table, &criteria).unwrap();

        // PRR = (100/1000) / (1000/9000) = 0.1 / 0.111 = 0.9
        assert!(result.point_estimate < 2.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn test_prr_zero_cases() {
        let table = ContingencyTable::new(0, 100, 100, 9800);
        let criteria = SignalCriteria::evans();
        let result = calculate_prr(&table, &criteria).unwrap();

        assert_eq!(result.point_estimate, 0.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn test_prr_chi_square_threshold() {
        // Test that chi-square threshold is 3.841 (not 4.0)
        let table = ContingencyTable::new(5, 95, 50, 9850);
        let criteria = SignalCriteria::evans();
        let result = calculate_prr(&table, &criteria).unwrap();

        // Chi-square should be calculated correctly
        assert!(result.chi_square.is_some());

        // Verify Evans threshold is 3.841
        assert!((criteria.chi_square_threshold - 3.841).abs() < 0.001);
    }
}
