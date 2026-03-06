//! Test history query parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{self, Deserialize};

/// Query test run history with optional filters.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct TestHistoryQueryParams {
    /// Filter by crate name (optional).
    pub crate_name: Option<String>,
    /// Only include runs from last N days (optional, default: 30).
    pub since_days: Option<u32>,
    /// Maximum rows to return (optional, default: 50).
    pub limit: Option<u32>,
}

/// Identify flaky tests that alternate between pass and fail.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(crate = "rmcp::schemars")]
pub struct TestHistoryFlakyParams {
    /// Filter by crate name (optional).
    pub crate_name: Option<String>,
    /// Minimum number of fail occurrences to count as flaky (optional, default: 2).
    pub min_flips: Option<u32>,
    /// Analysis window in days (optional, default: 14).
    pub window_days: Option<u32>,
}
