//! Parameter structs for signal-pipeline MCP tools.
//!
//! All structs prefixed with `Pipeline` to avoid collision with existing
//! signal detection params in `params/pv.rs`.

use schemars::JsonSchema;
use serde::Deserialize;

/// Compute all disproportionality metrics from a 2x2 contingency table.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineComputeAllParams {
    /// Cell a: drug+event co-reports
    pub a: u64,
    /// Cell b: drug without event
    pub b: u64,
    /// Cell c: event without drug
    pub c: u64,
    /// Cell d: neither drug nor event
    pub d: u64,
}

/// Batch compute metrics for multiple drug-event pairs.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineBatchComputeParams {
    /// Array of {drug, event, a, b, c, d} objects
    pub items: Vec<PipelineBatchItem>,
}

/// Single item in a batch compute request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineBatchItem {
    pub drug: String,
    pub event: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

/// Detect signals and apply Evans thresholds.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineDetectParams {
    pub drug: String,
    pub event: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    /// PRR minimum threshold (default: 2.0)
    pub prr_min: Option<f64>,
    /// Chi-square minimum threshold (default: 3.841)
    pub chi_square_min: Option<f64>,
    /// Minimum case count (default: 3)
    pub case_count_min: Option<u64>,
}

/// Validate a detection result against multiple checks.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineValidateParams {
    pub drug: String,
    pub event: String,
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    /// Use strict thresholds instead of Evans defaults
    pub strict: Option<bool>,
}

/// Get all threshold configurations (Evans, strict, sensitive).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineThresholdsParams {
    /// Optional: only return the named config ("evans", "strict", "sensitive")
    pub config: Option<String>,
}

/// Generate a report from batch detection results.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineReportParams {
    /// Array of {drug, event, a, b, c, d} objects
    pub items: Vec<PipelineBatchItem>,
    /// Report format: "json" or "table" (default: "json")
    pub format: Option<String>,
}

/// Get the PV pipeline relay chain with fidelity per stage.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineRelayChainParams {
    /// "full" (7-stage) or "core" (4-stage detection only)
    pub chain_type: Option<String>,
}

/// Look up cross-domain transfer mappings for signal-pipeline types.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelineTransferParams {
    /// Source type name to look up transfers for (e.g. "ContingencyTable", "DetectionResult")
    pub source_type: Option<String>,
    /// Target domain to filter by (e.g. "Biology", "Cloud")
    pub domain: Option<String>,
}

/// Get the crate's T1 primitive manifest.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipelinePrimitivesParams {
    /// If provided, look up the pipeline stage primitive for this stage name
    pub stage: Option<String>,
}
