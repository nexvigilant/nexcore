//! Theory of Vigilance (Grounded) MCP tool parameters.
//!
//! Typed parameter structs for signal strength calculation, safety margin,
//! stability shell analysis, harm classification, and meta-vigilance health.

use schemars::JsonSchema;
use serde::Deserialize;

/// Calculate signal strength S = U × R × T.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedSignalStrengthParams {
    /// Uniqueness U: information content in bits (≥ 0.0).
    pub uniqueness: f64,
    /// Recognition R: detection sensitivity × accuracy [0.0, 1.0].
    pub recognition: f64,
    /// Temporal T: decaying relevance factor [0.0, 1.0].
    pub temporal: f64,
}

/// Calculate safety margin d(s) = (threshold - s) / threshold.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedSafetyMarginParams {
    /// Current signal value.
    pub signal: f64,
    /// Safety threshold.
    pub threshold: f64,
}

/// Check if a complexity value sits on a stability shell (magic number).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedStabilityShellParams {
    /// Complexity chi value (number of architectural units).
    pub complexity: u64,
    /// Shell type: "complexity" or "connection". Default: "complexity".
    pub shell_type: Option<String>,
}

/// Classify a harm type and return its properties.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedHarmTypeParams {
    /// Harm type: "Acute", "Chronic", "Cascading", "Dormant",
    /// "Emergent", "Feedback", "Gateway", "Hidden".
    pub harm_type: String,
}

/// Check meta-vigilance health of the vigilance loop itself.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedMetaVigilanceParams {
    /// Total loop latency in milliseconds.
    pub loop_latency_ms: u64,
    /// Calibration overhead in milliseconds.
    pub calibration_overhead_ms: u64,
    /// Detection drift factor.
    pub detection_drift: f64,
    /// Apparatus integrity [0.0, 1.0].
    pub apparatus_integrity: f64,
}

/// Check EkaIntelligence emergence threshold.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedEkaIntelligenceParams {
    /// Complexity chi value.
    pub complexity: u64,
    /// Stability score [0.0, 1.0].
    pub stability: f64,
}

/// List all stability shell magic numbers for a given shell type.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TovGroundedMagicNumbersParams {
    /// Shell type: "complexity" or "connection". Default: "complexity".
    pub shell_type: Option<String>,
}
