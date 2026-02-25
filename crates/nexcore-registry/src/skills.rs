//! Active skills CRUD operations.

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{RegistryError, Result};

/// A row from the `active_skills` table.
///
/// Fields are grouped into four categories:
/// - **Anthropic official**: fields from the Claude Code skill frontmatter spec
/// - **Computed/operational**: derived during scan (line count, agent pairing, etc.)
/// - **SMST assessment**: per-component quality scores cached from assessment engine
/// - **NexVigilant KPI tracking**: our custom fields for quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRow {
    // --- Anthropic official frontmatter fields ---
    /// Skill name (primary key, from directory name)
    pub name: String,
    /// Filesystem path to SKILL.md
    pub path: String,
    /// What the skill does and when to use it
    pub description: Option<String>,
    /// Hint shown during autocomplete (e.g., `[issue-number]`)
    pub argument_hint: Option<String>,
    /// If true, only users can invoke (Claude cannot auto-load)
    pub disable_model_invocation: bool,
    /// If true, users can invoke via `/name` (default: true per Anthropic spec)
    pub user_invocable: bool,
    /// Tools Claude can use without permission when skill is active
    pub allowed_tools: Option<String>,
    /// Model override when skill is active
    pub model: Option<String>,
    /// Execution context (`fork` = run in subagent)
    pub context: Option<String>,
    /// Subagent type for `context: fork` (e.g., `Explore`, `Plan`)
    pub agent: Option<String>,
    /// Skill-scoped hooks (JSON)
    pub hooks: Option<String>,

    // --- Computed / operational fields ---
    /// Line count of SKILL.md
    pub line_count: Option<i32>,
    /// Whether this skill has a paired agent file
    pub has_agent: bool,
    /// Number of sub-skills
    pub sub_skill_count: i32,
    /// Parent skill name (if sub-skill)
    pub parent_skill: Option<String>,

    // --- Runtime feature detection (scanned from content body) ---
    /// Content body uses `$ARGUMENTS`, `$N`, or `$ARGUMENTS[N]`
    pub uses_arguments: bool,
    /// Content body uses `` !`command` `` dynamic context injection
    pub uses_dynamic_context: bool,
    /// Content body uses `${CLAUDE_SESSION_ID}`
    pub uses_session_id: bool,
    /// Total character count of SKILL.md (for 2% budget tracking)
    pub content_chars: Option<i32>,

    // --- SMST v2 component breakdown (assessment cache) ---
    /// Input quality score (0-15)
    pub smst_input: Option<i32>,
    /// Output quality score (0-15)
    pub smst_output: Option<i32>,
    /// Logic quality score (0-25)
    pub smst_logic: Option<i32>,
    /// Error handling quality score (0-20)
    pub smst_error_handling: Option<i32>,
    /// Examples quality score (0-15)
    pub smst_examples: Option<i32>,
    /// References quality score (0-10)
    pub smst_references: Option<i32>,
    /// When the skill was last assessed
    pub last_assessed_at: Option<String>,
    /// Who/what performed the last assessment
    pub assessed_by: Option<String>,

    // --- NexVigilant KPI tracking fields ---
    /// Skill version string (our convention)
    pub version: Option<String>,
    /// Compliance tier (our quality metric)
    pub compliance: Option<String>,
    /// SMST v1 score
    pub smst_v1: Option<f64>,
    /// SMST v2 score
    pub smst_v2: Option<i32>,
    /// JSON array of tags (our convention)
    pub tags: Option<String>,
    /// Position in skill chain (our convention)
    pub chain_position: Option<String>,
    /// Pipeline this skill belongs to (our convention)
    pub pipeline: Option<String>,

    // --- Timestamps ---
    /// When the skill was last scanned
    pub scanned_at: DateTime,
    /// When the row was last updated
    pub updated_at: DateTime,
}

/// Column list for all active_skills queries (36 columns).
const COLUMNS: &str = "\
    name, path, description, argument_hint, disable_model_invocation, \
    user_invocable, allowed_tools, model, context, agent, hooks, \
    line_count, has_agent, sub_skill_count, parent_skill, \
    uses_arguments, uses_dynamic_context, uses_session_id, content_chars, \
    smst_input, smst_output, smst_logic, smst_error_handling, \
    smst_examples, smst_references, last_assessed_at, assessed_by, \
    version, compliance, smst_v1, smst_v2, tags, chain_position, pipeline, \
    scanned_at, updated_at";

/// Placeholder list for 36 positional params.
const PLACEHOLDERS: &str = "\
    ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,\
    ?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36";

/// Insert a new skill row.
///
/// # Errors
///
/// Returns an error if the insert fails (e.g., duplicate name).
pub fn insert(conn: &Connection, row: &SkillRow) -> Result<()> {
    let scanned = row.scanned_at.to_rfc3339();
    let updated = row.updated_at.to_rfc3339();
    conn.execute(
        &format!("INSERT INTO active_skills ({COLUMNS}) VALUES ({PLACEHOLDERS})"),
        params![
            row.name,
            row.path,
            row.description,
            row.argument_hint,
            row.disable_model_invocation,
            row.user_invocable,
            row.allowed_tools,
            row.model,
            row.context,
            row.agent,
            row.hooks,
            row.line_count,
            row.has_agent,
            row.sub_skill_count,
            row.parent_skill,
            row.uses_arguments,
            row.uses_dynamic_context,
            row.uses_session_id,
            row.content_chars,
            row.smst_input,
            row.smst_output,
            row.smst_logic,
            row.smst_error_handling,
            row.smst_examples,
            row.smst_references,
            row.last_assessed_at,
            row.assessed_by,
            row.version,
            row.compliance,
            row.smst_v1,
            row.smst_v2,
            row.tags,
            row.chain_position,
            row.pipeline,
            scanned,
            updated,
        ],
    )?;
    Ok(())
}

/// Upsert a skill row (insert or update on conflict).
///
/// # Errors
///
/// Returns an error on query failure.
pub fn upsert(conn: &Connection, row: &SkillRow) -> Result<()> {
    let scanned = row.scanned_at.to_rfc3339();
    let updated = row.updated_at.to_rfc3339();
    conn.execute(
        &format!(
            "INSERT INTO active_skills ({COLUMNS}) VALUES ({PLACEHOLDERS})
             ON CONFLICT(name) DO UPDATE SET
              path=excluded.path, description=excluded.description,
              argument_hint=excluded.argument_hint,
              disable_model_invocation=excluded.disable_model_invocation,
              user_invocable=excluded.user_invocable,
              allowed_tools=excluded.allowed_tools, model=excluded.model,
              context=excluded.context, agent=excluded.agent, hooks=excluded.hooks,
              line_count=excluded.line_count, has_agent=excluded.has_agent,
              sub_skill_count=excluded.sub_skill_count, parent_skill=excluded.parent_skill,
              uses_arguments=excluded.uses_arguments,
              uses_dynamic_context=excluded.uses_dynamic_context,
              uses_session_id=excluded.uses_session_id,
              content_chars=excluded.content_chars,
              smst_input=excluded.smst_input, smst_output=excluded.smst_output,
              smst_logic=excluded.smst_logic, smst_error_handling=excluded.smst_error_handling,
              smst_examples=excluded.smst_examples, smst_references=excluded.smst_references,
              last_assessed_at=excluded.last_assessed_at, assessed_by=excluded.assessed_by,
              version=excluded.version, compliance=excluded.compliance,
              smst_v1=excluded.smst_v1, smst_v2=excluded.smst_v2,
              tags=excluded.tags, chain_position=excluded.chain_position,
              pipeline=excluded.pipeline,
              scanned_at=excluded.scanned_at, updated_at=excluded.updated_at"
        ),
        params![
            row.name,
            row.path,
            row.description,
            row.argument_hint,
            row.disable_model_invocation,
            row.user_invocable,
            row.allowed_tools,
            row.model,
            row.context,
            row.agent,
            row.hooks,
            row.line_count,
            row.has_agent,
            row.sub_skill_count,
            row.parent_skill,
            row.uses_arguments,
            row.uses_dynamic_context,
            row.uses_session_id,
            row.content_chars,
            row.smst_input,
            row.smst_output,
            row.smst_logic,
            row.smst_error_handling,
            row.smst_examples,
            row.smst_references,
            row.last_assessed_at,
            row.assessed_by,
            row.version,
            row.compliance,
            row.smst_v1,
            row.smst_v2,
            row.tags,
            row.chain_position,
            row.pipeline,
            scanned,
            updated,
        ],
    )?;
    Ok(())
}

/// Map a rusqlite row to a `SkillRow` (36 columns).
fn row_to_skill(row: &rusqlite::Row<'_>) -> rusqlite::Result<SkillRow> {
    Ok(SkillRow {
        name: row.get(0)?,
        path: row.get(1)?,
        description: row.get(2)?,
        argument_hint: row.get(3)?,
        disable_model_invocation: row.get(4)?,
        user_invocable: row.get(5)?,
        allowed_tools: row.get(6)?,
        model: row.get(7)?,
        context: row.get(8)?,
        agent: row.get(9)?,
        hooks: row.get(10)?,
        line_count: row.get(11)?,
        has_agent: row.get(12)?,
        sub_skill_count: row.get(13)?,
        parent_skill: row.get(14)?,
        uses_arguments: row.get(15)?,
        uses_dynamic_context: row.get(16)?,
        uses_session_id: row.get(17)?,
        content_chars: row.get(18)?,
        smst_input: row.get(19)?,
        smst_output: row.get(20)?,
        smst_logic: row.get(21)?,
        smst_error_handling: row.get(22)?,
        smst_examples: row.get(23)?,
        smst_references: row.get(24)?,
        last_assessed_at: row.get(25)?,
        assessed_by: row.get(26)?,
        version: row.get(27)?,
        compliance: row.get(28)?,
        smst_v1: row.get(29)?,
        smst_v2: row.get(30)?,
        tags: row.get(31)?,
        chain_position: row.get(32)?,
        pipeline: row.get(33)?,
        scanned_at: parse_dt(row.get::<_, String>(34)?),
        updated_at: parse_dt(row.get::<_, String>(35)?),
    })
}

/// Get a skill by name.
///
/// # Errors
///
/// Returns `NotFound` if the skill doesn't exist.
pub fn get(conn: &Connection, name: &str) -> Result<SkillRow> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM active_skills WHERE name = ?1"),
        [name],
        row_to_skill,
    )
    .map_err(|_| RegistryError::NotFound(format!("skill {name}")))
}

/// List all skills, ordered by name.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_all(conn: &Connection) -> Result<Vec<SkillRow>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM active_skills ORDER BY name"
    ))?;

    let rows = stmt
        .query_map([], row_to_skill)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Count total skills.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count(conn: &Connection) -> Result<i64> {
    let c: i64 = conn.query_row("SELECT COUNT(*) FROM active_skills", [], |row| row.get(0))?;
    Ok(c)
}

/// Delete a skill by name.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn delete(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM active_skills WHERE name = ?1", [name])?;
    Ok(())
}

/// Delete all skills (used before full rescan).
///
/// # Errors
///
/// Returns an error on query failure.
pub fn delete_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM active_skills", [])?;
    Ok(())
}

/// Parse an RFC3339/ISO8601 datetime string, falling back to `DateTime::now()`.
fn parse_dt(s: String) -> DateTime {
    s.parse::<DateTime>().unwrap_or_else(|_| DateTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::RegistryPool;

    fn make_skill(name: &str) -> SkillRow {
        SkillRow {
            name: name.to_string(),
            path: format!("/skills/{name}/SKILL.md"),
            description: Some("Test skill".to_string()),
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
            version: Some("1.0.0".to_string()),
            compliance: None,
            smst_v1: None,
            smst_v2: None,
            tags: None,
            chain_position: None,
            pipeline: None,
            scanned_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            let skill = make_skill("test-skill");
            insert(conn, &skill)?;
            let got = get(conn, "test-skill")?;
            assert_eq!(got.name, "test-skill");
            assert_eq!(got.description, Some("Test skill".to_string()));
            assert!(got.user_invocable);
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
            let mut skill = make_skill("upsert-test");
            upsert(conn, &skill)?;
            skill.line_count = Some(200);
            upsert(conn, &skill)?;
            let got = get(conn, "upsert-test")?;
            assert_eq!(got.line_count, Some(200));
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
            insert(conn, &make_skill("a"))?;
            insert(conn, &make_skill("b"))?;
            let all = list_all(conn)?;
            assert_eq!(all.len(), 2);
            assert_eq!(all[0].name, "a");
            let c = count(conn)?;
            assert_eq!(c, 2);
            Ok(())
        })
        .ok();
    }

    #[test]
    fn test_delete() {
        let pool = RegistryPool::open_in_memory().ok();
        assert!(pool.is_some());
        let pool = pool.unwrap_or_else(|| unreachable!());
        pool.with_conn(|conn| {
            insert(conn, &make_skill("del-me"))?;
            assert_eq!(count(conn)?, 1);
            delete(conn, "del-me")?;
            assert_eq!(count(conn)?, 0);
            Ok(())
        })
        .ok();
    }
}
