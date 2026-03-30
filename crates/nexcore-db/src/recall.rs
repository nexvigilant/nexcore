//! Smarter recall: full-text search across brain knowledge stores.
//!
//! Uses FTS5 virtual tables (schema v11) to search artifacts, beliefs,
//! patterns, corrections, and sessions by keyword or phrase.

use rusqlite::{params, Connection};

use crate::error::Result;

/// A search result from any knowledge store.
#[derive(Debug, Clone)]
pub struct RecallResult {
    /// Which store this result came from
    pub source: RecallSource,
    /// Unique identifier within the source
    pub id: String,
    /// Primary text content (matched field)
    pub title: String,
    /// Snippet of matching content
    pub snippet: String,
    /// FTS5 rank score (lower = more relevant)
    pub rank: f64,
}

/// Which knowledge store a recall result came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallSource {
    /// Brain artifact
    Artifact,
    /// Implicit belief
    Belief,
    /// Detected pattern
    Pattern,
    /// Learned correction
    Correction,
    /// Session description
    Session,
}

impl std::fmt::Display for RecallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecallSource::Artifact => write!(f, "artifact"),
            RecallSource::Belief => write!(f, "belief"),
            RecallSource::Pattern => write!(f, "pattern"),
            RecallSource::Correction => write!(f, "correction"),
            RecallSource::Session => write!(f, "session"),
        }
    }
}

/// Search across all knowledge stores and return ranked results.
///
/// Results are ranked by FTS5 relevance and deduplicated across stores.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_all(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut results = Vec::new();

    // Search each FTS table and collect results
    if let Ok(mut r) = search_artifacts(conn, query, limit) {
        results.append(&mut r);
    }
    if let Ok(mut r) = search_beliefs(conn, query, limit) {
        results.append(&mut r);
    }
    if let Ok(mut r) = search_patterns(conn, query, limit) {
        results.append(&mut r);
    }
    if let Ok(mut r) = search_corrections(conn, query, limit) {
        results.append(&mut r);
    }
    if let Ok(mut r) = search_sessions(conn, query, limit) {
        results.append(&mut r);
    }

    // Sort by rank (lower = more relevant in FTS5)
    results.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit as usize);

    Ok(results)
}

/// Search artifacts by content or name.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_artifacts(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.name, snippet(artifacts_fts, 1, '<b>', '</b>', '...', 32), rank
         FROM artifacts_fts
         JOIN artifacts a ON a.id = artifacts_fts.rowid
         WHERE artifacts_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![query, limit], |row| {
            Ok(RecallResult {
                source: RecallSource::Artifact,
                id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                snippet: row.get(2)?,
                rank: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Search beliefs by proposition or category.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_beliefs(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, proposition, snippet(beliefs_fts, 1, '<b>', '</b>', '...', 32), rank
         FROM beliefs_fts
         WHERE beliefs_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![query, limit], |row| {
            Ok(RecallResult {
                source: RecallSource::Belief,
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                rank: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Search patterns by description or type.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_patterns(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, description, snippet(patterns_fts, 1, '<b>', '</b>', '...', 32), rank
         FROM patterns_fts
         WHERE patterns_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![query, limit], |row| {
            Ok(RecallResult {
                source: RecallSource::Pattern,
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                rank: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Search corrections by mistake or correction text.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_corrections(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut stmt = conn.prepare(
        "SELECT rowid, mistake, snippet(corrections_fts, 1, '<b>', '</b>', '...', 32), rank
         FROM corrections_fts
         WHERE corrections_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![query, limit], |row| {
            Ok(RecallResult {
                source: RecallSource::Correction,
                id: row.get::<_, i64>(0)?.to_string(),
                title: row.get(1)?,
                snippet: row.get(2)?,
                rank: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Search sessions by description.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn search_sessions(conn: &Connection, query: &str, limit: u32) -> Result<Vec<RecallResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, description, snippet(sessions_fts, 1, '<b>', '</b>', '...', 32), rank
         FROM sessions_fts
         WHERE sessions_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map(params![query, limit], |row| {
            Ok(RecallResult {
                source: RecallSource::Session,
                id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
                rank: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Rebuild all FTS indexes from source tables.
///
/// Use after bulk data changes or if FTS indexes are stale.
///
/// # Errors
///
/// Returns an error on rebuild failure.
pub fn rebuild_fts_indexes(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        INSERT INTO artifacts_fts(artifacts_fts) VALUES('rebuild');
        INSERT INTO beliefs_fts(beliefs_fts) VALUES('rebuild');
        INSERT INTO patterns_fts(patterns_fts) VALUES('rebuild');
        INSERT INTO corrections_fts(corrections_fts) VALUES('rebuild');
        INSERT INTO sessions_fts(sessions_fts) VALUES('rebuild');
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;
    use crate::schema::initialize;

    fn pool() -> DbPool {
        let pool = DbPool::open_in_memory().expect("in-memory pool");
        pool.with_conn(|conn| {
            initialize(conn)?;
            // Seed test data
            conn.execute_batch(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('s1', 'nexcore', 'Implemented signal detection pipeline', datetime('now'));

                 INSERT INTO artifacts (session_id, name, artifact_type, content, summary, created_at, updated_at)
                 VALUES ('s1', 'task.md', 'task', 'Build PRR computation for pharmacovigilance', 'PRR task', datetime('now'), datetime('now'));

                 INSERT INTO beliefs (id, proposition, category, confidence, evidence, formed_at, updated_at)
                 VALUES ('b1', 'Rust prevents memory safety bugs', 'capability', 0.9, '[]', datetime('now'), datetime('now'));

                 INSERT INTO patterns (id, pattern_type, description, examples, detected_at, updated_at, confidence, occurrence_count)
                 VALUES ('p1', 'naming', 'Uses snake_case for function names', '[\"compute_prr\"]', datetime('now'), datetime('now'), 0.8, 5);

                 INSERT INTO corrections (mistake, correction, context, learned_at, application_count, source, believability)
                 VALUES ('used unwrap', 'use error propagation with ?', 'In library code', datetime('now'), 3, 'compiler', 1.0);",
            )?;

            // Populate FTS indexes
            conn.execute_batch(
                "INSERT INTO artifacts_fts(rowid, name, content, summary, artifact_type)
                     SELECT id, name, content, summary, artifact_type FROM artifacts;
                 INSERT INTO beliefs_fts(rowid, id, proposition, category)
                     SELECT rowid, id, proposition, category FROM beliefs;
                 INSERT INTO patterns_fts(rowid, id, description, pattern_type, examples)
                     SELECT rowid, id, description, pattern_type, examples FROM patterns;
                 INSERT INTO corrections_fts(rowid, mistake, correction, context)
                     SELECT id, mistake, correction, COALESCE(context, '') FROM corrections;
                 INSERT INTO sessions_fts(rowid, id, description, project)
                     SELECT rowid, id, description, project FROM sessions;",
            )?;
            Ok(())
        })
        .expect("seed data");
        pool
    }

    #[test]
    fn test_search_artifacts() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_artifacts(conn, "pharmacovigilance", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].source, RecallSource::Artifact);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_beliefs() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_beliefs(conn, "memory safety", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].source, RecallSource::Belief);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_patterns() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_patterns(conn, "snake_case", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].source, RecallSource::Pattern);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_corrections() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_corrections(conn, "unwrap", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].source, RecallSource::Correction);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_sessions() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_sessions(conn, "signal detection", 10)?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].source, RecallSource::Session);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_all() {
        let db = pool();
        db.with_conn(|conn| {
            // "PRR" appears in artifact content and pattern examples
            let results = search_all(conn, "PRR", 10)?;
            assert!(!results.is_empty());
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_search_no_results() {
        let db = pool();
        db.with_conn(|conn| {
            let results = search_all(conn, "xyznonexistent", 10)?;
            assert!(results.is_empty());
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_rebuild_fts() {
        let db = pool();
        db.with_conn(|conn| {
            rebuild_fts_indexes(conn)?;
            // Should still find results after rebuild
            let results = search_beliefs(conn, "memory", 10)?;
            assert_eq!(results.len(), 1);
            Ok(())
        })
        .expect("test");
    }
}
