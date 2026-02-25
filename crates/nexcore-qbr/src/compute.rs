//! Unified QBR computation entry point.
//!
//! Orchestrates all four forms from a single `BenefitRiskInput`.
//! Forms are computed when their required inputs are present:
//! - Form 1 (Simple): always computed (requires at least 1 benefit + 1 risk table)
//! - Form 2 (Bayesian): computed when method is EBGM
//! - Form 3 (Composite): computed when weights are provided
//! - Form 4 (Therapeutic Window): computed when Hill parameters are provided
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: →(Causality) + N(Quantity) + κ(Comparison) + ∂(Boundary) + Σ(Sum)

use crate::bayesian;
use crate::composite;
use crate::error::QbrError;
use crate::signal::extract_signal_strength;
use crate::simple;
use crate::therapeutic_window;
use crate::types::{BenefitRiskInput, IntegrationBounds, QBR, QbrMethodDetails, QbrSignalMethod};

/// Compute full QBR from a `BenefitRiskInput`.
///
/// Returns all applicable forms based on available inputs.
pub fn compute_qbr(input: &BenefitRiskInput) -> Result<QBR, QbrError> {
    // Validate minimum inputs
    if input.benefit_tables.is_empty() {
        return Err(QbrError::NoBenefitTables);
    }
    if input.risk_tables.is_empty() {
        return Err(QbrError::NoRiskTables);
    }

    let primary_benefit = &input.benefit_tables[0];
    let primary_risk = &input.risk_tables[0];

    // ── Form 1: Simple Ratio (always computed) ──────────────────────────
    let simple_result = simple::compute_simple(primary_benefit, primary_risk, input.method)?;

    // Extract signal details for audit trail
    let benefit_signal = extract_signal_strength(primary_benefit, input.method)?;
    let risk_signal = extract_signal_strength(primary_risk, input.method)?;

    // ── Form 2: Bayesian (only with EBGM method) ───────────────────────
    let (bayesian_result, benefit_eb05, risk_eb95, worst_case) =
        if input.method == QbrSignalMethod::Ebgm {
            match bayesian::compute_bayesian(primary_benefit, primary_risk) {
                Ok(bqbr) => (
                    Some(bqbr.primary),
                    Some(bqbr.benefit_eb05),
                    Some(bqbr.risk_eb95),
                    Some(bqbr.worst_case),
                ),
                Err(_) => (None, None, None, None),
            }
        } else {
            (None, None, None, None)
        };

    // ── Form 3: Composite Weighted (only with weights) ──────────────────
    let composite_result = match (&input.benefit_weights, &input.risk_weights) {
        (Some(bw), Some(rw)) => composite::compute_composite(
            &input.benefit_tables,
            &input.risk_tables,
            bw,
            rw,
            input.method,
        )
        .ok(),
        _ => None,
    };

    // ── Form 4: Therapeutic Window (only with Hill params) ──────────────
    let window_result = match (&input.hill_efficacy, &input.hill_toxicity) {
        (Some(eff), Some(tox)) => {
            let bounds = input
                .integration_bounds
                .as_ref()
                .cloned()
                .unwrap_or(IntegrationBounds {
                    dose_min: 0.1,
                    dose_max: 100.0,
                    intervals: 1000,
                });
            therapeutic_window::compute_therapeutic_window(eff, tox, &bounds).ok()
        }
        _ => None,
    };

    Ok(QBR {
        simple: simple_result,
        bayesian: bayesian_result,
        composite: composite_result,
        therapeutic_window: window_result,
        details: QbrMethodDetails {
            benefit_signal,
            risk_signal,
            benefit_eb05,
            risk_eb95,
            worst_case_bayesian: worst_case,
            method: input.method,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HillCurveParams, IntegrationBounds};
    use nexcore_constants::{Confidence, Measured};
    use nexcore_pv_core::signals::ContingencyTable;

    fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable::new(a, b, c, d)
    }

    fn weight(v: f64, c: f64) -> Measured<f64> {
        Measured::new(v, Confidence::new(c))
    }

    #[test]
    fn test_compute_qbr_minimal() {
        // Minimal input: 1 benefit table + 1 risk table
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(20, 80, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(qbr.simple.value > 0.0);
        assert!(qbr.bayesian.is_none()); // PRR method → no Bayesian
        assert!(qbr.composite.is_none()); // no weights
        assert!(qbr.therapeutic_window.is_none()); // no Hill params
    }

    #[test]
    fn test_compute_qbr_ebgm_enables_bayesian() {
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(25, 75, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Ebgm,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(
            qbr.bayesian.is_some(),
            "EBGM method should produce Bayesian form"
        );
    }

    #[test]
    fn test_compute_qbr_with_weights() {
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(20, 80, 5, 95), make_table(15, 85, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: Some(vec![weight(3.0, 0.9), weight(1.0, 0.8)]),
            risk_weights: Some(vec![weight(1.0, 0.9)]),
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(
            qbr.composite.is_some(),
            "Weights provided → composite should exist"
        );
    }

    #[test]
    fn test_compute_qbr_with_hill() {
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(20, 80, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: Some(HillCurveParams {
                k_half: 10.0,
                n_hill: 2.0,
            }),
            hill_toxicity: Some(HillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            }),
            integration_bounds: Some(IntegrationBounds {
                dose_min: 0.1,
                dose_max: 100.0,
                intervals: 1000,
            }),
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(
            qbr.therapeutic_window.is_some(),
            "Hill params provided → therapeutic window should exist"
        );
        let tw = qbr
            .therapeutic_window
            .unwrap_or_else(|| Measured::new(0.0, Confidence::NONE));
        assert!(tw.value > 0.0, "EC50=10, TC50=80 → positive window");
    }

    #[test]
    fn test_compute_qbr_all_forms() {
        // Full input: all four forms should be computed
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(25, 75, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: Some(vec![weight(2.0, 0.9)]),
            risk_weights: Some(vec![weight(1.0, 0.9)]),
            hill_efficacy: Some(HillCurveParams {
                k_half: 10.0,
                n_hill: 2.0,
            }),
            hill_toxicity: Some(HillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            }),
            integration_bounds: Some(IntegrationBounds {
                dose_min: 0.1,
                dose_max: 100.0,
                intervals: 1000,
            }),
            method: QbrSignalMethod::Ebgm,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(qbr.simple.value > 0.0, "Form 1 present");
        assert!(qbr.bayesian.is_some(), "Form 2 present");
        assert!(qbr.composite.is_some(), "Form 3 present");
        assert!(qbr.therapeutic_window.is_some(), "Form 4 present");
    }

    #[test]
    fn test_compute_qbr_no_benefit() {
        let input = BenefitRiskInput {
            benefit_tables: vec![],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(matches!(result, Err(QbrError::NoBenefitTables)));
    }

    #[test]
    fn test_compute_qbr_no_risk() {
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(20, 80, 5, 95)],
            risk_tables: vec![],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(matches!(result, Err(QbrError::NoRiskTables)));
    }

    #[test]
    fn test_audit_trail_populated() {
        let input = BenefitRiskInput {
            benefit_tables: vec![make_table(20, 80, 5, 95)],
            risk_tables: vec![make_table(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: QbrSignalMethod::Prr,
        };
        let result = compute_qbr(&input);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| panic!("QBR computation failed"));
        assert!(qbr.details.benefit_signal.value > 0.0);
        assert!(qbr.details.risk_signal.value > 0.0);
        assert_eq!(qbr.details.method, QbrSignalMethod::Prr);
    }
}
