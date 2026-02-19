//! Rank Fusion — Reciprocal Rank Fusion (RRF) and hybrid retrieval merging.
//!
//! Inspired by AI Engineering Bible Section 31 (Context & Retrieval Refinements):
//! formal rank fusion algorithms for combining multiple retrieval systems
//! (dense semantic + sparse keyword) into a single unified ranking.
//!
//! # Algorithms
//!
//! 1. **RRF** (Reciprocal Rank Fusion): `score(d) = Σ(w_i / (k + rank_i(d)))`
//!    Standard k=60. Robust, parameter-free, outperforms most learned merging.
//!    (Cormack, Clarke, Buettcher, 2009)
//!
//! 2. **Hybrid Interpolation**: `score(d) = α × dense(d) + (1-α) × sparse(d)`
//!    Linear combination of dense (semantic) and sparse (BM25) scores.
//!
//! 3. **Borda Count**: Position-based voting. Item at rank 1 gets N points,
//!    rank 2 gets N-1, etc. Sum across all rankers.
//!
//! # T1 Grounding: σ(Sequence) + μ(Mapping) + κ(Comparison) + N(Quantity)
//! - σ: Ranked sequences from multiple systems
//! - μ: Rank → score mapping functions
//! - κ: Score comparison for final ordering
//! - N: Numeric fusion scores

use crate::params::{RankFusionBordaParams, RankFusionHybridParams, RankFusionRrfParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// MCP Tools
// ============================================================================

/// `rank_fusion_rrf` — Reciprocal Rank Fusion.
///
/// Merges multiple ranked lists using: `score(d) = Σ(w_i / (k + rank_i(d)))`
/// Default k=60 (Cormack et al., 2009). Robust across diverse ranking systems.
pub fn rank_fusion_rrf(params: RankFusionRrfParams) -> Result<CallToolResult, McpError> {
    if params.rankings.is_empty() {
        return Err(McpError::invalid_params(
            "At least one ranking list is required".to_string(),
            None,
        ));
    }

    let k = params.k.unwrap_or(60);
    let limit = params.limit.unwrap_or(20);
    let default_weight = 1.0;

    // Compute RRF scores
    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut item_sources: HashMap<String, Vec<(String, usize)>> = HashMap::new(); // item → [(ranker, rank)]

    for (ranker_name, ranked_list) in &params.rankings {
        let weight = params
            .weights
            .as_ref()
            .and_then(|w| w.get(ranker_name))
            .copied()
            .unwrap_or(default_weight);

        for (rank_idx, item_id) in ranked_list.iter().enumerate() {
            let rank = rank_idx + 1; // 1-based rank
            let rrf_score = weight / (k as f64 + rank as f64);
            *scores.entry(item_id.clone()).or_insert(0.0) += rrf_score;

            item_sources
                .entry(item_id.clone())
                .or_default()
                .push((ranker_name.clone(), rank));
        }
    }

    // Sort by RRF score descending
    let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.truncate(limit);

    // Build results
    let results: Vec<serde_json::Value> = ranked
        .iter()
        .enumerate()
        .map(|(i, (item_id, score))| {
            let sources = item_sources.get(item_id).cloned().unwrap_or_default();
            let source_details: Vec<serde_json::Value> = sources
                .iter()
                .map(|(ranker, rank)| json!({"ranker": ranker, "rank": rank}))
                .collect();

            json!({
                "rank": i + 1,
                "item_id": item_id,
                "rrf_score": (*score * 100000.0).round() / 100000.0,
                "sources": source_details,
                "source_count": sources.len(),
            })
        })
        .collect();

    let ranker_stats: Vec<serde_json::Value> = params
        .rankings
        .iter()
        .map(|(name, list)| {
            let weight = params
                .weights
                .as_ref()
                .and_then(|w| w.get(name))
                .copied()
                .unwrap_or(default_weight);
            json!({
                "ranker": name,
                "items": list.len(),
                "weight": weight,
            })
        })
        .collect();

    let unique_items: usize = ranked.len();
    let total_items: usize = params.rankings.values().map(|v| v.len()).sum();

    let result = json!({
        "algorithm": "reciprocal_rank_fusion",
        "formula": "score(d) = Σ(w_i / (k + rank_i(d)))",
        "k": k,
        "ranker_count": params.rankings.len(),
        "rankers": ranker_stats,
        "total_items_across_rankers": total_items,
        "unique_items_after_fusion": unique_items,
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `rank_fusion_hybrid` — Hybrid dense+sparse score interpolation.
///
/// Combines semantic similarity scores with keyword-based scores:
/// `final(d) = α × dense(d) + (1-α) × normalized_sparse(d)`
pub fn rank_fusion_hybrid(params: RankFusionHybridParams) -> Result<CallToolResult, McpError> {
    if params.dense_scores.is_empty() && params.sparse_scores.is_empty() {
        return Err(McpError::invalid_params(
            "At least one score set must be non-empty".to_string(),
            None,
        ));
    }

    let alpha = params.alpha.unwrap_or(0.6);
    let limit = params.limit.unwrap_or(20);
    let normalize_sparse = params.normalize_sparse.unwrap_or(true);

    // Optionally normalize sparse scores to [0, 1]
    let sparse_scores = if normalize_sparse && !params.sparse_scores.is_empty() {
        let max_sparse = params
            .sparse_scores
            .values()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let min_sparse = params
            .sparse_scores
            .values()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let range = max_sparse - min_sparse;

        if range > f64::EPSILON {
            params
                .sparse_scores
                .iter()
                .map(|(k, &v)| (k.clone(), (v - min_sparse) / range))
                .collect()
        } else {
            params.sparse_scores.clone()
        }
    } else {
        params.sparse_scores.clone()
    };

    // Collect all unique item IDs
    let mut all_items: Vec<String> = Vec::new();
    for key in params.dense_scores.keys() {
        if !all_items.contains(key) {
            all_items.push(key.clone());
        }
    }
    for key in sparse_scores.keys() {
        if !all_items.contains(key) {
            all_items.push(key.clone());
        }
    }

    // Compute hybrid scores
    let mut scored: Vec<(String, f64, f64, f64)> = Vec::new(); // (id, hybrid, dense, sparse)

    for item_id in &all_items {
        let dense = params.dense_scores.get(item_id).copied().unwrap_or(0.0);
        let sparse = sparse_scores.get(item_id).copied().unwrap_or(0.0);
        let hybrid = alpha * dense + (1.0 - alpha) * sparse;
        scored.push((item_id.clone(), hybrid, dense, sparse));
    }

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.truncate(limit);

    let results: Vec<serde_json::Value> = scored
        .iter()
        .enumerate()
        .map(|(i, (item_id, hybrid, dense, sparse))| {
            let in_dense = params.dense_scores.contains_key(item_id);
            let in_sparse = params.sparse_scores.contains_key(item_id);

            json!({
                "rank": i + 1,
                "item_id": item_id,
                "hybrid_score": (*hybrid * 100000.0).round() / 100000.0,
                "dense_score": (*dense * 100000.0).round() / 100000.0,
                "sparse_score": (*sparse * 100000.0).round() / 100000.0,
                "in_dense": in_dense,
                "in_sparse": in_sparse,
                "coverage": if in_dense && in_sparse { "both" } else if in_dense { "dense_only" } else { "sparse_only" },
            })
        })
        .collect();

    let both_count = scored
        .iter()
        .filter(|(id, ..)| {
            params.dense_scores.contains_key(id) && params.sparse_scores.contains_key(id)
        })
        .count();

    let result = json!({
        "algorithm": "hybrid_interpolation",
        "formula": format!("score = {:.2} × dense + {:.2} × sparse", alpha, 1.0 - alpha),
        "alpha": alpha,
        "dense_items": params.dense_scores.len(),
        "sparse_items": params.sparse_scores.len(),
        "unique_items": all_items.len(),
        "overlap_items": both_count,
        "sparse_normalized": normalize_sparse,
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `rank_fusion_borda` — Borda count rank aggregation.
///
/// Position-based voting: item at rank 1 gets N points, rank 2 gets N-1, etc.
/// Simple but effective when rankings have different scales.
pub fn rank_fusion_borda(params: RankFusionBordaParams) -> Result<CallToolResult, McpError> {
    if params.rankings.is_empty() {
        return Err(McpError::invalid_params(
            "At least one ranking list is required".to_string(),
            None,
        ));
    }

    let limit = params.limit.unwrap_or(20);

    let mut scores: HashMap<String, f64> = HashMap::new();
    let mut item_ranks: HashMap<String, Vec<(String, usize, f64)>> = HashMap::new();

    for (ranker_name, ranked_list) in &params.rankings {
        let n = ranked_list.len();
        for (idx, item_id) in ranked_list.iter().enumerate() {
            let points = (n - idx) as f64; // Rank 1 = N points, Rank N = 1 point
            *scores.entry(item_id.clone()).or_insert(0.0) += points;
            item_ranks.entry(item_id.clone()).or_default().push((
                ranker_name.clone(),
                idx + 1,
                points,
            ));
        }
    }

    // Normalize by max possible score
    let max_possible: f64 = params.rankings.values().map(|v| v.len() as f64).sum();

    let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    ranked.truncate(limit);

    let results: Vec<serde_json::Value> = ranked
        .iter()
        .enumerate()
        .map(|(i, (item_id, score))| {
            let ranks = item_ranks.get(item_id).cloned().unwrap_or_default();
            let rank_details: Vec<serde_json::Value> = ranks
                .iter()
                .map(|(ranker, rank, pts)| json!({"ranker": ranker, "rank": rank, "points": pts}))
                .collect();

            json!({
                "rank": i + 1,
                "item_id": item_id,
                "borda_score": (*score * 100.0).round() / 100.0,
                "normalized_score": if max_possible > 0.0 {
                    (*score / max_possible * 10000.0).round() / 10000.0
                } else {
                    0.0
                },
                "appearances": ranks.len(),
                "rank_details": rank_details,
            })
        })
        .collect();

    let result = json!({
        "algorithm": "borda_count",
        "formula": "score(d) = Σ(N_i - rank_i(d) + 1) for each ranker i",
        "ranker_count": params.rankings.len(),
        "max_possible_score": max_possible,
        "unique_items": ranked.len(),
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
