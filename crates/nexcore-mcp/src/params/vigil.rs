//! Vigil System Parameters (Persistence-Boundary Frequency)
//! Tier: T1-T2 (Regulatory Monitoring)
//!
//! Daemon management, boundary specifications, ledger queries, and orchestrator control.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// Vigil System (vigil_system.rs) Parameters
// ============================================================================

/// Parameters for starting the vigilance daemon.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysStartParams {
    /// Path to TOML config file.
    #[serde(default)]
    pub config_path: Option<String>,
}

/// Parameters for adding a boundary specification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysAddBoundaryParams {
    /// Name for the boundary.
    pub name: String,
    /// Threshold type: "always", "severity", "count".
    pub threshold_type: String,
    /// Source filter.
    #[serde(default)]
    pub source_filter: Option<String>,
    /// Severity level.
    #[serde(default)]
    pub severity: Option<String>,
    /// Count for threshold.
    #[serde(default)]
    pub count: Option<u64>,
    /// Window in ms.
    #[serde(default)]
    pub window_ms: Option<u64>,
    /// Cooldown in ms.
    #[serde(default)]
    pub cooldown_ms: Option<u64>,
}

/// Parameters for querying ledger entries.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysLedgerQueryParams {
    /// Entry type filter.
    #[serde(default)]
    pub entry_type: Option<String>,
    /// Entries after this timestamp.
    #[serde(default)]
    pub since: Option<u64>,
    /// Maximum entries.
    #[serde(default)]
    pub limit: Option<u64>,
}

// ============================================================================
// Vigil Orchestrator (vigil.rs) Parameters
// ============================================================================

/// Parameters for vigil status query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilStatusParams {}

/// Parameters for vigil health check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilHealthParams {}

/// Parameters for emitting an event to the EventBus.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilEmitEventParams {
    /// Event source identifier.
    pub source: String,
    /// Event type.
    pub event_type: String,
    /// Event payload (arbitrary JSON).
    pub payload: serde_json::Value,
    /// Priority level.
    #[serde(default)]
    pub priority: Option<String>,
}

/// Parameters for memory search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilMemorySearchParams {
    /// Search query.
    pub query: String,
    /// Maximum results.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for memory stats.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilMemoryStatsParams {}

/// Parameters for LLM usage stats.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilLlmStatsParams {}

/// Parameters for source control (enable/disable/configure sources).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSourceControlParams {
    /// Source name.
    pub source: String,
    /// Action: "enable", "disable", "configure", "status".
    pub action: String,
    /// Optional configuration payload.
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

/// Parameters for executor control (manage LLM executors).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilExecutorControlParams {
    /// Action: "enable", "disable", "set-primary", "configure", "status".
    pub action: String,
    /// Provider name.
    #[serde(default)]
    pub provider: Option<String>,
    /// Complexity thresholds configuration.
    #[serde(default)]
    pub complexity_thresholds: Option<serde_json::Value>,
}

/// Parameters for authority/rule configuration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilAuthorityConfigParams {
    /// Action: "add-rule", "remove-rule", "list-rules", "set-mode".
    pub action: String,
    /// Rule type.
    #[serde(default)]
    pub rule_type: Option<String>,
    /// Rule value/config.
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Parameters for decision confidence estimation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilDecisionConfidenceParams {}

/// Parameters for memory persistence.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilMemoryPersistParams {}

/// Parameters for executor benchmarking.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilExecutorBenchmarkParams {}

/// Parameters for context cost estimation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilContextCostEstimateParams {}

/// Parameters for signal injection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSignalInjectionParams {}

/// Parameters for context assembly.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilContextAssembleParams {}

/// Parameters for authority verification.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilAuthorityVerifyParams {}

/// Parameters for webhook testing.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilWebhookTestParams {
    /// Webhook URL to test.
    #[serde(default)]
    pub url: Option<String>,
}

/// Parameters for source configuration.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSourceConfigParams {
    /// Source name.
    #[serde(default)]
    pub source: Option<String>,
}
