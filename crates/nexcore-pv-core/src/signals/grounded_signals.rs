//! Grounded signal detection — `Uncertain<T>`-returning wrappers.
//!
//! These functions wrap the core signal detection algorithms and return
//! `Uncertain<f64>` with confidence derived from statistical properties
//! of the result (CI width, case count).
//!
//! # T1 Primitive Grounding
//!
//! - `×(Product)`: Uncertain = value × confidence
//! - `N(Quantity)`: Confidence derived from case count and CI width
//! - `∂(Boundary)`: Confidence bands gate action thresholds

use grounded::{Confidence, ConfidenceBand, Uncertain};

use crate::signals::core::error::SignalError;
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalResult};
use crate::signals::{bayesian, disproportionality};

/// Derive confidence from signal result statistics.
///
/// Confidence is based on two factors:
/// 1. **Log-scale CI width** — for ratio statistics (PRR, ROR, EBGM), CIs are
///    computed as `exp(ln(est) ± Z*SE)`, so `ln(upper/lower) = 2*Z*SE` gives
///    the true precision independent of point estimate magnitude.
/// 2. **Case count** — more cases = more reliable
///
/// # CALIBRATION
///
/// Log CI width thresholds derived from pharmacoepidemiological practice:
/// - `ln(upper/lower) < 1.0` with n >= 10: High confidence (0.95)
///   (CI ratio < 2.7x — tight bounds)
/// - `ln(upper/lower) < 2.0` with n >= 5: Medium confidence (0.80)
///   (CI ratio < 7.4x — reasonable bounds)
/// - `ln(upper/lower) < 3.0` with n >= 3: Low confidence (0.50)
///   (CI ratio < 20x — wide but informative)
/// - Otherwise: Negligible confidence (0.30)
fn derive_confidence(result: &SignalResult) -> Confidence {
    // Use log-scale CI width: ln(upper/lower) = 2*Z*SE for ratio statistics.
    // This is invariant to point estimate magnitude, unlike (upper-lower)/estimate.
    let log_ci_width = if result.upper_ci > 0.0 && result.lower_ci > 0.0 {
        (result.upper_ci / result.lower_ci).ln()
    } else {
        f64::INFINITY
    };

    let raw = match (log_ci_width, result.case_count) {
        (w, n) if w < 1.0 && n >= 10 => 0.95,
        (w, n) if w < 2.0 && n >= 5 => 0.80,
        (w, n) if w < 3.0 && n >= 3 => 0.50,
        _ => 0.30,
    };

    // SAFETY: raw values are all in [0.0, 1.0] — new() cannot fail
    match Confidence::new(raw) {
        Ok(c) => c,
        Err(_) => Confidence::NONE,
    }
}

/// PRR with grounded uncertainty.
///
/// Returns `Uncertain<f64>` wrapping the PRR point estimate with
/// confidence derived from CI width and case count.
pub fn prr_uncertain(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Uncertain<f64>, SignalError> {
    let result = disproportionality::prr::calculate_prr(table, criteria)?;
    let confidence = derive_confidence(&result);
    Ok(Uncertain::with_provenance(
        result.point_estimate,
        confidence,
        "PRR disproportionality analysis",
    ))
}

/// ROR with grounded uncertainty.
///
/// Returns `Uncertain<f64>` wrapping the ROR point estimate with
/// confidence derived from CI width and case count.
pub fn ror_uncertain(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Uncertain<f64>, SignalError> {
    let result = disproportionality::ror::calculate_ror(table, criteria)?;
    let confidence = derive_confidence(&result);
    Ok(Uncertain::with_provenance(
        result.point_estimate,
        confidence,
        "ROR disproportionality analysis",
    ))
}

/// IC with grounded uncertainty.
///
/// Returns `Uncertain<f64>` wrapping the IC point estimate with
/// confidence derived from CI width and case count.
pub fn ic_uncertain(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Uncertain<f64>, SignalError> {
    let result = bayesian::ic::calculate_ic(table, criteria)?;
    let confidence = derive_confidence(&result);
    Ok(Uncertain::with_provenance(
        result.point_estimate,
        confidence,
        "IC Bayesian signal analysis",
    ))
}

/// EBGM with grounded uncertainty.
///
/// Returns `Uncertain<f64>` wrapping the EBGM point estimate with
/// confidence derived from CI width and case count.
pub fn ebgm_uncertain(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Uncertain<f64>, SignalError> {
    let result = bayesian::ebgm::calculate_ebgm(table, criteria)?;
    let confidence = derive_confidence(&result);
    Ok(Uncertain::with_provenance(
        result.point_estimate,
        confidence,
        "EBGM empirical Bayes analysis",
    ))
}

/// Evaluate all signal methods and return uncertain results.
///
/// Convenience function that runs PRR, ROR, IC, and EBGM with grounded
/// uncertainty. Returns a vec of (method_name, uncertain_result) pairs.
/// Methods that fail are silently skipped.
pub fn evaluate_all_uncertain(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Vec<(&'static str, Uncertain<f64>)> {
    let mut results = Vec::with_capacity(4);

    if let Ok(u) = prr_uncertain(table, criteria) {
        results.push(("PRR", u));
    }
    if let Ok(u) = ror_uncertain(table, criteria) {
        results.push(("ROR", u));
    }
    if let Ok(u) = ic_uncertain(table, criteria) {
        results.push(("IC", u));
    }
    if let Ok(u) = ebgm_uncertain(table, criteria) {
        results.push(("EBGM", u));
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prr_uncertain_high_confidence() {
        // Strong signal with many cases: a=15, b=100, c=5, d=1000
        let table = ContingencyTable::new(15, 100, 5, 1000);
        let criteria = SignalCriteria::evans();
        let result = prr_uncertain(&table, &criteria).unwrap();

        assert!(*result.value() > 1.0, "PRR should be positive");
        assert_eq!(result.provenance(), Some("PRR disproportionality analysis"));

        // With 15 cases and a well-defined ratio, confidence should be
        // at least Medium (0.80)
        assert!(
            result.confidence().value() >= 0.80,
            "15 cases should yield at least Medium confidence, got {}",
            result.confidence()
        );
    }

    #[test]
    fn test_prr_uncertain_low_confidence() {
        // Weak signal with few cases: a=3, b=1000, c=3, d=1000
        let table = ContingencyTable::new(3, 1000, 3, 1000);
        let criteria = SignalCriteria::evans();
        let result = prr_uncertain(&table, &criteria).unwrap();

        // PRR ~ 1.0 with only 3 cases — should have Low or Negligible confidence
        assert!(
            result.confidence().value() <= 0.80,
            "3 cases with PRR~1 should not yield Medium+ confidence, got {}",
            result.confidence()
        );
    }

    #[test]
    fn test_confidence_band_derivation() {
        // Test the derive_confidence function directly via signal results

        // High confidence scenario: many cases, narrow CI
        let table = ContingencyTable::new(50, 500, 100, 9350);
        let criteria = SignalCriteria::evans();
        let result = prr_uncertain(&table, &criteria).unwrap();
        let band = result.band();
        // 50 cases should produce at least Medium confidence
        assert!(
            band == ConfidenceBand::High || band == ConfidenceBand::Medium,
            "50 cases should produce High or Medium band, got {band:?}"
        );

        // Negligible confidence scenario: zero cases returns null result
        let table_zero = ContingencyTable::new(0, 500, 100, 9400);
        let result_zero = prr_uncertain(&table_zero, &criteria).unwrap();
        assert_eq!(
            result_zero.band(),
            ConfidenceBand::Negligible,
            "Zero-case PRR should have Negligible confidence"
        );
    }

    #[test]
    fn test_evaluate_all_uncertain() {
        let table = ContingencyTable::new(15, 100, 5, 1000);
        let criteria = SignalCriteria::evans();
        let results = evaluate_all_uncertain(&table, &criteria);

        // Should have results for all 4 methods
        assert_eq!(results.len(), 4, "Should return PRR, ROR, IC, EBGM");

        let names: Vec<&str> = results.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"PRR"));
        assert!(names.contains(&"ROR"));
        assert!(names.contains(&"IC"));
        assert!(names.contains(&"EBGM"));

        // All should have provenance
        for (name, uncertain) in &results {
            assert!(
                uncertain.provenance().is_some(),
                "{name} should have provenance"
            );
        }
    }

    #[test]
    fn test_cross_validation_vector() {
        // Cross-validation test vector: a=15, b=100, c=5, d=1000
        // This same vector will be used in the TypeScript tests (D2)
        let table = ContingencyTable::new(15, 100, 5, 1000);
        let criteria = SignalCriteria::evans();

        let prr = prr_uncertain(&table, &criteria).unwrap();
        let ror = ror_uncertain(&table, &criteria).unwrap();

        // PRR = (15/115) / (5/1005) ≈ 26.2
        assert!(
            (*prr.value() - 26.2).abs() < 1.0,
            "PRR should be approximately 26.2, got {}",
            prr.value()
        );

        // ROR = (15*1000) / (100*5) = 30.0
        assert!(
            (*ror.value() - 30.0).abs() < 1.0,
            "ROR should be approximately 30.0, got {}",
            ror.value()
        );
    }
}
