//! Routing Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Tool routing, DAG construction, and deterministic dispatch parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for tool routing: given a stimulus, return deterministic tool selection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolRouteParams {
    /// Stimulus or intent to route (e.g., "compare two strings", "drug safety review")
    pub stimulus: String,
    /// Maximum number of routing matches to return (default: 3)
    pub limit: Option<usize>,
}

/// Parameters for tool DAG construction: given tools, return dependency graph + execution plan.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolDagParams {
    /// Tool names to include in the DAG
    pub tools: Vec<String>,
    /// Whether to include transitive dependencies (default: true)
    pub transitive: Option<bool>,
}

/// Parameters for tool dependency lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolDepsParams {
    /// Tool name to look up
    pub tool: String,
}

/// Parameters for named workflow chain execution plan.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolChainParams {
    /// Chain name (e.g., "pv_signal_analysis", "concept_grounding", "drug_safety_review")
    /// Use "list" to see all available chains.
    pub chain: String,
}
