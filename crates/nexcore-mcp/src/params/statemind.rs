//! Statemind DNA pipeline MCP tool parameters.
//!
//! Typed parameter structs for word DNA analysis and constellation analysis.

use schemars::JsonSchema;
use serde::Deserialize;

/// Analyze a word through the full DNA pipeline (encode → FFT → thermo → project → safety).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatemindAnalyzeWordParams {
    /// The word to analyze.
    pub word: String,
}

/// Analyze a constellation of words (pairwise resonance, mutation stability, clustering).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatemindConstellationParams {
    /// List of words to analyze as a constellation.
    pub words: Vec<String>,
}
