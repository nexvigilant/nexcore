//! Database schema definition and migration.
//!
//! Schema version is tracked in a `schema_version` table.
//! V1 contains all 17 tables from the original `skills.db`.

use rusqlite::Connection;

use crate::error::{RegistryError, Result};

/// Current schema version. Increment when adding migrations.
pub const CURRENT_SCHEMA_VERSION: u32 = 4;

/// Initialize the database schema (create all tables if they don't exist).
///
/// # Errors
///
/// Returns an error if tables cannot be created.
pub fn initialize(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version     INTEGER PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            description TEXT
        );",
    )?;

    let current: Option<u32> = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    match current {
        None => {
            // Fresh database: V1 DDL already includes all V2 columns,
            // so no need to run apply_v2_columns().
            apply_v1(conn)?;
            conn.execute(
                "INSERT INTO schema_version (version, description) VALUES (?1, ?2)",
                rusqlite::params![CURRENT_SCHEMA_VERSION, "Initial schema + Anthropic official fields"],
            )?;
        }
        Some(v) if v == CURRENT_SCHEMA_VERSION => {}
        Some(v) if v < CURRENT_SCHEMA_VERSION => {
            migrate(conn, v)?;
        }
        Some(v) => {
            return Err(RegistryError::VersionMismatch {
                expected: CURRENT_SCHEMA_VERSION,
                found: v,
            });
        }
    }

    Ok(())
}

/// Run incremental migrations from `from_version` to `CURRENT_SCHEMA_VERSION`.
fn migrate(conn: &Connection, from_version: u32) -> Result<()> {
    if from_version < 2 {
        apply_v2_columns(conn)?;
    }
    if from_version < 3 {
        apply_v3_columns(conn)?;
    }
    if from_version < 4 {
        apply_v4_columns(conn)?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version, description) VALUES (?1, ?2)",
        rusqlite::params![
            CURRENT_SCHEMA_VERSION,
            "V4: SMST component breakdown + assessment tracking"
        ],
    )?;
    Ok(())
}

/// V2: Add 7 Anthropic official frontmatter columns and fix `user_invocable` default.
///
/// Official spec: <https://code.claude.com/docs/en/skills>
fn apply_v2_columns(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE active_skills ADD COLUMN argument_hint TEXT;
        ALTER TABLE active_skills ADD COLUMN disable_model_invocation INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE active_skills ADD COLUMN allowed_tools TEXT;
        ALTER TABLE active_skills ADD COLUMN model TEXT;
        ALTER TABLE active_skills ADD COLUMN context TEXT;
        ALTER TABLE active_skills ADD COLUMN agent TEXT;
        ALTER TABLE active_skills ADD COLUMN hooks TEXT;

        -- Fix: official default for user_invocable is true (1).
        -- Old scanner defaulted to false (0). Reset all to 1;
        -- next re-scan applies correct values from frontmatter.
        UPDATE active_skills SET user_invocable = 1;
        ",
    )?;
    Ok(())
}

/// V3: Add runtime feature detection columns.
///
/// Scanned from SKILL.md content body (not frontmatter):
/// - `$ARGUMENTS`, `$N`, `$ARGUMENTS[N]` → uses_arguments
/// - `` !`command` `` → uses_dynamic_context
/// - `${CLAUDE_SESSION_ID}` → uses_session_id
/// - Total character count → content_chars (for 2% budget tracking)
fn apply_v3_columns(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE active_skills ADD COLUMN uses_arguments INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE active_skills ADD COLUMN uses_dynamic_context INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE active_skills ADD COLUMN uses_session_id INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE active_skills ADD COLUMN content_chars INTEGER;
        ",
    )?;
    Ok(())
}

/// V4: Add SMST component breakdown columns and assessment tracking.
///
/// Stores per-component SMST v2 scores so compliance can be recomputed
/// without re-reading filesystem. Also tracks when/who last assessed.
fn apply_v4_columns(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        ALTER TABLE active_skills ADD COLUMN smst_input INTEGER;
        ALTER TABLE active_skills ADD COLUMN smst_output INTEGER;
        ALTER TABLE active_skills ADD COLUMN smst_logic INTEGER;
        ALTER TABLE active_skills ADD COLUMN smst_error_handling INTEGER;
        ALTER TABLE active_skills ADD COLUMN smst_examples INTEGER;
        ALTER TABLE active_skills ADD COLUMN smst_references INTEGER;
        ALTER TABLE active_skills ADD COLUMN last_assessed_at TEXT;
        ALTER TABLE active_skills ADD COLUMN assessed_by TEXT;
        ",
    )?;
    Ok(())
}

/// Apply the V1 schema — all 17 tables from `skills.db`.
fn apply_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Reference: frontmatter field definitions
        CREATE TABLE IF NOT EXISTS ref_frontmatter_fields (
            field           TEXT PRIMARY KEY,
            required        INTEGER NOT NULL DEFAULT 0,
            type            TEXT NOT NULL,
            default_value   TEXT,
            description     TEXT NOT NULL
        );

        -- Reference: invocation mode configurations
        CREATE TABLE IF NOT EXISTS ref_invocation_modes (
            configuration       TEXT PRIMARY KEY,
            user_can_invoke     INTEGER NOT NULL DEFAULT 1,
            claude_can_invoke   INTEGER NOT NULL DEFAULT 1,
            context_loading     TEXT NOT NULL
        );

        -- Reference: content type definitions
        CREATE TABLE IF NOT EXISTS ref_content_types (
            type            TEXT PRIMARY KEY,
            purpose         TEXT NOT NULL,
            invocation      TEXT NOT NULL,
            example         TEXT NOT NULL
        );

        -- Reference: context budget properties
        CREATE TABLE IF NOT EXISTS ref_context_budget (
            property        TEXT PRIMARY KEY,
            value           TEXT NOT NULL,
            description     TEXT NOT NULL
        );

        -- Reference: scope priority definitions
        CREATE TABLE IF NOT EXISTS ref_scopes (
            priority        INTEGER PRIMARY KEY,
            scope           TEXT NOT NULL UNIQUE,
            location        TEXT NOT NULL,
            applies_to      TEXT NOT NULL
        );

        -- Reference: substitution variable definitions
        CREATE TABLE IF NOT EXISTS ref_substitutions (
            variable        TEXT PRIMARY KEY,
            description     TEXT NOT NULL,
            example         TEXT NOT NULL
        );

        -- Reference: troubleshooting entries
        CREATE TABLE IF NOT EXISTS ref_troubleshooting (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            symptom         TEXT NOT NULL,
            cause           TEXT NOT NULL,
            fix             TEXT NOT NULL
        );

        -- Reference: subagent definitions
        CREATE TABLE IF NOT EXISTS ref_subagents (
            name            TEXT PRIMARY KEY,
            model           TEXT NOT NULL,
            tools           TEXT,
            purpose         TEXT NOT NULL,
            builtin         INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_ref_subagents_builtin ON ref_subagents(builtin);

        -- Live registry: active skills
        CREATE TABLE IF NOT EXISTS active_skills (
            name            TEXT PRIMARY KEY,
            path            TEXT NOT NULL,
            description     TEXT,
            -- Anthropic official frontmatter fields
            argument_hint             TEXT,
            disable_model_invocation  INTEGER NOT NULL DEFAULT 0,
            user_invocable            INTEGER NOT NULL DEFAULT 1,
            allowed_tools             TEXT,
            model                     TEXT,
            context                   TEXT,
            agent                     TEXT,
            hooks                     TEXT,
            -- Computed / operational fields
            line_count      INTEGER,
            has_agent       INTEGER NOT NULL DEFAULT 0,
            sub_skill_count INTEGER NOT NULL DEFAULT 0,
            parent_skill    TEXT REFERENCES active_skills(name),
            -- Runtime feature detection (scanned from content body)
            uses_arguments          INTEGER NOT NULL DEFAULT 0,
            uses_dynamic_context    INTEGER NOT NULL DEFAULT 0,
            uses_session_id         INTEGER NOT NULL DEFAULT 0,
            content_chars           INTEGER,
            -- SMST v2 component breakdown (assessment cache)
            smst_input              INTEGER,
            smst_output             INTEGER,
            smst_logic              INTEGER,
            smst_error_handling     INTEGER,
            smst_examples           INTEGER,
            smst_references         INTEGER,
            last_assessed_at        TEXT,
            assessed_by             TEXT,
            -- NexVigilant KPI tracking fields
            version         TEXT,
            compliance      TEXT CHECK(compliance IN (
                'Invalid','Bronze','Silver','Gold','Platinum','Diamond'
            )),
            smst_v1         REAL,
            smst_v2         INTEGER,
            tags            TEXT,
            chain_position  TEXT,
            pipeline        TEXT,
            -- Timestamps
            scanned_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_skills_compliance ON active_skills(compliance);
        CREATE INDEX IF NOT EXISTS idx_skills_parent ON active_skills(parent_skill);

        -- Live registry: active agents
        CREATE TABLE IF NOT EXISTS active_agents (
            name            TEXT PRIMARY KEY,
            path            TEXT NOT NULL,
            model           TEXT,
            tools           TEXT,
            description     TEXT,
            paired_skill    TEXT REFERENCES active_skills(name),
            scanned_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_agents_skill ON active_agents(paired_skill);

        -- Metrics: per-skill metric counters/gauges
        CREATE TABLE IF NOT EXISTS skill_metrics (
            source          TEXT NOT NULL,
            metric          TEXT NOT NULL,
            type            TEXT NOT NULL CHECK(type IN ('counter','gauge','histogram')),
            value           REAL NOT NULL DEFAULT 0,
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            PRIMARY KEY (source, metric)
        );
        CREATE INDEX IF NOT EXISTS idx_metrics_source ON skill_metrics(source);

        -- Metrics: individual invocation records
        CREATE TABLE IF NOT EXISTS skill_invocations (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            skill_name      TEXT NOT NULL,
            trigger_type    TEXT CHECK(trigger_type IN ('slash','auto','explicit','agent')),
            session_id      TEXT,
            duration_ms     INTEGER,
            invoked_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_invocations_skill ON skill_invocations(skill_name);
        CREATE INDEX IF NOT EXISTS idx_invocations_time ON skill_invocations(invoked_at);

        -- Goals: SMART goals per skill
        CREATE TABLE IF NOT EXISTS skill_goals (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            skill_name      TEXT NOT NULL,
            name            TEXT NOT NULL,
            specific        TEXT NOT NULL,
            measurable      TEXT NOT NULL,
            achievable      TEXT NOT NULL,
            relevant        TEXT NOT NULL,
            time_bound      TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'active'
                            CHECK(status IN ('active','achieved','abandoned','deferred')),
            current_value   REAL,
            target_value    REAL,
            unit            TEXT,
            created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_goals_skill ON skill_goals(skill_name);
        CREATE INDEX IF NOT EXISTS idx_goals_status ON skill_goals(status);

        -- KPIs: ecosystem-level key performance indicators
        CREATE TABLE IF NOT EXISTS skill_kpis (
            name            TEXT PRIMARY KEY,
            description     TEXT NOT NULL,
            formula         TEXT,
            current_value   REAL,
            target_value    REAL,
            unit            TEXT,
            direction       TEXT CHECK(direction IN ('higher_better','lower_better','target')),
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );

        -- Audit: action log
        CREATE TABLE IF NOT EXISTS audit_log (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            skill_name      TEXT NOT NULL,
            action          TEXT NOT NULL,
            details         TEXT,
            actor           TEXT,
            created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
        );
        CREATE INDEX IF NOT EXISTS idx_audit_skill ON audit_log(skill_name);
        CREATE INDEX IF NOT EXISTS idx_audit_time ON audit_log(created_at);
        ",
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_fresh_db() {
        let conn = Connection::open_in_memory().ok();
        assert!(conn.is_some());
        let conn = conn.unwrap_or_else(|| unreachable!());
        let result = initialize(&conn);
        assert!(result.is_ok());

        let version: u32 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_initialize_idempotent() {
        let conn = Connection::open_in_memory().ok();
        assert!(conn.is_some());
        let conn = conn.unwrap_or_else(|| unreachable!());
        let r1 = initialize(&conn);
        assert!(r1.is_ok());
        let r2 = initialize(&conn);
        assert!(r2.is_ok());
    }

    #[test]
    fn test_all_tables_exist() {
        let conn = Connection::open_in_memory().ok();
        assert!(conn.is_some());
        let conn = conn.unwrap_or_else(|| unreachable!());
        initialize(&conn).ok();

        let tables = [
            "ref_frontmatter_fields",
            "ref_invocation_modes",
            "ref_content_types",
            "ref_context_budget",
            "ref_scopes",
            "ref_substitutions",
            "ref_troubleshooting",
            "ref_subagents",
            "active_skills",
            "active_agents",
            "skill_metrics",
            "skill_invocations",
            "skill_goals",
            "skill_kpis",
            "audit_log",
            "schema_version",
        ];

        for table in tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            assert!(exists, "Table '{table}' should exist");
        }
    }
}
