//! # Signal Stats
//!
//! Statistical algorithms for disproportionality signal detection.

//! Wraps `ContingencyTable` computations with full metric output.
//!
//! ## T1 Primitive: Mapping
//! `ContingencyTable → SignalMetrics` — pure transformation.
//!
//! ## Algorithms
//! - **PRR**: Proportional Reporting Ratio
//! - **ROR**: Reporting Odds Ratio
//! - **IC**: Information Component (Bayesian)
//! - **EBGM**: Empirical Bayesian Geometric Mean (simplified)
//! - **Chi-square**: With Yates correction

use crate::core::{ChiSquare, ContingencyTable, Ebgm, Ic, Prr, Ror, SignalStrength};

/// All computed metrics for a contingency table.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalMetrics {
    /// PRR value (None if denominator zero).
    pub prr: Option<Prr>,
    /// ROR value (None if denominator zero).
    pub ror: Option<Ror>,
    /// Information Component.
    pub ic: Ic,
    /// Simplified EBGM.
    pub ebgm: Ebgm,
    /// Chi-square statistic.
    pub chi_square: ChiSquare,
    /// Overall signal strength from PRR.
    pub strength: SignalStrength,
}

/// Compute all disproportionality metrics from a contingency table.
pub fn compute_all(table: &ContingencyTable) -> SignalMetrics {
    let prr = table.prr().map(Prr);
    let ror = table.ror().map(Ror);
    let ic = compute_ic(table);
    let ebgm = compute_ebgm(table);
    let chi_sq = ChiSquare(table.chi_square());
    let strength = prr.map_or(SignalStrength::None, |p| SignalStrength::from_prr(p.0));

    SignalMetrics {
        prr,
        ror,
        ic,
        ebgm,
        chi_square: chi_sq,
        strength,
    }
}

/// Information Component: IC = log2(observed / expected).
#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "u64->f64 cast is intentional for statistical computation; saturating ops prevent overflow"
)]
pub fn compute_ic(table: &ContingencyTable) -> Ic {
    let n = table.total() as f64;
    if n == 0.0 {
        return Ic(0.0);
    }
    let observed = table.a as f64;
    let expected =
        table.a.saturating_add(table.b) as f64 * table.a.saturating_add(table.c) as f64 / n;
    if expected == 0.0 || observed == 0.0 {
        return Ic(0.0);
    }
    let ic = (observed / expected).log2();
    if ic.is_finite() { Ic(ic) } else { Ic(0.0) }
}

/// Simplified EBGM: shrinkage estimate `(a + 0.5) / (E + 0.5)`.
#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "u64->f64 cast is intentional for statistical computation; saturating ops prevent overflow"
)]
pub fn compute_ebgm(table: &ContingencyTable) -> Ebgm {
    let n = table.total() as f64;
    if n == 0.0 {
        return Ebgm(0.0);
    }
    let expected =
        table.a.saturating_add(table.b) as f64 * table.a.saturating_add(table.c) as f64 / n;
    Ebgm((table.a as f64 + 0.5) / (expected + 0.5))
}

/// Batch compute metrics for multiple drug-event pairs.
pub fn compute_batch(
    tables: &[(crate::core::DrugEventPair, ContingencyTable)],
) -> Vec<(crate::core::DrugEventPair, SignalMetrics)> {
    tables
        .iter()
        .map(|(pair, table)| (pair.clone(), compute_all(table)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_all_metrics() {
        let table = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let m = compute_all(&table);
        assert!(m.prr.is_some());
        assert!(m.ror.is_some());
        assert!(
            m.chi_square.0 > 3.841,
            "chi-sq should exceed Evans threshold"
        );
        assert!(m.strength >= SignalStrength::Moderate);
    }

    #[test]
    fn ic_positive_for_signal() {
        let table = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let ic = compute_ic(&table);
        assert!(ic.0 > 0.0, "IC should be positive for a signal");
    }

    #[test]
    fn ebgm_above_one_for_signal() {
        let table = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let ebgm = compute_ebgm(&table);
        assert!(ebgm.0 > 1.0, "EBGM should be >1 for a signal");
    }

    #[test]
    fn zero_table_handled() {
        let table = ContingencyTable {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        };
        let m = compute_all(&table);
        assert_eq!(m.strength, SignalStrength::None);
    }
}
