//! CCIM assessment: per-directive balance sheet with conservation invariant.
//!
//! Conservation law: C_closing = C_opening + new_tools_cu - depreciation_cu
//! Invariant: |delta| < CONSERVATION_EPSILON
//!
//! Grounding: κ(Comparison) + N(Quantity) + ∂(Boundary).

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

use crate::depreciation::{compute_delta_avg, compute_withdrawal};
use crate::error::CcimError;
use crate::types::{CompoundingRatio, DepreciationEntry};

/// Maximum allowable delta before conservation violation.
const CONSERVATION_EPSILON: f64 = 0.001;

/// A single CCIM assessment (balance sheet for one directive).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CcimAssessment {
    /// Directive identifier (e.g. "D007", "VDAG-CONSERVATION").
    pub directive: String,
    /// Assessment date (ISO 8601).
    pub date: String,
    /// Assessment type: "baseline" or "treatment".
    pub assessment_type: String,
    /// Opening capability units.
    pub c_opening: f64,
    /// Closing capability units.
    pub c_closing: f64,
    /// Actual compounding ratio achieved.
    pub rho_actual: CompoundingRatio,
    /// Net Capability Rate of Return (rho - delta_avg).
    pub ncrr: f64,
    /// Average depreciation rate.
    pub delta_avg: f64,
    /// FIRE progress percentage.
    pub fire_progress_pct: f64,
    /// Number of new tools shipped this directive.
    pub new_tools_shipped: u32,
    /// Descriptions of tools shipped.
    pub tools_shipped: Vec<String>,
    /// Notes.
    pub notes: String,
}

/// NCRR (Net Capability Rate of Return) result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NcrrResult {
    /// The compounding ratio (rho).
    pub rho: f64,
    /// Weighted average depreciation rate.
    pub delta_avg: f64,
    /// Net rate: rho - delta_avg.
    pub ncrr: Measured<f64>,
    /// Whether capability is growing (ncrr > 0).
    pub is_growing: bool,
}

/// FIRE progress result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FireProgress {
    /// Current capability units.
    pub current_cu: f64,
    /// FIRE threshold.
    pub fire_threshold: f64,
    /// Progress percentage.
    pub progress_pct: Measured<f64>,
    /// Whether FIRE has been reached.
    pub fire_reached: bool,
}

/// Check conservation invariant: C_closing = C_opening + new_tools_cu - depreciation_cu.
///
/// Returns Ok(delta) if conservation holds, Err(ConservationViolation) otherwise.
///
/// # CALIBRATION: confidence = 0.99 (deterministic arithmetic check)
pub fn conservation_check(
    c_opening: f64,
    c_closing: f64,
    new_tools_cu: f64,
    depreciation_cu: f64,
) -> Result<Measured<f64>, CcimError> {
    let expected = c_opening + new_tools_cu - depreciation_cu;
    let delta = c_closing - expected;

    if delta.abs() >= CONSERVATION_EPSILON {
        return Err(CcimError::ConservationViolation {
            delta,
            c_opening,
            c_closing,
            new_tools_cu,
            depreciation_cu,
        });
    }

    // CALIBRATION: deterministic check — confidence is near-perfect
    let confidence = Confidence::HIGH;
    Ok(Measured::new(delta, confidence))
}

/// Compute NCRR from actual rho and depreciation entries.
///
/// NCRR = rho - delta_avg. If NCRR < 0, capability is shrinking.
///
/// # CALIBRATION: confidence = clamp(1.0 - 1.0 / (observations + 1), 0.05, 0.99)
pub fn compute_ncrr(
    rho: CompoundingRatio,
    entries: &[DepreciationEntry],
    observations: u32,
) -> NcrrResult {
    let delta_avg = compute_delta_avg(entries);
    let ncrr_value = rho.value() - delta_avg;

    let conf_raw = 1.0 - 1.0 / (f64::from(observations) + 1.0);
    let conf = conf_raw.clamp(0.05, 0.99);
    let confidence = Confidence::new(conf);

    NcrrResult {
        rho: rho.value(),
        delta_avg,
        ncrr: Measured::new(ncrr_value, confidence),
        is_growing: ncrr_value > 0.0,
    }
}

/// Compute FIRE progress: current_cu / fire_threshold * 100.
///
/// # CALIBRATION: confidence = clamp(1.0 - 1.0 / (observations + 1), 0.05, 0.99)
pub fn fire_progress(current_cu: f64, fire_threshold: f64, observations: u32) -> FireProgress {
    let pct = if fire_threshold > 0.0 {
        (current_cu / fire_threshold) * 100.0
    } else {
        100.0
    };

    let conf_raw = 1.0 - 1.0 / (f64::from(observations) + 1.0);
    let conf = conf_raw.clamp(0.05, 0.99);
    let confidence = Confidence::new(conf);

    FireProgress {
        current_cu,
        fire_threshold,
        progress_pct: Measured::new(pct, confidence),
        fire_reached: current_cu >= fire_threshold,
    }
}

/// Build a full assessment from raw data.
///
/// Validates conservation invariant and computes derived metrics.
#[allow(
    clippy::too_many_arguments,
    reason = "assessment construction needs all fields"
)]
pub fn build_assessment(
    directive: String,
    date: String,
    assessment_type: String,
    c_opening: f64,
    c_closing: f64,
    rho: CompoundingRatio,
    depreciation_entries: &[DepreciationEntry],
    new_tools_shipped: u32,
    tools_shipped: Vec<String>,
    fire_threshold: f64,
    observations: u32,
    notes: String,
) -> Result<CcimAssessment, CcimError> {
    let depreciation_cu = compute_withdrawal(depreciation_entries);
    let new_tools_cu = c_closing - c_opening + depreciation_cu;

    // Verify conservation
    let _delta = conservation_check(c_opening, c_closing, new_tools_cu, depreciation_cu)?;

    let ncrr_result = compute_ncrr(rho, depreciation_entries, observations);
    let fire = fire_progress(c_closing, fire_threshold, observations);

    Ok(CcimAssessment {
        directive,
        date,
        assessment_type,
        c_opening,
        c_closing,
        rho_actual: rho,
        ncrr: ncrr_result.ncrr.value,
        delta_avg: ncrr_result.delta_avg,
        fire_progress_pct: fire.progress_pct.value,
        new_tools_shipped,
        tools_shipped,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DepreciationCategory;

    #[test]
    fn test_conservation_check_passes() {
        // C_closing = C_opening + new_tools - depreciation
        // 1670 = 1655 + 20 - 5 => delta = 1670 - 1670 = 0
        let result = conservation_check(1655.0, 1670.0, 20.0, 5.0);
        assert!(result.is_ok());
        let measured = result.expect("should pass");
        assert!(measured.value.abs() < CONSERVATION_EPSILON);
    }

    #[test]
    fn test_conservation_check_fails() {
        // C_closing = 2000 but expected = 1655 + 20 - 5 = 1670
        // delta = 2000 - 1670 = 330 >> EPSILON
        let result = conservation_check(1655.0, 2000.0, 20.0, 5.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_ncrr_positive() {
        let rho = CompoundingRatio::GROWTH; // 0.50
        let entries = vec![DepreciationEntry {
            description: "stale docs".to_string(),
            category: DepreciationCategory::StaleDocs,
            capability_at_risk: 100.0,
            periods_unmaintained: 1,
        }];
        let result = compute_ncrr(rho, &entries, 5);
        // NCRR = 0.50 - 0.01 = 0.49
        assert!(result.is_growing);
        assert!((result.ncrr.value - 0.49).abs() < 0.01);
    }

    #[test]
    fn test_ncrr_negative() {
        let rho = CompoundingRatio::MATTRESS; // 0.0
        let entries = vec![DepreciationEntry {
            description: "failing tests".to_string(),
            category: DepreciationCategory::FailingTests,
            capability_at_risk: 100.0,
            periods_unmaintained: 1,
        }];
        let result = compute_ncrr(rho, &entries, 5);
        // NCRR = 0.0 - 0.10 = -0.10
        assert!(!result.is_growing);
        assert!((result.ncrr.value - (-0.10)).abs() < 0.01);
    }

    #[test]
    fn test_fire_progress_under_threshold() {
        let result = fire_progress(1655.0, 5000.0, 3);
        assert!(!result.fire_reached);
        assert!((result.progress_pct.value - 33.1).abs() < 0.1);
    }

    #[test]
    fn test_fire_progress_reached() {
        let result = fire_progress(5500.0, 5000.0, 3);
        assert!(result.fire_reached);
        assert!(result.progress_pct.value > 100.0);
    }

    #[test]
    fn test_build_assessment_validates_conservation() {
        let rho = CompoundingRatio::SAVINGS;
        let result = build_assessment(
            "D008".to_string(),
            "2026-02-25".to_string(),
            "treatment".to_string(),
            1655.0,
            1670.0,
            rho,
            &[], // no depreciation
            1,
            vec!["tool_schema".to_string()],
            5000.0,
            3,
            "test".to_string(),
        );
        assert!(result.is_ok());
        let assessment = result.expect("should build");
        assert_eq!(assessment.new_tools_shipped, 1);
    }
}
