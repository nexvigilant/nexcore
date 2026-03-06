//! Boundary Detector MCP Parameters
//! T1 composition: ∂(Boundary) + κ(Comparison) + ς(State) + →(Causality) + N(Quantity)
//! Dominant: ∂

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for scanning a single value against named boundaries
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BoundaryDetectScanParams {
    /// The value(s) to scan. Single number or multi-dimensional vector.
    pub values: Vec<f64>,
    /// Named boundaries to scan against.
    pub boundaries: Vec<BoundaryDef>,
}

/// Parameters for scanning a stream of values and detecting crossings
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BoundaryDetectStreamParams {
    /// Stream of values to scan (each element is one time point).
    pub stream: Vec<f64>,
    /// Named boundaries to scan against.
    pub boundaries: Vec<BoundaryDef>,
}

/// Parameters for checking proximity to a boundary
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BoundaryDetectProximityParams {
    /// The value to check.
    pub value: f64,
    /// The boundary threshold.
    pub threshold: f64,
    /// Name of the boundary (for display).
    #[serde(default = "default_boundary_name")]
    pub name: String,
}

/// A boundary definition for MCP tool input
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BoundaryDef {
    /// Human-readable name for this boundary.
    pub name: String,
    /// Threshold value — above = one classification, below = the other.
    pub threshold: f64,
    /// Optional weight (default 1.0). Higher weight = more influence in multi-boundary scans.
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 {
    1.0
}

fn default_boundary_name() -> String {
    "boundary".to_string()
}
