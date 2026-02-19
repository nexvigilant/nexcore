//! Foundation tools: algorithms, crypto, YAML, graph, FSRS
//!
//! These tools wrap nexcore-vigilance::foundation algorithms for 10-63x speedup over Python.

use crate::params::{
    ConceptGrepParams, FsrsReviewParams, FuzzySearchParams, GraphLevelsParams, GraphTopsortParams,
    LevenshteinBoundedParams, LevenshteinParams, Sha256Params, YamlParseParams,
};
use nexcore_vigilance::foundation::{
    Card, FsrsScheduler, Rating, SkillGraph, SkillNode, expand_concept,
    fuzzy_search as nf_fuzzy_search, levenshtein as nf_levenshtein,
    levenshtein_bounded as nf_levenshtein_bounded, sha256_hash,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Calculate Levenshtein edit distance.
///
/// **Note:** For multi-algorithm support (Damerau, LCS), prefer
/// `edit_distance_compute` which is a superset of this tool.
/// This tool is retained for backward compatibility.
pub fn calc_levenshtein(params: LevenshteinParams) -> Result<CallToolResult, McpError> {
    let result = nf_levenshtein(&params.source, &params.target);
    let json = json!({
        "distance": result.distance,
        "similarity": result.similarity,
        "source_len": result.source_len,
        "target_len": result.target_len,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Calculate bounded Levenshtein edit distance with early termination
pub fn calc_levenshtein_bounded(
    params: LevenshteinBoundedParams,
) -> Result<CallToolResult, McpError> {
    match nf_levenshtein_bounded(&params.source, &params.target, params.max_distance) {
        Some(distance) => {
            let max_len = params
                .source
                .chars()
                .count()
                .max(params.target.chars().count());
            let similarity = if max_len == 0 {
                1.0
            } else {
                ((1.0 - (distance as f64 / max_len as f64)) * 10000.0).round() / 10000.0
            };
            let json = json!({
                "distance": distance,
                "exceeded": false,
                "similarity": similarity,
                "max_distance": params.max_distance,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        None => {
            let json = json!({
                "distance": null,
                "exceeded": true,
                "similarity": null,
                "max_distance": params.max_distance,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Batch fuzzy search
pub fn fuzzy_search(params: FuzzySearchParams) -> Result<CallToolResult, McpError> {
    let matches = nf_fuzzy_search(&params.query, &params.candidates, params.limit);
    let results: Vec<_> = matches
        .iter()
        .map(|m| {
            json!({
                "candidate": m.candidate,
                "distance": m.distance,
                "similarity": m.similarity,
            })
        })
        .collect();
    let json = json!({
        "query": params.query,
        "matches": results,
        "count": results.len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// SHA-256 hash
pub fn sha256(params: Sha256Params) -> Result<CallToolResult, McpError> {
    let result = sha256_hash(&params.input);
    let json = json!({
        "hex": result.hex,
        "input_len": params.input.len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Parse YAML to JSON
pub fn yaml_parse(params: YamlParseParams) -> Result<CallToolResult, McpError> {
    match serde_yaml::from_str::<serde_json::Value>(&params.content) {
        Ok(value) => Ok(CallToolResult::success(vec![Content::text(
            value.to_string(),
        )])),
        Err(e) => {
            let err_json = json!({
                "error": format!("YAML parse error: {}", e),
            });
            Ok(CallToolResult::success(vec![Content::text(
                err_json.to_string(),
            )]))
        }
    }
}

/// Topological sort
pub fn graph_topsort(params: GraphTopsortParams) -> Result<CallToolResult, McpError> {
    let mut graph = SkillGraph::new();

    // Build dependency map from edges
    let mut deps: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut all_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (from, to) in &params.edges {
        all_nodes.insert(from.clone());
        all_nodes.insert(to.clone());
        deps.entry(to.clone()).or_default().push(from.clone());
    }

    // Add all nodes with their dependencies
    for node_name in all_nodes {
        let node_deps: Vec<&str> = deps
            .get(&node_name)
            .map(|d| d.iter().map(String::as_str).collect())
            .unwrap_or_default();
        graph.add_node(SkillNode::simple(&node_name, node_deps));
    }

    match graph.topological_sort() {
        Ok(order) => {
            let json = json!({
                "order": order,
                "count": order.len(),
                "has_cycle": false,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(cycle) => {
            let json = json!({
                "error": format!("Cycle detected: {:?}", cycle),
                "cycle": cycle,
                "has_cycle": true,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Compute parallel execution levels
pub fn graph_levels(params: GraphLevelsParams) -> Result<CallToolResult, McpError> {
    let mut graph = SkillGraph::new();

    // Build dependency map from edges
    let mut deps: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut all_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (from, to) in &params.edges {
        all_nodes.insert(from.clone());
        all_nodes.insert(to.clone());
        deps.entry(to.clone()).or_default().push(from.clone());
    }

    // Add all nodes with their dependencies
    for node_name in all_nodes {
        let node_deps: Vec<&str> = deps
            .get(&node_name)
            .map(|d| d.iter().map(String::as_str).collect())
            .unwrap_or_default();
        graph.add_node(SkillNode::simple(&node_name, node_deps));
    }

    match graph.level_parallelization() {
        Ok(levels) => {
            let json = json!({
                "levels": levels,
                "depth": levels.len(),
                "max_parallel": levels.iter().map(|l| l.len()).max().unwrap_or(0),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(cycle) => {
            let json = json!({
                "error": format!("Cycle detected: {:?}", cycle),
                "cycle": cycle,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// FSRS spaced repetition review
pub fn fsrs_review(params: FsrsReviewParams) -> Result<CallToolResult, McpError> {
    let rating = match params.rating {
        1 => Rating::Again,
        2 => Rating::Hard,
        3 => Rating::Good,
        4 => Rating::Easy,
        _ => {
            let json = json!({
                "error": "Invalid rating. Must be 1 (Again), 2 (Hard), 3 (Good), or 4 (Easy).",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    // Create a card with the provided state
    let card = Card {
        stability: params.stability,
        difficulty: params.difficulty,
        elapsed_days: u64::from(params.elapsed_days),
        state: nexcore_vigilance::foundation::CardState::Review,
        ..Default::default()
    };

    let scheduler = FsrsScheduler::default();
    let result = scheduler.review(&card, rating, u64::from(params.elapsed_days));

    let json = json!({
        "new_stability": result.card.stability,
        "new_difficulty": result.card.difficulty,
        "scheduled_days": result.scheduled_days,
        "retrievability": result.retrievability,
        "rating": params.rating,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Deterministic concept expansion for multi-variant search
pub fn concept_grep(params: ConceptGrepParams) -> Result<CallToolResult, McpError> {
    let expansion = expand_concept(&params.concept, params.sections);
    let json = json!({
        "original": expansion.original,
        "patterns": expansion.patterns,
        "regex": expansion.regex,
        "pattern_count": expansion.patterns.len(),
        "sections": expansion.sections,
    });
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
