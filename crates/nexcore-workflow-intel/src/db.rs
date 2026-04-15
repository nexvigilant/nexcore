//! Database access layer for brain.db workflow data.

use crate::error::Result;
use crate::types::*;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;

/// Open a read-only connection to brain.db.
pub fn open_brain_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA query_only=ON;")?;
    Ok(conn)
}

/// Default brain.db path.
pub fn default_brain_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    std::path::PathBuf::from(home).join(".claude/brain/brain.db")
}

/// Extract tool events from decision_audit for a time window.
pub fn query_tool_events(conn: &Connection, days: u32) -> Result<Vec<ToolEvent>> {
    let sql = "SELECT tool, action, session_id, timestamp, risk_level, reversible
               FROM decision_audit
               WHERE timestamp >= datetime('now', ?1)
               ORDER BY timestamp ASC";
    let window = format!("-{days} days");
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([&window], |row| {
        Ok(ToolEvent {
            tool: row.get(0)?,
            action: row.get::<_, String>(1)?,
            session_id: row.get(2)?,
            timestamp: row.get(3)?,
            risk_level: row.get(4)?,
            reversible: row.get::<_, i32>(5)? != 0,
        })
    })?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}

/// Query tool usage aggregate stats.
pub fn query_tool_usage(conn: &Connection) -> Result<Vec<(String, u64, u64, u64)>> {
    let sql = "SELECT tool_name, total_calls, success_count, failure_count
               FROM tool_usage
               ORDER BY total_calls DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u64>(1)?,
            row.get::<_, u64>(2)?,
            row.get::<_, u64>(3)?,
        ))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query autopsy records for session-level metrics.
pub fn query_autopsy_records(conn: &Connection, days: u32) -> Result<Vec<AutopsyRow>> {
    let sql = "SELECT a.session_id, s.description,
                      a.outcome_verdict, a.tool_calls_total, a.mcp_calls,
                      a.files_modified, a.lesson_count, a.pattern_count,
                      a.session_started_at
               FROM autopsy_records a
               JOIN sessions s ON s.id = a.session_id
               WHERE a.session_started_at >= datetime('now', ?1)
               ORDER BY a.session_started_at DESC";
    let window = format!("-{days} days");
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([&window], |row| {
        Ok(AutopsyRow {
            session_id: row.get(0)?,
            description: row.get(1)?,
            verdict: row.get::<_, Option<String>>(2)?,
            tool_calls: row.get::<_, u64>(3)?,
            mcp_calls: row.get::<_, u64>(4)?,
            files_modified: row.get::<_, u64>(5)?,
            lesson_count: row.get::<_, u64>(6)?,
            pattern_count: row.get::<_, u64>(7)?,
            started_at: row.get(8)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query skill invocations.
pub fn query_skill_invocations(conn: &Connection, days: u32) -> Result<Vec<(String, u64)>> {
    let sql = "SELECT skill_name, COUNT(*) as cnt
               FROM skill_invocations
               WHERE invoked_at >= datetime('now', ?1)
               GROUP BY skill_name
               ORDER BY cnt DESC";
    let window = format!("-{days} days");
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([&window], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Build tool transition map from ordered events.
pub fn compute_transitions(events: &[ToolEvent]) -> Vec<ToolTransition> {
    let mut transition_counts: HashMap<(String, String), u64> = HashMap::new();
    let mut from_totals: HashMap<String, u64> = HashMap::new();

    // Group events by session, then compute sequential transitions
    let mut by_session: HashMap<&str, Vec<&ToolEvent>> = HashMap::new();
    for event in events {
        by_session
            .entry(event.session_id.as_str())
            .or_default()
            .push(event);
    }

    for (_sid, session_events) in &by_session {
        for window in session_events.windows(2) {
            let from = normalize_tool_name(&window[0].tool);
            let to = normalize_tool_name(&window[1].tool);
            *transition_counts.entry((from.clone(), to)).or_insert(0) += 1;
            *from_totals.entry(from).or_insert(0) += 1;
        }
    }

    let mut transitions: Vec<ToolTransition> = transition_counts
        .into_iter()
        .map(|((from, to), count)| {
            let total = from_totals.get(&from).copied().unwrap_or(1);
            ToolTransition {
                from,
                to,
                count,
                pct: (count as f64 / total as f64) * 100.0,
            }
        })
        .collect();

    transitions.sort_by(|a, b| b.count.cmp(&a.count));
    transitions
}

/// Normalize tool names to categories for cleaner transitions.
fn normalize_tool_name(name: &str) -> String {
    if name.starts_with("mcp__") {
        // Extract server name: mcp__nexcore__tool -> nexcore
        let parts: Vec<&str> = name.splitn(4, "__").collect();
        if parts.len() >= 3 {
            return format!("mcp:{}", parts[1]);
        }
    }
    name.to_string()
}

/// Classify a tool into a category.
pub fn classify_tool(name: &str) -> &'static str {
    if name.starts_with("mcp__") {
        "mcp"
    } else if matches!(
        name,
        "Read" | "Write" | "Edit" | "Glob" | "Grep" | "Bash" | "Agent" | "Skill"
    ) {
        "builtin"
    } else {
        "other"
    }
}

/// Intermediate type for autopsy query results.
#[derive(Debug, Clone)]
pub struct AutopsyRow {
    pub session_id: String,
    pub description: String,
    pub verdict: Option<String>,
    pub tool_calls: u64,
    pub mcp_calls: u64,
    pub files_modified: u64,
    pub lesson_count: u64,
    pub pattern_count: u64,
    pub started_at: String,
}
