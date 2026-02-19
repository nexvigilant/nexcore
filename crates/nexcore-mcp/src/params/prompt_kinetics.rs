//! Params for prompt kinetics (PK model) tools.

use serde::Deserialize;

/// Analyze a prompt through the ADME pharmacokinetic model.
#[derive(Debug, Deserialize)]
pub struct PromptKineticsAnalyzeParams {
    /// The prompt text to analyze
    pub prompt: String,
    /// Context: what system/model receives this prompt
    pub target_model: Option<String>,
}

/// Compute bioavailability of a prompt (information absorption rate).
#[derive(Debug, Deserialize)]
pub struct PromptBioavailabilityParams {
    /// The prompt text
    pub prompt: String,
    /// Expected output length (tokens)
    pub expected_output_tokens: Option<u64>,
}

/// Get PK model parameter definitions.
#[derive(Debug, Deserialize)]
pub struct PromptKineticsModelParams {}
