//! Engram knowledge store MCP tool parameters.
//!
//! Typed parameter structs for the unified knowledge daemon:
//! search, decay scoring, duplicate detection, store statistics,
//! and multi-source ingestion.

use schemars::JsonSchema;
use serde::Deserialize;

/// Search the engram store using TF-IDF ranking.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramSearchParams {
    /// Search query (keywords or phrases).
    pub query: String,
    /// Maximum results to return. Default: 10.
    pub limit: Option<usize>,
}

/// Search with temporal decay — recent knowledge ranks higher.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramSearchDecayParams {
    /// Search query (keywords or phrases).
    pub query: String,
    /// Maximum results to return. Default: 10.
    pub limit: Option<usize>,
}

/// Get a specific engram by ID (read-only, no access tracking).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramPeekParams {
    /// Engram ID to retrieve.
    pub id: u64,
}

/// Get store statistics (total, active, stale counts by layer).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramStatsParams {}

/// Find near-duplicate engrams by content similarity.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramFindDuplicatesParams {
    /// Jaccard similarity threshold [0.0, 1.0]. Default: 0.3.
    pub threshold: Option<f64>,
}

/// Compute temporal decay score for an engram.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramDecayScoreParams {
    /// ISO 8601 timestamp when the engram was created.
    pub created_at: String,
    /// ISO 8601 timestamp of last access.
    pub last_accessed: String,
    /// Number of times accessed.
    pub access_count: u64,
    /// Half-life in days. Default: 14.0.
    pub half_life_days: Option<f64>,
    /// Stale threshold [0.0, 1.0]. Default: 0.1.
    pub stale_threshold: Option<f64>,
}

/// Ingest knowledge from a source file/directory into the store.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramIngestParams {
    /// Source type: "memory_md", "brain_dir", "lessons_jsonl", or "implicit_json".
    pub source_type: String,
    /// Path to the source file or directory.
    pub path: String,
}

/// List engrams filtered by source layer.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EngramBySourceParams {
    /// Source layer: "memory", "brain", "lesson", "implicit", or "session".
    pub source: String,
}
