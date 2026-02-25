//! Artifact and artifact version CRUD operations.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{DbError, Result};

/// An artifact row (mutable current state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRow {
    /// Auto-increment ID (None for new inserts)
    pub id: Option<i64>,
    /// Parent session ID
    pub session_id: String,
    /// Artifact name (e.g., "phase1-plan")
    pub name: String,
    /// Type: task, implementation_plan, walkthrough, review, research, decision, custom
    pub artifact_type: String,
    /// Current content
    pub content: String,
    /// Human-readable summary
    pub summary: String,
    /// Current resolved version (0 = never resolved)
    pub current_version: u32,
    /// JSON array of tags
    pub tags: String,
    /// JSON custom metadata
    pub custom_meta: String,
    /// When created
    pub created_at: DateTime,
    /// When last updated
    pub updated_at: DateTime,
}

/// An immutable resolved version of an artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactVersionRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Parent session ID
    pub session_id: String,
    /// Artifact name
    pub artifact_name: String,
    /// Version number (1, 2, 3, ...)
    pub version: u32,
    /// Snapshot content
    pub content: String,
    /// When resolved
    pub resolved_at: DateTime,
}

/// Insert or update an artifact (upsert by session_id + name).
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert(conn: &Connection, art: &ArtifactRow) -> Result<()> {
    conn.execute(
        "INSERT INTO artifacts (session_id, name, artifact_type, content, summary,
                                current_version, tags, custom_meta, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(session_id, name) DO UPDATE SET
            content = excluded.content,
            summary = excluded.summary,
            current_version = excluded.current_version,
            tags = excluded.tags,
            custom_meta = excluded.custom_meta,
            updated_at = excluded.updated_at",
        params![
            art.session_id,
            art.name,
            art.artifact_type,
            art.content,
            art.summary,
            art.current_version,
            art.tags,
            art.custom_meta,
            art.created_at.to_rfc3339(),
            art.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get an artifact by session ID and name.
///
/// # Errors
///
/// Returns `NotFound` if the artifact doesn't exist.
pub fn get(conn: &Connection, session_id: &str, name: &str) -> Result<ArtifactRow> {
    conn.query_row(
        "SELECT id, session_id, name, artifact_type, content, summary,
                current_version, tags, custom_meta, created_at, updated_at
         FROM artifacts WHERE session_id = ?1 AND name = ?2",
        params![session_id, name],
        |row| {
            Ok(ArtifactRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                name: row.get(2)?,
                artifact_type: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                current_version: row.get(6)?,
                tags: row.get(7)?,
                custom_meta: row.get(8)?,
                created_at: parse_datetime(row.get::<_, String>(9)?),
                updated_at: parse_datetime(row.get::<_, String>(10)?),
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("artifact {session_id}/{name}")))
}

/// List all artifacts for a session.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_by_session(conn: &Connection, session_id: &str) -> Result<Vec<ArtifactRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, name, artifact_type, content, summary,
                current_version, tags, custom_meta, created_at, updated_at
         FROM artifacts WHERE session_id = ?1 ORDER BY created_at ASC",
    )?;

    let rows = stmt
        .query_map([session_id], |row| {
            Ok(ArtifactRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                name: row.get(2)?,
                artifact_type: row.get(3)?,
                content: row.get(4)?,
                summary: row.get(5)?,
                current_version: row.get(6)?,
                tags: row.get(7)?,
                custom_meta: row.get(8)?,
                created_at: parse_datetime(row.get::<_, String>(9)?),
                updated_at: parse_datetime(row.get::<_, String>(10)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Insert a resolved (immutable) artifact version.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert_version(conn: &Connection, ver: &ArtifactVersionRow) -> Result<()> {
    conn.execute(
        "INSERT INTO artifact_versions (session_id, artifact_name, version, content, resolved_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            ver.session_id,
            ver.artifact_name,
            ver.version,
            ver.content,
            ver.resolved_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a specific resolved version.
///
/// # Errors
///
/// Returns `NotFound` if the version doesn't exist.
pub fn get_version(
    conn: &Connection,
    session_id: &str,
    artifact_name: &str,
    version: u32,
) -> Result<ArtifactVersionRow> {
    conn.query_row(
        "SELECT id, session_id, artifact_name, version, content, resolved_at
         FROM artifact_versions
         WHERE session_id = ?1 AND artifact_name = ?2 AND version = ?3",
        params![session_id, artifact_name, version],
        |row| {
            Ok(ArtifactVersionRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                artifact_name: row.get(2)?,
                version: row.get(3)?,
                content: row.get(4)?,
                resolved_at: parse_datetime(row.get::<_, String>(5)?),
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("version {session_id}/{artifact_name}@{version}")))
}

/// List all versions of an artifact.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_versions(
    conn: &Connection,
    session_id: &str,
    artifact_name: &str,
) -> Result<Vec<ArtifactVersionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, artifact_name, version, content, resolved_at
         FROM artifact_versions
         WHERE session_id = ?1 AND artifact_name = ?2
         ORDER BY version ASC",
    )?;

    let rows = stmt
        .query_map(params![session_id, artifact_name], |row| {
            Ok(ArtifactVersionRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                artifact_name: row.get(2)?,
                version: row.get(3)?,
                content: row.get(4)?,
                resolved_at: parse_datetime(row.get::<_, String>(5)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Parse an RFC3339 datetime string, falling back to now on failure.
fn parse_datetime(s: String) -> DateTime {
    DateTime::parse_from_rfc3339(&s).unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;
    use crate::sessions;

    fn setup() -> DbPool {
        let pool = DbPool::open_in_memory().expect("open");
        pool.with_conn(|conn| {
            sessions::insert(
                conn,
                &sessions::SessionRow {
                    id: "s1".into(),
                    project: "test".into(),
                    git_commit: None,
                    description: "Test".into(),
                    created_at: DateTime::now(),
                },
            )
        })
        .expect("insert session");
        pool
    }

    #[test]
    fn test_upsert_and_get() {
        let pool = setup();
        pool.with_conn(|conn| {
            let art = ArtifactRow {
                id: None,
                session_id: "s1".into(),
                name: "task.md".into(),
                artifact_type: "task".into(),
                content: "# My Task".into(),
                summary: "My task".into(),
                current_version: 0,
                tags: "[]".into(),
                custom_meta: "null".into(),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            upsert(conn, &art)?;

            let loaded = get(conn, "s1", "task.md")?;
            assert_eq!(loaded.name, "task.md");
            assert_eq!(loaded.content, "# My Task");
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_upsert_updates_existing() {
        let pool = setup();
        pool.with_conn(|conn| {
            let art = ArtifactRow {
                id: None,
                session_id: "s1".into(),
                name: "plan.md".into(),
                artifact_type: "plan".into(),
                content: "v1".into(),
                summary: "Plan v1".into(),
                current_version: 0,
                tags: "[]".into(),
                custom_meta: "null".into(),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            upsert(conn, &art)?;

            let mut updated = art.clone();
            updated.content = "v2".into();
            updated.summary = "Plan v2".into();
            updated.current_version = 1;
            upsert(conn, &updated)?;

            let loaded = get(conn, "s1", "plan.md")?;
            assert_eq!(loaded.content, "v2");
            assert_eq!(loaded.current_version, 1);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_versions() {
        let pool = setup();
        pool.with_conn(|conn| {
            insert_version(
                conn,
                &ArtifactVersionRow {
                    id: None,
                    session_id: "s1".into(),
                    artifact_name: "task.md".into(),
                    version: 1,
                    content: "Resolved v1".into(),
                    resolved_at: DateTime::now(),
                },
            )?;
            insert_version(
                conn,
                &ArtifactVersionRow {
                    id: None,
                    session_id: "s1".into(),
                    artifact_name: "task.md".into(),
                    version: 2,
                    content: "Resolved v2".into(),
                    resolved_at: DateTime::now(),
                },
            )?;

            let v1 = get_version(conn, "s1", "task.md", 1)?;
            assert_eq!(v1.content, "Resolved v1");

            let all = list_versions(conn, "s1", "task.md")?;
            assert_eq!(all.len(), 2);
            Ok(())
        })
        .expect("test");
    }
}
