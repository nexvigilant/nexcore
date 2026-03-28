//! Vigilance Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Safety margin, risk scoring, and ToV mapping.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for safety margin calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SafetyMarginParams {
    /// PRR value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// Information Component 2.5th percentile
    pub ic025: f64,
    /// EBGM 5th percentile
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for risk score calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RiskScoreParams {
    /// Drug name
    pub drug: String,
    /// Adverse event
    pub event: String,
    /// PRR value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// Information Component 2.5th percentile
    pub ic025: f64,
    /// EBGM 5th percentile
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for non-compensatory (geometric) risk scoring
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RiskScoreGeometricParams {
    /// Drug name
    pub drug: String,
    /// Adverse event
    pub event: String,
    /// PRR value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// Information Component 2.5th percentile
    pub ic025: f64,
    /// EBGM 5th percentile
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
    /// Scoring mode: "geometric" (default), "additive", or "dual" (both side-by-side)
    #[serde(default = "default_scoring_mode")]
    pub mode: String,
    /// Optional custom weights [PRR, ROR, IC025, EBGM05] (must sum to 1.0). Default: Bayesian-heavy [0.15, 0.15, 0.35, 0.35]
    pub weights: Option<Vec<f64>>,
}

fn default_scoring_mode() -> String {
    "dual".to_string()
}

/// Parameters for ToV level mapping
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MapToTovParams {
    /// Safety level: 1-8 (Molecular to Regulatory)
    pub level: u8,
}
