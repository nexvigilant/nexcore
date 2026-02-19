//! Insight Engine Parameters (Pattern Detection & Novelty)
//! Tier: T2-T3 (Cognitive Pattern Mining)
//!
//! Ingestion, status, configuration, connection, compression, and pattern recognition.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// A single observation for the InsightEngine.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightObservationInput {
    /// Unique key (e.g., "drug_a").
    pub key: String,
    /// String value.
    #[serde(default)]
    pub value: Option<String>,
    /// Numeric value.
    #[serde(default)]
    pub numeric_value: Option<f64>,
    /// Optional tags.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Parameters for ingesting observations.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightIngestParams {
    /// Observations to ingest.
    pub observations: Vec<InsightObservationInput>,
    /// Min co-occurrences.
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
    /// Confidence threshold.
    #[serde(default)]
    pub pattern_confidence_threshold: Option<f64>,
    /// Enable suddenness detection.
    #[serde(default)]
    pub enable_suddenness: Option<bool>,
    /// Suddenness threshold.
    #[serde(default)]
    pub suddenness_threshold: Option<f64>,
    /// Enable recursive learning (ρ).
    #[serde(default)]
    pub enable_recursive_learning: Option<bool>,
    /// Connection strength threshold.
    #[serde(default)]
    pub connection_strength_threshold: Option<f64>,
    /// Min compression ratio.
    #[serde(default)]
    pub compression_min_ratio: Option<f64>,
}

/// Parameters for status query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightStatusParams {
    /// Optional observations to replay.
    #[serde(default)]
    pub observations: Option<Vec<InsightObservationInput>>,
}

/// Parameters for configuration management.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightConfigParams {
    /// Min co-occurrences.
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
    /// Confidence threshold.
    #[serde(default)]
    pub pattern_confidence_threshold: Option<f64>,
    /// Connection strength threshold.
    #[serde(default)]
    pub connection_strength_threshold: Option<f64>,
    /// Min compression ratio.
    #[serde(default)]
    pub compression_min_ratio: Option<f64>,
    /// Enable suddenness detection.
    #[serde(default)]
    pub enable_suddenness: Option<bool>,
    /// Suddenness threshold.
    #[serde(default)]
    pub suddenness_threshold: Option<f64>,
    /// Enable recursive learning.
    #[serde(default)]
    pub enable_recursive_learning: Option<bool>,
}

/// Parameters for establishing connections.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightConnectParams {
    /// Source key
    pub from: String,
    /// Target key
    pub to: String,
    /// Relationship type
    pub relation: String,
    /// Connection strength (0.0-1.0)
    pub strength: f64,
}

/// Parameters for key compression.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightCompressParams {
    /// Keys to compress.
    pub keys: Vec<String>,
    /// The unifying principle.
    pub principle: String,
}

/// Parameters for pattern retrieval.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightPatternsParams {
    /// Observations to process.
    pub observations: Vec<InsightObservationInput>,
    /// Min co-occurrences.
    pub pattern_min_occurrences: Option<u64>,
}

/// Parameters for auto-compression.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightCompressAutoParams {
    /// Observations to ingest.
    pub observations: Vec<InsightObservationInput>,
    /// Min co-occurrences.
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
}

/// Parameters for resetting engine state.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightResetParams {
    /// Confirmation flag.
    #[serde(default)]
    pub confirm: Option<bool>,
}

/// Parameters for querying the engine.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightQueryParams {
    /// Key prefix filter.
    pub key_prefix: Option<String>,
    /// Tag filter.
    pub tag: Option<String>,
    /// Domain filter.
    pub domain: Option<String>,
    /// Max results.
    pub limit: Option<usize>,
}

/// Parameters for novelty detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightNoveltiesParams {
    /// Threshold.
    pub min_score: Option<f64>,
    /// Max results.
    pub limit: Option<usize>,
}
