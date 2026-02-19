//! Reverse Transcriptase Parameters (Schema Inference)
//! Tier: T2-C (κ + σ + μ + ∂ — Comparison + Sequence + Mapping + Boundary)
//!
//! JSON schema inference, validation, and synthetic data generation.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Full transcriptase pipeline parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseProcessParams {
    /// JSON string to process
    pub json: String,
    /// Synthesize boundary violations
    #[serde(default)]
    pub violations: Option<bool>,
    /// Verify round-trip fidelity
    #[serde(default)]
    pub verify: Option<bool>,
}

/// Schema inference parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseInferParams {
    /// JSON string to infer schema from
    pub json: String,
}

/// Boundary violation analysis parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseViolationsParams {
    /// JSON string to analyze
    pub json: String,
}

/// Synthetic data generation parameters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseGenerateParams {
    /// JSON string to observe
    pub json: String,
    /// Number of synthetic records to generate
    #[serde(default)]
    pub count: Option<usize>,
}
