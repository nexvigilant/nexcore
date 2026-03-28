//! DAG Publish MCP tools — read-only introspection of the crate publish pipeline.
//!
//! Three tools, all safe to call at any time (no cargo publish invoked):
//! - `dag_publish_plan`: Phase plan (parallelisable waves of crates)
//! - `dag_publish_dry_run`: Flat topological publish order with optional filter/limit
//! - `dag_publish_status`: Resume state — what's published vs pending

use std::path::Path;

use crate::params::{DagPublishDryRunParams, DagPublishPlanParams, DagPublishStatusParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// dag_publish_plan
// ---------------------------------------------------------------------------

/// Build the dependency DAG and group crates into parallelisable phases.
///
/// Returns the phase plan as JSON.  Phase 0 contains crates with no internal
/// dependencies; each subsequent phase's crates can be published in parallel
/// once all prior phases are complete.
pub fn dag_publish_plan(params: DagPublishPlanParams) -> Result<CallToolResult, McpError> {
    let crates_dir = Path::new(&params.crates_dir);
    let dag = dag_publish::build_dag(crates_dir, &params.registry)
        .map_err(|e| McpError::internal_error(format!("Failed to build DAG: {e}"), None))?;

    let phases = dag_publish::group_into_phases(&dag)
        .map_err(|e| McpError::internal_error(format!("Failed to compute phases: {e}"), None))?;

    let total_crates: usize = phases.iter().map(|p| p.len()).sum();

    let phase_json: Vec<_> = phases
        .iter()
        .enumerate()
        .map(|(i, phase)| {
            json!({
                "phase": i,
                "crate_count": phase.len(),
                "crates": phase,
                "parallelisable": phase.len() > 1,
            })
        })
        .collect();

    let response = json!({
        "crates_dir": params.crates_dir,
        "registry": params.registry,
        "total_crates": total_crates,
        "total_phases": phases.len(),
        "phases": phase_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// dag_publish_dry_run
// ---------------------------------------------------------------------------

/// Return the flat topological publish order for the crate DAG.
///
/// Applies optional `filter` (name prefix) and `limit` before returning.
/// No cargo invocations — purely a DAG computation.
pub fn dag_publish_dry_run(params: DagPublishDryRunParams) -> Result<CallToolResult, McpError> {
    let crates_dir = Path::new(&params.crates_dir);
    let dag = dag_publish::build_dag(crates_dir, &params.registry)
        .map_err(|e| McpError::internal_error(format!("Failed to build DAG: {e}"), None))?;

    let order = dag_publish::topological_sort(&dag)
        .map_err(|e| McpError::internal_error(format!("Topological sort failed: {e}"), None))?;

    // Apply filter
    let mut filtered: Vec<&String> = order.iter().collect();
    if let Some(ref prefix) = params.filter {
        filtered.retain(|name| name.starts_with(prefix.as_str()));
    }

    // Apply limit
    if let Some(limit) = params.limit {
        filtered.truncate(limit);
    }

    let crate_entries: Vec<_> = filtered
        .iter()
        .enumerate()
        .map(|(i, name)| {
            json!({
                "position": i + 1,
                "name": name,
            })
        })
        .collect();

    let response = json!({
        "crates_dir": params.crates_dir,
        "registry": params.registry,
        "filter": params.filter,
        "limit": params.limit,
        "total_in_dag": order.len(),
        "total_shown": filtered.len(),
        "publish_order": crate_entries,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// dag_publish_status
// ---------------------------------------------------------------------------

/// Read the `.dag-publish-state.json` resume file and report what has been
/// published vs what is still pending.
///
/// When `show_pending` is true the crates directory is scanned and cross-
/// referenced against the checkpoint file to produce a pending list.
pub fn dag_publish_status(params: DagPublishStatusParams) -> Result<CallToolResult, McpError> {
    let workspace_root = Path::new(&params.workspace_root);

    let state = dag_publish::read_resume_state(workspace_root)
        .map_err(|e| McpError::internal_error(format!("Failed to read resume state: {e}"), None))?;

    let published_json: Vec<_> = state
        .published
        .iter()
        .map(|e| json!({"name": e.name, "version": e.version}))
        .collect();

    let mut response = json!({
        "workspace_root": params.workspace_root,
        "state_file": format!("{}/.dag-publish-state.json", params.workspace_root),
        "published_count": state.published.len(),
        "published": published_json,
    });

    if params.show_pending {
        let crates_dir = Path::new(&params.crates_dir);
        let dag = dag_publish::build_dag(crates_dir, &params.registry)
            .map_err(|e| McpError::internal_error(format!("Failed to build DAG: {e}"), None))?;

        let order = dag_publish::topological_sort(&dag)
            .map_err(|e| McpError::internal_error(format!("Topological sort failed: {e}"), None))?;

        let pending: Vec<&String> = order
            .iter()
            .filter(|name| {
                // A crate is pending if no checkpoint entry exists for it
                // (regardless of version — we check by name only here since
                // version lookup requires scanning files again).
                !state.published.iter().any(|e| &e.name == *name)
            })
            .collect();

        let pending_json: Vec<_> = pending
            .iter()
            .enumerate()
            .map(|(i, name)| json!({"position": i + 1, "name": name}))
            .collect();

        response["total_in_dag"] = json!(order.len());
        response["pending_count"] = json!(pending.len());
        response["pending"] = json!(pending_json);
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
