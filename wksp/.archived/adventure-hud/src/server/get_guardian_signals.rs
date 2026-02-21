//! Server function for fetching Guardian signals

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianSignalInfo {
    pub id: String,
    pub pattern: String,
    pub severity: String,
    pub timestamp: String,
    pub verdict: Option<String>,
    pub probability: Option<f64>,
}

#[server]
pub async fn get_guardian_signals() -> Result<Vec<GuardianSignalInfo>, ServerFnError> {
    // In a real implementation, this would call the Guardian MCP or query the history buffer.
    // For the HUD visualization, we return sample data representing the PAMP/DAMP cascade.
    
    Ok(vec![
        GuardianSignalInfo {
            id: "sig-001".to_string(),
            pattern: "adversarial_prompt".to_string(),
            severity: "High".to_string(),
            timestamp: "2026-02-11T05:00:00Z".to_string(),
            verdict: Some("generated".to_string()),
            probability: Some(0.92),
        },
        GuardianSignalInfo {
            id: "sig-002".to_string(),
            pattern: "engram_drift".to_string(),
            severity: "Medium".to_string(),
            timestamp: "2026-02-11T05:02:00Z".to_string(),
            verdict: Some("generated".to_string()),
            probability: Some(0.78),
        },
        GuardianSignalInfo {
            id: "sig-003".to_string(),
            pattern: "code_pathogen".to_string(),
            severity: "Medium".to_string(),
            timestamp: "2026-02-11T05:05:00Z".to_string(),
            verdict: Some("human".to_string()),
            probability: Some(0.45),
        },
    ])
}
