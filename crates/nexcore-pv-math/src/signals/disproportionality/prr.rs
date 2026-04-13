//! Proportional Reporting Ratio (PRR) — Evans et al. 2001.
//!
//! PRR = [a/(a+b)] / [c/(c+d)] = P(Event|Drug) / P(Event|No Drug)
//!
//! Signal criteria (Evans):
//! - PRR ≥ 2.0
//! - χ² ≥ 3.841 (p < 0.05, df = 1) — CRITICAL: NOT 4.0
//! - n ≥ 3 cases

use crate::error::PvMathError;
use crate::stats::{Z_95, chi_square_statistic};
use crate::types::{SignalCriteria, SignalResult, TwoByTwoTable};
use nexcore_signal_types::SignalMethod;

/// Calculate PRR and signal status from a 2×2 contingency table.
///
/// # Errors
///
/// Returns `PvMathError` when the table is invalid or division by zero occurs.
#[allow(clippy::many_single_char_names)]
pub fn calculate_prr(
    table: &TwoByTwoTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, PvMathError> {
    if !table.is_valid() {
        return Err(PvMathError::invalid_table("empty contingency table"));
    }

    if table.a == 0 {
        return Ok(SignalResult {
            method: SignalMethod::Prr,
            point_estimate: 0.0,
            lower_ci: 0.0,
            upper_ci: 0.0,
            chi_square: None,
            is_signal: false,
            case_count: 0,
            total_reports: table.total(),
        });
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;
    let n = table.total() as f64;

    let non_drug_rate = c / (c + d);
    if non_drug_rate == 0.0 {
        return Err(PvMathError::math_error(
            "division by zero: no events in non-drug group",
        ));
    }

    let prr = (a / (a + b)) / non_drug_rate;
    let log_prr = prr.ln();

    let se = if a > 0.0 && (a + b) > 0.0 && c > 0.0 && (c + d) > 0.0 {
        (1.0 / a - 1.0 / (a + b) + 1.0 / c - 1.0 / (c + d)).sqrt()
    } else {
        f64::INFINITY
    };

    let lower_ci = (-Z_95).mul_add(se, log_prr).exp();
    let upper_ci = Z_95.mul_add(se, log_prr).exp();

    let expected_a = (a + b) * (a + c) / n;
    let chi_square = if expected_a > 0.0 {
        (a - expected_a).powi(2) / expected_a
    } else {
        0.0
    };

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

/// Calculate PRR point estimate only — returns `None` for invalid or zero-case tables.
#[must_use]
pub fn prr_only(table: &TwoByTwoTable) -> Option<f64> {
    if !table.is_valid() || table.a == 0 {
        return None;
    }
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;
    let non_drug_rate = c / (c + d);
    if non_drug_rate == 0.0 {
        return None;
    }
    Some((a / (a + b)) / non_drug_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prr_strong_signal() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_prr(&table, &SignalCriteria::evans()).unwrap();
        // PRR = (10/100) / (100/9900) ≈ 9.9
        assert!(result.point_estimate > 9.0 && result.point_estimate < 10.5);
        assert!(result.is_signal);
    }

    #[test]
    fn prr_no_signal_below_threshold() {
        let table = TwoByTwoTable::new(100, 900, 1000, 8000);
        let result = calculate_prr(&table, &SignalCriteria::evans()).unwrap();
        assert!(result.point_estimate < 2.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn prr_zero_cases_returns_null() {
        let table = TwoByTwoTable::new(0, 100, 100, 9800);
        let result = calculate_prr(&table, &SignalCriteria::evans()).unwrap();
        assert_eq!(result.point_estimate, 0.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn prr_chi_square_threshold_is_3_841() {
        let criteria = SignalCriteria::evans();
        assert!((criteria.chi_square_threshold - 3.841).abs() < 0.001);
    }

    #[test]
    fn prr_only_smoke() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let v = prr_only(&table).unwrap();
        assert!(v > 9.0);
    }
}
