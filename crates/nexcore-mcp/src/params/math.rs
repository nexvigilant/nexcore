//! Mathematical & Aggregate Function Parameters
//! Tier: T2-C (Σ + ρ + κ — Sum + Recursion + Comparison)
//!
//! Numeric aggregation, tree folding, ranking, percentiles, and outlier detection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for fold_all aggregation over numeric values.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateFoldParams {
    /// Numeric values to aggregate.
    pub values: Vec<f64>,
}

/// Parameters for recursive tree fold.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateTreeFoldParams {
    /// Tree as JSON
    pub tree: serde_json::Value,
    /// Combine function: "sum", "max", or "mean"
    #[serde(default = "default_combine_fn")]
    pub combine: String,
}

fn default_combine_fn() -> String {
    "sum".to_string()
}

/// Parameters for ranking named values.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateRankParams {
    /// List of [name, value] pairs to rank.
    pub items: Vec<(String, f64)>,
    /// Number of top entries to return
    #[serde(default)]
    pub top_n: usize,
}

/// Parameters for percentile computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregatePercentileParams {
    /// Numeric values.
    pub values: Vec<f64>,
    /// Percentile to compute (0.0 to 1.0).
    pub percentile: f64,
}

/// Parameters for outlier detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateOutliersParams {
    /// List of [name, value] pairs to check.
    pub items: Vec<(String, f64)>,
}
