//! Directive Autopsy Engine record CRUD.
//!
//! Stores structured post-mortem analysis of session quality, linking PDP gate
//! evaluations, directive identity, session outcomes, and self-use compounding
//! metrics. One autopsy record per session (UNIQUE constraint on `session_id`).

use nexcore_chrono::DateTime;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::{DbError, Result};

/// An autopsy record row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopsyRow {
    /// Auto-increment ID (None for inserts).
    pub id: Option<i64>,
    /// Session ID (FK to sessions table).
    pub session_id: String,

    // -- Directive identity --
    /// Directive identifier (e.g., "D008", "VDAG-CONSERVATION").
    pub directive_id: Option<String>,
    /// Phase label (e.g., "Phase 1", "Phase 2").
    pub phase: Option<String>,
    /// Phase type: "audit", "design", "implementation", "migration".
    pub phase_type: Option<String>,

    // -- PDP Foundation Gate (G1/G2/G3) --
    /// G1 Proposition: "pass", "fail", or "not_evaluated".
    pub g1_proposition: String,
    /// G2 Specificity: "pass", "fail", or "not_evaluated".
    pub g2_specificity: String,
    /// G3 Singularity: "pass", "fail", or "not_evaluated".
    pub g3_singularity: String,

    // -- PDP Structural Gate (S1-S5) --
    /// S1: Count of bare qualifier (badjective) instances.
    pub s1_badjective: i64,
    /// S2: Count of throat-clearing preamble instances.
    pub s2_throat_clear: i64,
    /// S3: Count of hedging language instances.
    pub s3_hedging: i64,
    /// S4: 1 if structural context markers present, 0 if absent.
    pub s4_context: i64,
    /// S5: 1 if deliverable output spec present, 0 if absent.
    pub s5_output_spec: i64,

    // -- PDP Calibration Gate (C1-C5) --
    /// C1: 1 if evaluation criteria transferred, 0 if advisory.
    pub c1_eval_criteria: i64,
    /// C2: 1 if outcome constraints present, 0 if advisory.
    pub c2_outcome_focus: i64,
    /// C3: 1 if abstraction level consistent, 0 if advisory.
    pub c3_abstraction: i64,
    /// C4: 1 if decisive ending, 0 if advisory.
    pub c4_decisive_ending: i64,
    /// C5: 0 if no sell mode detected, 1 if detected (inverse — sell mode is bad).
    pub c5_sell_mode: i64,

    // -- Session Outcome --
    /// Verdict: "fully_demonstrated", "partially_demonstrated", "not_demonstrated".
    pub outcome_verdict: Option<String>,
    /// Count of lessons learned.
    pub lesson_count: i64,
    /// Count of identified patterns.
    pub pattern_count: i64,

    // -- Lesson Root Causes --
    /// Lessons tracing to PDP Proposition failure.
    pub rc_pdp_proposition: i64,
    /// Lessons tracing to "So What?" unanswered.
    pub rc_pdp_so_what: i64,
    /// Lessons tracing to "Why?" missing.
    pub rc_pdp_why: i64,
    /// Lessons tracing to hook/tool/infrastructure gap.
    pub rc_hook_gap: i64,

    // -- Quantitative Metrics --
    /// Total tool calls in the session.
    pub tool_calls_total: i64,
    /// MCP tool calls specifically.
    pub mcp_calls: i64,
    /// Times hooks blocked tool use.
    pub hook_blocks: i64,
    /// Files modified count.
    pub files_modified: i64,
    /// Lines written count.
    pub lines_written: i64,
    /// Commits made.
    pub commits: i64,
    /// Total tokens consumed.
    pub tokens_total: i64,

    // -- Self-Use Discipline --
    /// Compounding ratio: sovereign invocations / total analysis tasks.
    pub rho_session: Option<f64>,
    /// Count of sovereign tool uses.
    pub tools_sovereign: i64,
    /// Count of total analysis tasks.
    pub tools_analysis: i64,

    // -- Density Score --
    /// Compendious Score of the session reflection artifact.
    pub reflection_cs: Option<f64>,

    // -- Prose References --
    /// Artifact name for the session reflection (e.g., "session-reflection.md").
    pub reflection_artifact: Option<String>,
    /// Artifact name for chain closeout (multi-phase sessions).
    pub closeout_artifact: Option<String>,

    // -- Timestamps --
    /// When the session started (RFC 3339).
    pub session_started_at: String,
    /// When the session ended (RFC 3339, optional).
    pub session_ended_at: Option<String>,
    /// When the autopsy was performed (RFC 3339).
    pub autopsied_at: String,
}

impl Default for AutopsyRow {
    fn default() -> Self {
        let now = DateTime::now().to_rfc3339();
        Self {
            id: None,
            session_id: String::new(),
            directive_id: None,
            phase: None,
            phase_type: None,
            g1_proposition: "not_evaluated".into(),
            g2_specificity: "not_evaluated".into(),
            g3_singularity: "not_evaluated".into(),
            s1_badjective: 0,
            s2_throat_clear: 0,
            s3_hedging: 0,
            s4_context: 0,
            s5_output_spec: 0,
            c1_eval_criteria: 0,
            c2_outcome_focus: 0,
            c3_abstraction: 0,
            c4_decisive_ending: 0,
            c5_sell_mode: 0,
            outcome_verdict: None,
            lesson_count: 0,
            pattern_count: 0,
            rc_pdp_proposition: 0,
            rc_pdp_so_what: 0,
            rc_pdp_why: 0,
            rc_hook_gap: 0,
            tool_calls_total: 0,
            mcp_calls: 0,
            hook_blocks: 0,
            files_modified: 0,
            lines_written: 0,
            commits: 0,
            tokens_total: 0,
            rho_session: None,
            tools_sovereign: 0,
            tools_analysis: 0,
            reflection_cs: None,
            reflection_artifact: None,
            closeout_artifact: None,
            session_started_at: now.clone(),
            session_ended_at: None,
            autopsied_at: now,
        }
    }
}

/// Insert a new autopsy record.
///
/// Uses INSERT OR IGNORE to respect the UNIQUE(session_id) constraint.
/// Returns the number of rows inserted (0 if duplicate, 1 if new).
///
/// # Errors
///
/// Returns an error if the insert fails for reasons other than uniqueness.
pub fn insert_autopsy(conn: &Connection, row: &AutopsyRow) -> Result<usize> {
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO autopsy_records (
            session_id, directive_id, phase, phase_type,
            g1_proposition, g2_specificity, g3_singularity,
            s1_badjective, s2_throat_clear, s3_hedging, s4_context, s5_output_spec,
            c1_eval_criteria, c2_outcome_focus, c3_abstraction, c4_decisive_ending, c5_sell_mode,
            outcome_verdict, lesson_count, pattern_count,
            rc_pdp_proposition, rc_pdp_so_what, rc_pdp_why, rc_hook_gap,
            tool_calls_total, mcp_calls, hook_blocks, files_modified, lines_written, commits, tokens_total,
            rho_session, tools_sovereign, tools_analysis,
            reflection_cs,
            reflection_artifact, closeout_artifact,
            session_started_at, session_ended_at, autopsied_at
        ) VALUES (
            ?1, ?2, ?3, ?4,
            ?5, ?6, ?7,
            ?8, ?9, ?10, ?11, ?12,
            ?13, ?14, ?15, ?16, ?17,
            ?18, ?19, ?20,
            ?21, ?22, ?23, ?24,
            ?25, ?26, ?27, ?28, ?29, ?30, ?31,
            ?32, ?33, ?34,
            ?35,
            ?36, ?37,
            ?38, ?39, ?40
        )",
        params![
            row.session_id,
            row.directive_id,
            row.phase,
            row.phase_type,
            row.g1_proposition,
            row.g2_specificity,
            row.g3_singularity,
            row.s1_badjective,
            row.s2_throat_clear,
            row.s3_hedging,
            row.s4_context,
            row.s5_output_spec,
            row.c1_eval_criteria,
            row.c2_outcome_focus,
            row.c3_abstraction,
            row.c4_decisive_ending,
            row.c5_sell_mode,
            row.outcome_verdict,
            row.lesson_count,
            row.pattern_count,
            row.rc_pdp_proposition,
            row.rc_pdp_so_what,
            row.rc_pdp_why,
            row.rc_hook_gap,
            row.tool_calls_total,
            row.mcp_calls,
            row.hook_blocks,
            row.files_modified,
            row.lines_written,
            row.commits,
            row.tokens_total,
            row.rho_session,
            row.tools_sovereign,
            row.tools_analysis,
            row.reflection_cs,
            row.reflection_artifact,
            row.closeout_artifact,
            row.session_started_at,
            row.session_ended_at,
            row.autopsied_at,
        ],
    )?;
    Ok(inserted)
}

/// Get an autopsy record by session ID.
///
/// # Errors
///
/// Returns `DbError::NotFound` if no autopsy exists for the session.
pub fn get_by_session(conn: &Connection, session_id: &str) -> Result<AutopsyRow> {
    conn.query_row(
        "SELECT id, session_id, directive_id, phase, phase_type,
                g1_proposition, g2_specificity, g3_singularity,
                s1_badjective, s2_throat_clear, s3_hedging, s4_context, s5_output_spec,
                c1_eval_criteria, c2_outcome_focus, c3_abstraction, c4_decisive_ending, c5_sell_mode,
                outcome_verdict, lesson_count, pattern_count,
                rc_pdp_proposition, rc_pdp_so_what, rc_pdp_why, rc_hook_gap,
                tool_calls_total, mcp_calls, hook_blocks, files_modified, lines_written, commits, tokens_total,
                rho_session, tools_sovereign, tools_analysis,
                reflection_cs,
                reflection_artifact, closeout_artifact,
                session_started_at, session_ended_at, autopsied_at
         FROM autopsy_records WHERE session_id = ?1",
        [session_id],
        |row| {
            Ok(AutopsyRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                directive_id: row.get(2)?,
                phase: row.get(3)?,
                phase_type: row.get(4)?,
                g1_proposition: row.get(5)?,
                g2_specificity: row.get(6)?,
                g3_singularity: row.get(7)?,
                s1_badjective: row.get(8)?,
                s2_throat_clear: row.get(9)?,
                s3_hedging: row.get(10)?,
                s4_context: row.get(11)?,
                s5_output_spec: row.get(12)?,
                c1_eval_criteria: row.get(13)?,
                c2_outcome_focus: row.get(14)?,
                c3_abstraction: row.get(15)?,
                c4_decisive_ending: row.get(16)?,
                c5_sell_mode: row.get(17)?,
                outcome_verdict: row.get(18)?,
                lesson_count: row.get(19)?,
                pattern_count: row.get(20)?,
                rc_pdp_proposition: row.get(21)?,
                rc_pdp_so_what: row.get(22)?,
                rc_pdp_why: row.get(23)?,
                rc_hook_gap: row.get(24)?,
                tool_calls_total: row.get(25)?,
                mcp_calls: row.get(26)?,
                hook_blocks: row.get(27)?,
                files_modified: row.get(28)?,
                lines_written: row.get(29)?,
                commits: row.get(30)?,
                tokens_total: row.get(31)?,
                rho_session: row.get(32)?,
                tools_sovereign: row.get(33)?,
                tools_analysis: row.get(34)?,
                reflection_cs: row.get(35)?,
                reflection_artifact: row.get(36)?,
                closeout_artifact: row.get(37)?,
                session_started_at: row.get(38)?,
                session_ended_at: row.get(39)?,
                autopsied_at: row.get(40)?,
            })
        },
    )
    .map_err(|_| DbError::NotFound(format!("Autopsy for session '{session_id}'")))
}

/// List autopsy records for a directive, ordered by session start time.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_by_directive(conn: &Connection, directive_id: &str) -> Result<Vec<AutopsyRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, directive_id, phase, phase_type,
                g1_proposition, g2_specificity, g3_singularity,
                s1_badjective, s2_throat_clear, s3_hedging, s4_context, s5_output_spec,
                c1_eval_criteria, c2_outcome_focus, c3_abstraction, c4_decisive_ending, c5_sell_mode,
                outcome_verdict, lesson_count, pattern_count,
                rc_pdp_proposition, rc_pdp_so_what, rc_pdp_why, rc_hook_gap,
                tool_calls_total, mcp_calls, hook_blocks, files_modified, lines_written, commits, tokens_total,
                rho_session, tools_sovereign, tools_analysis,
                reflection_cs,
                reflection_artifact, closeout_artifact,
                session_started_at, session_ended_at, autopsied_at
         FROM autopsy_records WHERE directive_id = ?1
         ORDER BY session_started_at ASC",
    )?;
    let rows = stmt
        .query_map([directive_id], |row| {
            Ok(AutopsyRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                directive_id: row.get(2)?,
                phase: row.get(3)?,
                phase_type: row.get(4)?,
                g1_proposition: row.get(5)?,
                g2_specificity: row.get(6)?,
                g3_singularity: row.get(7)?,
                s1_badjective: row.get(8)?,
                s2_throat_clear: row.get(9)?,
                s3_hedging: row.get(10)?,
                s4_context: row.get(11)?,
                s5_output_spec: row.get(12)?,
                c1_eval_criteria: row.get(13)?,
                c2_outcome_focus: row.get(14)?,
                c3_abstraction: row.get(15)?,
                c4_decisive_ending: row.get(16)?,
                c5_sell_mode: row.get(17)?,
                outcome_verdict: row.get(18)?,
                lesson_count: row.get(19)?,
                pattern_count: row.get(20)?,
                rc_pdp_proposition: row.get(21)?,
                rc_pdp_so_what: row.get(22)?,
                rc_pdp_why: row.get(23)?,
                rc_hook_gap: row.get(24)?,
                tool_calls_total: row.get(25)?,
                mcp_calls: row.get(26)?,
                hook_blocks: row.get(27)?,
                files_modified: row.get(28)?,
                lines_written: row.get(29)?,
                commits: row.get(30)?,
                tokens_total: row.get(31)?,
                rho_session: row.get(32)?,
                tools_sovereign: row.get(33)?,
                tools_analysis: row.get(34)?,
                reflection_cs: row.get(35)?,
                reflection_artifact: row.get(36)?,
                closeout_artifact: row.get(37)?,
                session_started_at: row.get(38)?,
                session_ended_at: row.get(39)?,
                autopsied_at: row.get(40)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// List all autopsy records, ordered by session start time.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn list_all(conn: &Connection) -> Result<Vec<AutopsyRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, directive_id, phase, phase_type,
                g1_proposition, g2_specificity, g3_singularity,
                s1_badjective, s2_throat_clear, s3_hedging, s4_context, s5_output_spec,
                c1_eval_criteria, c2_outcome_focus, c3_abstraction, c4_decisive_ending, c5_sell_mode,
                outcome_verdict, lesson_count, pattern_count,
                rc_pdp_proposition, rc_pdp_so_what, rc_pdp_why, rc_hook_gap,
                tool_calls_total, mcp_calls, hook_blocks, files_modified, lines_written, commits, tokens_total,
                rho_session, tools_sovereign, tools_analysis,
                reflection_cs,
                reflection_artifact, closeout_artifact,
                session_started_at, session_ended_at, autopsied_at
         FROM autopsy_records ORDER BY session_started_at ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AutopsyRow {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                directive_id: row.get(2)?,
                phase: row.get(3)?,
                phase_type: row.get(4)?,
                g1_proposition: row.get(5)?,
                g2_specificity: row.get(6)?,
                g3_singularity: row.get(7)?,
                s1_badjective: row.get(8)?,
                s2_throat_clear: row.get(9)?,
                s3_hedging: row.get(10)?,
                s4_context: row.get(11)?,
                s5_output_spec: row.get(12)?,
                c1_eval_criteria: row.get(13)?,
                c2_outcome_focus: row.get(14)?,
                c3_abstraction: row.get(15)?,
                c4_decisive_ending: row.get(16)?,
                c5_sell_mode: row.get(17)?,
                outcome_verdict: row.get(18)?,
                lesson_count: row.get(19)?,
                pattern_count: row.get(20)?,
                rc_pdp_proposition: row.get(21)?,
                rc_pdp_so_what: row.get(22)?,
                rc_pdp_why: row.get(23)?,
                rc_hook_gap: row.get(24)?,
                tool_calls_total: row.get(25)?,
                mcp_calls: row.get(26)?,
                hook_blocks: row.get(27)?,
                files_modified: row.get(28)?,
                lines_written: row.get(29)?,
                commits: row.get(30)?,
                tokens_total: row.get(31)?,
                rho_session: row.get(32)?,
                tools_sovereign: row.get(33)?,
                tools_analysis: row.get(34)?,
                reflection_cs: row.get(35)?,
                reflection_artifact: row.get(36)?,
                closeout_artifact: row.get(37)?,
                session_started_at: row.get(38)?,
                session_ended_at: row.get(39)?,
                autopsied_at: row.get(40)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count total autopsy records.
///
/// # Errors
///
/// Returns an error on query failure.
pub fn count_autopsies(conn: &Connection) -> Result<i64> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM autopsy_records", [], |row| row.get(0))?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    fn make_test_autopsy(session_id: &str) -> AutopsyRow {
        AutopsyRow {
            session_id: session_id.into(),
            directive_id: Some("D008".into()),
            phase: Some("Phase 1".into()),
            phase_type: Some("audit".into()),
            g1_proposition: "pass".into(),
            g2_specificity: "pass".into(),
            g3_singularity: "fail".into(),
            s1_badjective: 2,
            s4_context: 1,
            s5_output_spec: 1,
            c1_eval_criteria: 1,
            c4_decisive_ending: 1,
            outcome_verdict: Some("partially_demonstrated".into()),
            lesson_count: 3,
            pattern_count: 2,
            rc_pdp_proposition: 1,
            rc_hook_gap: 2,
            tool_calls_total: 150,
            mcp_calls: 30,
            hook_blocks: 5,
            files_modified: 8,
            lines_written: 400,
            commits: 2,
            tokens_total: 50000,
            rho_session: Some(0.625),
            tools_sovereign: 10,
            tools_analysis: 16,
            reflection_cs: Some(0.73),
            reflection_artifact: Some("session-reflection.md".into()),
            session_started_at: "2026-02-28T00:00:00Z".into(),
            session_ended_at: Some("2026-02-28T04:00:00Z".into()),
            autopsied_at: "2026-02-28T04:05:00Z".into(),
            ..AutopsyRow::default()
        }
    }

    #[test]
    fn test_insert_and_get() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-autopsy-1', 'nexcore', 'test session', '2026-02-28T00:00:00Z')",
                [],
            )?;

            let row = make_test_autopsy("sess-autopsy-1");
            let inserted = insert_autopsy(conn, &row)?;
            assert_eq!(inserted, 1);

            let got = get_by_session(conn, "sess-autopsy-1")?;
            assert!(got.id.is_some());
            assert_eq!(got.session_id, "sess-autopsy-1");
            assert_eq!(got.directive_id.as_deref(), Some("D008"));
            assert_eq!(got.phase.as_deref(), Some("Phase 1"));
            assert_eq!(got.phase_type.as_deref(), Some("audit"));
            assert_eq!(got.g1_proposition, "pass");
            assert_eq!(got.g3_singularity, "fail");
            assert_eq!(got.s1_badjective, 2);
            assert_eq!(got.s4_context, 1);
            assert_eq!(
                got.outcome_verdict.as_deref(),
                Some("partially_demonstrated")
            );
            assert_eq!(got.lesson_count, 3);
            assert_eq!(got.rc_pdp_proposition, 1);
            assert_eq!(got.rc_hook_gap, 2);
            assert_eq!(got.tool_calls_total, 150);
            assert_eq!(got.mcp_calls, 30);

            let rho = got.rho_session.expect("rho should be Some");
            assert!((rho - 0.625).abs() < f64::EPSILON);

            let cs = got.reflection_cs.expect("cs should be Some");
            assert!((cs - 0.73).abs() < f64::EPSILON);

            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_duplicate_ignored() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-dup', 'test', 'dup test', '2026-02-28T00:00:00Z')",
                [],
            )?;

            let row = make_test_autopsy("sess-dup");
            let first = insert_autopsy(conn, &row)?;
            assert_eq!(first, 1);

            let second = insert_autopsy(conn, &row)?;
            assert_eq!(second, 0, "Duplicate should be silently ignored");

            assert_eq!(count_autopsies(conn)?, 1);
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_list_by_directive() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            for (id, started) in [
                ("sess-d008-p1", "2026-02-28T00:00:00Z"),
                ("sess-d008-p2", "2026-02-28T06:00:00Z"),
                ("sess-other", "2026-02-28T12:00:00Z"),
            ] {
                conn.execute(
                    "INSERT INTO sessions (id, project, description, created_at)
                     VALUES (?1, 'nexcore', 'test', ?2)",
                    params![id, started],
                )?;
            }

            let mut row1 = make_test_autopsy("sess-d008-p1");
            row1.phase = Some("Phase 1".into());
            insert_autopsy(conn, &row1)?;

            let mut row2 = make_test_autopsy("sess-d008-p2");
            row2.phase = Some("Phase 2".into());
            row2.session_started_at = "2026-02-28T06:00:00Z".into();
            insert_autopsy(conn, &row2)?;

            let mut row3 = make_test_autopsy("sess-other");
            row3.directive_id = Some("D009".into());
            insert_autopsy(conn, &row3)?;

            let d008 = list_by_directive(conn, "D008")?;
            assert_eq!(d008.len(), 2);
            assert_eq!(d008[0].phase.as_deref(), Some("Phase 1"));
            assert_eq!(d008[1].phase.as_deref(), Some("Phase 2"));

            let d009 = list_by_directive(conn, "D009")?;
            assert_eq!(d009.len(), 1);

            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_not_found() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            let result = get_by_session(conn, "nonexistent");
            assert!(result.is_err());
            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_count() {
        let db = DbPool::open_in_memory().expect("open");
        db.with_conn(|conn| {
            assert_eq!(count_autopsies(conn)?, 0);

            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-count', 'test', 'count test', '2026-02-28T00:00:00Z')",
                [],
            )?;
            insert_autopsy(conn, &make_test_autopsy("sess-count"))?;
            assert_eq!(count_autopsies(conn)?, 1);

            Ok(())
        })
        .expect("test");
    }
}
