//! Build Orchestrator Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! CI/CD pipeline execution, workspace discovery, history, and metrics.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for dry-running a pipeline definition
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildOrchestratorDryRunParams {
    /// Pipeline definition as JSON string, or built-in name ("validate", "validate-quick")
    pub pipeline: String,
}

/// Parameters for listing available pipeline stage types
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildOrchestratorStagesParams {}

/// Parameters for discovering crates in a workspace
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildOrchestratorWorkspaceParams {
    /// Workspace root path (default: ~/nexcore)
    #[serde(default = "default_workspace_path")]
    pub path: String,
}

fn default_workspace_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    format!("{home}/nexcore")
}

/// Parameters for querying pipeline run history
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildOrchestratorHistoryParams {
    /// Workspace root path (default: ~/nexcore)
    #[serde(default = "default_workspace_path")]
    pub path: String,
    /// Maximum number of runs to return (default: 10)
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    /// Filter by status: "completed", "failed", or "all" (default)
    #[serde(default)]
    pub status: Option<String>,
}

fn default_history_limit() -> usize {
    10
}

/// Parameters for getting build metrics summary
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildOrchestratorMetricsParams {
    /// Workspace root path (default: ~/nexcore)
    #[serde(default = "default_workspace_path")]
    pub path: String,
}
