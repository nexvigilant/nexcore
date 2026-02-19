//! Params for muscular system (tool execution) tools.

use serde::Deserialize;

/// Classify a tool by muscle type (skeletal/smooth/cardiac).
#[derive(Debug, Deserialize)]
pub struct MuscularClassifyParams {
    /// Tool name to classify
    pub tool: String,
}

/// Check fatigue level for the current session.
#[derive(Debug, Deserialize)]
pub struct MuscularFatigueParams {
    /// Total tool calls in this session
    pub total_calls: u64,
    /// Context tokens consumed
    pub tokens_consumed: u64,
    /// Context window size
    #[serde(default = "default_context_window")]
    pub context_window: u64,
}

fn default_context_window() -> u64 {
    200_000
}

/// Get muscular system health overview.
#[derive(Debug, Deserialize)]
pub struct MuscularHealthParams {}
