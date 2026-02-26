//! Persistent experience stores.
//!
//! SQLite-backed storage so learnings survive across sessions.
//! Grounds to: π(Persistence) + λ(Location)

use rusqlite::{Connection, params};

use crate::GroundedError;
use crate::confidence::Confidence;
use crate::feedback::{ExperienceStore, Learning, Verdict};

/// SQLite-backed experience store.
///
/// Persists learnings to a database file, enabling accumulation
/// across sessions — the "refined priors" from the spec.
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Open or create a SQLite store at the given path.
    pub fn open(path: &std::path::Path) -> Result<Self, GroundedError> {
        let conn = Connection::open(path)
            .map_err(|e| GroundedError::PersistenceFailed(format!("sqlite open: {e}")))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Create an in-memory SQLite store (useful for testing).
    pub fn in_memory() -> Result<Self, GroundedError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| GroundedError::PersistenceFailed(format!("sqlite memory: {e}")))?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), GroundedError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS learnings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    insight TEXT NOT NULL,
                    posterior REAL NOT NULL,
                    verdict TEXT NOT NULL,
                    hypothesis_claim TEXT NOT NULL,
                    observation TEXT NOT NULL,
                    learned_at TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_learnings_claim
                    ON learnings(hypothesis_claim);
                CREATE INDEX IF NOT EXISTS idx_learnings_verdict
                    ON learnings(verdict);",
            )
            .map_err(|e| GroundedError::PersistenceFailed(format!("schema init: {e}")))?;
        Ok(())
    }

    /// Count total learnings stored.
    pub fn count(&self) -> Result<u64, GroundedError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM learnings", [], |row| row.get(0))
            .map_err(|e| GroundedError::PersistenceFailed(format!("count: {e}")))?;
        Ok(count as u64)
    }

    /// Get learnings by verdict type.
    pub fn learnings_by_verdict(&self, verdict: Verdict) -> Result<Vec<Learning>, GroundedError> {
        let verdict_str = verdict_to_str(verdict);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT insight, posterior, verdict, hypothesis_claim, observation, learned_at
                 FROM learnings WHERE verdict = ?1 ORDER BY learned_at DESC",
            )
            .map_err(|e| GroundedError::PersistenceFailed(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map(params![verdict_str], |row| {
                Ok(LearningRow {
                    insight: row.get(0)?,
                    posterior: row.get(1)?,
                    verdict: row.get(2)?,
                    hypothesis_claim: row.get(3)?,
                    observation: row.get(4)?,
                    learned_at: row.get(5)?,
                })
            })
            .map_err(|e| GroundedError::PersistenceFailed(format!("query: {e}")))?;

        let mut learnings = Vec::new();
        for row in rows {
            let row = row.map_err(|e| GroundedError::PersistenceFailed(format!("row: {e}")))?;
            learnings.push(row_to_learning(row)?);
        }
        Ok(learnings)
    }
}

struct LearningRow {
    insight: String,
    posterior: f64,
    verdict: String,
    hypothesis_claim: String,
    observation: String,
    learned_at: String,
}

fn verdict_to_str(v: Verdict) -> &'static str {
    match v {
        Verdict::Supported => "supported",
        Verdict::Refuted => "refuted",
        Verdict::Inconclusive => "inconclusive",
    }
}

fn str_to_verdict(s: &str) -> Result<Verdict, GroundedError> {
    match s {
        "supported" => Ok(Verdict::Supported),
        "refuted" => Ok(Verdict::Refuted),
        "inconclusive" => Ok(Verdict::Inconclusive),
        other => Err(GroundedError::PersistenceFailed(format!(
            "unknown verdict value in database: {other:?}"
        ))),
    }
}

fn row_to_learning(row: LearningRow) -> Result<Learning, GroundedError> {
    let posterior = Confidence::new(row.posterior)?;
    let learned_at = nexcore_chrono::DateTime::parse_from_rfc3339(&row.learned_at)
        .map_err(|e| GroundedError::PersistenceFailed(format!("parse date: {e}")))?;
    let verdict = str_to_verdict(&row.verdict)?;

    Ok(Learning {
        insight: row.insight,
        posterior,
        verdict,
        hypothesis_claim: row.hypothesis_claim,
        observation: row.observation,
        learned_at,
    })
}

impl ExperienceStore for SqliteStore {
    fn persist(&mut self, learning: &Learning) -> Result<(), GroundedError> {
        self.conn
            .execute(
                "INSERT INTO learnings (insight, posterior, verdict, hypothesis_claim, observation, learned_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    learning.insight,
                    learning.posterior.value(),
                    verdict_to_str(learning.verdict),
                    learning.hypothesis_claim,
                    learning.observation,
                    learning.learned_at.to_rfc3339(),
                ],
            )
            .map_err(|e| GroundedError::PersistenceFailed(format!("insert: {e}")))?;
        Ok(())
    }

    fn all_learnings(&self) -> Result<Vec<Learning>, GroundedError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT insight, posterior, verdict, hypothesis_claim, observation, learned_at
                 FROM learnings ORDER BY learned_at DESC",
            )
            .map_err(|e| GroundedError::PersistenceFailed(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(LearningRow {
                    insight: row.get(0)?,
                    posterior: row.get(1)?,
                    verdict: row.get(2)?,
                    hypothesis_claim: row.get(3)?,
                    observation: row.get(4)?,
                    learned_at: row.get(5)?,
                })
            })
            .map_err(|e| GroundedError::PersistenceFailed(format!("query: {e}")))?;

        let mut learnings = Vec::new();
        for row in rows {
            let row = row.map_err(|e| GroundedError::PersistenceFailed(format!("row: {e}")))?;
            learnings.push(row_to_learning(row)?);
        }
        Ok(learnings)
    }

    fn learnings_for(&self, claim: &str) -> Result<Vec<Learning>, GroundedError> {
        let pattern = format!("%{claim}%");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT insight, posterior, verdict, hypothesis_claim, observation, learned_at
                 FROM learnings WHERE hypothesis_claim LIKE ?1 ORDER BY learned_at DESC",
            )
            .map_err(|e| GroundedError::PersistenceFailed(format!("prepare: {e}")))?;

        let rows = stmt
            .query_map(params![pattern], |row| {
                Ok(LearningRow {
                    insight: row.get(0)?,
                    posterior: row.get(1)?,
                    verdict: row.get(2)?,
                    hypothesis_claim: row.get(3)?,
                    observation: row.get(4)?,
                    learned_at: row.get(5)?,
                })
            })
            .map_err(|e| GroundedError::PersistenceFailed(format!("query: {e}")))?;

        let mut learnings = Vec::new();
        for row in rows {
            let row = row.map_err(|e| GroundedError::PersistenceFailed(format!("row: {e}")))?;
            learnings.push(row_to_learning(row)?);
        }
        Ok(learnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;

    fn make_learning(claim: &str, verdict: Verdict, posterior: f64) -> Learning {
        Learning {
            insight: format!("insight about {claim}"),
            posterior: Confidence::new(posterior).unwrap_or(Confidence::NONE),
            verdict,
            hypothesis_claim: claim.into(),
            observation: "observed".into(),
            learned_at: DateTime::now(),
        }
    }

    #[test]
    fn sqlite_store_persist_and_retrieve() {
        let store_result = SqliteStore::in_memory();
        assert!(store_result.is_ok(), "in-memory SQLite should always open");
        let mut store = store_result.unwrap_or_else(|_| unreachable!());

        let l1 = make_learning("gravity works", Verdict::Supported, 0.95);
        let l2 = make_learning("perpetual motion", Verdict::Refuted, 0.1);
        let l3 = make_learning("quantum effects", Verdict::Inconclusive, 0.5);

        assert!(store.persist(&l1).is_ok(), "persist l1 must succeed");
        assert!(store.persist(&l2).is_ok(), "persist l2 must succeed");
        assert!(store.persist(&l3).is_ok(), "persist l3 must succeed");

        let count = store.count();
        assert!(count.is_ok(), "count query must succeed");
        assert_eq!(count.unwrap_or(0), 3);

        let all = store.all_learnings();
        assert!(all.is_ok(), "all_learnings query must succeed");
        assert_eq!(all.unwrap_or_default().len(), 3);

        let supported = store.learnings_by_verdict(Verdict::Supported);
        assert!(supported.is_ok(), "verdict query must succeed");
        let supported = supported.unwrap_or_default();
        assert_eq!(supported.len(), 1);
        assert!(supported[0].hypothesis_claim.contains("gravity"));
    }

    #[test]
    fn sqlite_store_query_by_claim() {
        let store_result = SqliteStore::in_memory();
        assert!(store_result.is_ok(), "in-memory SQLite should always open");
        let mut store = store_result.unwrap_or_else(|_| unreachable!());

        assert!(
            store
                .persist(&make_learning("rust is fast", Verdict::Supported, 0.9))
                .is_ok(),
            "persist must succeed"
        );
        assert!(
            store
                .persist(&make_learning("rust is easy", Verdict::Inconclusive, 0.6))
                .is_ok(),
            "persist must succeed"
        );
        assert!(
            store
                .persist(&make_learning("python is slow", Verdict::Supported, 0.8))
                .is_ok(),
            "persist must succeed"
        );

        let rust_learnings = store.learnings_for("rust");
        assert!(rust_learnings.is_ok(), "learnings_for query must succeed");
        assert_eq!(rust_learnings.unwrap_or_default().len(), 2);

        let python_learnings = store.learnings_for("python");
        assert!(python_learnings.is_ok(), "learnings_for query must succeed");
        assert_eq!(python_learnings.unwrap_or_default().len(), 1);
    }
}
