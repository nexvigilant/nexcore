//! Station Parameters (WebMCP Hub Config Rails)
//! Tier: T3 (Domain × MCP integration)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for building a station config for a PV vertical.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationBuildConfigParams {
    /// PV vertical name (e.g., "faers", "dailymed", "pubmed", "ema", "clinical_trials").
    pub vertical: String,
    /// Config title for the hub listing.
    pub title: String,
    /// Config description (disclaimer auto-appended).
    pub description: String,
}

/// Parameters for adding a tool to a station config.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationAddToolParams {
    /// PV vertical name to add the tool to.
    pub vertical: String,
    /// Tool name (kebab-case).
    pub name: String,
    /// Tool description for agent discovery.
    pub description: String,
    /// Route path the tool targets.
    pub route: String,
    /// Execution type: "extract", "navigate", "fill", or "click".
    pub execution_type: String,
}

/// Parameters for listing station configs.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationListParams {
    /// Optional: filter by vertical name.
    pub vertical: Option<String>,
}

/// Parameters for generating MoltBrowser payloads.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationExportParams {
    /// PV vertical to export.
    pub vertical: String,
    /// MoltBrowser config ID (from contribute_create-config response).
    pub config_id: Option<String>,
}

/// Parameters for station coverage report.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationCoverageParams {}

/// Parameters for resolving the best tool for a domain via StationClient.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StationResolveParams {
    /// Domain to resolve (e.g., "api.fda.gov", "dailymed.nlm.nih.gov").
    pub domain: String,
    /// Optional task hint to guide tool selection.
    pub task_hint: Option<String>,
}
