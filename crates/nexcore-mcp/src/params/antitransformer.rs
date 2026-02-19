//! Antitransformer Parameters (AI Text Detection)
//! Tier: T3 (Domain-specific — AI text detection)
//!
//! Fingerprinting and batch analysis for AI-generated content.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for analyzing a single text.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerAnalyzeParams {
    /// Text to analyze.
    pub text: String,
    /// Decision threshold (0.0-1.0).
    pub threshold: Option<f64>,
    /// Entropy window size.
    pub window_size: Option<usize>,
}

/// A single text item in a batch request.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerBatchItem {
    /// Optional identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Text to analyze.
    pub text: String,
}

/// Parameters for batch analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerBatchParams {
    /// Array of texts to analyze
    pub texts: Vec<AntitransformerBatchItem>,
    /// Decision threshold.
    pub threshold: Option<f64>,
    /// Entropy window size.
    pub window_size: Option<usize>,
}
