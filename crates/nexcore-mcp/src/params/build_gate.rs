//! Build Gate Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Cargo build coordination, hash-based skip detection, and lock status.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for evaluating whether a crate needs rebuilding.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildGateEvaluateParams {
    /// Workspace root path (defaults to ~/nexcore)
    #[serde(default)]
    pub workspace: Option<String>,

    /// Force rebuild even if hash is unchanged
    #[serde(default)]
    pub force: bool,
}

/// Parameters for querying build gate status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildGateStatusParams {
    /// Workspace root path (defaults to ~/nexcore)
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Parameters for querying historical build results.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BuildGateHistoryParams {
    /// Workspace root path (defaults to ~/nexcore)
    #[serde(default)]
    pub workspace: Option<String>,
}
