//! Params for lymphatic system (overflow, output quality, inspection) tools.

use serde::Deserialize;

/// Analyze drainage capacity (overflow management).
#[derive(Debug, Deserialize)]
pub struct LymphaticDrainageParams {
    /// Items to drain/filter (e.g., output tokens, log entries)
    pub item_count: u64,
    /// Capacity limit
    pub capacity: Option<u64>,
}

/// Run thymic selection on a candidate (quality gate).
#[derive(Debug, Deserialize)]
pub struct LymphaticThymicParams {
    /// Candidate identifier (e.g., tool name, skill name)
    pub candidate: String,
    /// Quality criteria to check against
    pub criteria: Option<Vec<String>>,
}

/// Inspect a node in the lymphatic network.
#[derive(Debug, Deserialize)]
pub struct LymphaticInspectParams {
    /// Node identifier to inspect
    pub node: String,
}

/// Get lymphatic system health overview.
#[derive(Debug, Deserialize)]
pub struct LymphaticHealthParams {}
