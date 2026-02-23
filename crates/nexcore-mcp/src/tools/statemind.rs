//! Statemind DNA pipeline MCP tools.
//!
//! Word-as-DNA analysis: nucleotide encoding, FFT spectral analysis,
//! thermodynamic tension, 3D projection, pairwise resonance, clustering.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::statemind::{StatemindAnalyzeWordParams, StatemindConstellationParams};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Analyze a word through the full DNA pipeline.
pub fn statemind_analyze_word(p: StatemindAnalyzeWordParams) -> Result<CallToolResult, McpError> {
    let analysis = nexcore_statemind::DnaAnalysis::analyze(&p.word);
    ok_json(serde_json::to_value(&analysis).unwrap_or_default())
}

/// Analyze a constellation of words (pairwise resonance, mutation, clustering).
pub fn statemind_constellation(
    p: StatemindConstellationParams,
) -> Result<CallToolResult, McpError> {
    let word_refs: Vec<&str> = p.words.iter().map(|s| s.as_str()).collect();
    let analysis = nexcore_statemind::ConstellationAnalysis::analyze(&word_refs);
    ok_json(serde_json::to_value(&analysis).unwrap_or_default())
}
