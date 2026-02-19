//! Persistent Retrieval Pipeline Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Unified multi-source retrieval with caching, freshness tracking, and metrics.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for unified retrieval query across all knowledge sources.
///
/// Searches: brain artifacts, Qdrant vectors, filesystem KSB, implicit knowledge.
/// Results are ranked by composite score (relevance × freshness × source_weight).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RetrievalQueryParams {
    /// Search query text
    pub query: String,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Source filter: "all" (default), "brain", "qdrant", "filesystem", "implicit"
    #[serde(default)]
    pub source: Option<String>,
    /// Minimum relevance score threshold (0.0 - 1.0, default: 0.0)
    #[serde(default)]
    pub min_relevance: Option<f64>,
    /// Include cached results if available (default: true)
    #[serde(default)]
    pub use_cache: Option<bool>,
    /// Filter by domain tag (e.g., "pv", "guardian", "skills")
    #[serde(default)]
    pub domain: Option<String>,
}

/// Parameters for ingesting content into the retrieval index.
///
/// Stores content with metadata for later retrieval via `retrieval_query`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RetrievalIngestParams {
    /// Content to index
    pub content: String,
    /// Source identifier (e.g., file path, URL, session ID)
    pub source_id: String,
    /// Content title or label
    #[serde(default)]
    pub title: Option<String>,
    /// Domain tags for filtering (e.g., ["pv", "guardian"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// TTL in hours before this entry is considered stale (default: 168 = 7 days)
    #[serde(default)]
    pub ttl_hours: Option<u64>,
}

/// Parameters for retrieval pipeline statistics and health.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RetrievalStatsParams {
    /// Include top queries in response (default: true)
    #[serde(default)]
    pub include_top_queries: Option<bool>,
    /// Include freshness breakdown (default: true)
    #[serde(default)]
    pub include_freshness: Option<bool>,
}
