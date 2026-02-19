//! Formula-Derived Parameters (KU Extraction)
//! Tier: T3 (Algorithmic Logic)
//!
//! Signal strength, domain distance, flywheel velocity, and spectral overlap.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Signal Strength composite (S = U × R × T).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalStrengthParams {
    /// Unexpectedness factor (0.0 to 1.0).
    pub unexpectedness: f64,
    /// Robustness factor (0.0 to 1.0).
    pub robustness: f64,
    /// Therapeutic importance factor (0.0 to 1.0).
    pub therapeutic_importance: f64,
}

/// Parameters for Domain Distance calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainDistanceParams {
    /// Primitives in domain A.
    pub primitives_a: Vec<String>,
    /// Primitives in domain B.
    pub primitives_b: Vec<String>,
    /// Weight for T1 overlap.
    #[serde(default = "default_w1")]
    pub w1: f64,
    /// Weight for T2 overlap.
    #[serde(default = "default_w2")]
    pub w2: f64,
    /// Weight for T3 overlap.
    #[serde(default = "default_w3")]
    pub w3: f64,
}

fn default_w1() -> f64 {
    0.2
}
fn default_w2() -> f64 {
    0.3
}
fn default_w3() -> f64 {
    0.5
}

/// Parameters for Flywheel Velocity calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelVelocityParams {
    /// Failure timestamps (ms).
    pub failure_timestamps: Vec<u64>,
    /// Fix timestamps (ms).
    pub fix_timestamps: Vec<u64>,
}

/// Parameters for Token Ratio calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TokenRatioParams {
    /// LLM tokens consumed.
    pub token_count: u64,
    /// Semantic operations produced.
    pub operation_count: u64,
}

/// Parameters for Spectral Overlap calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SpectralOverlapParams {
    /// First spectrum vector.
    pub spectrum_a: Vec<f64>,
    /// Second spectrum vector.
    pub spectrum_b: Vec<f64>,
}
