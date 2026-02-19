//! Watchtower Monitoring & Telemetry Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Session analysis, telemetry statistics, and unified multi-agent monitoring.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for listing saved sessions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerSessionsListParams {}

/// Parameters for getting active sessions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerActiveSessionsParams {}

/// Parameters for analyzing a session
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerAnalyzeParams {
    /// Path to session log file
    #[serde(default)]
    pub session_path: Option<String>,
}

/// Parameters for getting telemetry statistics
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerTelemetryStatsParams {}

/// Parameters for getting recent log entries
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerRecentParams {
    /// Number of entries to return (default: 20)
    #[serde(default = "default_recent_count")]
    pub count: Option<usize>,
    /// Filter by session ID
    #[serde(default)]
    pub session_filter: Option<String>,
}

fn default_recent_count() -> Option<usize> {
    Some(20)
}

/// Parameters for symbol audit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerSymbolAuditParams {
    /// Path to file or directory to audit
    pub path: String,
    /// Confidence score for the digest
    #[serde(default)]
    pub confidence: Option<f64>,
}

// ============================================================================
// Monitoring Parameters
// ============================================================================

/// Parameters for monitoring alerts query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringAlertsParams {
    /// Filter by severity level (CRITICAL, HIGH, WARN, INFO).
    #[serde(default)]
    pub severity_filter: Option<String>,
    /// Maximum number of alerts to return
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for monitoring hook health query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringHookHealthParams {
    /// Specific hook name to analyze.
    #[serde(default)]
    pub hook_name: Option<String>,
}

/// Parameters for monitoring signal digest query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringSignalDigestParams {
    /// Time window in minutes
    #[serde(default)]
    pub window_minutes: Option<u64>,
}

/// Parameters for Gemini telemetry statistics
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerGeminiStatsParams {}

/// Parameters for getting recent Gemini calls
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerGeminiRecentParams {
    /// Number of recent entries to return (default: 20)
    #[serde(default = "default_gemini_recent_count")]
    pub count: usize,
}

fn default_gemini_recent_count() -> usize {
    20
}

/// Parameters for unified Claude + Gemini telemetry view
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerUnifiedParams {
    /// Include Claude Code telemetry (default: true)
    #[serde(default = "default_true")]
    pub include_claude: bool,
    /// Include Gemini telemetry (default: true)
    #[serde(default = "default_true")]
    pub include_gemini: bool,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Phase 4 Surveillance Parameters
// ============================================================================

/// Parameters for Phase 4 (post-market surveillance) tick.
///
/// Runs one SENSE-ANALYZE-DECIDE-RESPOND cycle across all monitoring signals,
/// CTVP events, drift detection, and hook health to produce a unified
/// surveillance verdict with auto-escalation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct Phase4SurveillanceTickParams {
    /// Capability ID to check (optional; checks all if omitted)
    #[serde(default)]
    pub capability_id: Option<String>,
    /// CAR value to record (0.0-1.0). If provided, records an observation.
    #[serde(default)]
    pub car_observation: Option<f64>,
    /// Time window in minutes for event correlation (default: 60)
    #[serde(default)]
    pub window_minutes: Option<u64>,
}
