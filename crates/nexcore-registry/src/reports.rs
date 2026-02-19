//! Gap report generation for skill ecosystem metadata coverage.
//!
//! Queries the database to identify skills missing critical fields,
//! producing an actionable gap report for prioritized remediation.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Ecosystem-wide gap report showing missing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReport {
    /// Total skills in ecosystem
    pub total_skills: i64,
    /// Skills missing a description
    pub missing_description: Vec<String>,
    /// Skills missing tags
    pub missing_tags: Vec<String>,
    /// User-invocable skills missing argument_hint
    pub missing_argument_hint: Vec<String>,
    /// Skills missing allowed_tools
    pub missing_allowed_tools: Vec<String>,
    /// Skills missing version
    pub missing_version: Vec<String>,
    /// Skills exceeding 16KB content budget: (name, content_chars)
    pub over_budget: Vec<(String, i32)>,
    /// Skills exceeding 500-line limit: (name, line_count)
    pub over_line_limit: Vec<(String, i32)>,
    /// Skills with no compliance rating
    pub missing_compliance: Vec<String>,
}

impl GapReport {
    /// Total number of distinct gap instances.
    #[must_use]
    pub fn total_gaps(&self) -> usize {
        self.missing_description.len()
            + self.missing_tags.len()
            + self.missing_argument_hint.len()
            + self.missing_allowed_tools.len()
            + self.missing_version.len()
            + self.over_budget.len()
            + self.over_line_limit.len()
            + self.missing_compliance.len()
    }

    /// Coverage percentage (fields populated / total possible).
    #[must_use]
    pub fn coverage_pct(&self) -> f64 {
        if self.total_skills == 0 {
            return 100.0;
        }
        let total = self.total_skills as f64;
        // 6 key fields tracked per skill
        let possible = total * 6.0;
        let missing = (self.missing_description.len()
            + self.missing_tags.len()
            + self.missing_allowed_tools.len()
            + self.missing_version.len()
            + self.over_budget.len()
            + self.over_line_limit.len()) as f64;
        ((possible - missing) / possible) * 100.0
    }
}

/// Generate a gap report from the current database state.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn generate_gap_report(conn: &Connection) -> Result<GapReport> {
    let total_skills: i64 =
        conn.query_row("SELECT COUNT(*) FROM active_skills", [], |r| r.get(0))?;

    let missing_description = query_names(
        conn,
        "SELECT name FROM active_skills WHERE description IS NULL ORDER BY name",
    )?;

    let missing_tags = query_names(
        conn,
        "SELECT name FROM active_skills WHERE tags IS NULL ORDER BY name",
    )?;

    let missing_argument_hint = query_names(
        conn,
        "SELECT name FROM active_skills WHERE user_invocable = 1 AND argument_hint IS NULL ORDER BY name",
    )?;

    let missing_allowed_tools = query_names(
        conn,
        "SELECT name FROM active_skills WHERE allowed_tools IS NULL ORDER BY name",
    )?;

    let missing_version = query_names(
        conn,
        "SELECT name FROM active_skills WHERE version IS NULL ORDER BY name",
    )?;

    let over_budget = query_name_int(
        conn,
        "SELECT name, content_chars FROM active_skills WHERE content_chars > 16384 ORDER BY content_chars DESC",
    )?;

    let over_line_limit = query_name_int(
        conn,
        "SELECT name, line_count FROM active_skills WHERE line_count > 500 ORDER BY line_count DESC",
    )?;

    let missing_compliance = query_names(
        conn,
        "SELECT name FROM active_skills WHERE compliance IS NULL ORDER BY name",
    )?;

    Ok(GapReport {
        total_skills,
        missing_description,
        missing_tags,
        missing_argument_hint,
        missing_allowed_tools,
        missing_version,
        over_budget,
        over_line_limit,
        missing_compliance,
    })
}

/// Query returning a list of skill names.
fn query_names(conn: &Connection, sql: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(rows)
}

/// Query returning (name, integer) pairs.
fn query_name_int(conn: &Connection, sql: &str) -> Result<Vec<(String, i32)>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<(String, i32)>, _>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::pool::RegistryPool;
    use crate::skills::{self, SkillRow};

    fn make_skill(name: &str) -> SkillRow {
        let now = Utc::now();
        SkillRow {
            name: name.to_string(),
            path: format!("/skills/{name}/SKILL.md"),
            description: Some("Test".to_string()),
            argument_hint: None,
            disable_model_invocation: false,
            user_invocable: true,
            allowed_tools: None,
            model: None,
            context: None,
            agent: None,
            hooks: None,
            line_count: Some(100),
            has_agent: false,
            sub_skill_count: 0,
            parent_skill: None,
            uses_arguments: false,
            uses_dynamic_context: false,
            uses_session_id: false,
            content_chars: Some(500),
            smst_input: None,
            smst_output: None,
            smst_logic: None,
            smst_error_handling: None,
            smst_examples: None,
            smst_references: None,
            last_assessed_at: None,
            assessed_by: None,
            version: None,
            compliance: None,
            smst_v1: None,
            smst_v2: None,
            tags: None,
            chain_position: None,
            pipeline: None,
            scanned_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_gap_report_empty_db() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let report = generate_gap_report(conn)?;
            assert_eq!(report.total_skills, 0);
            assert_eq!(report.total_gaps(), 0);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_gap_report_finds_missing_fields() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            skills::upsert(conn, &make_skill("s1"))?;
            let mut s2 = make_skill("s2");
            s2.description = None;
            skills::upsert(conn, &s2)?;

            let report = generate_gap_report(conn)?;
            assert_eq!(report.total_skills, 2);
            assert_eq!(report.missing_description.len(), 1);
            assert_eq!(report.missing_description[0], "s2");
            // Both missing tags, version, allowed_tools, compliance
            assert_eq!(report.missing_tags.len(), 2);
            assert_eq!(report.missing_version.len(), 2);
            assert_eq!(report.missing_allowed_tools.len(), 2);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_gap_report_over_budget() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let mut big = make_skill("big");
            big.content_chars = Some(20000);
            big.line_count = Some(600);
            skills::upsert(conn, &big)?;

            let report = generate_gap_report(conn)?;
            assert_eq!(report.over_budget.len(), 1);
            assert_eq!(report.over_budget[0].0, "big");
            assert_eq!(report.over_line_limit.len(), 1);
            Ok(())
        })
        .ok();
    }
}
