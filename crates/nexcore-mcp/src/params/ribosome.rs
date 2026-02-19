//! Ribosome Parameters (Data Contract Registry)
//! Tier: T2-C (κ + σ + μ + ∂ + N — Comparison + Sequence + Mapping + Boundary + Quantity)
//!
//! Contract storage, validation, and drift detection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for storing a baseline contract.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeStoreParams {
    /// Unique contract identifier
    pub contract_id: String,
    /// JSON string
    pub json: String,
}

/// Parameters for validating data against a stored contract.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeValidateParams {
    /// Contract ID to validate against
    pub contract_id: String,
    /// JSON string to validate
    pub json: String,
}

/// Parameters for generating synthetic data from a contract.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeGenerateParams {
    /// Contract ID to generate from
    pub contract_id: String,
    /// Number of synthetic records
    #[serde(default)]
    pub count: Option<usize>,
}

/// Parameters for batch drift detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeDriftParams {
    /// Map of contract_id → JSON string
    pub data: HashMap<String, String>,
}
