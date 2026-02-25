//! CCIM state file I/O.
//!
//! Reads and writes the ccim-assessments.json state file format.
//! Grounding: π(Persistence) + μ(Mapping) + ∂(Boundary).

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::CcimError;

/// Top-level state file structure matching ccim-assessments.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CcimState {
    /// Schema version (e.g. "1.1.0").
    pub version: String,
    /// Baseline measurement.
    pub baseline: BaselineRecord,
    /// Chronological assessment records.
    pub assessments: Vec<AssessmentRecord>,
    /// Trial tracking data.
    pub trial: Option<TrialRecord>,
}

/// Baseline capability measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BaselineRecord {
    /// Directive identifier.
    pub directive: String,
    /// Date of measurement (ISO 8601).
    pub date: String,
    /// Opening capability units.
    #[serde(rename = "C_opening")]
    pub c_opening: f64,
    /// Closing capability units.
    #[serde(rename = "C_closing")]
    pub c_closing: f64,
    /// Actual compounding ratio.
    pub rho_actual: f64,
    /// Target compounding ratio.
    pub rho_target: f64,
    /// Net capability rate of return.
    pub ncrr: f64,
    /// Weighted average depreciation rate.
    pub delta_avg: f64,
    /// FIRE threshold in CU.
    pub fire_target: f64,
    /// FIRE progress percentage.
    pub fire_progress_pct: f64,
    /// Count of new tools shipped.
    pub new_tools_shipped: u32,
    /// Depreciation items halted.
    pub depreciation_halted: Vec<String>,
    /// Superiority evidence entries.
    pub superiority_evidence: Vec<String>,
    /// Notes.
    pub notes: String,
}

/// A single assessment entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssessmentRecord {
    /// Directive identifier.
    pub directive: String,
    /// Date (ISO 8601).
    pub date: String,
    /// Type: "baseline" or "treatment".
    #[serde(rename = "type")]
    pub assessment_type: String,
    /// Opening capability units.
    #[serde(rename = "C_opening")]
    pub c_opening: f64,
    /// Closing capability units.
    #[serde(rename = "C_closing")]
    pub c_closing: f64,
    /// Actual compounding ratio achieved.
    pub rho_actual: f64,
    /// NCRR value.
    pub ncrr: f64,
    /// FIRE progress percentage.
    pub fire_progress_pct: f64,
    /// Notes about this assessment.
    pub notes: String,
    /// Average depreciation rate (optional — baseline may omit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_avg: Option<f64>,
    /// New tools shipped count (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_tools_shipped: Option<u32>,
    /// List of tools shipped (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_shipped: Option<Vec<String>>,
    /// Depreciation items halted (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depreciation_halted: Option<Vec<String>>,
    /// Superiority evidence (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superiority_evidence: Option<Vec<String>>,
    /// Sovereign tool invocations (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sovereign_invocations: Option<u32>,
    /// Total analysis tasks (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_analysis_tasks: Option<u32>,
    /// Conservation findings (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conservation_findings: Option<serde_json::Value>,
}

/// Trial tracking record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrialRecord {
    /// Protocol identifier.
    pub protocol_id: String,
    /// Hypothesis statement.
    pub hypothesis: String,
    /// Current interim verdict.
    pub interim_verdict: String,
    /// First treatment rho.
    pub first_treatment_rho: f64,
    /// Second treatment rho (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_treatment_rho: Option<f64>,
    /// Cumulative rho.
    pub cumulative_rho: f64,
    /// Number of observations.
    pub observations: u32,
    /// Planned observation count.
    pub planned: u32,
}

/// Load CCIM state from a JSON file.
pub fn load_state(path: &Path) -> Result<CcimState, CcimError> {
    if !path.exists() {
        return Err(CcimError::StateFileNotFound(path.display().to_string()));
    }

    let contents = std::fs::read_to_string(path)?;
    let state: CcimState = serde_json::from_str(&contents)?;
    Ok(state)
}

/// Save CCIM state to a JSON file.
pub fn save_state(path: &Path, state: &CcimState) -> Result<(), CcimError> {
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Get the latest assessment from state.
#[must_use]
pub fn latest_assessment(state: &CcimState) -> Option<&AssessmentRecord> {
    state.assessments.last()
}

/// Get the current capability units from the latest assessment.
#[must_use]
pub fn current_cu(state: &CcimState) -> f64 {
    state
        .assessments
        .last()
        .map_or(state.baseline.c_closing, |a| a.c_closing)
}

/// Count total observations across all assessments.
#[must_use]
pub fn total_observations(state: &CcimState) -> u32 {
    state.trial.as_ref().map_or(
        u32::try_from(state.assessments.len()).unwrap_or(u32::MAX),
        |t| t.observations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> CcimState {
        CcimState {
            version: "1.1.0".to_string(),
            baseline: BaselineRecord {
                directive: "D007".to_string(),
                date: "2026-02-24".to_string(),
                c_opening: 1655.0,
                c_closing: 1655.0,
                rho_actual: 0.0,
                rho_target: 0.15,
                ncrr: -0.025,
                delta_avg: 0.025,
                fire_target: 5000.0,
                fire_progress_pct: 33.1,
                new_tools_shipped: 0,
                depreciation_halted: vec![],
                superiority_evidence: vec![],
                notes: "baseline".to_string(),
            },
            assessments: vec![AssessmentRecord {
                directive: "D007".to_string(),
                date: "2026-02-24".to_string(),
                assessment_type: "baseline".to_string(),
                c_opening: 1049.0,
                c_closing: 1655.0,
                rho_actual: 0.0,
                ncrr: -0.025,
                fire_progress_pct: 33.1,
                notes: "baseline assessment".to_string(),
                delta_avg: None,
                new_tools_shipped: None,
                tools_shipped: None,
                depreciation_halted: None,
                superiority_evidence: None,
                sovereign_invocations: None,
                total_analysis_tasks: None,
                conservation_findings: None,
            }],
            trial: Some(TrialRecord {
                protocol_id: "TRIAL-2026-0224-SELFUSE".to_string(),
                hypothesis: "Self-Use increases rho".to_string(),
                interim_verdict: "POSITIVE".to_string(),
                first_treatment_rho: 0.875,
                second_treatment_rho: Some(0.875),
                cumulative_rho: 0.875,
                observations: 2,
                planned: 3,
            }),
        }
    }

    #[test]
    fn test_current_cu_from_latest_assessment() {
        let state = sample_state();
        assert!((current_cu(&state) - 1655.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_latest_assessment() {
        let state = sample_state();
        let latest = latest_assessment(&state);
        assert!(latest.is_some());
        assert_eq!(
            latest.map(|a| &a.directive).unwrap_or(&String::new()),
            "D007"
        );
    }

    #[test]
    fn test_total_observations_from_trial() {
        let state = sample_state();
        assert_eq!(total_observations(&state), 2);
    }

    #[test]
    fn test_load_state_file_not_found() {
        let result = load_state(Path::new("/nonexistent/path.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_state() {
        let state = sample_state();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test-state.json");

        save_state(&path, &state).expect("save");
        let loaded = load_state(&path).expect("load");

        assert_eq!(loaded.version, "1.1.0");
        assert_eq!(loaded.assessments.len(), 1);
        assert!((current_cu(&loaded) - 1655.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_load_actual_state_file() {
        let path = Path::new("/home/matthew/.claude/hooks/state/ccim-assessments.json");
        if path.exists() {
            let state = load_state(path).expect("should parse actual state file");
            assert_eq!(state.version, "1.1.0");
            assert!(state.assessments.len() >= 2);
        }
    }
}
