//! Anatomy MCP tools — workspace structural analysis.
//!
//! Exposes nexcore-anatomy functionality as MCP tools:
//! - `anatomy_health`: Full workspace health report
//! - `anatomy_blast_radius`: Blast radius for a specific crate
//! - `anatomy_chomsky`: Chomsky classification for a crate or all crates
//! - `anatomy_violations`: List boundary violations
//!
//! ## Primitive Foundation
//! - σ (Sequence): Topological ordering
//! - μ (Mapping): Crate → layer/score/level
//! - κ (Comparison): Threshold-based classification
//! - ∂ (Boundary): Layer boundary enforcement
//! - ρ (Recursion): Cycle detection via Kahn's algorithm
//! - N (Quantity): Fan-in/out, density, blast radius metrics

use crate::params::{AnatomyBlastRadiusParams, AnatomyChomskyParams};
use nexcore_anatomy::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;

/// Default workspace manifest (`~/nexcore/Cargo.toml`).
fn ws_manifest() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join("nexcore").join("Cargo.toml")
}

/// Load workspace dependency graph via nexcore-anatomy.
fn load_graph() -> Result<DependencyGraph, McpError> {
    let manifest = ws_manifest();
    DependencyGraph::from_manifest_path(&manifest)
        .map_err(|e| McpError::internal_error(e.to_string(), None))
}

// ---------------------------------------------------------------------------
// anatomy_health — full workspace health report
// ---------------------------------------------------------------------------

/// Full workspace anatomy: health status, crate count, density, bottleneck,
/// criticality distribution, boundary violations, and Chomsky distribution.
///
/// Tier: T3 (nexcore-anatomy × MCP integration)
pub fn anatomy_health_tool() -> Result<CallToolResult, McpError> {
    let graph = load_graph()?;
    let blast = BlastRadiusReport::from_graph(&graph);
    let chomsky = ChomskyReport::from_graph(&graph);
    let report = nexcore_anatomy::AnatomyReport::from_graph(graph);

    let result = json!({
        "health": report.summary.health.label(),
        "total_crates": report.summary.total_crates,
        "cycles": report.summary.cycle_count,
        "max_depth": report.summary.max_depth,
        "graph_density": report.summary.graph_density,
        "bottleneck": {
            "crate": report.summary.bottleneck,
            "fan_in": report.summary.max_fan_in,
            "blast_radius_pct": format!("{:.1}", blast.worst_case_ratio * 100.0),
        },
        "criticality": {
            "critical": report.summary.critical_count,
            "supporting": report.summary.supporting_count,
            "experimental": report.summary.experimental_count,
        },
        "violations": report.layers.violations.iter().map(|v| json!({
            "from": v.from_crate,
            "from_layer": format!("{:?}", v.from_layer),
            "to": v.to_crate,
            "to_layer": format!("{:?}", v.to_layer),
            "severity": v.severity,
        })).collect::<Vec<_>>(),
        "chomsky": {
            "distribution": chomsky.level_distribution,
            "avg_generators": chomsky.avg_generators,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// anatomy_blast_radius — per-crate blast radius
// ---------------------------------------------------------------------------

/// Compute blast radius for a specific crate: direct/transitive dependents,
/// impact ratio, cascade depth/width, and containment score.
///
/// Tier: T3 (nexcore-anatomy × MCP integration)
pub fn anatomy_blast_radius_tool(
    params: AnatomyBlastRadiusParams,
) -> Result<CallToolResult, McpError> {
    let graph = load_graph()?;

    let radius = BlastRadius::for_crate(&graph, &params.crate_name).ok_or_else(|| {
        McpError::invalid_params(
            format!("Crate '{}' not found in workspace", params.crate_name),
            None,
        )
    })?;

    let result = json!({
        "crate_name": radius.crate_name,
        "direct_dependents": radius.direct_count,
        "transitive_dependents": radius.transitive_count,
        "impact_ratio": radius.impact_ratio,
        "impact_pct": format!("{:.1}%", radius.impact_ratio * 100.0),
        "cascade_depth": radius.cascade_depth,
        "cascade_width": radius.cascade_width,
        "containment": radius.containment,
        "total_workspace_crates": graph.total_crates,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// anatomy_chomsky — Chomsky classification
// ---------------------------------------------------------------------------

/// Get Chomsky classification for a specific crate or all crates.
/// Returns generators used, Chomsky level, and automaton model.
///
/// Tier: T3 (nexcore-anatomy × MCP integration)
pub fn anatomy_chomsky_tool(params: AnatomyChomskyParams) -> Result<CallToolResult, McpError> {
    let graph = load_graph()?;
    let chomsky = ChomskyReport::from_graph(&graph);

    if let Some(name) = &params.crate_name {
        // Single crate lookup
        let profile = chomsky
            .profiles
            .iter()
            .find(|p| p.name == *name)
            .ok_or_else(|| {
                McpError::invalid_params(format!("Crate '{name}' not found in workspace"), None)
            })?;

        let result = json!({
            "name": profile.name,
            "generators": profile.generators,
            "generator_count": profile.generator_count,
            "level": profile.level.label(),
            "automaton": profile.architecture,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    } else {
        // All crates summary
        let result = json!({
            "total_crates": chomsky.profiles.len(),
            "distribution": chomsky.level_distribution,
            "avg_generators": chomsky.avg_generators,
            "overengineering_candidates": chomsky.overengineering_candidates,
            "profiles": chomsky.profiles.iter().map(|p| json!({
                "name": p.name,
                "level": p.level.label(),
                "generators": p.generators,
            })).collect::<Vec<_>>(),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }
}

// ---------------------------------------------------------------------------
// anatomy_violations — list boundary violations
// ---------------------------------------------------------------------------

/// List all layer boundary violations in the workspace with severity scores.
///
/// Tier: T3 (nexcore-anatomy × MCP integration)
pub fn anatomy_violations_tool() -> Result<CallToolResult, McpError> {
    let graph = load_graph()?;
    let layers = nexcore_anatomy::LayerMap::from_graph(&graph);

    let result = json!({
        "total_violations": layers.violations.len(),
        "has_severe": layers.violations.iter().any(|v| v.severity >= 2),
        "violations": layers.violations.iter().map(|v| json!({
            "from_crate": v.from_crate,
            "from_layer": format!("{:?}", v.from_layer),
            "to_crate": v.to_crate,
            "to_layer": format!("{:?}", v.to_layer),
            "severity": v.severity,
            "direction": if v.severity >= 2 { "upward (severe)" } else { "upward (minor)" },
        })).collect::<Vec<_>>(),
        "layer_counts": layers.layer_counts,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
