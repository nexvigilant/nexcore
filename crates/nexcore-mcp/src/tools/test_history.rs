//! Test history query tools.
//!
//! Queries test_runs table in brain.db for historical test results.

use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::test_history::{TestHistoryFlakyParams, TestHistoryQueryParams};

/// Query test run history with optional filters.
pub fn test_history_query(params: TestHistoryQueryParams) -> Result<CallToolResult, McpError> {
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/matthew"))
        .join(".claude/brain/brain.db");

    if !db_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "brain.db not found at ~/.claude/brain/brain.db",
        )]));
    }

    let since_days = params.since_days.unwrap_or(30);
    let limit = params.limit.unwrap_or(50);
    let since_clause = format!("-{since_days} days");

    let mut query = format!(
        "SELECT run_at, session_id, crate_name, runner, passed, failed, ignored, duration_s, fail_names FROM test_runs WHERE run_at >= datetime('now', '{since_clause}')"
    );

    if let Some(ref crate_name) = params.crate_name {
        query.push_str(&format!(
            " AND crate_name = '{}'",
            crate_name.replace('\'', "''")
        ));
    }
    query.push_str(" ORDER BY run_at DESC");
    query.push_str(&format!(" LIMIT {limit}"));

    let output = std::process::Command::new("sqlite3")
        .args(["-json", db_path.to_str().unwrap_or(""), &query])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !out.status.success() {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "SQLite error: {stderr}"
                ))]));
            }
            let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();
            let result = serde_json::json!({
                "success": true,
                "count": rows.len(),
                "since_days": since_days,
                "crate_filter": params.crate_name,
                "runs": rows,
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to execute sqlite3: {e}"
        ))])),
    }
}

/// Identify flaky tests that appear in fail_names across multiple runs.
pub fn test_history_flaky(params: TestHistoryFlakyParams) -> Result<CallToolResult, McpError> {
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/matthew"))
        .join(".claude/brain/brain.db");

    if !db_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "brain.db not found",
        )]));
    }

    let window_days = params.window_days.unwrap_or(14);
    let min_flips = params.min_flips.unwrap_or(2);

    let mut query = format!(
        "SELECT crate_name, fail_names, passed, failed FROM test_runs WHERE run_at >= datetime('now', '-{window_days} days') AND fail_names != '[]'"
    );
    if let Some(ref crate_name) = params.crate_name {
        query.push_str(&format!(
            " AND crate_name = '{}'",
            crate_name.replace('\'', "''")
        ));
    }

    let output = std::process::Command::new("sqlite3")
        .args(["-json", db_path.to_str().unwrap_or(""), &query])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let rows: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap_or_default();

            let mut test_counts: std::collections::HashMap<String, u32> =
                std::collections::HashMap::new();
            for row in &rows {
                if let Some(names_str) = row.get("fail_names").and_then(|v| v.as_str()) {
                    if let Ok(names) = serde_json::from_str::<Vec<String>>(names_str) {
                        for name in names {
                            *test_counts.entry(name).or_insert(0) += 1;
                        }
                    }
                }
            }

            let mut flaky: Vec<serde_json::Value> = test_counts
                .into_iter()
                .filter(|(_, fails)| *fails >= min_flips)
                .map(|(name, fails)| {
                    serde_json::json!({
                        "test_name": name,
                        "fail_count": fails,
                    })
                })
                .collect();
            flaky.sort_by(|a, b| b["fail_count"].as_u64().cmp(&a["fail_count"].as_u64()));

            let result = serde_json::json!({
                "success": true,
                "window_days": window_days,
                "min_flips": min_flips,
                "flaky_tests": flaky,
                "total_runs_analyzed": rows.len(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to execute sqlite3: {e}"
        ))])),
    }
}
