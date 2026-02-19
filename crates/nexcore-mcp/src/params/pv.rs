//! PV Signal Detection Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Pharmacovigilance signal detection and causality assessment parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Contingency table for signal detection (2x2)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ContingencyTableParams {
    /// Drug + Event count (cell a)
    pub a: u64,
    /// Drug + No Event count (cell b)
    pub b: u64,
    /// No Drug + Event count (cell c)
    pub c: u64,
    /// No Drug + No Event count (cell d)
    pub d: u64,
}

/// Parameters for complete signal analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalCompleteParams {
    /// Contingency table
    #[serde(flatten)]
    pub table: ContingencyTableParams,
    /// PRR threshold (default: 2.0)
    #[serde(default = "default_prr_threshold")]
    pub prr_threshold: f64,
    /// Minimum case count (default: 3)
    #[serde(default = "default_min_n")]
    pub min_n: u32,
}

fn default_prr_threshold() -> f64 {
    2.0
}

fn default_min_n() -> u32 {
    3
}

/// Parameters for individual signal algorithm
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalAlgorithmParams {
    /// Contingency table
    #[serde(flatten)]
    pub table: ContingencyTableParams,
}

/// Parameters for Naranjo causality assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NaranjoParams {
    /// Temporal relationship: 1=yes, 0=unknown, -1=no
    pub temporal: i32,
    /// Improved after withdrawal: 1=yes, 0=unknown, -1=no
    pub dechallenge: i32,
    /// Recurred on re-exposure: 1=yes, 0=unknown, -1=no
    pub rechallenge: i32,
    /// Alternative causes exist: 1=yes, -1=no, 0=unknown
    pub alternatives: i32,
    /// Previously reported: 1=yes, 0=no
    pub previous: i32,
}

// ============================================================================
// Signal Pipeline Parameters (signal-stats / signal-core)
// ============================================================================

/// Parameters for single drug-event signal detection via signal-stats pipeline.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalDetectParams {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// Cell a: drug+ event+
    pub a: u64,
    /// Cell b: drug+ event-
    pub b: u64,
    /// Cell c: drug- event+
    pub c: u64,
    /// Cell d: drug- event-
    pub d: u64,
}

/// Parameters for batch signal detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalBatchParams {
    /// List of drug-event pairs to analyze
    pub items: Vec<SignalDetectParams>,
}

// PvPipelineParams re-exported from regulatory for qualified path access
pub use super::regulatory::PvPipelineParams;

/// Parameters for WHO-UMC causality assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WhoUmcParams {
    /// Temporal relationship: 1=yes, 0=unknown, -1=no
    pub temporal: i32,
    /// Improved after withdrawal: 1=yes, 0=unknown, -1=no
    pub dechallenge: i32,
    /// Recurred on re-exposure: 1=yes, 0=unknown, -1=no
    pub rechallenge: i32,
    /// Alternative causes exist: 1=yes, -1=no, 0=unknown
    pub alternatives: i32,
    /// Pharmacological plausibility: 1=yes, 0=unknown, -1=no
    pub plausibility: i32,
}
