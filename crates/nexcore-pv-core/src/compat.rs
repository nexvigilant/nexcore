//! # PV Signals Compatibility Layer
//!
//! Provides wrapper functions that use `pv::types` instead of `signals::core::types`.
//! This allows the rest of the codebase to use the familiar pv types while
//! leveraging the full nexcore-signals algorithm implementations.

use crate::signals::core::types::{ContingencyTable as SigTable, SignalCriteria as SigCriteria};
use crate::thresholds::SignalCriteria as PvCriteria;
use crate::types::{ContingencyTable as PvTable, SignalResult as PvResult};
use signal::core::ContingencyTable as SignalCrateTable;

/// Convert pv::types::ContingencyTable to signals::core::types::ContingencyTable
fn to_sig_table(table: &PvTable) -> SigTable {
    SigTable::new(table.a, table.b, table.c, table.d)
}

/// Convert pv::thresholds::SignalCriteria to signals::core::types::SignalCriteria
fn to_sig_criteria(criteria: &PvCriteria) -> SigCriteria {
    SigCriteria {
        prr_threshold: criteria.prr_threshold,
        ror_threshold: criteria.ror_lower_threshold, // Map to ror_threshold
        ror_lower_ci_threshold: criteria.ror_lower_threshold,
        ic025_threshold: criteria.ic025_threshold,
        ebgm_threshold: criteria.eb05_threshold,
        eb05_threshold: criteria.eb05_threshold,
        chi_square_threshold: criteria.chi_square_threshold,
        min_cases: criteria.min_cases,
    }
}

/// Convert signals::core::types::SignalResult to pv::types::SignalResult
fn to_pv_result(result: crate::signals::core::types::SignalResult) -> PvResult {
    PvResult::new(
        result.point_estimate,
        result.lower_ci,
        result.upper_ci,
        result.is_signal,
    )
}

/// Calculate PRR using pv types.
#[must_use]
pub fn calculate_prr(table: &PvTable, criteria: &PvCriteria) -> PvResult {
    let sig_table = to_sig_table(table);
    let sig_criteria = to_sig_criteria(criteria);

    crate::signals::disproportionality::prr::calculate_prr(&sig_table, &sig_criteria)
        .map(to_pv_result)
        .unwrap_or_else(|_| PvResult::new(0.0, 0.0, 0.0, false))
}

/// Calculate ROR using pv types.
#[must_use]
pub fn calculate_ror(table: &PvTable, criteria: &PvCriteria) -> PvResult {
    let sig_table = to_sig_table(table);
    let sig_criteria = to_sig_criteria(criteria);

    crate::signals::disproportionality::ror::calculate_ror(&sig_table, &sig_criteria)
        .map(to_pv_result)
        .unwrap_or_else(|_| PvResult::new(0.0, 0.0, 0.0, false))
}

/// Calculate IC using pv types.
#[must_use]
pub fn calculate_ic(table: &PvTable, criteria: &PvCriteria) -> PvResult {
    let sig_table = to_sig_table(table);
    let sig_criteria = to_sig_criteria(criteria);

    crate::signals::bayesian::ic::calculate_ic(&sig_table, &sig_criteria)
        .map(to_pv_result)
        .unwrap_or_else(|_| PvResult::new(0.0, 0.0, 0.0, false))
}

/// Calculate EBGM using pv types.
#[must_use]
pub fn calculate_ebgm(table: &PvTable, criteria: &PvCriteria) -> PvResult {
    let sig_table = to_sig_table(table);
    let sig_criteria = to_sig_criteria(criteria);

    crate::signals::bayesian::ebgm::calculate_ebgm(&sig_table, &sig_criteria)
        .map(to_pv_result)
        .unwrap_or_else(|_| PvResult::new(0.0, 0.0, 0.0, false))
}

/// Calculate chi-square statistic.
#[must_use]
pub fn calculate_chi_square(table: &PvTable) -> f64 {
    crate::signals::core::stats::chi_square_statistic(
        table.a as f64,
        table.b as f64,
        table.c as f64,
        table.d as f64,
    )
}

/// Fisher exact test result
#[derive(Debug, Clone)]
pub struct FisherResult {
    /// Two-tailed p-value
    pub p_value_two_tailed: f64,
    /// One-tailed p-value (less)
    pub p_value_less: f64,
    /// One-tailed p-value (greater)
    pub p_value_greater: f64,
    /// Odds ratio
    pub odds_ratio: f64,
    /// Lower confidence interval
    pub lower_ci: f64,
    /// Upper confidence interval
    pub upper_ci: f64,
}

/// Fisher exact test using pv types.
#[must_use]
pub fn fisher_exact_test(table: &PvTable) -> FisherResult {
    // Use a simple implementation for Fisher's exact test
    // This is a simplified version - for production, use hypergeometric distribution
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    let odds_ratio = if b * c == 0.0 {
        f64::INFINITY
    } else {
        (a * d) / (b * c)
    };

    // Approximate p-value using chi-square
    let chi_sq = calculate_chi_square(table);
    let p_value = crate::signals::core::stats::chi_square_p_value(chi_sq);

    // Approximate CI using Woolf's method
    let ln_or = odds_ratio.ln();
    let se = if a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 {
        (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt()
    } else {
        1.0
    };
    let z = 1.96;
    let lower_ci = (ln_or - z * se).exp();
    let upper_ci = (ln_or + z * se).exp();

    FisherResult {
        p_value_two_tailed: p_value,
        p_value_less: p_value / 2.0,
        p_value_greater: p_value / 2.0,
        odds_ratio,
        lower_ci,
        upper_ci,
    }
}

// ── Cross-type ContingencyTable conversions ──────────────────────

/// Lossless: pv-core u64 → signal crate u64 (same width).
impl From<SigTable> for SignalCrateTable {
    fn from(t: SigTable) -> Self {
        Self::new(t.a, t.b, t.c, t.d)
    }
}

/// Lossless: signal crate u64 → pv-core u64 (same width now).
impl From<SignalCrateTable> for SigTable {
    fn from(t: SignalCrateTable) -> Self {
        Self::new(t.a, t.b, t.c, t.d)
    }
}

#[cfg(test)]
mod cross_type_tests {
    use super::*;

    #[test]
    fn from_vigilance_to_signal_crate() {
        let vig = SigTable::new(10, 20, 30, 40);
        let sig: SignalCrateTable = vig.into();
        assert_eq!(sig.a, 10u64);
        assert_eq!(sig.b, 20u64);
        assert_eq!(sig.c, 30u64);
        assert_eq!(sig.d, 40u64);
    }

    #[test]
    fn from_signal_crate_to_vigilance() {
        let sig = SignalCrateTable::new(5, 10, 15, 20);
        let t: SigTable = sig.into();
        assert_eq!(t.a, 5u64);
        assert_eq!(t.b, 10u64);
        assert_eq!(t.c, 15u64);
        assert_eq!(t.d, 20u64);
    }
}
