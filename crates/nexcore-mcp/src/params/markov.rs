//! Markov chain analysis and construction MCP tool parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Unified Markov chain analysis covering stationary distribution,
//! n-step probabilities, ergodicity classification, state classification,
//! communicating classes, absorbing states, entropy rate, and mean first passage time.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// A single transition definition for Markov chain construction.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransitionInput {
    /// Source state index (0-based).
    pub from: usize,
    /// Target state index (0-based).
    pub to: usize,
    /// Transition probability (or weight — will be row-normalized).
    pub probability: f64,
}

/// Parameters for the markov_analyze MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MarkovAnalyzeParams {
    /// State labels (required).
    pub states: Vec<String>,

    /// Transition definitions. Each specifies from, to, and probability.
    /// Rows are normalized to sum to 1.0 if they don't already.
    pub transitions: Vec<TransitionInput>,

    /// Analysis mode:
    /// - "summary": full chain summary (ergodicity, classes, stationary dist, entropy)
    /// - "stationary": stationary distribution only
    /// - "n_step": n-step transition probability (requires from_state, to_state, steps)
    /// - "classify": state classification (recurrent/transient/absorbing)
    /// - "classes": communicating classes
    /// - "ergodicity": ergodicity check
    /// - "entropy": entropy rate
    /// - "mfpt": mean first passage time (requires from_state, to_state)
    pub analysis: String,

    /// Source state index for n_step and mfpt queries (0-based).
    #[serde(default)]
    pub from_state: Option<usize>,

    /// Target state index for n_step and mfpt queries (0-based).
    #[serde(default)]
    pub to_state: Option<usize>,

    /// Number of steps for n_step query (default: 1).
    #[serde(default = "default_steps")]
    pub steps: u32,
}

fn default_steps() -> u32 {
    1
}

/// Parameters for the markov_from_data MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MarkovFromDataParams {
    /// State labels.
    pub states: Vec<String>,

    /// Observed state sequences as arrays of state indices.
    /// Transition probabilities are estimated from consecutive pairs.
    pub sequences: Vec<Vec<usize>>,
}
