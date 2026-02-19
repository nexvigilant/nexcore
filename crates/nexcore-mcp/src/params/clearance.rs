//! Security Clearance Parameters (5-level clearance system)
//! Tier: T2-T3 (Boundary Enforcement and State Management)
//!
//! evaluate, policy, validation, and metadata for security clearances.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for evaluating a gate operation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceEvaluateParams {
    /// Target kind: "project", "crate", "file", etc.
    pub target_kind: String,
    /// Target name/path
    pub target_name: String,
    /// Classification level: "Public", "Internal", "Confidential", "Secret", "TopSecret"
    pub level: String,
    /// Operation type: "access", "write", "external_call"
    #[serde(default = "default_clearance_op")]
    pub operation: String,
    /// External tool name
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Actor identity
    #[serde(default = "default_clearance_actor")]
    pub actor: String,
}

fn default_clearance_op() -> String {
    "access".to_string()
}

fn default_clearance_actor() -> String {
    "claude".to_string()
}

/// Parameters for looking up the policy for a specific level.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearancePolicyForParams {
    /// Classification level
    pub level: String,
}

/// Parameters for validating a classification change.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceValidateChangeParams {
    /// Current classification level
    pub from_level: String,
    /// Target classification level
    pub to_level: String,
    /// Current access mode: "Unrestricted", "Aware", "Guarded", "Enforced", "Lockdown"
    #[serde(default = "default_clearance_mode")]
    pub mode: String,
    /// Whether downgrade is explicitly permitted
    #[serde(default)]
    pub downgrade_permitted: bool,
}

fn default_clearance_mode() -> String {
    "Guarded".to_string()
}

/// Parameters for querying classification level metadata.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceLevelInfoParams {
    /// Classification level
    pub level: String,
}
