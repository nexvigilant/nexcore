//! Code tracker CRUD: content-addressable file snapshots.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{DbError, Result};

/// A tracked file row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFileRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Project identifier
    pub project: String,
    /// File path (relative or absolute)
    pub file_path: String,
    /// SHA-256 content hash (first 32 hex chars)
    pub content_hash: String,
    /// File size in bytes
    pub file_size: u64,
    /// When the file was tracked
    pub tracked_at: DateTime<Utc>,
    /// File modification time
    pub mtime: DateTime<Utc>,
}

/// Upsert a tracked file (by project + file_path).
///
/// # Errors
///
/// Returns an error if the upsert fails.
pub fn upsert(conn: &Connection, f: &TrackedFileRow) -> Result<()> {
    conn.execute(
        "INSERT INTO tracked_files (project, file_path, content_hash, file_size, tracked_at, mtime)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(project, file_path) DO UPDATE SET
            content_hash = excluded.content_hash,
            file_size = excluded.file_size,
            tracked_at = excluded.tracked_at,
            mtime = excluded.mtime",
        params![
            f.project,
            f.file_path,
            f.content_hash,
            f.file_size as i64,
            f.tracked_at.to_rfc3339(),
            f.mtime.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a tracked file by project and path.
///
/// # Errors
///
/// Returns `NotFound` if the file isn't tracked.
pub fn get(conn: &Connection, project: &str, file_path: &str) -> Result<TrackedFileRow> {
    conn.query_row(
        "SELECT id, project, file_path, content_hash, file_size, tracked_at, mtime
         FROM tracked_files WHERE project = ?1 AND file_path = ?2",
        params![project, file_path],
        |row| {
            Ok(TrackedFileRow {
                id: Some(row.get(0)?),
                project: row.get(1)?,
                file_path: row.get(2)?,
                content_hash: row.get(3)?,
                file_size: row.get::<_, i64>(4)? as u64,
                tracked_at: parse_dt(row.get::<_, String>(5)?),
                mtime: parse_dt(row.get::<_, String>(6)?),
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("tracked file {project}/{file_path}")))
}

/// List all tracked files for a project.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_by_project(conn: &Connection, project: &str) -> Result<Vec<TrackedFileRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, file_path, content_hash, file_size, tracked_at, mtime
         FROM tracked_files WHERE project = ?1 ORDER BY file_path ASC",
    )?;
    let rows = stmt
        .query_map([project], |row| {
            Ok(TrackedFileRow {
                id: Some(row.get(0)?),
                project: row.get(1)?,
                file_path: row.get(2)?,
                content_hash: row.get(3)?,
                file_size: row.get::<_, i64>(4)? as u64,
                tracked_at: parse_dt(row.get::<_, String>(5)?),
                mtime: parse_dt(row.get::<_, String>(6)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn parse_dt(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    #[test]
    fn test_tracked_file_crud() {
        let pool = DbPool::open_in_memory().expect("open");
        pool.with_conn(|conn| {
            let f = TrackedFileRow {
                id: None,
                project: "nexcore".into(),
                file_path: "src/lib.rs".into(),
                content_hash: "abc123def456".into(),
                file_size: 1024,
                tracked_at: Utc::now(),
                mtime: Utc::now(),
            };
            upsert(conn, &f)?;

            let loaded = get(conn, "nexcore", "src/lib.rs")?;
            assert_eq!(loaded.content_hash, "abc123def456");
            assert_eq!(loaded.file_size, 1024);

            let all = list_by_project(conn, "nexcore")?;
            assert_eq!(all.len(), 1);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_upsert_updates_hash() {
        let pool = DbPool::open_in_memory().expect("open");
        pool.with_conn(|conn| {
            let f = TrackedFileRow {
                id: None,
                project: "p".into(),
                file_path: "a.rs".into(),
                content_hash: "hash1".into(),
                file_size: 100,
                tracked_at: Utc::now(),
                mtime: Utc::now(),
            };
            upsert(conn, &f)?;

            let mut updated = f.clone();
            updated.content_hash = "hash2".into();
            updated.file_size = 200;
            upsert(conn, &updated)?;

            let loaded = get(conn, "p", "a.rs")?;
            assert_eq!(loaded.content_hash, "hash2");
            assert_eq!(loaded.file_size, 200);
            Ok(())
        })
        .expect("test");
    }
}
