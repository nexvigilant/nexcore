//! Security & Boot Parameters
//! Tier: T2-C (σ + → + ∂ — Boot Chain State)
//!
//! Secure boot status and chain of trust.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for querying secure boot chain status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootStatusParams {
    /// Boot policy: "Strict", "Degraded", "Permissive"
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

/// Parameters for verifying a boot stage measurement.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootVerifyParams {
    /// Boot stage
    pub stage: String,
    /// Data to measure
    pub data: String,
    /// Optional expected hash (hex)
    #[serde(default)]
    pub expected_hex: Option<String>,
    /// Boot policy
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

/// Parameters for generating a boot quote.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootQuoteParams {
    /// Boot stages to measure
    pub stages: Vec<SecureBootStageInput>,
    /// Boot policy
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

/// A single stage input for boot quote generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootStageInput {
    /// Boot stage name
    pub stage: String,
    /// Data to measure
    pub data: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
}

fn default_boot_policy() -> String {
    "Permissive".to_string()
}
