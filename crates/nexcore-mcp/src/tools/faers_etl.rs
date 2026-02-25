//! FAERS ETL MCP tools — local bulk data pipeline for signal detection.
//!
//! Wraps `nexcore-faers-etl` functions as MCP tools:
//! - `faers_etl_run`: Full pipeline → top signals by PRR
//! - `faers_etl_signals`: Pipeline + drug/event filter
//! - `faers_etl_known_pairs`: Pipeline + known-pair validation
//! - `faers_etl_status`: Filesystem check on cached Parquet output
//!
//! Heavy pipeline functions use `tokio::task::spawn_blocking` since
//! Polars is CPU-bound (~30-60s on 370MB FAERS data, ~7GB RSS).

use crate::params::{
    FaersEtlKnownPairsParams, FaersEtlRunParams, FaersEtlSignalsParams, FaersEtlStatusParams,
};
use nexcore_faers_etl::SignalDetectionResult;
use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Round to 4 decimal places for readability.
fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

fn signal_to_json(r: &SignalDetectionResult) -> serde_json::Value {
    json!({
        "drug": r.drug.as_str(),
        "event": r.event.as_str(),
        "n": r.case_count.value(),
        "prr": round4(r.prr.point.value()),
        "prr_lower_ci": round4(r.prr.lower_ci),
        "prr_signal": r.prr.is_signal,
        "ror": round4(r.ror.point.value()),
        "ror_lower_ci": round4(r.ror.lower_ci),
        "ror_signal": r.ror.is_signal,
        "ic": round4(r.ic.point.value()),
        "ic025": round4(r.ic.lower_ci),
        "ic_signal": r.ic.is_signal,
        "ebgm": round4(r.ebgm.point.value()),
        "eb05": round4(r.ebgm.lower_ci),
        "ebgm_signal": r.ebgm.is_signal,
    })
}

fn text_result(value: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

fn validate_dir(path: &str) -> Result<(), McpError> {
    if !Path::new(path).is_dir() {
        return Err(McpError::invalid_params(
            format!("FAERS directory not found: {path}"),
            None,
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pipeline execution (sync, runs in spawn_blocking)
// ---------------------------------------------------------------------------

struct PipelineResult {
    results: Vec<SignalDetectionResult>,
    total_pairs: usize,
    signal_count: usize,
    elapsed_secs: f64,
}

fn run_pipeline(
    faers_dir: &Path,
    include_all_roles: bool,
    min_cases: i64,
) -> Result<PipelineResult, String> {
    let start = Instant::now();

    let output = nexcore_faers_etl::run_full_pipeline(faers_dir, include_all_roles, min_cases)
        .map_err(|e| format!("Pipeline failed: {e}"))?;

    let signal_count = output.results.iter().filter(|r| r.is_any_signal()).count();

    Ok(PipelineResult {
        total_pairs: output.total_pairs,
        signal_count,
        elapsed_secs: start.elapsed().as_secs_f64(),
        results: output.results,
    })
}

fn empty_result(start: Instant) -> PipelineResult {
    PipelineResult {
        results: Vec::new(),
        total_pairs: 0,
        signal_count: 0,
        elapsed_secs: start.elapsed().as_secs_f64(),
    }
}

async fn run_blocking_pipeline(
    faers_dir: String,
    include_all_roles: bool,
    min_cases: i64,
) -> Result<PipelineResult, McpError> {
    tokio::task::spawn_blocking(move || {
        run_pipeline(Path::new(&faers_dir), include_all_roles, min_cases)
    })
    .await
    .map_err(|e| McpError::internal_error(format!("Task join failed: {e}"), None))?
    .map_err(|e| McpError::internal_error(e, None))
}

// ---------------------------------------------------------------------------
// faers_etl_run
// ---------------------------------------------------------------------------

/// Run full FAERS ETL pipeline. Returns top signals sorted by PRR descending.
pub async fn run(params: FaersEtlRunParams) -> Result<CallToolResult, McpError> {
    validate_dir(&params.faers_dir)?;

    let include_all = params.include_all_roles.unwrap_or(false);
    let min_cases = params.min_cases.unwrap_or(3);
    let top_n = params.top_n.unwrap_or(50);

    let result = run_blocking_pipeline(params.faers_dir, include_all, min_cases).await?;

    let mut sorted = result.results;
    sorted.sort_by(|a, b| {
        b.prr
            .point
            .value()
            .partial_cmp(&a.prr.point.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.truncate(top_n);

    let signal_rate = compute_rate(result.signal_count, result.total_pairs);
    let output = json!({
        "pipeline": {
            "pairs_total": result.total_pairs,
            "signals_detected": result.signal_count,
            "signal_rate_pct": round4(signal_rate),
            "elapsed_secs": round4(result.elapsed_secs),
        },
        "top_signals": sorted.iter().map(signal_to_json).collect::<Vec<_>>(),
    });

    Ok(text_result(&output))
}

fn compute_rate(count: usize, total: usize) -> f64 {
    if total > 0 {
        (count as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

// ---------------------------------------------------------------------------
// faers_etl_signals
// ---------------------------------------------------------------------------

/// Search FAERS ETL signals by drug and/or event name.
pub async fn signals(params: FaersEtlSignalsParams) -> Result<CallToolResult, McpError> {
    validate_dir(&params.faers_dir)?;

    let include_all = params.include_all_roles.unwrap_or(false);
    let min_cases = params.min_cases.unwrap_or(3);
    let drug_filter = params.drug.clone();
    let event_filter = params.event.clone();

    let result = run_blocking_pipeline(params.faers_dir, include_all, min_cases).await?;
    let filtered = filter_by_drug_event(&result.results, &drug_filter, &event_filter);

    let output = json!({
        "query": { "drug": drug_filter, "event": event_filter },
        "matches": filtered.len(),
        "signals": filtered.iter().map(|r| signal_to_json(r)).collect::<Vec<_>>(),
    });

    Ok(text_result(&output))
}

fn filter_by_drug_event<'a>(
    results: &'a [SignalDetectionResult],
    drug: &Option<String>,
    event: &Option<String>,
) -> Vec<&'a SignalDetectionResult> {
    results
        .iter()
        .filter(|r| r.is_any_signal())
        .filter(|r| matches_filter(r.drug.as_str(), drug))
        .filter(|r| matches_filter(r.event.as_str(), event))
        .collect()
}

fn matches_filter(value: &str, filter: &Option<String>) -> bool {
    match filter {
        Some(f) => value.to_uppercase().contains(&f.to_uppercase()),
        None => true,
    }
}

// ---------------------------------------------------------------------------
// faers_etl_known_pairs
// ---------------------------------------------------------------------------

/// Validate known drug-event pairs against local FAERS data.
pub async fn known_pairs(params: FaersEtlKnownPairsParams) -> Result<CallToolResult, McpError> {
    validate_dir(&params.faers_dir)?;

    if params.pairs.is_empty() {
        return Err(McpError::invalid_params(
            "At least one drug-event pair is required".to_string(),
            None,
        ));
    }

    let min_cases = params.min_cases.unwrap_or(3);
    let result = run_blocking_pipeline(params.faers_dir, false, min_cases).await?;

    let pair_results: Vec<serde_json::Value> = params
        .pairs
        .iter()
        .map(|pair| check_single_pair(&result.results, &pair.drug, &pair.event))
        .collect();

    let validated = pair_results.iter().filter(|r| r["found"] == true).count();
    let total = params.pairs.len();

    let output = json!({
        "validated": validated,
        "total": total,
        "hit_rate_pct": round4(compute_rate(validated, total)),
        "results": pair_results,
    });

    Ok(text_result(&output))
}

fn check_single_pair(
    results: &[SignalDetectionResult],
    drug: &str,
    event: &str,
) -> serde_json::Value {
    let drug_upper = drug.to_uppercase();
    let event_upper = event.to_uppercase();

    let found = results
        .iter()
        .find(|r| r.drug.as_str().contains(&drug_upper) && r.event.as_str().contains(&event_upper));

    match found {
        Some(r) => json!({
            "drug": drug, "event": event, "found": true,
            "n": r.case_count.value(),
            "prr": round4(r.prr.point.value()),
            "ror": round4(r.ror.point.value()),
            "ic": round4(r.ic.point.value()),
            "ebgm": round4(r.ebgm.point.value()),
            "is_signal": r.is_any_signal(),
        }),
        None => json!({
            "drug": drug, "event": event, "found": false,
            "n": 0, "prr": 0.0, "is_signal": false,
        }),
    }
}

// ---------------------------------------------------------------------------
// faers_etl_status
// ---------------------------------------------------------------------------

/// Check status of cached FAERS Parquet output files.
pub fn status(params: FaersEtlStatusParams) -> Result<CallToolResult, McpError> {
    let output_dir = resolve_output_dir(params.output_dir);
    let dir_path = Path::new(&output_dir);

    if !dir_path.is_dir() {
        return Ok(text_result(&json!({
            "output_dir": output_dir, "exists": false, "files": [],
        })));
    }

    let files = scan_parquet_files(dir_path);
    let output = json!({
        "output_dir": output_dir,
        "exists": true,
        "file_count": files.len(),
        "files": files,
        "note": "FAERS ETL requires ~7GB RSS for 370MB quarterly data.",
    });

    Ok(text_result(&output))
}

fn resolve_output_dir(custom: Option<String>) -> String {
    custom.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("nexcore/output").display().to_string())
            .unwrap_or_else(|| "/home/matthew/nexcore/output".to_string())
    })
}

fn scan_parquet_files(dir: &Path) -> Vec<serde_json::Value> {
    let mut files = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("faers_") || !name.ends_with(".parquet") {
            continue;
        }
        if let Some(info) = file_info(&entry) {
            files.push(info);
        }
    }

    files.sort_by(|a, b| sort_by_name(a, b));
    files
}

fn file_info(entry: &std::fs::DirEntry) -> Option<serde_json::Value> {
    let meta = entry.metadata().ok()?;
    let name = entry.file_name().to_string_lossy().to_string();
    let size_mb = meta.len() as f64 / (1024.0 * 1024.0);
    let modified = meta
        .modified()
        .ok()
        .map(|t| {
            let dt: nexcore_chrono::DateTime = t.into();
            dt.to_rfc3339()
        })
        .unwrap_or_else(|| "unknown".to_string());

    Some(json!({
        "name": name,
        "size_mb": round4(size_mb),
        "modified": modified,
    }))
}

fn sort_by_name(a: &serde_json::Value, b: &serde_json::Value) -> std::cmp::Ordering {
    let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
    a_name.cmp(b_name)
}
