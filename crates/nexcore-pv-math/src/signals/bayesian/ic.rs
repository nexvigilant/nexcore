//! Information Component (IC) — WHO Uppsala Monitoring Centre Bayesian measure.
//!
//! IC = log₂(observed / expected), with shrinkage: IC = log₂((a + 0.5) / (E + 0.5))
//!
//! Signal criterion: IC025 > 0 (lower 95% credibility bound above zero), n ≥ 3.
//!
//! # References
//!
//! Bate A et al. (1998). Eur J Clin Pharmacol 54(4):315-321.

use crate::error::PvMathError;
use crate::stats::{Z_95, log2};
use crate::types::{SignalCriteria, SignalResult, TwoByTwoTable};
use nexcore_signal_types::SignalMethod;

/// Calculate IC and determine signal status.
///
/// Uses shrinkage estimator: IC = log₂((a + 0.5) / (E + 0.5))
/// Variance approximation: Var(IC) ≈ 1 / ((a + 0.5) × ln(2)²)
///
/// # Errors
///
/// Returns `PvMathError` when the table is invalid.
pub fn calculate_ic(
    table: &TwoByTwoTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, PvMathError> {
    if !table.is_valid() {
        return Err(PvMathError::invalid_table("empty contingency table"));
    }

    let a = table.a as f64;
    let expected = table.expected_count();

    let ic = log2((a + 0.5) / (expected + 0.5));

    let ln2_sq = std::f64::consts::LN_2.powi(2);
    let sd = (1.0 / ((a + 0.5) * ln2_sq)).sqrt();

    let ic025 = (-Z_95).mul_add(sd, ic);
    let ic975 = Z_95.mul_add(sd, ic);

    let is_signal = ic025 > criteria.ic025_threshold && table.a >= u64::from(criteria.min_cases);

    Ok(SignalResult {
        method: SignalMethod::Ic,
        point_estimate: ic,
        lower_ci: ic025,
        upper_ci: ic975,
        chi_square: None,
        is_signal,
        case_count: table.a,
        total_reports: table.total(),
    })
}

/// IC point estimate only.
#[must_use]
pub fn ic_only(table: &TwoByTwoTable) -> Option<f64> {
    if !table.is_valid() {
        return None;
    }
    let a = table.a as f64;
    let expected = table.expected_count();
    Some(log2((a + 0.5) / (expected + 0.5)))
}

/// IC025 — lower 95% credibility bound.
#[must_use]
pub fn ic025(table: &TwoByTwoTable) -> Option<f64> {
    if !table.is_valid() {
        return None;
    }
    let a = table.a as f64;
    let expected = table.expected_count();
    let ic = log2((a + 0.5) / (expected + 0.5));
    let sd = (1.0 / ((a + 0.5) * std::f64::consts::LN_2.powi(2))).sqrt();
    Some(ic - Z_95 * sd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ic_strong_signal() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_ic(&table, &SignalCriteria::evans()).unwrap();
        // Expected ≈ 1.1, IC = log2(10.5 / 1.6) ≈ 2.71
        assert!(result.point_estimate > 2.0);
        assert!(result.is_signal);
        assert!(result.lower_ci > 0.0);
    }

    #[test]
    fn ic_no_signal_too_few_cases() {
        let table = TwoByTwoTable::new(2, 98, 200, 9700);
        let result = calculate_ic(&table, &SignalCriteria::evans()).unwrap();
        // min_cases = 3 not met
        assert!(!result.is_signal);
    }

    #[test]
    fn ic_near_independence_is_near_zero() {
        // When observed ≈ expected, IC ≈ 0
        let table = TwoByTwoTable::new(100, 900, 1000, 8000);
        let result = calculate_ic(&table, &SignalCriteria::evans()).unwrap();
        assert!(result.point_estimate.abs() < 0.5);
    }
}
