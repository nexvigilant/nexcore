//! Parameter types for Oracle MCP tools (Bayesian event prediction).

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for ingesting an event sequence into the Oracle.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OracleIngestParams {
    /// Ordered list of event kinds (e.g., ["read", "edit", "build", "test"]).
    pub events: Vec<String>,
}

/// Parameters for predicting the next event.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OraclePredictParams {
    /// The current event kind.
    pub current: String,
    /// Optional previous event for second-order (trigram) prediction.
    #[serde(default)]
    pub previous: Option<String>,
}

/// Parameters for recording what actually happened (accuracy tracking).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OracleObserveParams {
    /// The event we predicted from.
    pub predicted_from: String,
    /// What actually happened next.
    pub actual_next: String,
}

/// Parameters for Oracle status/report (no params needed).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OracleStatusParams {
    /// Placeholder (no params needed, but MCP requires a struct).
    #[serde(default)]
    pub _unused: Option<String>,
}

/// Parameters for resetting the Oracle.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OracleResetParams {
    /// Smoothing parameter for the new Oracle (default: 1.0).
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Accuracy tracking window size (default: 100).
    #[serde(default = "default_window")]
    pub accuracy_window: usize,
}

fn default_alpha() -> f64 {
    1.0
}
fn default_window() -> usize {
    100
}

/// Parameters for getting top-N predictions from a state.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct OracleTopPredictionsParams {
    /// The current event kind.
    pub current: String,
    /// Optional previous event for second-order context.
    #[serde(default)]
    pub previous: Option<String>,
    /// Number of top predictions to return (default: 5).
    #[serde(default = "default_top_n")]
    pub top_n: usize,
}

fn default_top_n() -> usize {
    5
}
