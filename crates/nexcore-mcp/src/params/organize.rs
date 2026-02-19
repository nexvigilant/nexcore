//! ORGANIZE pipeline MCP tool parameters.
//!
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! File organization pipeline: observe, rank, group, assign, name, integrate, zero-out, enforce.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for running the ORGANIZE pipeline analysis (dry-run).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeAnalyzeParams {
    /// Directory path to analyze
    pub path: String,
    /// Maximum directory depth (0 = unlimited)
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Parameters for getting default ORGANIZE configuration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeConfigDefaultParams {
    /// Directory path to generate config for
    pub path: String,
    /// Output format: "json" or "toml"
    #[serde(default)]
    pub format: Option<String>,
}

/// Parameters for generating a markdown report.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeReportMarkdownParams {
    /// Directory path to analyze and report on
    pub path: String,
}

/// Parameters for generating a JSON report.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeReportJsonParams {
    /// Directory path to analyze and report on
    pub path: String,
}

/// Parameters for running the observe step (inventory only).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeObserveParams {
    /// Directory path to observe
    pub path: String,
    /// Maximum directory depth (0 = unlimited)
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Parameters for running observe + rank steps.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OrganizeRankParams {
    /// Directory path to observe and rank
    pub path: String,
    /// Maximum number of entries to return (0 = all)
    #[serde(default)]
    pub limit: Option<usize>,
}
