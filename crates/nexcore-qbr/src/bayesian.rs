//! Form 2: Bayesian QBR (EBGM-based).
//!
//! ```text
//! QBR_bayes = EBGM_benefit / EBGM_risk
//! QBR_worst = EB05_benefit / EB95_risk  (most conservative)
//! ```
//!
//! Credibility intervals from EB05/EB95 on both sides.
//! The worst-case ratio (lower bound of benefit / upper bound of risk)
//! yields the most conservative defensible position.
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: κ(Comparison) + N(Quantity) + →(Causality) + ∂(Boundary)

use crate::error::QbrError;
use crate::signal::extract_ebgm_with_bounds;
use nexcore_constants::{Confidence, Measured};
use nexcore_pv_core::signals::ContingencyTable;

/// Bayesian QBR result with both primary and worst-case estimates.
#[derive(Debug, Clone)]
pub struct BayesianQbr {
    /// Primary: EBGM_benefit / EBGM_risk
    pub primary: Measured<f64>,
    /// Worst-case: EB05_benefit / EB95_risk (most conservative)
    pub worst_case: Measured<f64>,
    /// EB05 for benefit (lower credibility bound)
    pub benefit_eb05: f64,
    /// EB95 for risk (upper credibility bound)
    pub risk_eb95: f64,
}

/// Compute Form 2 Bayesian QBR using EBGM with credibility intervals.
pub fn compute_bayesian(
    benefit_table: &ContingencyTable,
    risk_table: &ContingencyTable,
) -> Result<BayesianQbr, QbrError> {
    let (benefit_measured, benefit_eb05, _benefit_eb95) = extract_ebgm_with_bounds(benefit_table)?;
    let (risk_measured, _risk_eb05, risk_eb95) = extract_ebgm_with_bounds(risk_table)?;

    if risk_measured.value.abs() < f64::EPSILON {
        return Err(QbrError::ZeroRiskSignal);
    }

    // Primary ratio: EBGM_benefit / EBGM_risk with confidence propagation
    let primary = benefit_measured.combine_with(risk_measured, |b, r| b / r);

    // Worst-case: EB05_benefit / EB95_risk
    // Uses the most conservative estimate from each side
    let worst_case_value = if risk_eb95.abs() < f64::EPSILON {
        return Err(QbrError::ZeroRiskSignal);
    } else {
        benefit_eb05 / risk_eb95
    };

    // Worst-case confidence is further degraded (we're using extreme bounds)
    // 20% penalty for using tail estimates instead of point estimates
    let degraded_confidence = Confidence::new(primary.confidence.value() * 0.8);
    let worst_case = Measured::new(worst_case_value, degraded_confidence);

    Ok(BayesianQbr {
        primary,
        worst_case,
        benefit_eb05,
        risk_eb95,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable::new(a, b, c, d)
    }

    #[test]
    fn test_bayesian_qbr_benefit_dominant() {
        let benefit = make_table(30, 70, 5, 95);
        let risk = make_table(8, 92, 5, 95);
        let result = compute_bayesian(&benefit, &risk);
        assert!(result.is_ok());
        let bqbr = result.unwrap_or_else(|_| BayesianQbr {
            primary: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            worst_case: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            benefit_eb05: 0.0,
            risk_eb95: 0.0,
        });
        assert!(bqbr.primary.value > 1.0, "EBGM benefit should dominate");
    }

    #[test]
    fn test_worst_case_more_conservative() {
        let benefit = make_table(20, 80, 5, 95);
        let risk = make_table(15, 85, 5, 95);
        let result = compute_bayesian(&benefit, &risk);
        assert!(result.is_ok());
        let bqbr = result.unwrap_or_else(|_| BayesianQbr {
            primary: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            worst_case: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            benefit_eb05: 0.0,
            risk_eb95: 0.0,
        });
        // Worst case should be <= primary (EB05 <= EBGM, EB95 >= EBGM)
        assert!(
            bqbr.worst_case.value <= bqbr.primary.value,
            "Worst case ({}) should be <= primary ({})",
            bqbr.worst_case.value,
            bqbr.primary.value
        );
    }

    #[test]
    fn test_bayesian_confidence_propagates() {
        let benefit = make_table(25, 75, 5, 95);
        let risk = make_table(10, 90, 5, 95);
        let result = compute_bayesian(&benefit, &risk);
        assert!(result.is_ok());
        let bqbr = result.unwrap_or_else(|_| BayesianQbr {
            primary: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            worst_case: Measured::new(0.0, nexcore_constants::Confidence::NONE),
            benefit_eb05: 0.0,
            risk_eb95: 0.0,
        });
        assert!(bqbr.primary.confidence.value() > 0.0);
        // Worst-case confidence should be lower than primary
        assert!(
            bqbr.worst_case.confidence.value() <= bqbr.primary.confidence.value(),
            "Worst-case confidence should be <= primary confidence"
        );
    }
}
