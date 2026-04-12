//! Parameter structs for DNA-ML pipeline MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `dna_ml_encode`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DnaMlEncodeParams {
    /// Feature vector to encode as DNA (12-element PV features).
    pub features: Vec<f64>,
    /// Per-feature minimum bounds (for quantization). If omitted, uses 0.0 for all.
    #[serde(default)]
    pub mins: Option<Vec<f64>>,
    /// Per-feature maximum bounds (for quantization). If omitted, uses 1.0 for all.
    #[serde(default)]
    pub maxs: Option<Vec<f64>>,
}

/// Parameters for `dna_ml_similarity`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DnaMlSimilarityParams {
    /// First feature vector.
    pub features_a: Vec<f64>,
    /// Second feature vector.
    pub features_b: Vec<f64>,
    /// Per-feature minimums for quantization.
    #[serde(default)]
    pub mins: Option<Vec<f64>>,
    /// Per-feature maximums for quantization.
    #[serde(default)]
    pub maxs: Option<Vec<f64>>,
}

/// Parameters for `dna_ml_pipeline_run`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DnaMlPipelineRunParams {
    /// Raw FAERS-like data entries.
    pub data: Vec<super::ml_pipeline::MlRawEntry>,
    /// Labels: "signal" or "noise" for each entry (same order as data).
    pub labels: Vec<String>,
    /// Number of trees (default: 50).
    #[serde(default)]
    pub n_trees: Option<usize>,
    /// Maximum tree depth (default: 8).
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Include DNA similarity features (default: true).
    #[serde(default)]
    pub use_dna_features: Option<bool>,
}
