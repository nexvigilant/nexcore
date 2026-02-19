//! ORGANIZE MCP tools — 8-step file organization pipeline.
//!
//! Exposes nexcore-organize functionality as MCP tools:
//! - `organize_analyze`: Run full pipeline in dry-run mode
//! - `organize_config_default`: Get default configuration for a path
//! - `organize_report_markdown`: Generate markdown report
//! - `organize_report_json`: Generate JSON report
//! - `organize_observe`: Inventory filesystem entries
//! - `organize_rank`: Observe + rank entries by recency/size/relevance

use crate::params::{
    OrganizeAnalyzeParams, OrganizeConfigDefaultParams, OrganizeObserveParams, OrganizeRankParams,
    OrganizeReportJsonParams, OrganizeReportMarkdownParams,
};
use nexcore_organize::config::OrganizeConfig;
use nexcore_organize::pipeline::OrganizePipeline;
use nexcore_organize::report;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// organize_analyze — full pipeline dry-run
// ---------------------------------------------------------------------------

/// Run the full ORGANIZE pipeline in analysis (dry-run) mode.
pub fn organize_analyze(params: OrganizeAnalyzeParams) -> Result<CallToolResult, McpError> {
    let path = std::path::Path::new(&params.path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let mut config = OrganizeConfig::default_for(path);
    if let Some(depth) = params.max_depth {
        config.max_depth = depth;
    }
    if !params.exclude.is_empty() {
        config.exclude_patterns.extend(params.exclude);
    }

    let pipeline = OrganizePipeline::new(config);
    let result = pipeline
        .analyze()
        .map_err(|e| McpError::internal_error(format!("Pipeline error: {e}"), None))?;

    let summary = json!({
        "root": result.root.display().to_string(),
        "mode": "dry_run",
        "integration": {
            "total_operations": result.plan.total,
            "succeeded": result.plan.succeeded,
            "failed": result.plan.failed,
            "bytes_affected": result.plan.bytes_affected,
            "bytes_human": report::format_bytes(result.plan.bytes_affected),
        },
        "cleanup": {
            "empty_dirs": result.cleanup.empty_dirs.len(),
            "duplicate_groups": result.cleanup.duplicates.len(),
            "wasted_bytes": result.cleanup.total_wasted_bytes,
            "wasted_human": report::format_bytes(result.cleanup.total_wasted_bytes),
        },
        "state": {
            "files_tracked": result.state.count,
            "snapshot_time": result.state.timestamp,
        },
        "drift": result.drift.as_ref().map(|d| json!({
            "has_drift": d.has_drift,
            "added": d.added.len(),
            "removed": d.removed.len(),
            "modified": d.modified.len(),
        })),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// organize_config_default — show default config
// ---------------------------------------------------------------------------

/// Return default ORGANIZE configuration for a directory.
pub fn organize_config_default(
    params: OrganizeConfigDefaultParams,
) -> Result<CallToolResult, McpError> {
    let config = OrganizeConfig::default_for(&params.path);

    let output = match params.format.as_deref() {
        Some("toml") => config.to_toml().map_err(|e| {
            McpError::internal_error(format!("TOML serialization error: {e}"), None)
        })?,
        _ => serde_json::to_string_pretty(&config).unwrap_or_else(|_| "{}".to_string()),
    };

    Ok(CallToolResult::success(vec![Content::text(output)]))
}

// ---------------------------------------------------------------------------
// organize_report_markdown — analyze + markdown report
// ---------------------------------------------------------------------------

/// Run ORGANIZE pipeline and return a markdown report.
pub fn organize_report_markdown(
    params: OrganizeReportMarkdownParams,
) -> Result<CallToolResult, McpError> {
    let path = std::path::Path::new(&params.path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let config = OrganizeConfig::default_for(path);
    let pipeline = OrganizePipeline::new(config);
    let result = pipeline
        .analyze()
        .map_err(|e| McpError::internal_error(format!("Pipeline error: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        report::markdown_report(&result),
    )]))
}

// ---------------------------------------------------------------------------
// organize_report_json — analyze + JSON report
// ---------------------------------------------------------------------------

/// Run ORGANIZE pipeline and return a JSON report.
pub fn organize_report_json(params: OrganizeReportJsonParams) -> Result<CallToolResult, McpError> {
    let path = std::path::Path::new(&params.path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let config = OrganizeConfig::default_for(path);
    let pipeline = OrganizePipeline::new(config);
    let result = pipeline
        .analyze()
        .map_err(|e| McpError::internal_error(format!("Pipeline error: {e}"), None))?;

    let json_str = report::json_report(&result)
        .map_err(|e| McpError::internal_error(format!("JSON serialization error: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json_str)]))
}

// ---------------------------------------------------------------------------
// organize_observe — inventory only (step 1)
// ---------------------------------------------------------------------------

/// Run the observe step only — inventory filesystem entries with metadata.
pub fn organize_observe(params: OrganizeObserveParams) -> Result<CallToolResult, McpError> {
    let path = std::path::Path::new(&params.path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let mut config = OrganizeConfig::default_for(path);
    if let Some(depth) = params.max_depth {
        config.max_depth = depth;
    }
    if !params.exclude.is_empty() {
        config.exclude_patterns.extend(params.exclude);
    }

    let inventory = nexcore_organize::observe::observe(&config)
        .map_err(|e| McpError::internal_error(format!("Observe error: {e}"), None))?;

    let summary = json!({
        "root": inventory.root.display().to_string(),
        "entry_count": inventory.entries.len(),
        "excluded_count": inventory.excluded_count,
        "observed_at": inventory.observed_at.to_rfc3339(),
        "entries": inventory.entries.iter().take(100).map(|e| json!({
            "path": e.path.display().to_string(),
            "is_dir": e.is_dir,
            "size_bytes": e.size_bytes,
            "extension": e.extension,
            "depth": e.depth,
            "name": e.name,
        })).collect::<Vec<_>>(),
        "truncated": inventory.entries.len() > 100,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// organize_rank — observe + rank (steps 1-2)
// ---------------------------------------------------------------------------

/// Run observe + rank steps — inventory and score entries by recency/size/relevance.
pub fn organize_rank(params: OrganizeRankParams) -> Result<CallToolResult, McpError> {
    let path = std::path::Path::new(&params.path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let config = OrganizeConfig::default_for(path);

    // Step 1: Observe
    let inventory = nexcore_organize::observe::observe(&config)
        .map_err(|e| McpError::internal_error(format!("Observe error: {e}"), None))?;

    // Step 2: Rank
    let ranked = nexcore_organize::rank::rank(inventory, &config)
        .map_err(|e| McpError::internal_error(format!("Rank error: {e}"), None))?;

    let limit = params.limit.unwrap_or(50).min(200);
    let entries: Vec<_> = ranked
        .entries
        .iter()
        .take(limit)
        .map(|e| {
            json!({
                "path": e.meta.path.display().to_string(),
                "name": e.meta.name,
                "score": e.score,
                "size_bytes": e.meta.size_bytes,
                "extension": e.meta.extension,
                "is_dir": e.meta.is_dir,
            })
        })
        .collect();

    let summary = json!({
        "root": ranked.root.display().to_string(),
        "total_entries": ranked.entries.len(),
        "showing": entries.len(),
        "entries": entries,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
