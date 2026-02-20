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

/// Parameters for querying current token budget state (ATP/ADP/AMP levels).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyBudgetParams {
    /// Total token budget
    pub budget: u64,
    /// Tokens spent on productive work
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Tokens recycled (tADP -> tATP via compression/caching)
    #[serde(default)]
    pub recycled: Option<u64>,
    /// Tokens degraded (tADP -> tAMP, productive work that was discarded)
    #[serde(default)]
    pub degraded: Option<u64>,
    /// Average token cost per operation (for remaining-ops estimate)
    #[serde(default)]
    pub avg_cost_per_op: Option<u64>,
}

/// Parameters for querying historical energy consumption patterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyHistoryParams {
    /// Total token budget
    pub budget: u64,
    /// Sequence of (productive_spent, wasted) pairs representing operations over time
    pub operations: Vec<EnergyHistoryOp>,
}

/// A single operation in the energy history timeline.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyHistoryOp {
    /// Label for this operation
    pub label: String,
    /// Tokens spent productively
    pub productive: u64,
    /// Tokens wasted
    pub wasted: u64,
}

/// Parameters for running one ATP/ADP cycle step.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyCycleParams {
    /// Total token budget
    pub budget: u64,
    /// Tokens spent on productive work before this cycle
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted before this cycle
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Cycle action to perform
    pub action: EnergyCycleAction,
    /// Amount of tokens for the action
    pub amount: u64,
}

/// The action to perform in an energy cycle step.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub enum EnergyCycleAction {
    /// Spend tokens productively: tATP -> tADP
    SpendProductive,
    /// Waste tokens: tATP -> tAMP
    SpendWaste,
    /// Recycle spent tokens: tADP -> tATP (compression/caching)
    Recycle,
    /// Degrade productive to waste: tADP -> tAMP (discarded work)
    Degrade,
}
