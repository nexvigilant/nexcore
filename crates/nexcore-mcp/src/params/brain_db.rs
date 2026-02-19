//! Brain Database Parameters (Persistent Memory)
//!
//! Handoff management and limit-based filtering.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for brain handoff retrieval.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainDbHandoffsParams {
    /// Optional project filter.
    pub project: Option<String>,
    /// Maximum results.
    pub limit: Option<u32>,
}

/// Parameters for raw read-only SQL query against brain.db.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainDbQueryParams {
    /// SQL query to execute. Must be a SELECT statement (read-only).
    /// Examples: "SELECT * FROM beliefs", "SELECT tool_name, total_calls FROM tool_usage ORDER BY total_calls DESC LIMIT 10"
    pub sql: String,
    /// Maximum rows to return (default: 100, max: 500).
    #[serde(default)]
    pub limit: Option<u32>,
}
