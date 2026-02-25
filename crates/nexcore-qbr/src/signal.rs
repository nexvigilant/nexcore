//! Signal strength extraction from contingency tables.
//!
//! Bridges QBR's `QbrSignalMethod` enum to the concrete signal detection
//! algorithms in `nexcore-pv-core`. Returns `Measured<f64>` with confidence
//! derived from the statistical properties of the result.
//!
//! Tier: T2-C (Cross-Domain Composite)
//! Grounding: N(Quantity) + κ(Comparison) + →(Causality)

use crate::error::QbrError;
use crate::types::QbrSignalMethod;
use nexcore_constants::{Confidence, Measured};
use nexcore_pv_core::signals::{ContingencyTable, SignalCriteria};

/// Extract signal strength as `Measured<f64>` from a contingency table.
///
/// Maps `QbrSignalMethod` to the appropriate algorithm and wraps the
/// result with confidence derived from the data quality (case count,
/// confidence interval width).
pub fn extract_signal_strength(
    table: &ContingencyTable,
    method: QbrSignalMethod,
) -> Result<Measured<f64>, QbrError> {
    let criteria = SignalCriteria::evans();

    match method {
        QbrSignalMethod::Prr => {
            let result = nexcore_pv_core::signals::calculate_prr(table, &criteria)
                .map_err(|e| QbrError::SignalDetection(e.to_string()))?;
            let confidence = signal_confidence(table, result.lower_ci, result.upper_ci);
            Ok(Measured::new(result.point_estimate, confidence))
        }
        QbrSignalMethod::Ror => {
            let result = nexcore_pv_core::signals::calculate_ror(table, &criteria)
                .map_err(|e| QbrError::SignalDetection(e.to_string()))?;
            let confidence = signal_confidence(table, result.lower_ci, result.upper_ci);
            Ok(Measured::new(result.point_estimate, confidence))
        }
        QbrSignalMethod::Ic => {
            let result = nexcore_pv_core::signals::calculate_ic(table, &criteria)
                .map_err(|e| QbrError::SignalDetection(e.to_string()))?;
            let confidence = signal_confidence(table, result.lower_ci, result.upper_ci);
            Ok(Measured::new(result.point_estimate, confidence))
        }
        QbrSignalMethod::Ebgm => {
            let result = nexcore_pv_core::signals::calculate_ebgm(table, &criteria)
                .map_err(|e| QbrError::SignalDetection(e.to_string()))?;
            let confidence = signal_confidence(table, result.lower_ci, result.upper_ci);
            Ok(Measured::new(result.point_estimate, confidence))
        }
    }
}

/// EBGM-specific extraction that also returns EB05 and EB95.
///
/// EB05 = lower 5th percentile (conservative estimate)
/// EB95 = upper 95th percentile (liberal estimate)
pub fn extract_ebgm_with_bounds(
    table: &ContingencyTable,
) -> Result<(Measured<f64>, f64, f64), QbrError> {
    let criteria = SignalCriteria::evans();
    let result = nexcore_pv_core::signals::calculate_ebgm(table, &criteria)
        .map_err(|e| QbrError::SignalDetection(e.to_string()))?;
    let confidence = signal_confidence(table, result.lower_ci, result.upper_ci);
    Ok((
        Measured::new(result.point_estimate, confidence),
        result.lower_ci, // EB05
        result.upper_ci, // EB95
    ))
}

/// Derive confidence from data quality indicators.
///
/// Factors:
/// - Case count (`a` cell): more cases = higher confidence
/// - CI width: narrower interval = higher confidence
fn signal_confidence(table: &ContingencyTable, ci_lower: f64, ci_upper: f64) -> Confidence {
    // Case count factor: asymptotic from 0 to 1 as cases increase
    let case_factor = 1.0 - 1.0 / (1.0 + table.a as f64 / 10.0);

    // CI width factor: narrower is better
    let ci_width = (ci_upper - ci_lower).abs();
    let ci_factor = if ci_width > 0.0 {
        1.0 / (1.0 + ci_width / 5.0)
    } else {
        0.5
    };

    // Geometric mean of factors
    let raw = (case_factor * ci_factor).sqrt().clamp(0.1, 0.99);
    Confidence::new(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(a: u64, b: u64, c: u64, d: u64) -> ContingencyTable {
        ContingencyTable::new(a, b, c, d)
    }

    #[test]
    fn test_extract_prr() {
        // a=15, b=85, c=5, d=95 → PRR should be > 1
        let table = make_table(15, 85, 5, 95);
        let result = extract_signal_strength(&table, QbrSignalMethod::Prr);
        assert!(result.is_ok());
        let measured = result.unwrap_or_else(|_| Measured::new(0.0, Confidence::NONE));
        assert!(
            measured.value > 1.0,
            "PRR should be > 1, got {}",
            measured.value
        );
        assert!(measured.confidence.value() > 0.0);
    }

    #[test]
    fn test_extract_ebgm_with_bounds() {
        let table = make_table(20, 80, 10, 90);
        let result = extract_ebgm_with_bounds(&table);
        assert!(result.is_ok());
        let (measured, eb05, eb95) =
            result.unwrap_or_else(|_| (Measured::new(0.0, Confidence::NONE), 0.0, 0.0));
        assert!(measured.value > 0.0);
        assert!(eb05 <= measured.value);
        assert!(eb95 >= measured.value);
    }

    #[test]
    fn test_confidence_increases_with_cases() {
        let small = make_table(3, 97, 3, 97);
        let large = make_table(50, 50, 10, 90);
        let c_small = signal_confidence(&small, 0.5, 5.0);
        let c_large = signal_confidence(&large, 1.5, 3.0);
        assert!(
            c_large.value() > c_small.value(),
            "More cases should yield higher confidence: {} vs {}",
            c_large.value(),
            c_small.value()
        );
    }
}
