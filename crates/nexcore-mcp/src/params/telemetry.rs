//! Telemetry Intelligence Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Telemetry source analysis, intelligence report generation, and evolution tracking.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for listing telemetry sources
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySourcesListParams {}

/// Parameters for analyzing a telemetry source
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySourceAnalyzeParams {
    /// Path to session file
    #[serde(default)]
    pub session_path: Option<String>,
    /// Project hash to find latest session
    #[serde(default)]
    pub project_hash: Option<String>,
}

/// Parameters for governance cross-reference
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryGovernanceCrossrefParams {
    /// Filter by category: "primitives", "governance", "capabilities", "constitutional"
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for snapshot evolution tracking
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySnapshotEvolutionParams {
    /// Session ID to get snapshots for
    #[serde(default)]
    pub session_id: Option<String>,
    /// Maximum number of sessions to return
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for intelligence report generation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryIntelReportParams {
    /// Maximum recent activity entries
    #[serde(default)]
    pub activity_limit: Option<usize>,
    /// Maximum file patterns to return
    #[serde(default)]
    pub file_limit: Option<usize>,
}

/// Parameters for recent telemetry activity
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryRecentParams {
    /// Number of recent operations to return
    #[serde(default)]
    pub count: Option<usize>,
}

/// Parameters for telemetry summary query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySummaryParams {
    #[serde(default)]
    pub _unused: Option<()>,
}

/// Parameters for per-tool telemetry query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryByToolParams {
    /// Tool name to query
    pub tool_name: String,
}

/// Parameters for slow calls query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySlowCallsParams {
    /// Duration threshold in milliseconds
    pub threshold_ms: u64,
    /// Maximum number of results
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for audit trail query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AuditTrailParams {
    /// Filter by tool name
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Only return records after this ISO-8601 timestamp
    #[serde(default)]
    pub since: Option<String>,
    /// Filter by success status
    #[serde(default)]
    pub success_only: Option<bool>,
    /// Maximum number of results
    #[serde(default)]
    pub limit: Option<usize>,
}
