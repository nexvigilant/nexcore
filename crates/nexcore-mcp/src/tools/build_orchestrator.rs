//! Build Orchestrator MCP tools — CI/CD pipeline management.
//!
//! Exposes nexcore-build-orchestrator functionality:
//! - `build_orchestrator_dry_run`: Preview pipeline execution plan
//! - `build_orchestrator_stages`: List available stage types
//! - `build_orchestrator_workspace`: Discover crates in workspace
//! - `build_orchestrator_history`: Query pipeline run history
//! - `build_orchestrator_metrics`: Get build timing metrics

use crate::params::{
    BuildOrchestratorDryRunParams, BuildOrchestratorHistoryParams, BuildOrchestratorMetricsParams,
    BuildOrchestratorStagesParams, BuildOrchestratorWorkspaceParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Dry-run a pipeline definition and return the execution plan
pub fn build_orchestrator_dry_run(
    params: BuildOrchestratorDryRunParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_build_orchestrator::PipelineDefinition;

    // Try built-in first, then parse JSON
    let definition = PipelineDefinition::builtin(&params.pipeline).unwrap_or_else(|| {
        serde_json::from_str::<PipelineDefinition>(&params.pipeline).unwrap_or_else(|_| {
            PipelineDefinition::validate() // fallback to validate
        })
    });

    let waves = nexcore_build_orchestrator::dry_run(&definition);

    let wave_json: Vec<_> = waves
        .iter()
        .enumerate()
        .map(|(i, wave)| {
            json!({
                "wave": i + 1,
                "stages": wave.iter().map(|s| s.0.as_str()).collect::<Vec<_>>(),
                "parallel": wave.len() > 1,
            })
        })
        .collect();

    let response = json!({
        "pipeline": definition.name,
        "description": definition.description,
        "total_stages": definition.stages.len(),
        "execution_waves": wave_json,
        "total_waves": waves.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// List all available pipeline stage types
pub fn build_orchestrator_stages(
    _params: BuildOrchestratorStagesParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_build_orchestrator::PipelineStage;

    let stages = vec![
        ("Fmt", "cargo fmt --all -- --check"),
        ("Check", "cargo check --workspace"),
        ("Clippy", "cargo clippy --workspace -- -D warnings"),
        ("Test", "cargo test --workspace"),
        ("Build", "cargo build --workspace"),
        ("Docs", "cargo doc --workspace --no-deps"),
        ("Audit", "cargo audit"),
        ("Coverage", "cargo llvm-cov"),
        ("Custom(cmd)", "Execute arbitrary command"),
    ];

    let builtins = nexcore_build_orchestrator::PipelineDefinition::builtin_names();

    let response = json!({
        "stage_types": stages.iter().map(|(name, cmd)| json!({
            "name": name,
            "default_command": cmd,
        })).collect::<Vec<_>>(),
        "builtin_pipelines": builtins,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Discover crates in a workspace
pub fn build_orchestrator_workspace(
    params: BuildOrchestratorWorkspaceParams,
) -> Result<CallToolResult, McpError> {
    let workspace_root = Path::new(&params.path);

    let scan = nexcore_build_orchestrator::workspace::scanner::scan_workspace(workspace_root)
        .map_err(|e| McpError::internal_error(format!("Workspace scan failed: {e}"), None))?;

    let response = json!({
        "workspace_root": scan.workspace_root,
        "crate_count": scan.crate_count,
        "dirty_count": scan.dirty_count(),
        "clean_count": scan.clean_count(),
        "scanned_at": scan.scanned_at.to_rfc3339(),
        "targets": scan.targets.iter().take(50).map(|t| json!({
            "name": t.name,
            "path": t.path,
            "needs_build": t.needs_build,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Query pipeline run history
pub fn build_orchestrator_history(
    params: BuildOrchestratorHistoryParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_build_orchestrator::RunStatus;
    use nexcore_build_orchestrator::history::query::{HistoryQuery, query_history};
    use nexcore_build_orchestrator::history::store::HistoryStore;

    let workspace_root = Path::new(&params.path);

    let store = HistoryStore::new(workspace_root)
        .map_err(|e| McpError::internal_error(format!("History store init failed: {e}"), None))?;

    let mut query = HistoryQuery::new().with_limit(params.limit);

    if let Some(ref status_filter) = params.status {
        match status_filter.to_lowercase().as_str() {
            "completed" => {
                query = query.with_status(RunStatus::Completed);
            }
            "failed" => {
                query = query.with_status(RunStatus::Failed);
            }
            _ => {} // "all" — no filter
        }
    }

    let runs = query_history(&store, &query)
        .map_err(|e| McpError::internal_error(format!("History query failed: {e}"), None))?;

    let response = json!({
        "total_runs": runs.len(),
        "runs": runs.iter().map(|r| json!({
            "id": r.id.0,
            "pipeline": r.definition_name,
            "status": format!("{:?}", r.status),
            "started_at": r.started_at.to_rfc3339(),
            "duration_ms": r.total_duration.as_ref().map(|d| d.millis),
            "stages_completed": r.completed_count(),
            "stages_succeeded": r.success_count(),
            "total_stages": r.stages.len(),
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Get build metrics summary from history
pub fn build_orchestrator_metrics(
    params: BuildOrchestratorMetricsParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_build_orchestrator::history::store::HistoryStore;
    use nexcore_build_orchestrator::metrics::collector::{TimingAggregates, collect_timings};

    let workspace_root = Path::new(&params.path);

    let store = HistoryStore::new(workspace_root)
        .map_err(|e| McpError::internal_error(format!("History store init failed: {e}"), None))?;

    let all_runs = store
        .load_all()
        .map_err(|e| McpError::internal_error(format!("History load failed: {e}"), None))?;

    let mut aggregates = TimingAggregates::new();
    for run in &all_runs {
        aggregates.add_run(run);
    }

    let response = json!({
        "total_runs": all_runs.len(),
        "avg_pipeline_ms": aggregates.avg_pipeline_ms(),
        "stage_averages": aggregates.stage_durations.keys().map(|stage| {
            json!({
                "stage": stage,
                "avg_ms": aggregates.avg_stage_ms(stage),
            })
        }).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
