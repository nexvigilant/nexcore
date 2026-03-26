//! Brain Database MCP tools — query accumulated knowledge from SQLite.
//!
//! These tools expose the persistent brain.db for reading:
//! - Summary: table counts overview
//! - Decisions: decision audit statistics by tool/risk
//! - Tool stats: tool usage leaderboard
//! - Antibodies: immunity antibody listings
//! - Handoffs: recent session handoff summaries
//! - Tasks: task completion statistics
//! - Efficiency: token efficiency across sessions
//! - Sync: re-ingest from dotfile sources

use nexcore_brain::db::get_pool;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params;

/// Helper: get pool or return a descriptive error.
fn pool_or_err() -> Result<nexcore_db::pool::DbPool, McpError> {
    get_pool().ok_or_else(|| {
        McpError::internal_error(
            "Brain SQLite backend unavailable (pool not initialized)",
            None,
        )
    })
}

/// `brain_db_summary` — Full table counts overview from brain.db.
///
/// Returns row counts for all 16 tables in a structured summary.
pub fn summary() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            // Count all tables via raw SQL (avoids needing count functions in every module)
            let count = |table: &str| -> i64 {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap_or(0)
            };

            // V1 tables
            let sessions = count("sessions");
            let artifacts = count("artifacts");
            let versions = count("artifact_versions");
            let tracked = count("tracked_files");
            let preferences = count("preferences");
            let patterns = count("patterns");
            let corrections = count("corrections");
            let beliefs = count("beliefs");
            let trust = count("trust_accumulators");
            let implications = count("implications");

            // V2 tables
            let decisions = count("decision_audit");
            let tools = count("tool_usage");
            let efficiency = count("token_efficiency");
            let tasks = count("tasks_history");
            let handoffs = count("handoffs");
            let antibodies = count("antibodies");

            let total = sessions
                + artifacts
                + versions
                + tracked
                + preferences
                + patterns
                + corrections
                + beliefs
                + trust
                + implications
                + decisions
                + tools
                + efficiency
                + tasks
                + handoffs
                + antibodies;

            let summary = serde_json::json!({
                "database": "brain.db",
                "schema_version": 2,
                "total_rows": total,
                "v1_tables": {
                    "sessions": sessions,
                    "artifacts": artifacts,
                    "artifact_versions": versions,
                    "tracked_files": tracked,
                    "preferences": preferences,
                    "patterns": patterns,
                    "corrections": corrections,
                    "beliefs": beliefs,
                    "trust_accumulators": trust,
                    "implications": implications,
                },
                "v2_tables": {
                    "decisions": decisions,
                    "tool_usage": tools,
                    "token_efficiency": efficiency,
                    "tasks_history": tasks,
                    "handoffs": handoffs,
                    "antibodies": antibodies,
                }
            });

            Ok(serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_decisions_stats` — Decision audit statistics.
///
/// Returns counts grouped by tool and risk level.
pub fn decisions_stats() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let total = nexcore_db::decisions::count(conn).unwrap_or(0);
            let by_tool = nexcore_db::decisions::count_by_tool(conn).unwrap_or_default();
            let by_risk = nexcore_db::decisions::count_by_risk(conn).unwrap_or_default();

            let stats = serde_json::json!({
                "total_decisions": total,
                "by_tool": by_tool.into_iter()
                    .map(|(tool, count)| serde_json::json!({"tool": tool, "count": count}))
                    .collect::<Vec<_>>(),
                "by_risk_level": by_risk.into_iter()
                    .map(|(risk, count)| serde_json::json!({"risk": risk, "count": count}))
                    .collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_tool_stats` — Tool usage leaderboard.
///
/// Returns all tools sorted by total calls descending.
pub fn tool_stats() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let tools = nexcore_db::telemetry::list_tool_usage(conn)?;
            let total_calls: i64 = tools.iter().map(|t| t.total_calls).sum();
            let total_tools = tools.len();

            let stats = serde_json::json!({
                "total_tools": total_tools,
                "total_calls": total_calls,
                "tools": tools.iter().map(|t| {
                    let success_rate = if t.total_calls > 0 {
                        (t.success_count as f64 / t.total_calls as f64) * 100.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "name": t.tool_name,
                        "total": t.total_calls,
                        "success": t.success_count,
                        "failure": t.failure_count,
                        "success_rate_pct": format!("{success_rate:.1}"),
                    })
                }).collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_antibodies` — List all immunity antibodies.
///
/// Returns each antibody with threat type, severity, confidence, and application count.
pub fn antibodies() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let abs = nexcore_db::knowledge::list_antibodies(conn)?;

            let data = serde_json::json!({
                "total": abs.len(),
                "antibodies": abs.iter().map(|a| {
                    serde_json::json!({
                        "id": a.id,
                        "name": a.name,
                        "threat_type": a.threat_type,
                        "severity": a.severity,
                        "description": a.description,
                        "confidence": a.confidence,
                        "applications": a.applications,
                        "false_positives": a.false_positives,
                        "learned_from": a.learned_from,
                        "t1_grounding": a.t1_grounding,
                    })
                }).collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_handoffs` — List handoffs, optionally filtered by project.
///
/// Returns handoff summaries with metrics (files modified, lines written, commits).
pub fn handoffs(params: params::BrainDbHandoffsParams) -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let total = nexcore_db::knowledge::count_handoffs(conn).unwrap_or(0);

            let handoffs = if let Some(ref proj) = params.project {
                nexcore_db::knowledge::list_handoffs_by_project(conn, proj)?
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, project, handoff_number, session_id, generated_at, status,
                            duration, files_modified, lines_written, commits, uncommitted, content
                     FROM handoffs ORDER BY handoff_number DESC LIMIT ?1",
                )?;
                let limit = params.limit.unwrap_or(20) as i64;
                stmt.query_map([limit], |row| {
                    Ok(nexcore_db::knowledge::HandoffRow {
                        id: Some(row.get(0)?),
                        project: row.get(1)?,
                        handoff_number: row.get(2)?,
                        session_id: row.get(3)?,
                        generated_at: row.get(4)?,
                        status: row.get(5)?,
                        duration: row.get(6)?,
                        files_modified: row.get(7)?,
                        lines_written: row.get(8)?,
                        commits: row.get(9)?,
                        uncommitted: row.get(10)?,
                        content: row.get(11)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
            };

            let data = serde_json::json!({
                "total_handoffs": total,
                "returned": handoffs.len(),
                "filter": params.project.as_deref().unwrap_or("all"),
                "handoffs": handoffs.iter().map(|h| {
                    serde_json::json!({
                        "project": h.project,
                        "number": h.handoff_number,
                        "session_id": h.session_id,
                        "generated_at": h.generated_at,
                        "status": h.status,
                        "duration": h.duration,
                        "files_modified": h.files_modified,
                        "lines_written": h.lines_written,
                        "commits": h.commits,
                        "uncommitted": h.uncommitted,
                    })
                }).collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_tasks` — Task completion statistics.
///
/// Returns task counts grouped by status.
pub fn tasks() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let total = nexcore_db::knowledge::count_tasks(conn).unwrap_or(0);
            let by_status = nexcore_db::knowledge::count_tasks_by_status(conn).unwrap_or_default();

            let data = serde_json::json!({
                "total_tasks": total,
                "by_status": by_status.into_iter()
                    .map(|(status, count)| serde_json::json!({"status": status, "count": count}))
                    .collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_efficiency` — Token efficiency across sessions.
///
/// Returns per-session token usage statistics.
pub fn efficiency() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let records = nexcore_db::telemetry::list_token_efficiency(conn)?;
            let total_tokens: i64 = records.iter().map(|r| r.total_tokens).sum();
            let total_actions: i64 = records.iter().map(|r| r.action_count).sum();

            let data = serde_json::json!({
                "total_sessions": records.len(),
                "total_tokens": total_tokens,
                "total_actions": total_actions,
                "avg_tokens_per_action": if total_actions > 0 {
                    total_tokens as f64 / total_actions as f64
                } else {
                    0.0
                },
                "sessions": records.iter().take(20).map(|r| {
                    serde_json::json!({
                        "session_id": r.session_id,
                        "action_count": r.action_count,
                        "total_tokens": r.total_tokens,
                        "tokens_per_action": if r.action_count > 0 {
                            r.total_tokens as f64 / r.action_count as f64
                        } else {
                            0.0
                        },
                    })
                }).collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_query` — Execute a read-only SQL SELECT against brain.db.
///
/// Runs an arbitrary SELECT query and returns rows as JSON objects.
/// Only SELECT statements are permitted (read-only access enforced).
pub fn query(params: params::BrainDbQueryParams) -> Result<CallToolResult, McpError> {
    let sql_trimmed = params.sql.trim().to_ascii_lowercase();
    if !sql_trimmed.starts_with("select") {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Only SELECT queries are permitted (read-only access)".to_string(),
        )]));
    }

    let max_rows = params.limit.unwrap_or(100).min(500) as usize;
    let pool = pool_or_err()?;

    let result = pool
        .with_conn(|conn| {
            let mut stmt = conn.prepare(&params.sql)?;
            let col_names: Vec<String> = stmt
                .column_names()
                .into_iter()
                .map(str::to_string)
                .collect();

            let rows: Vec<serde_json::Value> = stmt
                .query_map([], |row| {
                    let mut obj = serde_json::Map::new();
                    for (i, name) in col_names.iter().enumerate() {
                        // Try integer, then float, then string, else null
                        let json_val = row
                            .get::<_, Option<i64>>(i)
                            .ok()
                            .flatten()
                            .map(|n| serde_json::json!(n))
                            .or_else(|| {
                                row.get::<_, Option<f64>>(i)
                                    .ok()
                                    .flatten()
                                    .map(|f| serde_json::json!(f))
                            })
                            .or_else(|| {
                                row.get::<_, Option<String>>(i)
                                    .ok()
                                    .flatten()
                                    .map(|s| serde_json::json!(s))
                            })
                            .unwrap_or(serde_json::Value::Null);
                        obj.insert(name.clone(), json_val);
                    }
                    Ok(serde_json::Value::Object(obj))
                })?
                .filter_map(|r| r.ok())
                .take(max_rows)
                .collect();

            let data = serde_json::json!({
                "sql": params.sql,
                "columns": col_names,
                "row_count": rows.len(),
                "rows": rows,
                "limit": max_rows,
            });

            Ok(serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string()))
        })
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// `brain_db_sync` — Re-ingest V2 data from dotfile sources into SQLite.
///
/// Runs the V2 migration pipeline to refresh decisions, tool usage, token efficiency,
/// tasks, handoffs, and antibodies from their source files.
pub fn sync() -> Result<CallToolResult, McpError> {
    let pool = pool_or_err()?;

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let claude_dir = std::path::PathBuf::from(home).join(".claude");

    let result = pool
        .with_conn(|conn| nexcore_db::migrate::run_v2(conn, &claude_dir))
        .map_err(|e| McpError::internal_error(format!("V2 sync failed: {e}"), None))?;

    let output = format!(
        "V2 sync complete:\n  Decisions: {}\n  Tool usage: {}\n  Token efficiency: {}\n  \
         Tasks: {}\n  Handoffs: {}\n  Antibodies: {}\n  \
         Beliefs: {}\n  Preferences: {}\n  Patterns: {}\n  Corrections: {}\n  \
         Trust: {}\n  Errors: {}",
        result.decisions,
        result.tool_usage,
        result.token_efficiency,
        result.tasks,
        result.handoffs,
        result.antibodies,
        result.beliefs,
        result.preferences,
        result.patterns,
        result.corrections,
        result.trust_accumulators,
        result.errors.len(),
    );

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        output.to_string(),
    )]))
}
