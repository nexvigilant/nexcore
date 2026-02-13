//! Edit distance MCP tools
//!
//! Exposes the nexcore-edit-distance framework as MCP tools:
//! - Generic distance computation (Levenshtein, Damerau, LCS)
//! - Cross-domain transfer confidence
//! - Domain-adapted distance

use crate::params::{
    EditDistanceBatchParams, EditDistanceParams, EditDistanceSimilarityParams,
    EditDistanceTracebackParams, EditDistanceTransferParams,
};
use nexcore_edit_distance::prelude::*;
use nexcore_edit_distance::transfer::TransferRegistry;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Compute edit distance with selectable algorithm
pub fn edit_distance_compute(params: EditDistanceParams) -> Result<CallToolResult, McpError> {
    let algorithm = params.algorithm.as_deref().unwrap_or("levenshtein");
    let distance = match algorithm {
        "levenshtein" | "lev" => levenshtein(&params.source, &params.target),
        "damerau" | "damerau-levenshtein" | "dl" => {
            damerau_levenshtein(&params.source, &params.target)
        }
        "lcs" | "indel" => lcs_distance(&params.source, &params.target),
        other => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown algorithm: {other}. Use: levenshtein, damerau, lcs"
            ))]));
        }
    };

    let src_len = params.source.chars().count();
    let tgt_len = params.target.chars().count();
    let max_len = src_len.max(tgt_len);
    let similarity = if max_len == 0 {
        1.0
    } else {
        1.0 - (distance / max_len as f64)
    };

    let json = json!({
        "distance": distance,
        "similarity": similarity,
        "algorithm": algorithm,
        "source_len": src_len,
        "target_len": tgt_len,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Compute similarity with threshold check
pub fn edit_distance_similarity(
    params: EditDistanceSimilarityParams,
) -> Result<CallToolResult, McpError> {
    let m = Levenshtein::default();
    let sim = m.str_similarity(&params.source, &params.target);
    let threshold = params.threshold.unwrap_or(0.8);
    let matches = sim >= threshold;

    let json = json!({
        "similarity": sim,
        "distance": m.str_distance(&params.source, &params.target),
        "threshold": threshold,
        "matches": matches,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Compute edit distance with full traceback (operation sequence)
pub fn edit_distance_traceback(
    params: EditDistanceTracebackParams,
) -> Result<CallToolResult, McpError> {
    let m = LevenshteinTraceback::default();
    let result = m.str_solve(&params.source, &params.target);

    let ops: Vec<_> = result
        .operations
        .unwrap_or_default()
        .iter()
        .map(|op| match op {
            EditOp::Insert { pos, elem } => json!({
                "type": "insert",
                "pos": pos,
                "elem": elem.to_string(),
            }),
            EditOp::Delete { pos, elem } => json!({
                "type": "delete",
                "pos": pos,
                "elem": elem.to_string(),
            }),
            EditOp::Substitute { pos, from, to } => json!({
                "type": "substitute",
                "pos": pos,
                "from": from.to_string(),
                "to": to.to_string(),
            }),
            EditOp::Transpose { pos, first, second } => json!({
                "type": "transpose",
                "pos": pos,
                "first": first.to_string(),
                "second": second.to_string(),
            }),
        })
        .collect();

    let json = json!({
        "distance": result.distance,
        "operations": ops,
        "cells_computed": result.cells_computed,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Look up cross-domain transfer confidence
pub fn edit_distance_transfer(
    params: EditDistanceTransferParams,
) -> Result<CallToolResult, McpError> {
    let reg = TransferRegistry::with_defaults();
    let maps = reg.lookup(&params.source_domain, &params.target_domain);

    if maps.is_empty() {
        let all_domains: Vec<_> = reg
            .all()
            .iter()
            .flat_map(|m| vec![m.source_domain.clone(), m.target_domain.clone()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let json = json!({
            "found": false,
            "message": format!("No transfer map for '{}' → '{}'", params.source_domain, params.target_domain),
            "available_domains": all_domains,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let results: Vec<_> = maps
        .iter()
        .map(|m| {
            json!({
                "source_domain": m.source_domain,
                "target_domain": m.target_domain,
                "target_equivalent": m.target_equivalent,
                "confidence": m.confidence(),
                "structural": m.structural,
                "functional": m.functional,
                "contextual": m.contextual,
                "limiting_factor": m.limiting_factor(),
                "caveat": m.caveat,
            })
        })
        .collect();

    let json = json!({
        "found": true,
        "transfers": results,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Batch edit distance: compare query against multiple candidates
pub fn edit_distance_batch(params: EditDistanceBatchParams) -> Result<CallToolResult, McpError> {
    let algorithm = params.algorithm.as_deref().unwrap_or("levenshtein");
    let min_sim = params.min_similarity.unwrap_or(0.0);

    let dist_fn: fn(&str, &str) -> f64 = match algorithm {
        "levenshtein" | "lev" => levenshtein,
        "damerau" | "damerau-levenshtein" | "dl" => damerau_levenshtein,
        "lcs" | "indel" => lcs_distance,
        other => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown algorithm: {other}. Use: levenshtein, damerau, lcs"
            ))]));
        }
    };

    let mut results: Vec<_> = params
        .candidates
        .iter()
        .map(|candidate| {
            let distance = dist_fn(&params.query, candidate);
            let q_len = params.query.chars().count();
            let c_len = candidate.chars().count();
            let max_len = q_len.max(c_len);
            let similarity = if max_len == 0 {
                1.0
            } else {
                1.0 - (distance / max_len as f64)
            };
            (candidate, distance, similarity)
        })
        .filter(|(_, _, sim)| *sim >= min_sim)
        .collect();

    // Sort by similarity descending
    results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    if let Some(limit) = params.limit {
        results.truncate(limit);
    }

    let matches: Vec<_> = results
        .iter()
        .map(|(candidate, distance, similarity)| {
            json!({
                "candidate": candidate,
                "distance": distance,
                "similarity": similarity,
            })
        })
        .collect();

    let json = json!({
        "query": params.query,
        "algorithm": algorithm,
        "total_candidates": params.candidates.len(),
        "matches_returned": matches.len(),
        "matches": matches,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
