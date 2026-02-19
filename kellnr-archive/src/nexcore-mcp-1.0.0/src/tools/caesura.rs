//! Caesura detection MCP tools — structural seam detection in codebases.
//!
//! ## Primitive Foundation
//! - ∂ (Boundary): Discontinuity detection across strata
//! - ς (State): Codebase state shifts at seams
//! - ∝ (Irreversibility): Seams mark permanent architectural decisions
//! - ν (Frequency): Pattern recurrence analysis

use crate::params::{CaesuraMetricsParams, CaesuraReportParams, CaesuraScanParams};
use nexcore_caesura::{CaesuraDetector, Stratum};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

// ---------------------------------------------------------------------------
// caesura_scan — scan directory for caesuras across all strata
// ---------------------------------------------------------------------------

/// Scan a directory for structural seams (caesuras) across style, architecture,
/// and dependency strata. Returns JSON array of detected caesuras sorted by severity.
///
/// Tier: T3 (nexcore-caesura × MCP integration)
pub fn caesura_scan_tool(params: CaesuraScanParams) -> Result<CallToolResult, McpError> {
    let dir = Path::new(&params.path);
    if !dir.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let sensitivity = params.sensitivity.unwrap_or(2.0);
    let mut detector = CaesuraDetector::with_sensitivity(sensitivity);

    // Parse strata filter
    if let Some(strata_names) = params.strata {
        let strata: Vec<Stratum> = strata_names
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "style" => Some(Stratum::Style),
                "architecture" | "arch" => Some(Stratum::Architecture),
                "dependency" | "dep" => Some(Stratum::Dependency),
                "temporal" => Some(Stratum::Temporal),
                _ => None,
            })
            .collect();
        detector = detector.with_strata(strata);
    }

    let caesuras = detector
        .scan(dir)
        .map_err(|e| McpError::internal_error(format!("Scan failed: {e}"), None))?;

    let result = json!({
        "count": caesuras.len(),
        "caesuras": caesuras.iter().map(|c| json!({
            "type": c.caesura_type.label(),
            "stratum": format!("{:?}", c.stratum),
            "score": c.score.value(),
            "severity": c.score.severity().label(),
            "path": c.location.path,
            "description": c.description,
            "boundary_files": c.boundary_files,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// caesura_metrics — compute stratum metrics for a single file
// ---------------------------------------------------------------------------

/// Compute style and architecture metrics for a single Rust source file.
///
/// Tier: T3 (nexcore-caesura × MCP integration)
pub fn caesura_metrics_tool(params: CaesuraMetricsParams) -> Result<CallToolResult, McpError> {
    let path = Path::new(&params.file_path);
    if !path.exists() {
        return Err(McpError::invalid_params(
            format!("File does not exist: {}", params.file_path),
            None,
        ));
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|e| McpError::internal_error(format!("Read failed: {e}"), None))?;

    let style = nexcore_caesura::style::StyleDetector::compute_metrics(&contents);
    let arch = nexcore_caesura::architecture::ArchDetector::compute_metrics(&contents);

    let result = json!({
        "file": params.file_path,
        "style": {
            "snake_case_ratio": style.snake_case_ratio,
            "camel_case_ratio": style.camel_case_ratio,
            "naming_entropy": style.naming_entropy(),
            "comment_density": style.comment_density,
            "mean_line_length": style.mean_line_length,
            "stddev_line_length": style.stddev_line_length,
            "total_lines": style.total_lines,
        },
        "architecture": {
            "import_count": arch.import_count,
            "pub_surface": arch.pub_surface,
            "mod_count": arch.mod_count,
            "coupling_density": arch.coupling_density,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// caesura_report — summary report from scan results
// ---------------------------------------------------------------------------

/// Scan a directory and produce a markdown report of detected caesuras,
/// sorted by severity.
///
/// Tier: T3 (nexcore-caesura × MCP integration)
pub fn caesura_report_tool(params: CaesuraReportParams) -> Result<CallToolResult, McpError> {
    let dir = Path::new(&params.path);
    if !dir.exists() {
        return Err(McpError::invalid_params(
            format!("Path does not exist: {}", params.path),
            None,
        ));
    }

    let sensitivity = params.sensitivity.unwrap_or(2.0);
    let detector = CaesuraDetector::with_sensitivity(sensitivity);

    let caesuras = detector
        .scan(dir)
        .map_err(|e| McpError::internal_error(format!("Scan failed: {e}"), None))?;

    let report = CaesuraDetector::report(&caesuras);

    Ok(CallToolResult::success(vec![Content::text(report)]))
}
