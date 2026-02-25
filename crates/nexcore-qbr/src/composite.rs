//! Form 3: Composite Weighted QBR.
//!
//! ```text
//! QBR_composite = Σ(w_i × signal_strength(benefit_i)) / Σ(w_j × signal_strength(risk_j))
//! ```
//!
//! Weights encode clinical significance. Death as outcome weighted differently
//! than nausea. Complete tumor response weighted differently than marginal
//! symptom improvement.
//!
//! Multi-hop confidence degradation is the feature, not a bug:
//! 5 benefit outcomes × 3 risk outcomes = 8 confidence products.
//! An honest QBR with wide confidence bounds is infinitely more valuable
//! than a precise number with hidden uncertainty.
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: κ(Comparison) + N(Quantity) + Σ(Sum) + →(Causality)

use crate::error::QbrError;
use crate::signal::extract_signal_strength;
use crate::types::QbrSignalMethod;
use nexcore_constants::{Confidence, Measured};
use nexcore_pv_core::signals::ContingencyTable;

/// Compute Form 3 Composite Weighted QBR.
///
/// Requires matching vectors of tables and weights for both sides.
pub fn compute_composite(
    benefit_tables: &[ContingencyTable],
    risk_tables: &[ContingencyTable],
    benefit_weights: &[Measured<f64>],
    risk_weights: &[Measured<f64>],
    method: QbrSignalMethod,
) -> Result<Measured<f64>, QbrError> {
    if benefit_tables.is_empty() {
        return Err(QbrError::NoBenefitTables);
    }
    if risk_tables.is_empty() {
        return Err(QbrError::NoRiskTables);
    }
    if benefit_weights.len() != benefit_tables.len() {
        return Err(QbrError::WeightMismatch {
            weights: benefit_weights.len(),
            tables: benefit_tables.len(),
        });
    }
    if risk_weights.len() != risk_tables.len() {
        return Err(QbrError::WeightMismatch {
            weights: risk_weights.len(),
            tables: risk_tables.len(),
        });
    }

    // Compute weighted benefit sum: Σ(w_i × signal(benefit_i))
    let mut benefit_sum = 0.0_f64;
    let mut benefit_confidence = Confidence::PERFECT;
    for (table, weight) in benefit_tables.iter().zip(benefit_weights.iter()) {
        let signal = extract_signal_strength(table, method)?;
        let weighted = weight.combine_with(signal, |w, s| w * s);
        benefit_sum += weighted.value;
        benefit_confidence = benefit_confidence.combine(weighted.confidence);
    }

    // Compute weighted risk sum: Σ(w_j × signal(risk_j))
    let mut risk_sum = 0.0_f64;
    let mut risk_confidence = Confidence::PERFECT;
    for (table, weight) in risk_tables.iter().zip(risk_weights.iter()) {
        let signal = extract_signal_strength(table, method)?;
        let weighted = weight.combine_with(signal, |w, s| w * s);
        risk_sum += weighted.value;
        risk_confidence = risk_confidence.combine(weighted.confidence);
    }

    if risk_sum.abs() < f64::EPSILON {
        return Err(QbrError::ZeroRiskSignal);
    }

    let ratio = benefit_sum / risk_sum;
    let combined_confidence = benefit_confidence.combine(risk_confidence);

    Ok(Measured::new(ratio, combined_confidence))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable::new(a, b, c, d)
    }

    fn weight(v: f64, c: f64) -> Measured<f64> {
        Measured::new(v, Confidence::new(c))
    }

    #[test]
    fn test_composite_single_pair() {
        // Single benefit + single risk = same as simple ratio * weight ratio
        let benefit = [make_table(20, 80, 5, 95)];
        let risk = [make_table(10, 90, 5, 95)];
        let bw = [weight(1.0, 0.9)];
        let rw = [weight(1.0, 0.9)];
        let result = compute_composite(&benefit, &risk, &bw, &rw, QbrSignalMethod::Prr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_composite_multiple_outcomes() {
        // 2 benefit + 2 risk outcomes
        let benefits = [
            make_table(25, 75, 5, 95), // strong benefit
            make_table(15, 85, 5, 95), // moderate benefit
        ];
        let risks = [
            make_table(8, 92, 5, 95),  // weak risk
            make_table(12, 88, 5, 95), // moderate risk
        ];
        let bw = [weight(3.0, 0.9), weight(1.0, 0.8)]; // tumor response weighted 3x
        let rw = [weight(1.0, 0.9), weight(2.0, 0.8)]; // severe AE weighted 2x

        let result = compute_composite(&benefits, &risks, &bw, &rw, QbrSignalMethod::Prr);
        assert!(result.is_ok());
        let qbr = result.unwrap_or_else(|_| Measured::new(0.0, Confidence::NONE));
        assert!(qbr.value > 0.0, "Composite QBR should be positive");
    }

    #[test]
    fn test_composite_confidence_degrades_with_more_endpoints() {
        // More endpoints = more confidence hops = lower confidence
        let tables_1 = [make_table(20, 80, 5, 95)];
        let tables_3 = [
            make_table(20, 80, 5, 95),
            make_table(15, 85, 5, 95),
            make_table(10, 90, 5, 95),
        ];
        let w1 = [weight(1.0, 0.9)];
        let w3 = [weight(1.0, 0.9), weight(1.0, 0.9), weight(1.0, 0.9)];
        let risk = [make_table(10, 90, 5, 95)];
        let rw = [weight(1.0, 0.9)];

        let r1 = compute_composite(&tables_1, &risk, &w1, &rw, QbrSignalMethod::Prr);
        let r3 = compute_composite(&tables_3, &risk, &w3, &rw, QbrSignalMethod::Prr);
        assert!(r1.is_ok());
        assert!(r3.is_ok());
        let c1 = r1.map(|m| m.confidence.value()).unwrap_or(0.0);
        let c3 = r3.map(|m| m.confidence.value()).unwrap_or(0.0);
        assert!(
            c3 < c1,
            "More endpoints should degrade confidence: {} vs {}",
            c3,
            c1
        );
    }

    #[test]
    fn test_composite_weight_mismatch() {
        let benefits = [make_table(20, 80, 5, 95)];
        let risks = [make_table(10, 90, 5, 95)];
        let bw = [weight(1.0, 0.9), weight(2.0, 0.8)]; // 2 weights for 1 table
        let rw = [weight(1.0, 0.9)];
        let result = compute_composite(&benefits, &risks, &bw, &rw, QbrSignalMethod::Prr);
        assert!(matches!(result, Err(QbrError::WeightMismatch { .. })));
    }

    #[test]
    fn test_composite_empty_benefit() {
        let risks = [make_table(10, 90, 5, 95)];
        let rw = [weight(1.0, 0.9)];
        let result = compute_composite(&[], &risks, &[], &rw, QbrSignalMethod::Prr);
        assert!(matches!(result, Err(QbrError::NoBenefitTables)));
    }
}
