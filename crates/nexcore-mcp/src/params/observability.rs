//! AI Observability Metrics Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Inference latency tracking, prediction confidence, feature drift,
//! and data freshness for AI-specific observability.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for recording an inference latency measurement.
///
/// Tracks per-tool or per-model latency with percentile computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ObservabilityRecordLatencyParams {
    /// Identifier for the model/tool/endpoint being measured
    pub endpoint_id: String,
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Whether the inference succeeded
    #[serde(default)]
    pub success: Option<bool>,
    /// Optional metadata tags (e.g., "model:gpt4", "region:us-east")
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Parameters for querying observability metrics.
///
/// Returns latency percentiles, error rates, and throughput for
/// a specific endpoint or all endpoints.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ObservabilityQueryParams {
    /// Endpoint ID to query (omit for all)
    #[serde(default)]
    pub endpoint_id: Option<String>,
    /// Time window in seconds to analyze (default: 300 = 5 min)
    #[serde(default)]
    pub window_secs: Option<u64>,
}

/// Parameters for recording data freshness.
///
/// Tracks when data sources were last updated and flags stale data
/// that could degrade model performance.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ObservabilityFreshnessParams {
    /// Data source identifier
    pub source_id: String,
    /// Last update timestamp (Unix seconds). Omit to record current time.
    #[serde(default)]
    pub last_updated: Option<f64>,
    /// Maximum acceptable age in seconds (SLA). Default: 3600 (1 hour)
    #[serde(default)]
    pub max_age_secs: Option<u64>,
}
