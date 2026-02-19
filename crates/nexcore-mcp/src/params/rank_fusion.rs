//! Rank Fusion Parameters
//! Tier: T2-C (Cross-domain composed primitive)
//!
//! Reciprocal Rank Fusion (RRF) and hybrid retrieval merging
//! for combining multiple ranking systems.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for Reciprocal Rank Fusion (RRF).
///
/// Merges multiple ranked lists using the formula:
/// `RRF(d) = sum(1 / (k + rank_i(d)))` for each ranking system i.
///
/// Standard k=60 works well for most use cases (Cormack et al., 2009).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RankFusionRrfParams {
    /// Multiple ranked lists, each keyed by ranker name.
    /// Each list contains item IDs in rank order (best first).
    /// Example: {"bm25": ["doc1","doc3","doc2"], "semantic": ["doc2","doc1","doc4"]}
    pub rankings: HashMap<String, Vec<String>>,
    /// RRF constant k (default: 60). Higher k reduces impact of high-ranking items.
    #[serde(default)]
    pub k: Option<u64>,
    /// Maximum results to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Optional weights per ranker (default: equal weight 1.0)
    /// Example: {"bm25": 0.4, "semantic": 0.6}
    #[serde(default)]
    pub weights: Option<HashMap<String, f64>>,
}

/// Parameters for hybrid retrieval fusion.
///
/// Combines dense (semantic) and sparse (keyword) retrieval scores
/// using configurable interpolation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RankFusionHybridParams {
    /// Dense retrieval results: item_id → similarity score (0.0-1.0)
    pub dense_scores: HashMap<String, f64>,
    /// Sparse retrieval results: item_id → BM25/TF-IDF score
    pub sparse_scores: HashMap<String, f64>,
    /// Interpolation weight for dense scores (default: 0.6).
    /// Final = alpha * dense + (1-alpha) * sparse
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Maximum results to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Normalize sparse scores to 0-1 range (default: true)
    #[serde(default)]
    pub normalize_sparse: Option<bool>,
}

/// Parameters for Borda count rank aggregation.
///
/// Each item receives points based on its position in each ranking.
/// Item at rank 1 gets N points, rank 2 gets N-1, etc.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RankFusionBordaParams {
    /// Multiple ranked lists keyed by ranker name.
    pub rankings: HashMap<String, Vec<String>>,
    /// Maximum results to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
}
