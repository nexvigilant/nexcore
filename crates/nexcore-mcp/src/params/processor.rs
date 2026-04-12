//! Processor Framework MCP Parameters
//! T1 composition: μ(Mapping) + σ(Sequence) + ∂(Boundary) + κ(Comparison)
//! Dominant: μ

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// An antibody pattern for entry boundary validation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntibodyPatternInput {
    /// Human-readable description of what this pattern catches.
    pub description: String,
    /// Regex pattern to match against string representations of PRR values.
    pub regex: String,
}

/// Parameters for the processor demo pipeline.
/// Runs PRR values through a bounded classification pipeline.
/// Optionally applies antibody patterns as entry boundaries.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ProcessorDemoParams {
    /// PRR values to process through the pipeline.
    pub values: Vec<f64>,
    /// Signal threshold (default: 2.0 = Evans PRR criterion).
    pub threshold: Option<f64>,
    /// Optional antibody patterns — regex antipatterns that reject matching inputs.
    /// Each pattern is checked against the string representation of each PRR value.
    /// Example: [{"description": "reject exact zeros", "regex": "^0\\.0$"}]
    pub antibody_patterns: Option<Vec<AntibodyPatternInput>>,
}
