//! Tool usage and token efficiency telemetry CRUD.
//!
//! Stores aggregated tool call statistics and per-session token metrics.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;

// ========== Tool Usage ==========

/// A tool usage statistics row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageRow {
    /// Tool name (e.g., "Read", "Bash", "Edit")
    pub tool_name: String,
    /// Total invocations
    pub total_calls: i64,
    /// Successful calls
    pub success_count: i64,
    /// Failed calls
    pub failure_count: i64,
    /// Unix timestamp of last use
    pub last_used: i64,
}

/// Upsert tool usage statistics.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_tool_usage(conn: &Connection, row: &ToolUsageRow) -> Result<()> {
    conn.execute(
        "INSERT INTO tool_usage (tool_name, total_calls, success_count, failure_count, last_used)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(tool_name) DO UPDATE SET
            total_calls = excluded.total_calls,
            success_count = excluded.success_count,
            failure_count = excluded.failure_count,
            last_used = MAX(tool_usage.last_used, excluded.last_used)",
        params![
            row.tool_name,
            row.total_calls,
            row.success_count,
            row.failure_count,
            row.last_used,
        ],
    )?;
    Ok(())
}

/// List all tool usage, ordered by total calls descending.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_tool_usage(conn: &Connection) -> Result<Vec<ToolUsageRow>> {
    let mut stmt = conn.prepare(
        "SELECT tool_name, total_calls, success_count, failure_count, last_used
         FROM tool_usage ORDER BY total_calls DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ToolUsageRow {
                tool_name: row.get(0)?,
                total_calls: row.get(1)?,
                success_count: row.get(2)?,
                failure_count: row.get(3)?,
                last_used: row.get(4)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count total tool records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_tools(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM tool_usage", [], |row| row.get(0))?;
    Ok(n)
}

// ========== Token Efficiency ==========

/// A token efficiency row (per-session).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEfficiencyRow {
    /// Session ID
    pub session_id: String,
    /// Number of tool actions
    pub action_count: i64,
    /// Total tokens used
    pub total_tokens: i64,
    /// Unix timestamp when session started
    pub started_at: i64,
}

/// Upsert token efficiency for a session.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_token_efficiency(conn: &Connection, row: &TokenEfficiencyRow) -> Result<()> {
    conn.execute(
        "INSERT INTO token_efficiency (session_id, action_count, total_tokens, started_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(session_id) DO UPDATE SET
            action_count = excluded.action_count,
            total_tokens = excluded.total_tokens",
        params![
            row.session_id,
            row.action_count,
            row.total_tokens,
            row.started_at,
        ],
    )?;
    Ok(())
}

/// List all token efficiency records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_token_efficiency(conn: &Connection) -> Result<Vec<TokenEfficiencyRow>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, action_count, total_tokens, started_at
         FROM token_efficiency ORDER BY started_at DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TokenEfficiencyRow {
                session_id: row.get(0)?,
                action_count: row.get(1)?,
                total_tokens: row.get(2)?,
                started_at: row.get(3)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count token efficiency records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_efficiency(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM token_efficiency", [], |row| {
        row.get(0)
    })?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    #[test]
    fn test_tool_usage_crud() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            upsert_tool_usage(
                conn,
                &ToolUsageRow {
                    tool_name: "Read".into(),
                    total_calls: 490,
                    success_count: 480,
                    failure_count: 10,
                    last_used: 1770599318,
                },
            )?;
            let all = list_tool_usage(conn)?;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].total_calls, 490);

            let n = count_tools(conn)?;
            assert_eq!(n, 1);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_token_efficiency_crud() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            upsert_token_efficiency(
                conn,
                &TokenEfficiencyRow {
                    session_id: "sess-1".into(),
                    action_count: 10,
                    total_tokens: 5000,
                    started_at: 1770562031,
                },
            )?;
            let all = list_token_efficiency(conn)?;
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].total_tokens, 5000);
            Ok(())
        })
        .expect("test");
    }
}
