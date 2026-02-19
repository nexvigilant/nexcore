//! Compendious Parameters (Information Density Optimization)
//!
//! Scoring, compression, comparison, pattern analysis, and domain targeting.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for compendious score_text.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousScoreParams {
    /// Text to score for information density.
    pub text: String,
    /// Required elements that must be present.
    pub required_elements: Option<Vec<String>>,
}

/// Parameters for compendious compress_text.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousCompressParams {
    /// Text to compress using BLUFF method.
    pub text: String,
    /// Terms to preserve.
    pub preserve: Option<Vec<String>>,
}

/// Parameters for compendious compare_texts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousCompareParams {
    /// Original (uncompressed) text.
    pub original: String,
    /// Optimized (compressed) text.
    pub optimized: String,
}

/// Parameters for compendious analyze_patterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousAnalyzeParams {
    /// Text to analyze for verbose patterns.
    pub text: String,
}

/// Parameters for compendious get_domain_target.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousDomainTargetParams {
    /// Domain
    pub domain: String,
    /// Content type
    pub content_type: String,
}
