//! Accumulated knowledge CRUD: tasks history, handoffs, and antibodies.
//!
//! These tables store operational knowledge extracted from dotfiles.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::Result;

// ========== Tasks History ==========

/// A task history row (snapshot from ~/.claude/tasks/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Session the task belongs to
    pub session_id: String,
    /// Task ID within session (e.g., "1", "2")
    pub task_id: String,
    /// Task subject line
    pub subject: String,
    /// Detailed description
    pub description: String,
    /// Present-continuous form for spinners
    pub active_form: String,
    /// Task status: pending, in_progress, completed
    pub status: String,
    /// JSON array of task IDs this blocks
    pub blocks: String,
    /// JSON array of task IDs blocking this
    pub blocked_by: String,
}

/// Upsert a task history record.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_task(conn: &Connection, t: &TaskHistoryRow) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks_history (session_id, task_id, subject, description, active_form,
                                    status, blocks, blocked_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(session_id, task_id) DO UPDATE SET
            subject = excluded.subject,
            description = excluded.description,
            active_form = excluded.active_form,
            status = excluded.status,
            blocks = excluded.blocks,
            blocked_by = excluded.blocked_by",
        params![
            t.session_id,
            t.task_id,
            t.subject,
            t.description,
            t.active_form,
            t.status,
            t.blocks,
            t.blocked_by,
        ],
    )?;
    Ok(())
}

/// Count all task records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_tasks(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM tasks_history", [], |row| row.get(0))?;
    Ok(n)
}

/// Count tasks by status.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_tasks_by_status(conn: &Connection) -> Result<Vec<(String, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT status, COUNT(*) FROM tasks_history GROUP BY status ORDER BY COUNT(*) DESC",
    )?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Handoffs ==========

/// A handoff summary row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Project name (directory component)
    pub project: String,
    /// Handoff sequence number
    pub handoff_number: i32,
    /// Session ID (extracted from handoff header)
    pub session_id: String,
    /// When generated (ISO 8601)
    pub generated_at: String,
    /// Status string (e.g., "Session Complete")
    pub status: String,
    /// Duration string (e.g., "29h 19m")
    pub duration: String,
    /// Files modified count
    pub files_modified: i32,
    /// Lines written count
    pub lines_written: i32,
    /// Commits made
    pub commits: i32,
    /// Uncommitted changes count
    pub uncommitted: i32,
    /// Full markdown content
    pub content: String,
}

/// Upsert a handoff record.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_handoff(conn: &Connection, h: &HandoffRow) -> Result<()> {
    conn.execute(
        "INSERT INTO handoffs (project, handoff_number, session_id, generated_at, status,
                               duration, files_modified, lines_written, commits, uncommitted, content)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(project, handoff_number) DO UPDATE SET
            session_id = excluded.session_id,
            generated_at = excluded.generated_at,
            status = excluded.status,
            duration = excluded.duration,
            files_modified = excluded.files_modified,
            lines_written = excluded.lines_written,
            commits = excluded.commits,
            uncommitted = excluded.uncommitted,
            content = excluded.content",
        params![
            h.project,
            h.handoff_number,
            h.session_id,
            h.generated_at,
            h.status,
            h.duration,
            h.files_modified,
            h.lines_written,
            h.commits,
            h.uncommitted,
            h.content,
        ],
    )?;
    Ok(())
}

/// Count total handoff records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_handoffs(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM handoffs", [], |row| row.get(0))?;
    Ok(n)
}

/// List handoffs for a project.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_handoffs_by_project(conn: &Connection, project: &str) -> Result<Vec<HandoffRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, handoff_number, session_id, generated_at, status,
                duration, files_modified, lines_written, commits, uncommitted, content
         FROM handoffs WHERE project = ?1 ORDER BY handoff_number ASC",
    )?;
    let rows = stmt
        .query_map([project], |row| {
            Ok(HandoffRow {
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
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ========== Antibodies ==========

/// An antipattern antibody row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntibodyRow {
    /// Unique identifier (e.g., "PANIC-001")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Threat type: PAMP (external) or DAMP (internal)
    pub threat_type: String,
    /// Severity: critical, high, medium, low
    pub severity: String,
    /// Description of the antipattern
    pub description: String,
    /// JSON-serialized detection rules
    pub detection: String,
    /// JSON-serialized response strategy
    pub response: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Times successfully applied
    pub applications: i32,
    /// False positive count
    pub false_positives: i32,
    /// Where the antibody was learned from
    pub learned_from: String,
    /// T1 primitive grounding (optional)
    pub t1_grounding: Option<String>,
}

/// Upsert an antibody.
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert_antibody(conn: &Connection, a: &AntibodyRow) -> Result<()> {
    conn.execute(
        "INSERT INTO antibodies (id, name, threat_type, severity, description, detection,
                                 response, confidence, applications, false_positives,
                                 learned_from, t1_grounding)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            threat_type = excluded.threat_type,
            severity = excluded.severity,
            description = excluded.description,
            detection = excluded.detection,
            response = excluded.response,
            confidence = excluded.confidence,
            applications = excluded.applications,
            false_positives = excluded.false_positives,
            learned_from = excluded.learned_from,
            t1_grounding = COALESCE(excluded.t1_grounding, antibodies.t1_grounding)",
        params![
            a.id,
            a.name,
            a.threat_type,
            a.severity,
            a.description,
            a.detection,
            a.response,
            a.confidence,
            a.applications,
            a.false_positives,
            a.learned_from,
            a.t1_grounding,
        ],
    )?;
    Ok(())
}

/// List all antibodies.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_antibodies(conn: &Connection) -> Result<Vec<AntibodyRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, threat_type, severity, description, detection, response,
                confidence, applications, false_positives, learned_from, t1_grounding
         FROM antibodies ORDER BY severity ASC, confidence DESC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AntibodyRow {
                id: row.get(0)?,
                name: row.get(1)?,
                threat_type: row.get(2)?,
                severity: row.get(3)?,
                description: row.get(4)?,
                detection: row.get(5)?,
                response: row.get(6)?,
                confidence: row.get(7)?,
                applications: row.get(8)?,
                false_positives: row.get(9)?,
                learned_from: row.get(10)?,
                t1_grounding: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count antibodies.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_antibodies(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM antibodies", [], |row| row.get(0))?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    #[test]
    fn test_task_history_crud() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            upsert_task(
                conn,
                &TaskHistoryRow {
                    id: None,
                    session_id: "sess-1".into(),
                    task_id: "1".into(),
                    subject: "Fix authentication bug".into(),
                    description: "The auth module has a race condition".into(),
                    active_form: "Fixing authentication bug".into(),
                    status: "completed".into(),
                    blocks: "[]".into(),
                    blocked_by: "[]".into(),
                },
            )?;
            assert_eq!(count_tasks(conn)?, 1);

            let by_status = count_tasks_by_status(conn)?;
            assert_eq!(by_status.len(), 1);
            assert_eq!(by_status[0].0, "completed");
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_handoff_crud() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            upsert_handoff(
                conn,
                &HandoffRow {
                    id: None,
                    project: "nexcore".into(),
                    handoff_number: 97,
                    session_id: "sess-1".into(),
                    generated_at: "2026-02-02 06:51:26 UTC".into(),
                    status: "Session Complete".into(),
                    duration: "29h 19m".into(),
                    files_modified: 5,
                    lines_written: 40,
                    commits: 2,
                    uncommitted: 7,
                    content: "# Handoff content".into(),
                },
            )?;
            assert_eq!(count_handoffs(conn)?, 1);

            let by_proj = list_handoffs_by_project(conn, "nexcore")?;
            assert_eq!(by_proj.len(), 1);
            assert_eq!(by_proj[0].handoff_number, 97);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_antibody_crud() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            upsert_antibody(
                conn,
                &AntibodyRow {
                    id: "PANIC-001".into(),
                    name: "unwrap-eliminator".into(),
                    threat_type: "DAMP".into(),
                    severity: "critical".into(),
                    description: ".unwrap() causes panic".into(),
                    detection: r#"{"patterns": [".unwrap()"]}"#.into(),
                    response: r#"{"strategy": "suggest_safe_alternative"}"#.into(),
                    confidence: 0.95,
                    applications: 0,
                    false_positives: 0,
                    learned_from: "NexCore workspace audit".into(),
                    t1_grounding: Some("causality".into()),
                },
            )?;
            assert_eq!(count_antibodies(conn)?, 1);

            let all = list_antibodies(conn)?;
            assert_eq!(all[0].id, "PANIC-001");
            assert!((all[0].confidence - 0.95).abs() < f64::EPSILON);
            Ok(())
        })
        .expect("test");
    }
}
