//! Params for Clinical Trial Validation Paradigm (CTVP) tools.

use serde::Deserialize;

/// Score a deliverable against the 5-phase CTVP model.
#[derive(Debug, Deserialize)]
pub struct CtvpScoreParams {
    /// Path to file or crate to evaluate
    pub target: String,
    /// Phase to score (0-4, or "all" for full pipeline). Default: "all"
    pub phase: Option<String>,
}

/// Run the Five Problems Protocol discovery.
#[derive(Debug, Deserialize)]
pub struct CtvpFiveProblemsParams {
    /// Path to file or crate to analyze
    pub target: String,
    /// Domain context (e.g., "rust-crate", "api", "frontend")
    pub domain: Option<String>,
}

/// Get CTVP phase definitions and scoring criteria.
#[derive(Debug, Deserialize)]
pub struct CtvpPhasesListParams {}
