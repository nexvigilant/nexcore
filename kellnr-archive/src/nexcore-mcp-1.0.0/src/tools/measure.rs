//! Measure MCP tools — workspace quality measurement.
//!
//! Exposes nexcore-measure functionality as MCP tools:
//! - `measure_crate`: Single crate health assessment
//! - `measure_workspace`: Full workspace assessment
//! - `measure_entropy`: Shannon entropy calculation
//! - `measure_graph`: Dependency graph analysis
//! - `measure_drift`: Metric drift detection
//! - `measure_compare`: Side-by-side crate comparison
//! - `measure_stats`: Statistical summary of numeric data

use crate::params::{
    MeasureCompareParams, MeasureCrateParams, MeasureDriftParams, MeasureEntropyParams,
    MeasureStatsParams,
};
use nexcore_measure::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;

/// Default workspace root (`~/nexcore`).
fn ws_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("nexcore")
}

// ---------------------------------------------------------------------------
// measure_crate — single crate health
// ---------------------------------------------------------------------------

/// Measure a single crate's health: LOC, test count, entropy, and composite score.
pub fn measure_crate_tool(params: MeasureCrateParams) -> Result<CallToolResult, McpError> {
    let ws = ws_root();
    let measurement = collect::measure_crate(&ws, &params.name)
        .map_err(|e| McpError::internal_error(format!("measure_crate: {e}"), None))?;
    let now = MeasureTimestamp::now().epoch_secs();
    let health = composite::crate_health(&measurement, now, now);

    let result = json!({
        "measurement": measurement,
        "health": health,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// measure_workspace — full workspace assessment
// ---------------------------------------------------------------------------

/// Measure all crates in the workspace with aggregate health scoring.
pub fn measure_workspace_tool() -> Result<CallToolResult, McpError> {
    let ws = ws_root();
    let wm = collect::measure_workspace(&ws)
        .map_err(|e| McpError::internal_error(format!("measure_workspace: {e}"), None))?;
    let now = MeasureTimestamp::now().epoch_secs();
    let wh = composite::workspace_health(&wm.crates, now);

    let result = json!({
        "measurement": wm,
        "health": wh,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// measure_entropy — Shannon entropy from counts
// ---------------------------------------------------------------------------

/// Compute Shannon entropy, max entropy, and redundancy from category counts.
pub fn measure_entropy_tool(params: MeasureEntropyParams) -> Result<CallToolResult, McpError> {
    let h = entropy::shannon_entropy(&params.counts)
        .map_err(|e| McpError::internal_error(format!("entropy: {e}"), None))?;
    let h_max = entropy::max_entropy(params.counts.len())
        .map_err(|e| McpError::internal_error(format!("max_entropy: {e}"), None))?;
    let r = entropy::redundancy(&params.counts)
        .map_err(|e| McpError::internal_error(format!("redundancy: {e}"), None))?;

    let result = json!({
        "entropy": h.value(),
        "max_entropy": h_max.value(),
        "redundancy": r.value(),
        "n_categories": params.counts.len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// measure_graph — dependency graph analysis
// ---------------------------------------------------------------------------

/// Analyze the workspace dependency graph: density, depth, cycles, centrality.
pub fn measure_graph_tool() -> Result<CallToolResult, McpError> {
    let ws = ws_root();
    let dep_graph = graph::build_dep_graph(&ws)
        .map_err(|e| McpError::internal_error(format!("build_dep_graph: {e}"), None))?;
    let analysis = dep_graph.analyze();

    let result = json!({
        "density": analysis.density.value(),
        "depth": analysis.max_depth,
        "scc_count": analysis.cycle_count,
        "node_count": analysis.node_count,
        "edge_count": analysis.edge_count,
        "top_central": top_central_nodes(&analysis),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Extract top-10 nodes by betweenness centrality for the graph analysis result.
fn top_central_nodes(analysis: &GraphAnalysis) -> Vec<serde_json::Value> {
    let mut nodes: Vec<&GraphNode> = analysis.nodes.iter().collect();
    nodes.sort_by(|a, b| {
        b.betweenness_centrality
            .value()
            .total_cmp(&a.betweenness_centrality.value())
    });
    nodes
        .iter()
        .take(10)
        .map(|n| {
            json!({
                "crate": n.crate_id.name(),
                "betweenness": n.betweenness_centrality.value(),
                "degree": n.degree_centrality.value(),
                "fan_in": n.fan_in,
                "fan_out": n.fan_out,
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// measure_drift — detect metric drift in history
// ---------------------------------------------------------------------------

/// Detect statistically significant metric drift using Welch's t-test on history.
pub fn measure_drift_tool(params: MeasureDriftParams) -> Result<CallToolResult, McpError> {
    let path = history::default_history_path();
    let hist = history::MeasureHistory::load(&path)
        .map_err(|e| McpError::internal_error(format!("load_history: {e}"), None))?;
    let window = params.window.unwrap_or(5);
    let drifts = history::detect_drift(&hist, window)
        .map_err(|e| McpError::internal_error(format!("detect_drift: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&drifts).unwrap_or_else(|_| "[]".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// measure_compare — side-by-side crate comparison
// ---------------------------------------------------------------------------

/// Compare two crates side-by-side: health scores, LOC ratio, test density diff.
pub fn measure_compare_tool(params: MeasureCompareParams) -> Result<CallToolResult, McpError> {
    let ws = ws_root();
    let m_a = collect::measure_crate(&ws, &params.crate_a)
        .map_err(|e| McpError::internal_error(format!("crate_a: {e}"), None))?;
    let m_b = collect::measure_crate(&ws, &params.crate_b)
        .map_err(|e| McpError::internal_error(format!("crate_b: {e}"), None))?;
    let now = MeasureTimestamp::now().epoch_secs();
    let h_a = composite::crate_health(&m_a, now, now);
    let h_b = composite::crate_health(&m_b, now, now);

    let result = json!({
        "crate_a": {
            "name": params.crate_a,
            "health": h_a,
            "measurement": m_a,
        },
        "crate_b": {
            "name": params.crate_b,
            "health": h_b,
            "measurement": m_b,
        },
        "comparison": {
            "score_diff": h_a.score.value() - h_b.score.value(),
            "loc_ratio": if m_b.loc > 0 { m_a.loc as f64 / m_b.loc as f64 } else { 0.0 },
            "test_density_diff": m_a.test_density.value() - m_b.test_density.value(),
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// measure_stats — statistical summary
// ---------------------------------------------------------------------------

/// Compute statistical summary: mean, variance, 95% CI, and linear regression.
pub fn measure_stats_tool(params: MeasureStatsParams) -> Result<CallToolResult, McpError> {
    let data = &params.counts;
    if data.len() < 3 {
        return Err(McpError::invalid_params(
            "Need at least 3 data points for statistical analysis",
            None,
        ));
    }
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let se = (var / n).sqrt();
    let z = 1.96; // 95% CI
    let ci_lower = mean - z * se;
    let ci_upper = mean + z * se;

    let x: Vec<f64> = (0..data.len()).map(|i| i as f64).collect();
    let reg = stats::linear_regression(&x, data)
        .map_err(|e| McpError::internal_error(format!("regression: {e}"), None))?;

    let result = json!({
        "mean": mean,
        "variance": var,
        "std_dev": var.sqrt(),
        "ci_95": { "lower": ci_lower, "upper": ci_upper },
        "regression": {
            "slope": reg.slope,
            "intercept": reg.intercept,
            "r_squared": reg.r_squared,
            "p_value": reg.p_value,
        },
        "n": data.len(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
