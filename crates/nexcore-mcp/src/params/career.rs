//! Career Transitions Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Career transition graph from KSB corpus.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `career_transitions`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CareerTransitionsParams {
    /// List of career role IDs to include (default: all seed roles)
    #[serde(default)]
    pub roles: Option<Vec<String>>,
    /// Minimum similarity to include an edge (default 0.15)
    #[serde(default)]
    pub threshold: Option<f64>,
    /// Include value-mining salary signals (default false)
    #[serde(default)]
    pub include_salary: Option<bool>,
}
