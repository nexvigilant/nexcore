//! PV Domain Embedding Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Domain-specific embeddings for pharmacovigilance terminology using
//! TF-IDF sparse vectors and graph-based semantic similarity.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for computing PV term similarity.
///
/// Uses TF-IDF definition vectors + see_also graph distance to find
/// the most semantically similar PV terms to a query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvEmbeddingSimilarityParams {
    /// Term or concept to find similar PV terms for
    pub query: String,
    /// Maximum results (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Minimum similarity threshold (0.0 - 1.0, default: 0.1)
    #[serde(default)]
    pub min_similarity: Option<f64>,
    /// Include definition vectors in output (default: false)
    #[serde(default)]
    pub include_vectors: Option<bool>,
}

/// Parameters for computing the PV embedding for a specific term.
///
/// Returns the sparse TF-IDF vector, graph neighbors, and metadata.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvEmbeddingGetParams {
    /// ICH term name to get embedding for
    pub term: String,
}

/// Parameters for PV embedding index statistics.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvEmbeddingStatsParams {}
