//! Params for FDA-inspired code inspection tools.

use serde::Deserialize;

/// Run a code inspection audit on a file or directory.
#[derive(Debug, Deserialize)]
pub struct CodeInspectAuditParams {
    /// Path to file or directory to inspect
    pub target: String,
    /// Audit dimensions to check (default: all three)
    /// Options: "safety", "efficacy", "purity"
    pub dimensions: Option<Vec<String>>,
}

/// Score code against a specific inspection dimension.
#[derive(Debug, Deserialize)]
pub struct CodeInspectScoreParams {
    /// Source code content to score
    pub code: String,
    /// Language: "rust", "typescript", "zsh"
    pub language: Option<String>,
}

/// Get inspection criteria definitions.
#[derive(Debug, Deserialize)]
pub struct CodeInspectCriteriaParams {}
