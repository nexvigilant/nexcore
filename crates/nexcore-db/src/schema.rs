//! Database schema definition and migration.
//!
//! Schema version is tracked in a `schema_version` table.
//! Each version bump adds a migration function.

use rusqlite::Connection;

use crate::error::{DbError, Result};

/// Current schema version. Increment when adding migrations.
pub const CURRENT_SCHEMA_VERSION: u32 = 7;

/// Initialize the database schema (create all tables if they don't exist).
///
/// # Errors
///
/// Returns an error if tables cannot be created.
pub fn initialize(conn: &Connection) -> Result<()> {
    // Enable WAL mode for concurrent read access
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Schema versioning table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL
        );",
    )?;

    let current: Option<u32> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    match current {
        None => {
            // Fresh database — apply full schema (all versions)
            apply_v1(conn)?;
            apply_v2(conn)?;
            // V3 is a dedup migration — no new tables, skip on fresh install
            apply_v4(conn)?;
            apply_v5(conn)?;
            apply_v6(conn)?;
            apply_v7(conn)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [CURRENT_SCHEMA_VERSION],
            )?;
        }
        Some(v) if v == CURRENT_SCHEMA_VERSION => {
            // Already up to date
        }
        Some(v) if v < CURRENT_SCHEMA_VERSION => {
            // Run incremental migrations
            migrate(conn, v)?;
        }
        Some(v) => {
            return Err(DbError::VersionMismatch {
                expected: CURRENT_SCHEMA_VERSION,
                found: v,
            });
        }
    }

    Ok(())
}

/// Apply the V1 schema (initial full creation).
fn apply_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Brain sessions
        CREATE TABLE IF NOT EXISTS sessions (
            id              TEXT PRIMARY KEY,
            project         TEXT NOT NULL DEFAULT '',
            git_commit      TEXT,
            description     TEXT NOT NULL DEFAULT '',
            created_at      TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project);
        CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);

        -- Session artifacts (mutable current state)
        CREATE TABLE IF NOT EXISTS artifacts (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            name            TEXT NOT NULL,
            artifact_type   TEXT NOT NULL DEFAULT 'custom',
            content         TEXT NOT NULL DEFAULT '',
            summary         TEXT NOT NULL DEFAULT '',
            current_version INTEGER NOT NULL DEFAULT 0,
            tags            TEXT NOT NULL DEFAULT '[]',
            custom_meta     TEXT NOT NULL DEFAULT 'null',
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            UNIQUE(session_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_artifacts_session ON artifacts(session_id);
        CREATE INDEX IF NOT EXISTS idx_artifacts_type ON artifacts(artifact_type);

        -- Immutable resolved artifact versions
        CREATE TABLE IF NOT EXISTS artifact_versions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id      TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            artifact_name   TEXT NOT NULL,
            version         INTEGER NOT NULL,
            content         TEXT NOT NULL,
            resolved_at     TEXT NOT NULL,
            UNIQUE(session_id, artifact_name, version)
        );

        -- Code tracker: content-addressable file snapshots
        CREATE TABLE IF NOT EXISTS tracked_files (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project         TEXT NOT NULL,
            file_path       TEXT NOT NULL,
            content_hash    TEXT NOT NULL,
            file_size       INTEGER NOT NULL DEFAULT 0,
            tracked_at      TEXT NOT NULL,
            mtime           TEXT NOT NULL,
            UNIQUE(project, file_path)
        );
        CREATE INDEX IF NOT EXISTS idx_tracked_project ON tracked_files(project);

        -- Implicit knowledge: preferences
        CREATE TABLE IF NOT EXISTS preferences (
            key             TEXT PRIMARY KEY,
            value           TEXT NOT NULL DEFAULT 'null',
            description     TEXT,
            confidence      REAL NOT NULL DEFAULT 0.5,
            reinforcement_count INTEGER NOT NULL DEFAULT 1,
            updated_at      TEXT NOT NULL
        );

        -- Implicit knowledge: patterns
        CREATE TABLE IF NOT EXISTS patterns (
            id              TEXT PRIMARY KEY,
            pattern_type    TEXT NOT NULL,
            description     TEXT NOT NULL DEFAULT '',
            examples        TEXT NOT NULL DEFAULT '[]',
            detected_at     TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            confidence      REAL NOT NULL DEFAULT 0.5,
            occurrence_count INTEGER NOT NULL DEFAULT 1,
            t1_grounding    TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_patterns_type ON patterns(pattern_type);
        CREATE INDEX IF NOT EXISTS idx_patterns_grounding ON patterns(t1_grounding);

        -- Implicit knowledge: corrections
        CREATE TABLE IF NOT EXISTS corrections (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            mistake         TEXT NOT NULL,
            correction      TEXT NOT NULL,
            context         TEXT,
            learned_at      TEXT NOT NULL,
            application_count INTEGER NOT NULL DEFAULT 0
        );

        -- Implicit knowledge: beliefs (PROJECT GROUNDED)
        CREATE TABLE IF NOT EXISTS beliefs (
            id              TEXT PRIMARY KEY,
            proposition     TEXT NOT NULL,
            category        TEXT NOT NULL DEFAULT '',
            confidence      REAL NOT NULL DEFAULT 0.5,
            evidence        TEXT NOT NULL DEFAULT '[]',
            t1_grounding    TEXT,
            formed_at       TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            validation_count INTEGER NOT NULL DEFAULT 0,
            user_confirmed  INTEGER NOT NULL DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_beliefs_category ON beliefs(category);
        CREATE INDEX IF NOT EXISTS idx_beliefs_grounding ON beliefs(t1_grounding);

        -- Trust accumulators per domain (PROJECT GROUNDED)
        CREATE TABLE IF NOT EXISTS trust_accumulators (
            domain          TEXT PRIMARY KEY,
            demonstrations  INTEGER NOT NULL DEFAULT 0,
            failures        INTEGER NOT NULL DEFAULT 0,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            t1_grounding    TEXT
        );

        -- Belief implication graph edges (PROJECT GROUNDED)
        CREATE TABLE IF NOT EXISTS belief_implications (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            from_belief     TEXT NOT NULL REFERENCES beliefs(id) ON DELETE CASCADE,
            to_belief       TEXT NOT NULL REFERENCES beliefs(id) ON DELETE CASCADE,
            strength        TEXT NOT NULL DEFAULT 'moderate',
            established_at  TEXT NOT NULL,
            UNIQUE(from_belief, to_belief)
        );
        CREATE INDEX IF NOT EXISTS idx_implications_from ON belief_implications(from_belief);
        CREATE INDEX IF NOT EXISTS idx_implications_to ON belief_implications(to_belief);
        ",
    )?;

    Ok(())
}

/// Run incremental migrations from `from_version` to `CURRENT_SCHEMA_VERSION`.
fn migrate(conn: &Connection, from_version: u32) -> Result<()> {
    if from_version < 2 {
        apply_v2(conn)?;
    }
    if from_version < 3 {
        apply_v3(conn)?;
    }
    if from_version < 4 {
        apply_v4(conn)?;
    }
    if from_version < 5 {
        apply_v5(conn)?;
    }
    if from_version < 6 {
        apply_v6(conn)?;
    }
    if from_version < 7 {
        apply_v7(conn)?;
    }

    conn.execute(
        "UPDATE schema_version SET version = ?1",
        [CURRENT_SCHEMA_VERSION],
    )?;

    Ok(())
}

/// V2 schema: operational telemetry and accumulated knowledge tables.
///
/// New tables:
/// - `decision_audit` — tool usage decisions captured by hooks
/// - `tool_usage` — aggregated tool call statistics
/// - `token_efficiency` — per-session token efficiency metrics
/// - `tasks_history` — completed/pending task snapshots across sessions
/// - `handoffs` — session handoff summaries
/// - `antibodies` — antipattern immunity registry entries
fn apply_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Decision audit log (from hooks decision-journal)
        CREATE TABLE IF NOT EXISTS decision_audit (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp       TEXT NOT NULL,
            session_id      TEXT NOT NULL,
            tool            TEXT NOT NULL,
            action          TEXT NOT NULL DEFAULT '',
            target          TEXT NOT NULL DEFAULT '',
            risk_level      TEXT NOT NULL DEFAULT 'LOW',
            reversible      INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_decision_session ON decision_audit(session_id);
        CREATE INDEX IF NOT EXISTS idx_decision_tool ON decision_audit(tool);
        CREATE INDEX IF NOT EXISTS idx_decision_risk ON decision_audit(risk_level);
        CREATE INDEX IF NOT EXISTS idx_decision_ts ON decision_audit(timestamp);

        -- Tool usage telemetry (aggregated counts)
        CREATE TABLE IF NOT EXISTS tool_usage (
            tool_name       TEXT PRIMARY KEY,
            total_calls     INTEGER NOT NULL DEFAULT 0,
            success_count   INTEGER NOT NULL DEFAULT 0,
            failure_count   INTEGER NOT NULL DEFAULT 0,
            last_used       INTEGER NOT NULL DEFAULT 0
        );

        -- Token efficiency per session
        CREATE TABLE IF NOT EXISTS token_efficiency (
            session_id      TEXT PRIMARY KEY,
            action_count    INTEGER NOT NULL DEFAULT 0,
            total_tokens    INTEGER NOT NULL DEFAULT 0,
            started_at      INTEGER NOT NULL DEFAULT 0
        );

        -- Task history (snapshots from ~/.claude/tasks/)
        CREATE TABLE IF NOT EXISTS tasks_history (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id      TEXT NOT NULL,
            task_id         TEXT NOT NULL,
            subject         TEXT NOT NULL DEFAULT '',
            description     TEXT NOT NULL DEFAULT '',
            active_form     TEXT NOT NULL DEFAULT '',
            status          TEXT NOT NULL DEFAULT 'pending',
            blocks          TEXT NOT NULL DEFAULT '[]',
            blocked_by      TEXT NOT NULL DEFAULT '[]',
            UNIQUE(session_id, task_id)
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_session ON tasks_history(session_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks_history(status);

        -- Session handoff summaries
        CREATE TABLE IF NOT EXISTS handoffs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project         TEXT NOT NULL,
            handoff_number  INTEGER NOT NULL DEFAULT 0,
            session_id      TEXT NOT NULL DEFAULT '',
            generated_at    TEXT NOT NULL DEFAULT '',
            status          TEXT NOT NULL DEFAULT '',
            duration        TEXT NOT NULL DEFAULT '',
            files_modified  INTEGER NOT NULL DEFAULT 0,
            lines_written   INTEGER NOT NULL DEFAULT 0,
            commits         INTEGER NOT NULL DEFAULT 0,
            uncommitted     INTEGER NOT NULL DEFAULT 0,
            content         TEXT NOT NULL DEFAULT '',
            UNIQUE(project, handoff_number)
        );
        CREATE INDEX IF NOT EXISTS idx_handoffs_project ON handoffs(project);
        CREATE INDEX IF NOT EXISTS idx_handoffs_session ON handoffs(session_id);

        -- Antipattern immunity antibodies
        CREATE TABLE IF NOT EXISTS antibodies (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            threat_type     TEXT NOT NULL DEFAULT 'DAMP',
            severity        TEXT NOT NULL DEFAULT 'medium',
            description     TEXT NOT NULL DEFAULT '',
            detection       TEXT NOT NULL DEFAULT '{}',
            response        TEXT NOT NULL DEFAULT '{}',
            confidence      REAL NOT NULL DEFAULT 0.5,
            applications    INTEGER NOT NULL DEFAULT 0,
            false_positives INTEGER NOT NULL DEFAULT 0,
            learned_from    TEXT NOT NULL DEFAULT '',
            t1_grounding    TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_antibodies_severity ON antibodies(severity);
        CREATE INDEX IF NOT EXISTS idx_antibodies_threat ON antibodies(threat_type);
        ",
    )?;

    Ok(())
}

/// V3 schema: deduplicate `decision_audit` and add UNIQUE constraint.
///
/// The V2 schema used plain `INSERT` for decisions, so `brain_db_sync`
/// created duplicates on every re-run. This migration:
/// 1. Deletes duplicate rows (keeps the lowest rowid per unique combo).
/// 2. Creates a UNIQUE index on `(timestamp, session_id, tool, target)`.
fn apply_v3(conn: &Connection) -> Result<()> {
    // Step 1: Remove duplicates, keeping the first occurrence (lowest rowid).
    conn.execute_batch(
        "DELETE FROM decision_audit
         WHERE rowid NOT IN (
             SELECT MIN(rowid)
             FROM decision_audit
             GROUP BY timestamp, session_id, tool, target
         );",
    )?;

    // Step 2: Add unique index to prevent future duplicates.
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_decision_audit_dedup
         ON decision_audit(timestamp, session_id, tool, target);",
    )?;

    Ok(())
}

/// V4 schema: Directive Autopsy Engine records.
///
/// New table:
/// - `autopsy_records` — structured post-mortem analysis of session quality,
///   linking PDP gate evaluations, directive identity, session outcomes, and
///   self-use compounding metrics.
fn apply_v4(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS autopsy_records (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id          TEXT    NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

            -- Directive identity (Directive Protocol §Directive Protocol)
            directive_id        TEXT,
            phase               TEXT,
            phase_type          TEXT,

            -- PDP Foundation Gate (CLAUDE.md §Foundation Gate: G1/G2/G3)
            g1_proposition      TEXT    NOT NULL DEFAULT 'not_evaluated',
            g2_specificity      TEXT    NOT NULL DEFAULT 'not_evaluated',
            g3_singularity      TEXT    NOT NULL DEFAULT 'not_evaluated',

            -- PDP Structural Gate (CLAUDE.md §Structural Gate: S1-S5)
            s1_badjective       INTEGER NOT NULL DEFAULT 0,
            s2_throat_clear     INTEGER NOT NULL DEFAULT 0,
            s3_hedging          INTEGER NOT NULL DEFAULT 0,
            s4_context          INTEGER NOT NULL DEFAULT 0,
            s5_output_spec      INTEGER NOT NULL DEFAULT 0,

            -- PDP Calibration Gate (CLAUDE.md §Calibration Gate: C1-C5)
            c1_eval_criteria    INTEGER NOT NULL DEFAULT 0,
            c2_outcome_focus    INTEGER NOT NULL DEFAULT 0,
            c3_abstraction      INTEGER NOT NULL DEFAULT 0,
            c4_decisive_ending  INTEGER NOT NULL DEFAULT 0,
            c5_sell_mode        INTEGER NOT NULL DEFAULT 0,

            -- Session Outcome (session-closeout-reflection rule §Output)
            outcome_verdict     TEXT,
            lesson_count        INTEGER NOT NULL DEFAULT 0,
            pattern_count       INTEGER NOT NULL DEFAULT 0,

            -- Lesson Root Causes (session-closeout-reflection rule §Lessons Learned)
            rc_pdp_proposition  INTEGER NOT NULL DEFAULT 0,
            rc_pdp_so_what      INTEGER NOT NULL DEFAULT 0,
            rc_pdp_why          INTEGER NOT NULL DEFAULT 0,
            rc_hook_gap         INTEGER NOT NULL DEFAULT 0,

            -- Quantitative Metrics (handoffs + tool_usage)
            tool_calls_total    INTEGER NOT NULL DEFAULT 0,
            mcp_calls           INTEGER NOT NULL DEFAULT 0,
            hook_blocks         INTEGER NOT NULL DEFAULT 0,
            files_modified      INTEGER NOT NULL DEFAULT 0,
            lines_written       INTEGER NOT NULL DEFAULT 0,
            commits             INTEGER NOT NULL DEFAULT 0,
            tokens_total        INTEGER NOT NULL DEFAULT 0,

            -- Self-Use Discipline (CLAUDE.md §Self-Use Discipline, CCIM)
            rho_session         REAL,
            tools_sovereign     INTEGER NOT NULL DEFAULT 0,
            tools_analysis      INTEGER NOT NULL DEFAULT 0,

            -- Density Score (compendious-machine MCP)
            reflection_cs       REAL,

            -- Prose References
            reflection_artifact TEXT,
            closeout_artifact   TEXT,

            -- Timestamps
            session_started_at  TEXT    NOT NULL,
            session_ended_at    TEXT,
            autopsied_at        TEXT    NOT NULL,

            UNIQUE(session_id)
        );
        CREATE INDEX IF NOT EXISTS idx_autopsy_directive ON autopsy_records(directive_id);
        CREATE INDEX IF NOT EXISTS idx_autopsy_verdict ON autopsy_records(outcome_verdict);
        CREATE INDEX IF NOT EXISTS idx_autopsy_date ON autopsy_records(autopsied_at);
        CREATE INDEX IF NOT EXISTS idx_autopsy_phase ON autopsy_records(phase_type);
        ",
    )?;

    Ok(())
}

/// V5 schema: extended autopsy metrics + skill proposals + test history.
///
/// Changes:
/// - `autopsy_records` gains 9 columns for PDP prompt quality metrics
/// - `skill_proposals` — tracks pattern→skill promotion pipeline
/// - `test_runs` — per-crate test execution history for flake detection
fn apply_v5(conn: &Connection) -> Result<()> {
    // Extended autopsy columns (ALTER TABLE is idempotent-safe with IF NOT EXISTS
    // unavailable in SQLite ALTER, so we check column existence first)
    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(autopsy_records)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    let new_cols = [
        ("restart_count", "INTEGER NOT NULL DEFAULT 0"),
        ("clarification_count", "INTEGER NOT NULL DEFAULT 0"),
        ("deviation_count", "INTEGER NOT NULL DEFAULT 0"),
        ("ac_passed", "INTEGER NOT NULL DEFAULT 0"),
        ("ac_total", "INTEGER NOT NULL DEFAULT 0"),
        ("blocker_count", "INTEGER NOT NULL DEFAULT 0"),
        ("constraint_specificity", "REAL"),
        ("prior_art_refs", "INTEGER NOT NULL DEFAULT 0"),
        ("prompt_tokens", "INTEGER NOT NULL DEFAULT 0"),
    ];

    for (name, typ) in new_cols {
        if !cols.iter().any(|c| c == name) {
            conn.execute_batch(&format!(
                "ALTER TABLE autopsy_records ADD COLUMN {name} {typ};"
            ))?;
        }
    }

    // Skill proposals table
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS skill_proposals (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern_id          INTEGER,
            proposed_name       TEXT NOT NULL,
            proposed_description TEXT,
            source_pattern      TEXT,
            confidence          REAL DEFAULT 0.0,
            status              TEXT DEFAULT 'pending',
            created_at          TEXT,
            promoted_at         TEXT,
            skill_path          TEXT,
            FOREIGN KEY (pattern_id) REFERENCES patterns(id)
        );

        -- Test run history for flake detection
        CREATE TABLE IF NOT EXISTS test_runs (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            run_at      TEXT NOT NULL DEFAULT (datetime('now')),
            session_id  TEXT,
            crate_name  TEXT NOT NULL,
            runner      TEXT NOT NULL DEFAULT 'cargo-test',
            passed      INTEGER NOT NULL DEFAULT 0,
            failed      INTEGER NOT NULL DEFAULT 0,
            ignored     INTEGER NOT NULL DEFAULT 0,
            duration_s  REAL NOT NULL DEFAULT 0,
            fail_names  TEXT NOT NULL DEFAULT '[]'
        );
        ",
    )?;

    Ok(())
}

/// V6 schema: Guardian observability — health snapshots for continuous monitoring.
///
/// New table:
/// - `health_snapshots` — periodic system health readings from the Guardian observer daemon
fn apply_v6(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS health_snapshots (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            captured_at     TEXT NOT NULL DEFAULT (datetime('now')),
            session_velocity    REAL NOT NULL DEFAULT 0.0,
            mcp_backend_health  REAL NOT NULL DEFAULT 1.0,
            microgram_integrity REAL NOT NULL DEFAULT 1.0,
            station_activity    REAL NOT NULL DEFAULT 0.0,
            guardian_threat     TEXT NOT NULL DEFAULT 'Low',
            artifact_freshness  REAL NOT NULL DEFAULT 1.0,
            hook_error_rate     REAL NOT NULL DEFAULT 0.0,
            composite_score     REAL NOT NULL DEFAULT 1.0,
            alerts_json         TEXT NOT NULL DEFAULT '[]',
            vitals_json         TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_health_snapshots_captured
            ON health_snapshots(captured_at);
        ",
    )?;

    Ok(())
}

/// V7 schema: Discharge tracking — when stored knowledge was last surfaced to a session.
///
/// Adds `last_surfaced_at TEXT` to 5 tables that accumulate knowledge without
/// evidence of retrieval. This column enables measuring the discharge rate:
/// what fraction of stored knowledge actually flows back into session behavior.
///
/// Tables modified: antibodies, corrections, patterns, autopsy_records, beliefs.
fn apply_v7(conn: &Connection) -> Result<()> {
    // Use PRAGMA table_info to check column existence (ALTER TABLE has no IF NOT EXISTS)
    let tables_and_cols: [(&str, &str); 5] = [
        ("antibodies", "last_surfaced_at"),
        ("corrections", "last_surfaced_at"),
        ("patterns", "last_surfaced_at"),
        ("autopsy_records", "last_surfaced_at"),
        ("beliefs", "last_surfaced_at"),
    ];

    for (table, col) in tables_and_cols {
        let cols: Vec<String> = conn
            .prepare(&format!("PRAGMA table_info({table})"))?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .collect();

        if !cols.iter().any(|c| c == col) {
            conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {col} TEXT;"))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_fresh_db() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        let result = initialize(&conn);
        assert!(result.is_ok());

        // Verify schema version
        let version: u32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .expect("query version");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn test_initialize_idempotent() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        initialize(&conn).expect("first init");
        initialize(&conn).expect("second init should be idempotent");
    }

    #[test]
    fn test_migrate_v1_to_current() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        // Manually apply V1 only
        conn.execute_batch("PRAGMA journal_mode=WAL;").expect("wal");
        conn.execute_batch("PRAGMA foreign_keys=ON;").expect("fk");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")
            .expect("sv");
        apply_v1(&conn).expect("v1");
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])
            .expect("insert v1");

        // Now run full initialize — should migrate through V2 and V3
        initialize(&conn).expect("migrate to current");

        let version: u32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .expect("version");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        // V2 tables should exist
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='decision_audit'",
                [],
                |row| row.get(0),
            )
            .expect("check");
        assert!(exists, "decision_audit table should exist after migration");

        // V3 dedup index should exist
        let idx_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_decision_audit_dedup'",
                [],
                |row| row.get(0),
            )
            .expect("check dedup index");
        assert!(idx_exists, "dedup index should exist after V3 migration");
    }

    #[test]
    fn test_v3_dedup_migration() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        // Set up V2 schema
        conn.execute_batch("PRAGMA journal_mode=WAL;").expect("wal");
        conn.execute_batch("PRAGMA foreign_keys=ON;").expect("fk");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")
            .expect("sv");
        apply_v1(&conn).expect("v1");
        apply_v2(&conn).expect("v2");
        conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])
            .expect("insert v2");

        // Insert 3 duplicate rows
        for _ in 0..3 {
            conn.execute(
                "INSERT INTO decision_audit (timestamp, session_id, tool, action, target, risk_level, reversible)
                 VALUES ('2025-01-01T00:00:00Z', 'sess-1', 'Edit', 'structural', '/foo/lib.rs', 'LOW', 1)",
                [],
            )
            .expect("insert dup");
        }
        // And one unique row
        conn.execute(
            "INSERT INTO decision_audit (timestamp, session_id, tool, action, target, risk_level, reversible)
             VALUES ('2025-01-01T00:00:01Z', 'sess-1', 'Write', 'dependency', '/foo/Cargo.toml', 'MEDIUM', 1)",
            [],
        )
        .expect("insert unique");

        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_audit", [], |row| row.get(0))
            .expect("count before");
        assert_eq!(before, 4); // 3 dupes + 1 unique

        // Run V3 migration
        apply_v3(&conn).expect("v3 migration");

        let after: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_audit", [], |row| row.get(0))
            .expect("count after");
        assert_eq!(after, 2); // 1 deduped + 1 unique

        // Verify that inserting a duplicate is now silently ignored
        let inserted = conn.execute(
            "INSERT OR IGNORE INTO decision_audit (timestamp, session_id, tool, action, target, risk_level, reversible)
             VALUES ('2025-01-01T00:00:00Z', 'sess-1', 'Edit', 'structural', '/foo/lib.rs', 'LOW', 1)",
            [],
        )
        .expect("insert or ignore");
        assert_eq!(inserted, 0); // 0 rows changed = ignored

        let still: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_audit", [], |row| row.get(0))
            .expect("count still");
        assert_eq!(still, 2); // unchanged
    }

    #[test]
    fn test_v4_autopsy_migration() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        // Set up V3 schema manually
        conn.execute_batch("PRAGMA journal_mode=WAL;").expect("wal");
        conn.execute_batch("PRAGMA foreign_keys=ON;").expect("fk");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")
            .expect("sv");
        apply_v1(&conn).expect("v1");
        apply_v2(&conn).expect("v2");
        apply_v3(&conn).expect("v3");
        conn.execute("INSERT INTO schema_version (version) VALUES (3)", [])
            .expect("insert v3");

        // Run initialize — should migrate V3 → V4
        initialize(&conn).expect("migrate to v4");

        let version: u32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .expect("version");
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        // autopsy_records table should exist
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='autopsy_records'",
                [],
                |row| row.get(0),
            )
            .expect("check autopsy table");
        assert!(exists, "autopsy_records should exist after V4 migration");

        // All 4 indexes should exist
        for idx in [
            "idx_autopsy_directive",
            "idx_autopsy_verdict",
            "idx_autopsy_date",
            "idx_autopsy_phase",
        ] {
            let idx_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                    [idx],
                    |row| row.get(0),
                )
                .expect("check index");
            assert!(
                idx_exists,
                "Index '{}' should exist after V4 migration",
                idx
            );
        }

        // Insert a test autopsy record
        conn.execute(
            "INSERT INTO sessions (id, project, description, created_at)
             VALUES ('test-sess', 'test', 'test session', '2026-02-28T00:00:00Z')",
            [],
        )
        .expect("insert session");

        conn.execute(
            "INSERT INTO autopsy_records (session_id, g1_proposition, session_started_at, autopsied_at)
             VALUES ('test-sess', 'pass', '2026-02-28T00:00:00Z', '2026-02-28T01:00:00Z')",
            [],
        )
        .expect("insert autopsy");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM autopsy_records", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_v4_idempotent() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch("PRAGMA journal_mode=WAL;").expect("wal");
        conn.execute_batch("PRAGMA foreign_keys=ON;").expect("fk");
        apply_v1(&conn).expect("v1");
        apply_v2(&conn).expect("v2");

        // Apply V4 twice — should not error (CREATE TABLE IF NOT EXISTS)
        apply_v4(&conn).expect("v4 first");
        apply_v4(&conn).expect("v4 second should be idempotent");
    }

    #[test]
    fn test_autopsy_unique_session() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        initialize(&conn).expect("init");

        conn.execute(
            "INSERT INTO sessions (id, project, description, created_at)
             VALUES ('dup-test', 'test', 'test', '2026-02-28T00:00:00Z')",
            [],
        )
        .expect("insert session");

        conn.execute(
            "INSERT INTO autopsy_records (session_id, session_started_at, autopsied_at)
             VALUES ('dup-test', '2026-02-28T00:00:00Z', '2026-02-28T01:00:00Z')",
            [],
        )
        .expect("first autopsy");

        // Second insert for same session should fail (UNIQUE constraint)
        let dup = conn.execute(
            "INSERT INTO autopsy_records (session_id, session_started_at, autopsied_at)
             VALUES ('dup-test', '2026-02-28T00:00:00Z', '2026-02-28T02:00:00Z')",
            [],
        );
        assert!(dup.is_err(), "Duplicate session autopsy should be rejected");
    }

    #[test]
    fn test_tables_exist() {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        initialize(&conn).expect("init");

        let tables = [
            "sessions",
            "artifacts",
            "artifact_versions",
            "tracked_files",
            "preferences",
            "patterns",
            "corrections",
            "beliefs",
            "trust_accumulators",
            "belief_implications",
            // V2 tables
            "decision_audit",
            "tool_usage",
            "token_efficiency",
            "tasks_history",
            "handoffs",
            "antibodies",
            // V4 tables
            "autopsy_records",
        ];

        for table in tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .expect("check table");
            assert!(exists, "Table '{}' should exist", table);
        }
    }
}
