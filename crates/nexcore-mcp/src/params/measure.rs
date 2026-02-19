//! Workspace Quality Measurement Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Crate health analysis, entropy calculation, drift detection, and comparison.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for measuring a single crate's health.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureCrateParams {
    /// Crate name
    pub name: String,
}

/// Parameters for Shannon entropy calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureEntropyParams {
    /// Category counts
    pub counts: Vec<usize>,
}

/// Parameters for metric drift detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureDriftParams {
    /// Window size for drift comparison
    #[serde(default)]
    pub window: Option<usize>,
}

/// Parameters for side-by-side crate comparison.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureCompareParams {
    /// First crate name
    pub crate_a: String,
    /// Second crate name
    pub crate_b: String,
}

/// Parameters for statistical summary.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureStatsParams {
    /// Numeric data points
    pub counts: Vec<f64>,
}
