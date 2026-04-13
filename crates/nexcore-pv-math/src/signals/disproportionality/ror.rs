//! Reporting Odds Ratio (ROR).
//!
//! ROR = (a × d) / (b × c)
//!
//! Applies Haldane-Anscombe correction (+0.5) when any cell is zero.
//! Signal criteria: ROR ≥ 2.0, lower 95% CI ≥ 1.0, n ≥ 3.

use crate::error::PvMathError;
use crate::stats::{Z_95, apply_continuity_correction, chi_square_statistic, log_ratio_se};
use crate::types::{SignalCriteria, SignalResult, TwoByTwoTable};
use nexcore_signal_types::SignalMethod;

/// Calculate ROR and signal status from a 2×2 contingency table.
///
/// # Errors
///
/// Returns `PvMathError` when the table is invalid or division by zero occurs.
pub fn calculate_ror(
    table: &TwoByTwoTable,
    criteria: &SignalCriteria,
) -> Result<SignalResult, PvMathError> {
    if !table.is_valid() {
        return Err(PvMathError::invalid_table("empty contingency table"));
    }

    if table.a == 0 {
        return Ok(SignalResult {
            method: SignalMethod::Ror,
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

    let (a_adj, b_adj, c_adj, d_adj) = apply_continuity_correction(a, b, c, d);

    if b_adj * c_adj == 0.0 {
        return Err(PvMathError::math_error("division by zero in ROR"));
    }

    let ror = (a_adj * d_adj) / (b_adj * c_adj);
    let log_ror = ror.ln();
    let se = log_ratio_se(a_adj, b_adj, c_adj, d_adj);

    let lower_ci = (-Z_95).mul_add(se, log_ror).exp();
    let upper_ci = Z_95.mul_add(se, log_ror).exp();
    let chi_square = chi_square_statistic(a, b, c, d);

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

/// Calculate ROR point estimate only — `None` for invalid or zero-case tables.
#[must_use]
pub fn ror_only(table: &TwoByTwoTable) -> Option<f64> {
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
    fn ror_strong_signal() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_ror(&table, &SignalCriteria::evans()).unwrap();
        // ROR = (10 × 9800) / (90 × 100) ≈ 10.89
        assert!(result.point_estimate > 10.0 && result.point_estimate < 12.0);
        assert!(result.is_signal);
        assert!(result.lower_ci >= 1.0);
    }

    #[test]
    fn ror_zero_cases_returns_null() {
        let table = TwoByTwoTable::new(0, 100, 100, 9800);
        let result = calculate_ror(&table, &SignalCriteria::evans()).unwrap();
        assert_eq!(result.point_estimate, 0.0);
        assert!(!result.is_signal);
    }

    #[test]
    fn ror_continuity_correction_applied() {
        // Zero b cell — Haldane correction should prevent division by zero
        let table = TwoByTwoTable::new(5, 0, 100, 9895);
        let result = calculate_ror(&table, &SignalCriteria::evans()).unwrap();
        assert!(result.point_estimate > 0.0);
    }
}
