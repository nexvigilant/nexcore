//! Params for respiratory system (context window, gas exchange, dead space) tools.

use serde::Deserialize;

/// Analyze gas exchange (useful info extracted from context).
#[derive(Debug, Deserialize)]
pub struct RespiratoryExchangeParams {
    /// Total context tokens inhaled
    pub inhaled_tokens: u64,
    /// Useful tokens extracted
    pub extracted_tokens: Option<u64>,
}

/// Detect dead space in context (wasted tokens).
#[derive(Debug, Deserialize)]
pub struct RespiratoryDeadSpaceParams {
    /// Total context window size
    pub context_size: u64,
    /// Active/useful portion
    pub active_tokens: Option<u64>,
}

/// Measure tidal volume (per-turn context usage).
#[derive(Debug, Deserialize)]
pub struct RespiratoryTidalParams {
    /// Tokens per turn
    pub tokens_per_turn: Vec<u64>,
}

/// Get respiratory system health overview.
#[derive(Debug, Deserialize)]
pub struct RespiratoryHealthParams {}
