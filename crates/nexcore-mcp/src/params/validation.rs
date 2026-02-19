//! Validation Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! L1-L5 validation framework and Rust test classification.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for L1-L5 validation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationRunParams {
    /// Target path to validate (file or directory)
    pub target: String,
    /// Domain type (auto-detected if not specified)
    /// Options: skill, agent, config, architecture, pv_terminology, construct
    #[serde(default)]
    pub domain: Option<String>,
    /// Maximum validation level: L1, L2, L3, L4, or L5 (default: L5)
    #[serde(default)]
    pub max_level: Option<String>,
    /// Stop on first failing level (default: true)
    #[serde(default)]
    pub fail_fast: Option<bool>,
}

/// Parameters for quick check (L1-L2 only)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationCheckParams {
    /// Target path to validate
    pub target: String,
    /// Domain type (auto-detected if not specified)
    #[serde(default)]
    pub domain: Option<String>,
}

/// Parameters for listing validation domains
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationDomainsParams {}

/// Parameters for classifying tests in Rust source
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationClassifyTestsParams {
    /// Path to Rust file or directory to analyze
    pub path: String,
}
