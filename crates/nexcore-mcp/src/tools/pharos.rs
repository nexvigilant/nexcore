//! PHAROS MCP tools — autonomous signal surveillance pipeline.
//!
//! Tools:
//! - `pharos_run`: Execute full PHAROS pipeline (FAERS ETL → signal detect → filter → report)
//! - `pharos_status`: Check for existing PHAROS reports and timer status
//! - `pharos_report`: Retrieve a specific or latest surveillance report

use crate::params::{PharosReportParams, PharosRunParams, PharosStatusParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

fn text_result(value: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

fn error_result(msg: &str) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg.to_string())])
}

// ---------------------------------------------------------------------------
// pharos_run
// ---------------------------------------------------------------------------

/// Run the full PHAROS surveillance pipeline.
pub async fn run(params: PharosRunParams) -> Result<CallToolResult, McpError> {
    let faers_dir = params.faers_dir.clone();
    if !Path::new(&faers_dir).is_dir() {
        return Err(McpError::invalid_params(
            format!("FAERS directory not found: {faers_dir}"),
            None,
        ));
    }

    let output_dir = params
        .output_dir
        .unwrap_or_else(|| "./output/pharos".to_string());
    let min_cases = params.min_cases.unwrap_or(3);
    let include_all_roles = params.include_all_roles.unwrap_or(false);
    let top_n = params.top_n.unwrap_or(50);
    let threshold_mode = params
        .threshold_mode
        .unwrap_or_else(|| "default".to_string());

    let result = tokio::task::spawn_blocking(move || {
        use nexcore_pharos::{PharosConfig, PharosPipeline, SignalThresholds};

        let thresholds = match threshold_mode.as_str() {
            "strict" => SignalThresholds::strict(),
            "sensitive" => SignalThresholds::sensitive(),
            _ => SignalThresholds::default(),
        };

        let config = PharosConfig {
            faers_dir: std::path::PathBuf::from(&faers_dir),
            output_dir: std::path::PathBuf::from(&output_dir),
            min_cases,
            include_all_roles,
            thresholds,
            top_n_report: top_n,
            inject_guardian: true,
            emit_cytokines: true,
            qdrant_url: "http://localhost:6333".to_string(),
            qdrant_collection: "pharos_signals".to_string(),
        };

        if let Err(e) = config.validate() {
            return Err(format!("Config validation failed: {e}"));
        }

        let pipeline = PharosPipeline::new(config);
        match pipeline.execute() {
            Ok(output) => {
                let top_signals: Vec<serde_json::Value> = output
                    .report
                    .top_signals
                    .iter()
                    .map(|s| {
                        json!({
                            "drug": s.drug,
                            "event": s.event,
                            "case_count": s.case_count,
                            "prr": s.prr,
                            "ror_lower_ci": s.ror_lower_ci,
                            "ic025": s.ic025,
                            "eb05": s.eb05,
                            "algorithms_flagged": s.algorithms_flagged,
                            "threat_level": s.threat_level,
                        })
                    })
                    .collect();

                Ok(json!({
                    "success": true,
                    "run_id": output.report.run_id,
                    "total_pairs": output.report.total_pairs,
                    "raw_signals": output.report.raw_signals,
                    "actionable_signals": output.report.actionable_signals,
                    "guardian_injections": output.report.guardian_injections,
                    "duration_ms": output.report.duration_ms,
                    "top_signals": top_signals,
                    "summary": output.report.summary(),
                }))
            }
            Err(e) => Err(format!("Pipeline failed: {e}")),
        }
    })
    .await
    .map_err(|e| McpError::internal_error(format!("Task join failed: {e}"), None))?;

    match result {
        Ok(value) => Ok(text_result(&value)),
        Err(e) => Ok(error_result(&e)),
    }
}

// ---------------------------------------------------------------------------
// pharos_status
// ---------------------------------------------------------------------------

/// Check PHAROS status: existing reports, timer state.
pub fn status(params: PharosStatusParams) -> Result<CallToolResult, McpError> {
    let output_dir = params
        .output_dir
        .unwrap_or_else(|| "./output/pharos".to_string());
    let output_path = Path::new(&output_dir);

    let mut reports: Vec<serde_json::Value> = Vec::new();

    if output_path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(output_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("pharos-report-") && n.ends_with(".json"))
                {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let size = metadata.len();
                            reports.push(json!({
                                "file": path.display().to_string(),
                                "size_bytes": size,
                                "modified": format!("{:?}", modified),
                            }));
                        }
                    }
                }
            }
        }
    }

    // Check for Parquet file
    let parquet_path = output_path.join("signals.parquet");
    let parquet_info = if parquet_path.exists() {
        if let Ok(meta) = std::fs::metadata(&parquet_path) {
            Some(json!({
                "path": parquet_path.display().to_string(),
                "size_bytes": meta.len(),
            }))
        } else {
            None
        }
    } else {
        None
    };

    let result = json!({
        "output_dir": output_dir,
        "report_count": reports.len(),
        "reports": reports,
        "parquet": parquet_info,
        "timer": "pharos.timer (systemd user timer, quarterly)",
    });

    Ok(text_result(&result))
}

// ---------------------------------------------------------------------------
// pharos_report
// ---------------------------------------------------------------------------

/// Retrieve a specific or latest PHAROS surveillance report.
pub fn report(params: PharosReportParams) -> Result<CallToolResult, McpError> {
    let output_dir = params
        .output_dir
        .unwrap_or_else(|| "./output/pharos".to_string());
    let output_path = Path::new(&output_dir);

    if !output_path.is_dir() {
        return Ok(error_result(&format!(
            "Output directory not found: {output_dir}"
        )));
    }

    // If run_id specified, look for exact file
    if let Some(run_id) = &params.run_id {
        let filename = format!("pharos-report-{run_id}.json");
        let path = output_path.join(&filename);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(value) => return Ok(text_result(&value)),
                    Err(e) => return Ok(error_result(&format!("Failed to parse report: {e}"))),
                },
                Err(e) => return Ok(error_result(&format!("Failed to read report: {e}"))),
            }
        }
        return Ok(error_result(&format!(
            "Report not found for run_id: {run_id}"
        )));
    }

    // Otherwise find latest report by modification time
    let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;

    if let Ok(entries) = std::fs::read_dir(output_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("pharos-report-") && n.ends_with(".json"))
            {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(modified) = meta.modified() {
                        if latest.as_ref().is_none_or(|(_, t)| modified > *t) {
                            latest = Some((path, modified));
                        }
                    }
                }
            }
        }
    }

    match latest {
        Some((path, _)) => match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(value) => Ok(text_result(&value)),
                Err(e) => Ok(error_result(&format!("Failed to parse report: {e}"))),
            },
            Err(e) => Ok(error_result(&format!("Failed to read report: {e}"))),
        },
        None => Ok(error_result("No PHAROS reports found")),
    }
}
