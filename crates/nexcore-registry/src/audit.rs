//! Audit log CRUD operations.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A row from the `audit_log` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Skill name this audit entry relates to
    pub skill_name: String,
    /// Action performed (e.g., "created", "updated", "validated", "audited")
    pub action: String,
    /// JSON details
    pub details: Option<String>,
    /// Actor (e.g., "vigil", "user", "hook", "scan")
    pub actor: Option<String>,
    /// When the action occurred
    pub created_at: DateTime<Utc>,
}

/// Record an audit entry.
///
/// # Errors
///
/// Returns an error on insert failure.
pub fn record(conn: &Connection, row: &AuditRow) -> Result<i64> {
    conn.execute(
        "INSERT INTO audit_log (skill_name, action, details, actor, created_at)
         VALUES (?1,?2,?3,?4,?5)",
        params![
            row.skill_name,
            row.action,
            row.details,
            row.actor,
            row.created_at.to_rfc3339(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List recent audit entries, newest first.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<AuditRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_name, action, details, actor, created_at
         FROM audit_log ORDER BY created_at DESC LIMIT ?1",
    )?;

    let rows = stmt
        .query_map([limit], |row| {
            Ok(AuditRow {
                id: row.get(0)?,
                skill_name: row.get(1)?,
                action: row.get(2)?,
                details: row.get(3)?,
                actor: row.get(4)?,
                created_at: parse_dt(row.get::<_, String>(5)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// List audit entries for a specific skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_for_skill(conn: &Connection, skill_name: &str, limit: i64) -> Result<Vec<AuditRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_name, action, details, actor, created_at
         FROM audit_log WHERE skill_name = ?1
         ORDER BY created_at DESC LIMIT ?2",
    )?;

    let rows = stmt
        .query_map(params![skill_name, limit], |row| {
            Ok(AuditRow {
                id: row.get(0)?,
                skill_name: row.get(1)?,
                action: row.get(2)?,
                details: row.get(3)?,
                actor: row.get(4)?,
                created_at: parse_dt(row.get::<_, String>(5)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Count total audit entries.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count(conn: &Connection) -> Result<i64> {
    let c: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;
    Ok(c)
}

fn parse_dt(s: String) -> DateTime<Utc> {
    s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    #[test]
    fn test_record_and_list() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let entry = AuditRow {
                id: None,
                skill_name: "test-skill".to_string(),
                action: "created".to_string(),
                details: Some("{\"source\":\"scan\"}".to_string()),
                actor: Some("scan".to_string()),
                created_at: Utc::now(),
            };
            record(conn, &entry)?;
            let recent = list_recent(conn, 10)?;
            assert_eq!(recent.len(), 1);
            assert_eq!(recent[0].action, "created");
            let for_skill = list_for_skill(conn, "test-skill", 10)?;
            assert_eq!(for_skill.len(), 1);
            let c = count(conn)?;
            assert_eq!(c, 1);
            Ok(())
        })
        .ok();
    }
}
