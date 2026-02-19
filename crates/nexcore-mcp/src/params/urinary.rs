//! Params for urinary system (waste management, telemetry pruning, session expiry) tools.

use serde::Deserialize;

/// Analyze telemetry pruning needs.
#[derive(Debug, Deserialize)]
pub struct UrinaryPruningParams {
    /// Directory or file path to analyze for pruning
    pub target_path: Option<String>,
    /// Max age in hours before items are pruning candidates
    pub max_age_hours: Option<u64>,
}

/// Check session expiry status.
#[derive(Debug, Deserialize)]
pub struct UrinaryExpiryParams {
    /// Session age in hours
    pub session_age_hours: Option<f64>,
}

/// Evaluate retention policy compliance.
#[derive(Debug, Deserialize)]
pub struct UrinaryRetentionParams {
    /// Category to evaluate (e.g., "telemetry", "artifacts", "logs", "sessions")
    pub category: String,
}

/// Get urinary system health overview.
#[derive(Debug, Deserialize)]
pub struct UrinaryHealthParams {}
