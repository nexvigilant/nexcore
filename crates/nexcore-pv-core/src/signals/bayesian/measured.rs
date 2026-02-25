//! Measured<T> confidence wrappers for Bayesian signal detection methods.
//!
//! Wraps existing BCPNN, EBGM, IC, and Omega functions with `Measured<T>` from
//! `nexcore-constants`, providing confidence-propagating outputs without modifying
//! core algorithm code.
//!
//! # Confidence Mappings
//!
//! Each mapping is marked with `// CALIBRATION:` for searchability and future refinement.
//!
//! | Method | Confidence Source | Rationale |
//! |--------|-----------------|-----------|
//! | BCPNN  | CI width / 8.0  | IC range typically [-4, 4], narrow CI = high confidence |
//! | EBGM   | EB05/EBGM ratio | Close to 1.0 = narrow posterior = high confidence |
//! | IC     | CI width / 8.0  | Same scale as BCPNN (log2 information component) |
//! | Omega  | CI width / 8.0  | Same log2 scale as IC |
//!
//! # Grounding
//!
//! GroundsTo: →(Causality) + N(Quantity) + ∂(Boundary)
//! - → drives evidence update (prior → posterior)
//! - N carries the numerical estimate
//! - ∂ bounds the credible interval

use nexcore_constants::{Confidence, Measured};

use crate::signals::bayesian::bcpnn::calculate_bcpnn;
use crate::signals::bayesian::ebgm::calculate_ebgm;
use crate::signals::bayesian::ic::calculate_ic;
use crate::signals::bayesian::omega_shrinkage::{
    DDITable, OmegaConfig, OmegaResult, calculate_omega,
};
use crate::signals::core::error::SignalError;
use crate::signals::core::types::{ContingencyTable, SignalCriteria, SignalResult};

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED BCPNN
// ═══════════════════════════════════════════════════════════════════════════════

/// BCPNN result with Measured<SignalResult> confidence.
///
/// Wraps `calculate_bcpnn()` — does NOT modify the core BCPNN algorithm.
/// Confidence derived from credible interval width.
pub fn bcpnn_measured(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Measured<SignalResult>, SignalError> {
    let result = calculate_bcpnn(table, criteria)?;
    let confidence = bcpnn_confidence(&result);
    Ok(Measured::new(result, confidence))
}

/// Map BCPNN credible interval width to Confidence.
///
/// # CALIBRATION: BCPNN CI width → Confidence
///
/// `confidence = clamp(1.0 - (ic_upper - ic_lower) / 8.0, 0.05, 0.99)`
///
/// Rationale: IC credible interval width — narrow = high confidence, wide = low confidence.
/// Divisor 8.0: IC range is typically [-4, 4], so full range = 8.
fn bcpnn_confidence(result: &SignalResult) -> Confidence {
    // CALIBRATION: BCPNN CI width → Confidence
    let ci_width = result.upper_ci - result.lower_ci;
    Confidence::new((1.0 - ci_width / 8.0).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED EBGM
// ═══════════════════════════════════════════════════════════════════════════════

/// EBGM result with Measured<SignalResult> confidence.
///
/// Wraps `calculate_ebgm()` — does NOT modify the core EBGM/MGPS algorithm.
/// Confidence derived from EB05/EBGM posterior precision ratio.
pub fn ebgm_measured(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Measured<SignalResult>, SignalError> {
    let result = calculate_ebgm(table, criteria)?;
    let confidence = ebgm_confidence(&result);
    Ok(Measured::new(result, confidence))
}

/// Map EBGM posterior precision to Confidence.
///
/// # CALIBRATION: EBGM EB05/EBGM → Confidence
///
/// `confidence = clamp(eb05 / ebgm, 0.05, 0.99)`
///
/// Rationale: EB05/EBGM ratio approaches 1.0 when posterior is narrow (high confidence).
/// Ratio near 0 = wide posterior = low confidence.
fn ebgm_confidence(result: &SignalResult) -> Confidence {
    // CALIBRATION: EBGM EB05/EBGM ratio → Confidence
    let ratio = if result.point_estimate > 0.0 {
        result.lower_ci / result.point_estimate
    } else {
        0.05
    };
    Confidence::new(ratio.clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED IC
// ═══════════════════════════════════════════════════════════════════════════════

/// IC result with Measured<SignalResult> confidence.
///
/// Wraps `calculate_ic()` — does NOT modify the core IC algorithm.
/// Confidence derived from credible interval width (same scale as BCPNN).
pub fn ic_measured(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Measured<SignalResult>, SignalError> {
    let result = calculate_ic(table, criteria)?;
    let confidence = ic_confidence(&result);
    Ok(Measured::new(result, confidence))
}

/// Map IC credible interval width to Confidence.
///
/// # CALIBRATION: IC CI width → Confidence
///
/// `confidence = clamp(1.0 - (ic_upper - ic_lower) / 8.0, 0.05, 0.99)`
///
/// Rationale: IC shares the same log2 scale as BCPNN.
/// Same calibration formula applies.
fn ic_confidence(result: &SignalResult) -> Confidence {
    // CALIBRATION: IC CI width → Confidence
    let ci_width = result.upper_ci - result.lower_ci;
    Confidence::new((1.0 - ci_width / 8.0).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED OMEGA
// ═══════════════════════════════════════════════════════════════════════════════

/// Omega shrinkage result with Measured<OmegaResult> confidence.
///
/// Wraps `calculate_omega()` — does NOT modify the core Omega algorithm.
/// Confidence derived from credible interval width (log2 scale like IC).
///
/// Note: Omega uses `DDITable` (not `ContingencyTable`) because it models
/// drug-drug interactions requiring additional marginal counts.
pub fn omega_measured(
    table: &DDITable,
    config: &OmegaConfig,
) -> Result<Measured<OmegaResult>, SignalError> {
    let result = calculate_omega(table, config)?;
    let confidence = omega_confidence(&result);
    Ok(Measured::new(result, confidence))
}

/// Map Omega credible interval width to Confidence.
///
/// # CALIBRATION: Omega CI width → Confidence
///
/// `confidence = clamp(1.0 - (omega_upper - omega_lower) / 8.0, 0.05, 0.99)`
///
/// Rationale: Omega uses the same log2 scale as IC.
fn omega_confidence(result: &OmegaResult) -> Confidence {
    // CALIBRATION: Omega CI width → Confidence
    let ci_width = result.omega_upper - result.omega_lower;
    Confidence::new((1.0 - ci_width / 8.0).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn strong_signal_table() -> ContingencyTable {
        ContingencyTable::new(50, 50, 100, 9800)
    }

    fn weak_signal_table() -> ContingencyTable {
        ContingencyTable::new(3, 97, 300, 9600)
    }

    fn no_signal_table() -> ContingencyTable {
        ContingencyTable::new(10, 90, 1000, 8900)
    }

    fn large_sample_table() -> ContingencyTable {
        ContingencyTable::new(500, 500, 1000, 98000)
    }

    // ================================================================
    // Confidence bounds [0.05, 0.99]
    // ================================================================

    #[test]
    fn bcpnn_confidence_in_range() {
        for table in [
            strong_signal_table(),
            weak_signal_table(),
            no_signal_table(),
        ] {
            let result = bcpnn_measured(&table, &SignalCriteria::evans());
            if let Ok(m) = result {
                assert!(
                    m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                    "BCPNN confidence out of range: {}",
                    m.confidence.value()
                );
            }
        }
    }

    #[test]
    fn ebgm_confidence_in_range() {
        for table in [
            strong_signal_table(),
            weak_signal_table(),
            no_signal_table(),
        ] {
            let result = ebgm_measured(&table, &SignalCriteria::evans());
            if let Ok(m) = result {
                assert!(
                    m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                    "EBGM confidence out of range: {}",
                    m.confidence.value()
                );
            }
        }
    }

    #[test]
    fn ic_confidence_in_range() {
        for table in [
            strong_signal_table(),
            weak_signal_table(),
            no_signal_table(),
        ] {
            let result = ic_measured(&table, &SignalCriteria::evans());
            if let Ok(m) = result {
                assert!(
                    m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                    "IC confidence out of range: {}",
                    m.confidence.value()
                );
            }
        }
    }

    #[test]
    fn omega_confidence_in_range() {
        let table = DDITable::new(15, 100, 50, 40, 500, 400, 200, 10000);
        let config = OmegaConfig::default();
        let result = omega_measured(&table, &config);
        if let Ok(m) = result {
            assert!(
                m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                "Omega confidence out of range: {}",
                m.confidence.value()
            );
        }
    }

    // ================================================================
    // Strong signal → higher confidence than marginal
    // ================================================================

    #[test]
    fn bcpnn_strong_higher_confidence_than_weak() {
        let strong = bcpnn_measured(&strong_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let weak = bcpnn_measured(&weak_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        assert!(
            strong > weak,
            "Strong signal confidence ({strong}) should exceed weak ({weak})"
        );
    }

    #[test]
    fn ic_strong_higher_confidence_than_weak() {
        let strong = ic_measured(&strong_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let weak = ic_measured(&weak_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        assert!(
            strong > weak,
            "Strong IC confidence ({strong}) should exceed weak ({weak})"
        );
    }

    // ================================================================
    // Large samples → higher confidence
    // ================================================================

    #[test]
    fn bcpnn_large_sample_higher_confidence() {
        let large = bcpnn_measured(&large_sample_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let small = bcpnn_measured(&weak_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        assert!(
            large > small,
            "Large sample confidence ({large}) should exceed small ({small})"
        );
    }

    #[test]
    fn ic_large_sample_higher_confidence() {
        let large = ic_measured(&large_sample_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let small = ic_measured(&weak_signal_table(), &SignalCriteria::evans())
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        assert!(
            large > small,
            "Large IC confidence ({large}) should exceed small ({small})"
        );
    }

    // ================================================================
    // Consistency: all methods on same table produce comparable ranges
    // ================================================================

    #[test]
    fn all_methods_comparable_confidence_range() {
        let table = strong_signal_table();
        let criteria = SignalCriteria::evans();

        let bcpnn_conf = bcpnn_measured(&table, &criteria)
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let ebgm_conf = ebgm_measured(&table, &criteria)
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);
        let ic_conf = ic_measured(&table, &criteria)
            .ok()
            .map(|m| m.confidence.value())
            .unwrap_or(0.0);

        // All should be > 0.5 for a strong signal
        assert!(
            bcpnn_conf > 0.5,
            "BCPNN confidence {bcpnn_conf} should be > 0.5 for strong signal"
        );
        assert!(
            ic_conf > 0.5,
            "IC confidence {ic_conf} should be > 0.5 for strong signal"
        );
        // EBGM may have different calibration, but should be reasonable
        assert!(
            ebgm_conf > 0.1,
            "EBGM confidence {ebgm_conf} should be > 0.1 for strong signal"
        );
    }

    // ================================================================
    // Raw value preserved
    // ================================================================

    #[test]
    fn measured_preserves_raw_bcpnn() {
        let table = strong_signal_table();
        let criteria = SignalCriteria::evans();
        let raw = calculate_bcpnn(&table, &criteria).unwrap();
        let measured = bcpnn_measured(&table, &criteria).unwrap();
        assert_eq!(raw.point_estimate, measured.value.point_estimate);
        assert_eq!(raw.lower_ci, measured.value.lower_ci);
        assert_eq!(raw.upper_ci, measured.value.upper_ci);
        assert_eq!(raw.is_signal, measured.value.is_signal);
    }

    #[test]
    fn measured_preserves_raw_ebgm() {
        let table = strong_signal_table();
        let criteria = SignalCriteria::evans();
        let raw = calculate_ebgm(&table, &criteria).unwrap();
        let measured = ebgm_measured(&table, &criteria).unwrap();
        assert_eq!(raw.point_estimate, measured.value.point_estimate);
        assert_eq!(raw.is_signal, measured.value.is_signal);
    }

    #[test]
    fn measured_preserves_raw_ic() {
        let table = strong_signal_table();
        let criteria = SignalCriteria::evans();
        let raw = calculate_ic(&table, &criteria).unwrap();
        let measured = ic_measured(&table, &criteria).unwrap();
        assert_eq!(raw.point_estimate, measured.value.point_estimate);
        assert_eq!(raw.lower_ci, measured.value.lower_ci);
        assert_eq!(raw.upper_ci, measured.value.upper_ci);
    }

    #[test]
    fn measured_preserves_raw_omega() {
        let table = DDITable::new(15, 100, 50, 40, 500, 400, 200, 10000);
        let config = OmegaConfig::default();
        let raw = calculate_omega(&table, &config).unwrap();
        let measured = omega_measured(&table, &config).unwrap();
        assert_eq!(raw.omega, measured.value.omega);
        assert_eq!(raw.is_signal, measured.value.is_signal);
    }
}
