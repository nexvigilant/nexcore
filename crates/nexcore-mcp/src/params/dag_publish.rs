//! DAG Publish Parameters
//! Tier: T3 (Tooling MCP tool parameters)
//!
//! Read-only tools that expose dag-publish library functions:
//! - `dag_publish_plan`: Phase plan from build_dag + group_into_phases
//! - `dag_publish_dry_run`: Flat publish order from topological_sort
//! - `dag_publish_status`: Resume state from .dag-publish-state.json

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

fn default_nexcore_crates_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    format!("{home}/Projects/Active/nexcore/crates")
}

fn default_registry() -> String {
    "nexcore".to_string()
}

/// Parameters for `dag_publish_plan` — build the DAG and return phase groupings.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DagPublishPlanParams {
    /// Path to the crates directory to scan (default: ~/Projects/Active/nexcore/crates)
    #[serde(default = "default_nexcore_crates_dir")]
    pub crates_dir: String,

    /// Registry name used for dependency resolution (default: "nexcore")
    #[serde(default = "default_registry")]
    pub registry: String,
}

/// Parameters for `dag_publish_dry_run` — return flat topological publish order.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DagPublishDryRunParams {
    /// Path to the crates directory to scan (default: ~/Projects/Active/nexcore/crates)
    #[serde(default = "default_nexcore_crates_dir")]
    pub crates_dir: String,

    /// Registry name used for dependency resolution (default: "nexcore")
    #[serde(default = "default_registry")]
    pub registry: String,

    /// Only include crates whose name starts with this prefix (e.g. "stem-")
    #[serde(default)]
    pub filter: Option<String>,

    /// Maximum number of crates to include in the result
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for `dag_publish_status` — read the resume state file.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DagPublishStatusParams {
    /// Workspace root that contains `.dag-publish-state.json`
    /// (default: ~/Projects/Active/nexcore)
    #[serde(default = "default_workspace_root")]
    pub workspace_root: String,

    /// Also scan the crates directory and report which crates are pending
    /// (have no checkpoint yet).  Requires `crates_dir`.
    #[serde(default)]
    pub show_pending: bool,

    /// Path to the crates directory (required when `show_pending` is true)
    #[serde(default = "default_nexcore_crates_dir")]
    pub crates_dir: String,

    /// Registry name used for dependency resolution when `show_pending` is true
    #[serde(default = "default_registry")]
    pub registry: String,
}

fn default_workspace_root() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    format!("{home}/Projects/Active/nexcore")
}
