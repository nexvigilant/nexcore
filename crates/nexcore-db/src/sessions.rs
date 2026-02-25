//! Session CRUD operations.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{DbError, Result};

/// A brain session row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    /// Unique session ID (UUID)
    pub id: String,
    /// Project name
    pub project: String,
    /// Git commit at session creation
    pub git_commit: Option<String>,
    /// Human-readable description
    pub description: String,
    /// When the session was created
    pub created_at: DateTime,
}

/// Insert a new session.
///
/// # Errors
///
/// Returns an error if the insert fails (e.g., duplicate ID).
pub fn insert(conn: &Connection, session: &SessionRow) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (id, project, git_commit, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session.id,
            session.project,
            session.git_commit,
            session.description,
            session.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a session by ID.
///
/// # Errors
///
/// Returns `NotFound` if the session doesn't exist.
pub fn get(conn: &Connection, id: &str) -> Result<SessionRow> {
    conn.query_row(
        "SELECT id, project, git_commit, description, created_at FROM sessions WHERE id = ?1",
        [id],
        |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                project: row.get(1)?,
                git_commit: row.get(2)?,
                description: row.get(3)?,
                created_at: parse_datetime(row.get::<_, String>(4)?),
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("session {id}")))
}

/// List all sessions, newest first.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_all(conn: &Connection) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, git_commit, description, created_at
         FROM sessions ORDER BY created_at DESC",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                project: row.get(1)?,
                git_commit: row.get(2)?,
                description: row.get(3)?,
                created_at: parse_datetime(row.get::<_, String>(4)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// List sessions for a specific project.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_by_project(conn: &Connection, project: &str) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, git_commit, description, created_at
         FROM sessions WHERE project = ?1 ORDER BY created_at DESC",
    )?;

    let rows = stmt
        .query_map([project], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                project: row.get(1)?,
                git_commit: row.get(2)?,
                description: row.get(3)?,
                created_at: parse_datetime(row.get::<_, String>(4)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Insert a session, ignoring if the ID already exists.
///
/// Used by dual-write paths where the session may already exist from migration.
///
/// # Errors
///
/// Returns an error on query failure (not on conflict).
pub fn insert_or_ignore(conn: &Connection, session: &SessionRow) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO sessions (id, project, git_commit, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session.id,
            session.project,
            session.git_commit,
            session.description,
            session.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Count total sessions.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
    Ok(count)
}

/// Parse an RFC3339 datetime string, falling back to now on failure.
fn parse_datetime(s: String) -> DateTime {
    DateTime::parse_from_rfc3339(&s).unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    fn test_pool() -> DbPool {
        DbPool::open_in_memory().expect("in-memory db")
    }

    #[test]
    fn test_insert_and_get() {
        let pool = test_pool();
        pool.with_conn(|conn| {
            let session = SessionRow {
                id: "test-session-1".into(),
                project: "nucleus".into(),
                git_commit: Some("abc123".into()),
                description: "Test session".into(),
                created_at: DateTime::now(),
            };
            insert(conn, &session)?;

            let loaded = get(conn, "test-session-1")?;
            assert_eq!(loaded.id, "test-session-1");
            assert_eq!(loaded.project, "nucleus");
            assert_eq!(loaded.description, "Test session");
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_list_all() {
        let pool = test_pool();
        pool.with_conn(|conn| {
            for i in 0..3 {
                insert(
                    conn,
                    &SessionRow {
                        id: format!("session-{i}"),
                        project: "test".into(),
                        git_commit: None,
                        description: format!("Session {i}"),
                        created_at: DateTime::now(),
                    },
                )?;
            }
            let all = list_all(conn)?;
            assert_eq!(all.len(), 3);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_get_not_found() {
        let pool = test_pool();
        let result = pool.with_conn(|conn| get(conn, "nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_count() {
        let pool = test_pool();
        pool.with_conn(|conn| {
            assert_eq!(count(conn)?, 0);
            insert(
                conn,
                &SessionRow {
                    id: "s1".into(),
                    project: "p".into(),
                    git_commit: None,
                    description: "d".into(),
                    created_at: DateTime::now(),
                },
            )?;
            assert_eq!(count(conn)?, 1);
            Ok(())
        })
        .expect("test");
    }
}
