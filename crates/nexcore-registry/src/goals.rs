//! Skill goals and KPI CRUD operations.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{RegistryError, Result};

/// A row from the `skill_goals` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRow {
    /// Auto-increment ID
    pub id: Option<i64>,
    /// Skill this goal belongs to
    pub skill_name: String,
    /// Goal name
    pub name: String,
    /// SMART: Specific
    pub specific: String,
    /// SMART: Measurable
    pub measurable: String,
    /// SMART: Achievable
    pub achievable: String,
    /// SMART: Relevant
    pub relevant: String,
    /// SMART: Time-bound
    pub time_bound: String,
    /// Goal status
    pub status: String,
    /// Current measured value
    pub current_value: Option<f64>,
    /// Target value
    pub target_value: Option<f64>,
    /// Unit of measurement
    pub unit: Option<String>,
    /// When created
    pub created_at: DateTime,
    /// When last updated
    pub updated_at: DateTime,
}

/// A row from the `skill_kpis` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiRow {
    /// KPI name (primary key)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// SQL or expression formula
    pub formula: Option<String>,
    /// Current computed value
    pub current_value: Option<f64>,
    /// Target value
    pub target_value: Option<f64>,
    /// Unit of measurement
    pub unit: Option<String>,
    /// Direction: "higher_better", "lower_better", or "target"
    pub direction: Option<String>,
    /// When last updated
    pub updated_at: DateTime,
}

// --- Goal CRUD ---

/// Insert a new goal.
///
/// # Errors
///
/// Returns an error on insert failure.
pub fn insert_goal(conn: &Connection, row: &GoalRow) -> Result<i64> {
    conn.execute(
        "INSERT INTO skill_goals
         (skill_name, name, specific, measurable, achievable, relevant,
          time_bound, status, current_value, target_value, unit, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
        params![
            row.skill_name,
            row.name,
            row.specific,
            row.measurable,
            row.achievable,
            row.relevant,
            row.time_bound,
            row.status,
            row.current_value,
            row.target_value,
            row.unit,
            row.created_at.to_rfc3339(),
            row.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// List goals for a skill.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_goals(conn: &Connection, skill_name: &str) -> Result<Vec<GoalRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, skill_name, name, specific, measurable, achievable, relevant,
                time_bound, status, current_value, target_value, unit, created_at, updated_at
         FROM skill_goals WHERE skill_name = ?1 ORDER BY id",
    )?;

    let rows = stmt
        .query_map([skill_name], |row| {
            Ok(GoalRow {
                id: row.get(0)?,
                skill_name: row.get(1)?,
                name: row.get(2)?,
                specific: row.get(3)?,
                measurable: row.get(4)?,
                achievable: row.get(5)?,
                relevant: row.get(6)?,
                time_bound: row.get(7)?,
                status: row.get(8)?,
                current_value: row.get(9)?,
                target_value: row.get(10)?,
                unit: row.get(11)?,
                created_at: parse_dt(row.get::<_, String>(12)?),
                updated_at: parse_dt(row.get::<_, String>(13)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

// --- KPI CRUD ---

/// Upsert a KPI row.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn upsert_kpi(conn: &Connection, row: &KpiRow) -> Result<()> {
    conn.execute(
        "INSERT INTO skill_kpis (name, description, formula, current_value, target_value, unit, direction, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(name) DO UPDATE SET
          description=excluded.description, formula=excluded.formula,
          current_value=excluded.current_value, target_value=excluded.target_value,
          unit=excluded.unit, direction=excluded.direction, updated_at=excluded.updated_at",
        params![
            row.name,
            row.description,
            row.formula,
            row.current_value,
            row.target_value,
            row.unit,
            row.direction,
            row.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get a KPI by name.
///
/// # Errors
///
/// Returns `NotFound` if the KPI doesn't exist.
pub fn get_kpi(conn: &Connection, name: &str) -> Result<KpiRow> {
    conn.query_row(
        "SELECT name, description, formula, current_value, target_value, unit, direction, updated_at
         FROM skill_kpis WHERE name = ?1",
        [name],
        |row| {
            Ok(KpiRow {
                name: row.get(0)?,
                description: row.get(1)?,
                formula: row.get(2)?,
                current_value: row.get(3)?,
                target_value: row.get(4)?,
                unit: row.get(5)?,
                direction: row.get(6)?,
                updated_at: parse_dt(row.get::<_, String>(7)?),
            })
        },
    )
    .map_err(|_| RegistryError::NotFound(format!("kpi {name}")))
}

/// List all KPIs.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_kpis(conn: &Connection) -> Result<Vec<KpiRow>> {
    let mut stmt = conn.prepare(
        "SELECT name, description, formula, current_value, target_value, unit, direction, updated_at
         FROM skill_kpis ORDER BY name",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(KpiRow {
                name: row.get(0)?,
                description: row.get(1)?,
                formula: row.get(2)?,
                current_value: row.get(3)?,
                target_value: row.get(4)?,
                unit: row.get(5)?,
                direction: row.get(6)?,
                updated_at: parse_dt(row.get::<_, String>(7)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

fn parse_dt(s: String) -> DateTime {
    s.parse::<DateTime>().unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    #[test]
    fn test_upsert_and_get_kpi() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let kpi = KpiRow {
                name: "total_skills".to_string(),
                description: "Total skills".to_string(),
                formula: Some("SELECT COUNT(*) FROM active_skills".to_string()),
                current_value: Some(199.0),
                target_value: None,
                unit: Some("count".to_string()),
                direction: Some("higher_better".to_string()),
                updated_at: DateTime::now(),
            };
            upsert_kpi(conn, &kpi)?;
            let got = get_kpi(conn, "total_skills")?;
            assert_eq!(got.description, "Total skills");
            assert!((got.current_value.unwrap_or(0.0) - 199.0).abs() < f64::EPSILON);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_insert_and_list_goals() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            // Need a skill for FK (but FK not enforced on missing parent in this test
            // since the skill table exists but is empty — SQLite doesn't enforce FK on
            // insert unless the referenced table has the row, but our schema uses
            // REFERENCES without ON DELETE which means insert succeeds for non-existent parent)
            let goal = GoalRow {
                id: None,
                skill_name: "test-skill".to_string(),
                name: "Improve coverage".to_string(),
                specific: "Increase test coverage to 80%".to_string(),
                measurable: "Coverage percentage".to_string(),
                achievable: "Yes".to_string(),
                relevant: "Quality".to_string(),
                time_bound: "2026-Q1".to_string(),
                status: "active".to_string(),
                current_value: Some(60.0),
                target_value: Some(80.0),
                unit: Some("percent".to_string()),
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
            };
            insert_goal(conn, &goal)?;
            let goals = list_goals(conn, "test-skill")?;
            assert_eq!(goals.len(), 1);
            assert_eq!(goals[0].name, "Improve coverage");
            Ok(())
        })
        .ok();
    }
}
