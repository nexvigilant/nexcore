//! Skill metrics and invocation CRUD operations.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{RegistryError, Result};

/// A row from the `skill_metrics` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRow {
    /// Skill name (source)
    pub source: String,
    /// Metric name (e.g., "invocations", "query_misses")
    pub metric: String,
    /// Metric type
    pub metric_type: String,
    /// Current value
    pub value: f64,
    /// Last updated
    pub updated_at: DateTime,
}

/// A row from the `skill_invocations` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Skill that was invoked
    pub skill_name: String,
    /// How it was triggered
    pub trigger_type: Option<String>,
    /// Session ID where invocation happened
    pub session_id: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<i64>,
    /// When the invocation occurred
    pub invoked_at: DateTime,
}

/// Upsert a metric row.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn upsert_metric(conn: &Connection, row: &MetricRow) -> Result<()> {
    conn.execute(
        "INSERT INTO skill_metrics (source, metric, type, value, updated_at)
         VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(source, metric) DO UPDATE SET
          value=excluded.value, updated_at=excluded.updated_at",
        params![
            row.source,
            row.metric,
            row.metric_type,
            row.value,
            row.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a specific metric.
///
/// # Errors
///
/// Returns `NotFound` if the metric doesn't exist.
pub fn get_metric(conn: &Connection, source: &str, metric: &str) -> Result<MetricRow> {
    conn.query_row(
        "SELECT source, metric, type, value, updated_at
         FROM skill_metrics WHERE source = ?1 AND metric = ?2",
        [source, metric],
        |row| {
            Ok(MetricRow {
                source: row.get(0)?,
                metric: row.get(1)?,
                metric_type: row.get(2)?,
                value: row.get(3)?,
                updated_at: parse_dt(row.get::<_, String>(4)?),
            })
        },
    )
    .map_err(|_| RegistryError::NotFound(format!("metric {source}/{metric}")))
}

/// List all metrics for a skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_metrics_for(conn: &Connection, source: &str) -> Result<Vec<MetricRow>> {
    let mut stmt = conn.prepare(
        "SELECT source, metric, type, value, updated_at
         FROM skill_metrics WHERE source = ?1 ORDER BY metric",
    )?;

    let rows = stmt
        .query_map([source], |row| {
            Ok(MetricRow {
                source: row.get(0)?,
                metric: row.get(1)?,
                metric_type: row.get(2)?,
                value: row.get(3)?,
                updated_at: parse_dt(row.get::<_, String>(4)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Record a skill invocation.
///
/// # Errors
///
/// Returns an error on insert failure.
pub fn record_invocation(conn: &Connection, row: &InvocationRow) -> Result<i64> {
    conn.execute(
        "INSERT INTO skill_invocations (skill_name, trigger_type, session_id, duration_ms, invoked_at)
         VALUES (?1,?2,?3,?4,?5)",
        params![
            row.skill_name,
            row.trigger_type,
            row.session_id,
            row.duration_ms,
            row.invoked_at.to_rfc3339(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List recent invocations for a skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_invocations(
    conn: &Connection,
    skill_name: &str,
    limit: i64,
) -> Result<Vec<InvocationRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_name, trigger_type, session_id, duration_ms, invoked_at
         FROM skill_invocations WHERE skill_name = ?1
         ORDER BY invoked_at DESC LIMIT ?2",
    )?;

    let rows = stmt
        .query_map(params![skill_name, limit], |row| {
            Ok(InvocationRow {
                id: row.get(0)?,
                skill_name: row.get(1)?,
                trigger_type: row.get(2)?,
                session_id: row.get(3)?,
                duration_ms: row.get(4)?,
                invoked_at: parse_dt(row.get::<_, String>(5)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Count invocations for a skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_invocations(conn: &Connection, skill_name: &str) -> Result<i64> {
    let c: i64 = conn.query_row(
        "SELECT COUNT(*) FROM skill_invocations WHERE skill_name = ?1",
        [skill_name],
        |row| row.get(0),
    )?;
    Ok(c)
}

fn parse_dt(s: String) -> DateTime {
    s.parse::<DateTime>().unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    #[test]
    fn test_upsert_and_get_metric() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let m = MetricRow {
                source: "test-skill".to_string(),
                metric: "invocations".to_string(),
                metric_type: "counter".to_string(),
                value: 42.0,
                updated_at: DateTime::now(),
            };
            upsert_metric(conn, &m)?;
            let got = get_metric(conn, "test-skill", "invocations")?;
            assert!((got.value - 42.0).abs() < f64::EPSILON);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_record_and_list_invocations() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let inv = InvocationRow {
                id: None,
                skill_name: "my-skill".to_string(),
                trigger_type: Some("slash".to_string()),
                session_id: Some("sess-1".to_string()),
                duration_ms: Some(150),
                invoked_at: DateTime::now(),
            };
            record_invocation(conn, &inv)?;
            record_invocation(conn, &inv)?;
            let list = list_invocations(conn, "my-skill", 10)?;
            assert_eq!(list.len(), 2);
            let c = count_invocations(conn, "my-skill")?;
            assert_eq!(c, 2);
            Ok(())
        })
        .ok();
    }
}
