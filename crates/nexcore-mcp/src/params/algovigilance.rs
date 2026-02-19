//! Algovigilance Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Automated pharmacovigilance (deduplication, triage, reinforcement).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for comparing two ICSR narratives
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilDedupPairParams {
    /// First ICSR narrative text
    pub narrative_a: String,
    /// Second ICSR narrative text
    pub narrative_b: String,
    /// Similarity threshold (0.0-1.0)
    #[serde(default = "default_dedup_threshold")]
    pub threshold: f64,
}

fn default_dedup_threshold() -> f64 {
    0.85
}

/// Parameters for batch FAERS deduplication
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilDedupBatchParams {
    /// Drug name to fetch FAERS cases for
    pub drug: String,
    /// Similarity threshold (0.0-1.0)
    #[serde(default = "default_dedup_threshold")]
    pub threshold: f64,
    /// Maximum cases to fetch
    #[serde(default = "default_batch_limit")]
    pub limit: usize,
}

fn default_batch_limit() -> usize {
    50
}

/// Parameters for signal triage with decay
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageDecayParams {
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// Half-life in days
    #[serde(default = "default_half_life")]
    pub half_life_days: f64,
}

fn default_half_life() -> f64 {
    30.0
}

/// Parameters for reinforcing a signal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageReinforceParams {
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// Number of new supporting cases
    pub new_cases: u32,
}

/// Parameters for getting the signal triage queue
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageQueueParams {
    /// Drug name to get queue for
    pub drug: String,
    /// Half-life in days
    #[serde(default = "default_half_life")]
    pub half_life_days: f64,
    /// Minimum relevance cutoff
    #[serde(default = "default_cutoff")]
    pub cutoff: f64,
    /// Maximum signals to return
    #[serde(default = "default_queue_limit")]
    pub limit: usize,
}

fn default_cutoff() -> f64 {
    0.1
}

fn default_queue_limit() -> usize {
    10
}
