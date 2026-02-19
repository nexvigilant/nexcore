//! Counter-Awareness Parameters (Electronic Warfare)
//!
//! Detection, sensor fusion, optimization, and effectiveness matrix.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Single-sensor detection probability parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaDetectParams {
    /// Sensor name.
    pub sensor: String,
    /// Active counter-primitive names.
    #[serde(default)]
    pub counters: Vec<String>,
    /// Range to target in meters.
    pub range_m: f64,
    /// Raw target signature strength.
    pub raw_signature: f64,
}

/// Multi-sensor fused detection probability parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaFusionParams {
    /// Sensor names.
    pub sensors: Vec<String>,
    /// Active counter-primitive names.
    #[serde(default)]
    pub counters: Vec<String>,
    /// Range to target in meters.
    pub range_m: f64,
    /// Raw target signature strength.
    pub raw_signature: f64,
    /// Detection threshold.
    pub threshold: Option<f64>,
}

/// Optimal countermeasure loadout selection parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaOptimizeParams {
    /// Threat sensor names.
    pub sensors: Vec<String>,
    /// Available countermeasure names.
    pub countermeasures: Vec<String>,
    /// Maximum weight budget in kg.
    pub weight_budget_kg: f64,
    /// Engagement range in meters.
    pub range_m: f64,
    /// Raw target signature strength.
    pub raw_signature: f64,
}

/// Parameters for querying the effectiveness matrix.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaMatrixParams {
    /// Optional sensing primitive name.
    pub sensing: Option<String>,
    /// Optional counter-primitive name.
    pub counter: Option<String>,
}

/// Parameters for listing available assets.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaCatalogParams {
    /// Filter: "sensors", "countermeasures", or omit.
    pub category: Option<String>,
}
