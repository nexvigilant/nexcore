//! Disney Loop MCP parameters
//!
//! Forward-only compound discovery pipeline: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for running the full Disney Loop pipeline
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DisneyLoopRunParams {
    /// JSON array of records with fields: domain, direction, novelty_score, discovery
    pub records: Vec<DisneyRecord>,
}

/// Parameters for the anti-regression gate stage
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DisneyAntiRegressionParams {
    /// JSON array of records to filter
    pub records: Vec<DisneyRecord>,
}

/// Parameters for the curiosity search aggregation stage
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DisneyCuriositySearchParams {
    /// JSON array of records to aggregate (should be pre-filtered by anti-regression)
    pub records: Vec<DisneyRecord>,
}

/// Parameters for assessing current state
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DisneyStateAssessParams {
    /// JSON array of records to analyze
    pub records: Vec<DisneyRecord>,
}

/// A single record in the Disney Loop pipeline
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DisneyRecord {
    /// Domain category
    pub domain: String,
    /// Direction: "forward" or "backward"
    pub direction: String,
    /// Novelty score (higher = more novel)
    pub novelty_score: i64,
    /// Discovery label
    pub discovery: String,
}
