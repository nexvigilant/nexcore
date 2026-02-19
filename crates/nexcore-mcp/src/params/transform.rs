//! Text Transform Parameters (Cross-Domain Translation)
//!
//! Profile retrieval, segmentation, plan compilation, and fidelity scoring.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for transform_get_profile.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformGetProfileParams {
    /// Profile name.
    pub name: String,
}

/// Parameters for transform_segment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformSegmentParams {
    /// Title of the source document.
    pub title: String,
    /// Raw text to segment.
    pub text: String,
}

/// Parameters for transform_compile_plan.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformCompilePlanParams {
    /// Title of the source document.
    pub title: String,
    /// Raw text to transform.
    pub text: String,
    /// Source domain name.
    pub source_domain: String,
    /// Target profile name.
    pub target_profile: String,
}

/// Parameters for transform_score_fidelity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformScoreFidelityParams {
    /// Title of the source document.
    pub title: String,
    /// Raw source text.
    pub text: String,
    /// Source domain name.
    pub source_domain: String,
    /// Target profile name.
    pub target_profile: String,
    /// Number of paragraphs in the output.
    pub output_paragraph_count: usize,
    /// Per-paragraph concept hit counts.
    #[serde(default)]
    pub concept_hits: Vec<usize>,
}
