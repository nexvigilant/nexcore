//! Form 1: Simple Ratio QBR.
//!
//! ```text
//! QBR_simple = signal_strength(benefit) / signal_strength(risk)
//! ```
//!
//! QBR > 1.0 means benefit signal outweighs harm signal.
//! Both numerator and denominator are `Measured<T>`, so the ratio
//! inherits propagated confidence via product rule.
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: κ(Comparison) + N(Quantity) + →(Causality)

use crate::error::QbrError;
use crate::signal::extract_signal_strength;
use crate::types::QbrSignalMethod;
use nexcore_constants::Measured;
use nexcore_pv_core::signals::ContingencyTable;

/// Compute Form 1 Simple Ratio QBR.
///
/// Uses the first benefit table and first risk table.
/// Signal strength is extracted using the specified method.
pub fn compute_simple(
    benefit_table: &ContingencyTable,
    risk_table: &ContingencyTable,
    method: QbrSignalMethod,
) -> Result<Measured<f64>, QbrError> {
    let benefit_signal = extract_signal_strength(benefit_table, method)?;
    let risk_signal = extract_signal_strength(risk_table, method)?;

    if risk_signal.value.abs() < f64::EPSILON {
        return Err(QbrError::ZeroRiskSignal);
    }

    Ok(benefit_signal.combine_with(risk_signal, |b, r| b / r))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable::new(a, b, c, d)
    }

    #[test]
    fn test_simple_ratio_benefit_dominant() {
        // Benefit: strong association (a=30, b=70, c=5, d=95)
        // Risk: weak association (a=8, b=92, c=5, d=95)
        let benefit = make_table(30, 70, 5, 95);
        let risk = make_table(8, 92, 5, 95);
        let result = compute_simple(&benefit, &risk, QbrSignalMethod::Prr);
        assert!(result.is_ok());
        let qbr =
            result.unwrap_or_else(|_| Measured::new(0.0, nexcore_constants::Confidence::NONE));
        assert!(
            qbr.value > 1.0,
            "Benefit-dominant QBR should be > 1.0, got {}",
            qbr.value
        );
    }

    #[test]
    fn test_simple_ratio_risk_dominant() {
        // Benefit: weak association
        // Risk: strong association
        let benefit = make_table(5, 95, 5, 95);
        let risk = make_table(30, 70, 5, 95);
        let result = compute_simple(&benefit, &risk, QbrSignalMethod::Prr);
        assert!(result.is_ok());
        let qbr =
            result.unwrap_or_else(|_| Measured::new(999.0, nexcore_constants::Confidence::NONE));
        assert!(
            qbr.value < 1.0,
            "Risk-dominant QBR should be < 1.0, got {}",
            qbr.value
        );
    }

    #[test]
    fn test_simple_ratio_confidence_propagates() {
        let benefit = make_table(20, 80, 5, 95);
        let risk = make_table(15, 85, 5, 95);
        let result = compute_simple(&benefit, &risk, QbrSignalMethod::Prr);
        assert!(result.is_ok());
        let qbr =
            result.unwrap_or_else(|_| Measured::new(0.0, nexcore_constants::Confidence::NONE));
        // Confidence should be < both individual confidences (product rule)
        assert!(
            qbr.confidence.value() > 0.0 && qbr.confidence.value() < 1.0,
            "Confidence should be in (0, 1), got {}",
            qbr.confidence.value()
        );
    }

    #[test]
    fn test_simple_ratio_ror_method() {
        let benefit = make_table(25, 75, 5, 95);
        let risk = make_table(10, 90, 5, 95);
        let result = compute_simple(&benefit, &risk, QbrSignalMethod::Ror);
        assert!(result.is_ok());
        let qbr =
            result.unwrap_or_else(|_| Measured::new(0.0, nexcore_constants::Confidence::NONE));
        assert!(
            qbr.value > 1.0,
            "ROR-based QBR should also show benefit > risk"
        );
    }
}
