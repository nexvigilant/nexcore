//! Energy Dynamics Parameters (Token Metabolism)
//! Tier: T2-C (N + κ + ∝ — Quantity + Comparison + Proportion)
//!
//! Token-as-ATP/ADP biochemistry, coupling efficiency, and strategy selection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for computing energy charge and full state snapshot.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyChargeParams {
    /// Total token budget
    pub budget: u64,
    /// Tokens spent on productive work
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Total value produced
    #[serde(default)]
    pub total_value: Option<f64>,
}

/// Parameters for deciding the optimal strategy based on energy state.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyDecideParams {
    /// Total token budget
    pub budget: u64,
    /// Tokens spent on productive work
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Label for the operation being considered
    pub operation_label: String,
    /// Estimated token cost
    pub estimated_cost: u64,
    /// Estimated value
    pub estimated_value: f64,
    /// Whether a cached result might exist
    #[serde(default)]
    pub cache_possible: Option<bool>,
}
