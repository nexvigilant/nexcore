//! Active agents CRUD operations.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{RegistryError, Result};

/// A row from the `active_agents` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRow {
    /// Agent name (primary key)
    pub name: String,
    /// Filesystem path to agent .md file
    pub path: String,
    /// Model (e.g., "sonnet", "haiku", "opus")
    pub model: Option<String>,
    /// Tools available (JSON array string)
    pub tools: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Paired skill name (FK to `active_skills`)
    pub paired_skill: Option<String>,
    /// When the agent was last scanned
    pub scanned_at: DateTime<Utc>,
}

/// Insert a new agent row.
///
/// # Errors
///
/// Returns an error if the insert fails.
pub fn insert(conn: &Connection, row: &AgentRow) -> Result<()> {
    conn.execute(
        "INSERT INTO active_agents (name, path, model, tools, description, paired_skill, scanned_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            row.name,
            row.path,
            row.model,
            row.tools,
            row.description,
            row.paired_skill,
            row.scanned_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Upsert an agent row.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn upsert(conn: &Connection, row: &AgentRow) -> Result<()> {
    conn.execute(
        "INSERT INTO active_agents (name, path, model, tools, description, paired_skill, scanned_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7)
         ON CONFLICT(name) DO UPDATE SET
          path=excluded.path, model=excluded.model, tools=excluded.tools,
          description=excluded.description, paired_skill=excluded.paired_skill,
          scanned_at=excluded.scanned_at",
        params![
            row.name,
            row.path,
            row.model,
            row.tools,
            row.description,
            row.paired_skill,
            row.scanned_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get an agent by name.
///
/// # Errors
///
/// Returns `NotFound` if the agent doesn't exist.
pub fn get(conn: &Connection, name: &str) -> Result<AgentRow> {
    conn.query_row(
        "SELECT name, path, model, tools, description, paired_skill, scanned_at
         FROM active_agents WHERE name = ?1",
        [name],
        |row| {
            Ok(AgentRow {
                name: row.get(0)?,
                path: row.get(1)?,
                model: row.get(2)?,
                tools: row.get(3)?,
                description: row.get(4)?,
                paired_skill: row.get(5)?,
                scanned_at: parse_dt(row.get::<_, String>(6)?),
            })
        },
    )
    .map_err(|_| RegistryError::NotFound(format!("agent {name}")))
}

/// List all agents, ordered by name.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_all(conn: &Connection) -> Result<Vec<AgentRow>> {
    let mut stmt = conn.prepare(
        "SELECT name, path, model, tools, description, paired_skill, scanned_at
         FROM active_agents ORDER BY name",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AgentRow {
                name: row.get(0)?,
                path: row.get(1)?,
                model: row.get(2)?,
                tools: row.get(3)?,
                description: row.get(4)?,
                paired_skill: row.get(5)?,
                scanned_at: parse_dt(row.get::<_, String>(6)?),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Count total agents.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count(conn: &Connection) -> Result<i64> {
    let c: i64 = conn.query_row("SELECT COUNT(*) FROM active_agents", [], |row| row.get(0))?;
    Ok(c)
}

/// Delete an agent by name.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn delete(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM active_agents WHERE name = ?1", [name])?;
    Ok(())
}

/// Delete all agents (used before full rescan).
///
/// # Errors
///
/// Returns an error on query failure.
pub fn delete_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM active_agents", [])?;
    Ok(())
}

fn parse_dt(s: String) -> DateTime<Utc> {
    s.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    fn make_agent(name: &str) -> AgentRow {
        AgentRow {
            name: name.to_string(),
            path: format!("/agents/{name}.md"),
            model: Some("sonnet".to_string()),
            tools: Some("[\"Read\",\"Grep\"]".to_string()),
            description: Some("Test agent".to_string()),
            paired_skill: None,
            scanned_at: Utc::now(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let agent = make_agent("test-agent");
            insert(conn, &agent)?;
            let got = get(conn, "test-agent")?;
            assert_eq!(got.name, "test-agent");
            assert_eq!(got.model, Some("sonnet".to_string()));
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_upsert() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let mut agent = make_agent("upsert-agent");
            upsert(conn, &agent)?;
            agent.model = Some("opus".to_string());
            upsert(conn, &agent)?;
            let got = get(conn, "upsert-agent")?;
            assert_eq!(got.model, Some("opus".to_string()));
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_list_and_count() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            insert(conn, &make_agent("a"))?;
            insert(conn, &make_agent("b"))?;
            let all = list_all(conn)?;
            assert_eq!(all.len(), 2);
            let c = count(conn)?;
            assert_eq!(c, 2);
            Ok(())
        })
        .ok();
    }
}
