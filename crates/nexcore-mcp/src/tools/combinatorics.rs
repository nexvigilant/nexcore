//! Combinatorics MCP tools.
//!
//! Dudeney-derived combinatorial algorithms: Catalan numbers, derangements,
//! cycle decomposition, Josephus problem, grid paths, linear extensions.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::combinatorics::{
    CombBinomialParams, CombCatalanParams, CombCatalanTableParams, CombCycleDecompositionParams,
    CombDerangementParams, CombDerangementProbabilityParams, CombEliminationOrderParams,
    CombGridPathsParams, CombJosephusParams, CombLinearExtensionsParams,
    CombMinTranspositionsParams, CombMultinomialParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compute the nth Catalan number C(n).
pub fn comb_catalan(p: CombCatalanParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::catalan::catalan(p.n);
    ok_json(json!({
        "n": p.n,
        "catalan": result.to_string(),
    }))
}

/// Get the first 20 Catalan numbers as a lookup table.
pub fn comb_catalan_table(_p: CombCatalanTableParams) -> Result<CallToolResult, McpError> {
    let table = nexcore_combinatorics::catalan::catalan_table();
    let items: Vec<serde_json::Value> = table
        .iter()
        .map(|&(n, c)| json!({"n": n, "catalan": c.to_string()}))
        .collect();
    ok_json(json!({
        "count": items.len(),
        "table": items,
    }))
}

/// Decompose a permutation into disjoint cycles.
pub fn comb_cycle_decomposition(
    p: CombCycleDecompositionParams,
) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::cycle_decomposition(&p.permutation);
    ok_json(serde_json::to_value(&result).unwrap_or_default())
}

/// Compute minimum transpositions to sort a permutation.
pub fn comb_min_transpositions(p: CombMinTranspositionsParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::min_transpositions(&p.permutation);
    ok_json(json!({
        "n": p.permutation.len(),
        "min_transpositions": result,
    }))
}

/// Compute D(n), the number of derangements of n elements.
pub fn comb_derangement(p: CombDerangementParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::derangement(p.n);
    ok_json(json!({
        "n": p.n,
        "derangements": result.to_string(),
    }))
}

/// Compute the probability D(n)/n! that a random permutation is a derangement.
pub fn comb_derangement_probability(
    p: CombDerangementProbabilityParams,
) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::derangement_probability(p.n);
    let inv_e = 1.0_f64 / std::f64::consts::E;
    ok_json(json!({
        "n": p.n,
        "probability": result,
        "converges_to_1_over_e": inv_e,
        "deviation": (result - inv_e).abs(),
    }))
}

/// Count monotone lattice paths from (0,0) to (m,n).
pub fn comb_grid_paths(p: CombGridPathsParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::grid_paths(p.m, p.n);
    ok_json(json!({
        "m": p.m,
        "n": p.n,
        "paths": result.to_string(),
    }))
}

/// Compute the binomial coefficient C(n, k).
pub fn comb_binomial(p: CombBinomialParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::grid_paths::binomial(p.n, p.k);
    ok_json(json!({
        "n": p.n,
        "k": p.k,
        "binomial": result.to_string(),
    }))
}

/// Compute the multinomial coefficient.
pub fn comb_multinomial(p: CombMultinomialParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::grid_paths::multinomial(&p.lengths);
    let total: u32 = p.lengths.iter().sum();
    ok_json(json!({
        "lengths": p.lengths,
        "total_elements": total,
        "multinomial": result.to_string(),
    }))
}

/// Compute the Josephus survivor position (0-indexed).
pub fn comb_josephus(p: CombJosephusParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::josephus(p.n, p.k);
    ok_json(json!({
        "n": p.n,
        "k": p.k,
        "survivor_position": result,
    }))
}

/// Compute the full Josephus elimination order.
pub fn comb_elimination_order(p: CombEliminationOrderParams) -> Result<CallToolResult, McpError> {
    let order = nexcore_combinatorics::josephus::elimination_order(p.n, p.k);
    ok_json(json!({
        "n": p.n,
        "k": p.k,
        "elimination_order": order,
        "survivor": order.last(),
    }))
}

/// Count linear extensions for independent chains.
pub fn comb_linear_extensions(p: CombLinearExtensionsParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_combinatorics::count_linear_extensions_chains(&p.chain_lengths);
    let total_nodes: u32 = p.chain_lengths.iter().sum();
    ok_json(json!({
        "chain_lengths": p.chain_lengths,
        "total_nodes": total_nodes,
        "linear_extensions": result.to_string(),
    }))
}
