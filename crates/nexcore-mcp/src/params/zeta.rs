//! Zeta function and telescope pipeline parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parse LMFDB zero data from JSON string.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaLmfdbParseParams {
    /// JSON string containing zero data (supports raw array, labeled, or API response format)
    pub json: String,
}

/// Run telescope pipeline on computed zeros within a height range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaTelescopeRunParams {
    /// Minimum height for zero search
    pub t_min: f64,
    /// Maximum height for zero search
    pub t_max: f64,
    /// Number of zeros to predict beyond input set (default: 10)
    #[serde(default = "default_n_predict")]
    pub n_predict: usize,
}

fn default_n_predict() -> usize {
    10
}

/// Run batch telescope on multiple height ranges.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaBatchRunParams {
    /// List of (t_min, t_max) height ranges
    pub ranges: Vec<(f64, f64)>,
    /// Minimum zeros per range (default: 20)
    #[serde(default = "default_min_zeros")]
    pub min_zeros: usize,
}

fn default_min_zeros() -> usize {
    20
}

/// Fit scaling law to telescope confidence data.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaScalingFitParams {
    /// List of (N, confidence) pairs
    pub points: Vec<(usize, f64)>,
}

/// Predict telescope confidence at a given N.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaScalingPredictParams {
    /// Amplitude parameter a
    pub a: f64,
    /// Decay exponent b
    pub b: f64,
    /// Number of zeros to predict confidence for
    pub n: usize,
}

/// Cayley transform of CMV matrix from zeros in a height range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaCayleyParams {
    /// Minimum height for zero search
    pub t_min: f64,
    /// Maximum height for zero search
    pub t_max: f64,
}

/// Run operator hunt on zeros in a height range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaOperatorHuntParams {
    /// Minimum height for zero search
    pub t_min: f64,
    /// Maximum height for zero search
    pub t_max: f64,
}

/// Run a specific operator candidate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaOperatorCandidateParams {
    /// Operator name: "berry_keating", "xp_potential", or "cmv_truncation"
    pub operator: String,
    /// Minimum height for zero search
    pub t_min: f64,
    /// Maximum height for zero search
    pub t_max: f64,
}

/// Verify RH up to a given height.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaVerifyRhParams {
    /// Maximum height to verify
    pub max_height: f64,
    /// Step size for zero search (default: 0.05)
    #[serde(default = "default_step")]
    pub step: f64,
}

fn default_step() -> f64 {
    0.05
}

/// Compute zeta function at a complex point.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaComputeParams {
    /// Real part of s
    pub re: f64,
    /// Imaginary part of s
    pub im: f64,
}

/// Find zeros in a height range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaFindZerosParams {
    /// Minimum height
    pub t_min: f64,
    /// Maximum height
    pub t_max: f64,
    /// Step size for bracket search (default: 0.05)
    #[serde(default = "default_step")]
    pub step: f64,
}

/// Compare to GUE random matrix statistics.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaGueCompareParams {
    /// Minimum height for zero search
    pub t_min: f64,
    /// Maximum height for zero search
    pub t_max: f64,
}

/// Get embedded Riemann zeros.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ZetaEmbeddedZerosParams {
    /// Number of zeros to return (max 30)
    #[serde(default = "default_embedded_count")]
    pub count: usize,
}

fn default_embedded_count() -> usize {
    30
}
