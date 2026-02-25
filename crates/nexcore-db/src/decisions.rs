//! Decision audit CRUD operations.
//!
//! Stores tool usage decisions captured by cognitive hooks.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A decision audit row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRow {
    /// Auto-increment ID (None for new)
    pub id: Option<i64>,
    /// When the decision occurred
    pub timestamp: DateTime,
    /// Session in which the decision was made
    pub session_id: String,
    /// Tool used
    pub tool: String,
    /// Action description
    pub action: String,
    /// Target file/path/resource
    pub target: String,
    /// Risk level: LOW, MEDIUM, HIGH
    pub risk_level: String,
    /// Whether the action is reversible
    pub reversible: bool,
}

/// Insert a decision audit record (idempotent).
///
/// Uses `INSERT OR IGNORE` — safe to call multiple times with the same
/// `(timestamp, session_id, tool, target)` tuple. Duplicates are silently skipped.
///
/// # Errors
///
/// Returns an error if the insert fails (not counting ignored duplicates).
pub fn insert(conn: &Connection, d: &DecisionRow) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO decision_audit (timestamp, session_id, tool, action, target, risk_level, reversible)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            d.timestamp.to_rfc3339(),
            d.session_id,
            d.tool,
            d.action,
            d.target,
            d.risk_level,
            d.reversible as i32,
        ],
    )?;
    Ok(())
}

/// Count total decision records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM decision_audit", [], |row| row.get(0))?;
    Ok(n)
}

/// Count decisions by tool.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_by_tool(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT tool, COUNT(*) as cnt FROM decision_audit GROUP BY tool ORDER BY cnt DESC",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count decisions by risk level.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_by_risk(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT risk_level, COUNT(*) as cnt FROM decision_audit GROUP BY risk_level ORDER BY cnt DESC",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List decisions for a session.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_by_session(conn: &Connection, session_id: &str) -> Result<Vec<DecisionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, session_id, tool, action, target, risk_level, reversible
         FROM decision_audit WHERE session_id = ?1 ORDER BY timestamp ASC",
    )?;
    let rows = stmt
        .query_map([session_id], |row| {
            Ok(DecisionRow {
                id: Some(row.get(0)?),
                timestamp: parse_dt(row.get::<_, String>(1)?),
                session_id: row.get(2)?,
                tool: row.get(3)?,
                action: row.get(4)?,
                target: row.get(5)?,
                risk_level: row.get(6)?,
                reversible: row.get::<_, i32>(7)? != 0,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn parse_dt(s: String) -> DateTime {
    DateTime::parse_from_rfc3339(&s).unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    #[test]
    fn test_decision_insert_and_count() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            let d = DecisionRow {
                id: None,
                timestamp: DateTime::now(),
                session_id: "test-session".into(),
                tool: "Bash".into(),
                action: "Bash operation".into(),
                target: "cargo test".into(),
                risk_level: "LOW".into(),
                reversible: true,
            };
            insert(conn, &d)?;

            let n = count(conn)?;
            assert_eq!(n, 1);

            let by_tool = count_by_tool(conn)?;
            assert_eq!(by_tool.len(), 1);
            assert_eq!(by_tool[0].0, "Bash");

            let by_session = list_by_session(conn, "test-session")?;
            assert_eq!(by_session.len(), 1);
            assert_eq!(by_session[0].tool, "Bash");
            Ok(())
        })
        .expect("test");
    }
}
