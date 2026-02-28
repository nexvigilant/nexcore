//! Directive Autopsy Engine — structured post-mortem analysis of sessions.
//!
//! Two modes of operation:
//!
//! - **Retroactive**: Reads existing sessions, artifacts, handoffs, and telemetry
//!   from the database. Extracts what is available, defaults what is not.
//!   PDP gate results are marked `not_evaluated` for historical sessions.
//!
//! - **Prospective**: Reads enriched session-reflection.md artifacts (with YAML
//!   frontmatter) and PDP evaluation telemetry from `pdp_evaluations.jsonl`.
//!   Produces complete autopsy records with all 40 fields populated.
//!
//! Both modes use [`crate::autopsy::insert_autopsy`] with `INSERT OR IGNORE`
//! for idempotent operation.

use rusqlite::{Connection, params};
use serde::Deserialize;

use crate::autopsy::{self, AutopsyRow};
use crate::error::Result;
use crate::pdp_telemetry::{self, PdpGate};

// ── YAML Frontmatter Parser ───────────────────────────────────────────────

/// Parsed autopsy YAML frontmatter from a session-reflection.md artifact.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AutopsyFrontmatter {
    /// Directive identifier.
    #[serde(default)]
    pub directive_id: Option<String>,
    /// Phase label.
    #[serde(default)]
    pub phase: Option<String>,
    /// Phase type.
    #[serde(default)]
    pub phase_type: Option<String>,
    /// Session outcome verdict.
    #[serde(default)]
    pub outcome_verdict: Option<String>,
    /// Count of lessons learned.
    #[serde(default)]
    pub lesson_count: i64,
    /// Count of identified patterns.
    #[serde(default)]
    pub pattern_count: i64,
    /// Root cause breakdown.
    #[serde(default)]
    pub root_causes: RootCauses,
    /// Quantitative metrics.
    #[serde(default)]
    pub metrics: Metrics,
    /// Self-use compounding data.
    #[serde(default)]
    pub compounding: Compounding,
}

/// Root cause breakdown from lessons learned.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RootCauses {
    #[serde(default)]
    pub pdp_proposition: i64,
    #[serde(default)]
    pub pdp_so_what: i64,
    #[serde(default)]
    pub pdp_why: i64,
    #[serde(default)]
    pub hook_gap: i64,
}

/// Quantitative session metrics.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Metrics {
    #[serde(default)]
    pub tool_calls_total: i64,
    #[serde(default)]
    pub mcp_calls: i64,
    #[serde(default)]
    pub hook_blocks: i64,
    #[serde(default)]
    pub files_modified: i64,
    #[serde(default)]
    pub lines_written: i64,
    #[serde(default)]
    pub commits: i64,
    #[serde(default)]
    pub tokens_total: i64,
}

/// Self-use compounding metrics.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Compounding {
    #[serde(default)]
    pub rho_session: Option<f64>,
    #[serde(default)]
    pub tools_sovereign: i64,
    #[serde(default)]
    pub tools_analysis: i64,
}

/// Extract the `autopsy:` YAML block from markdown content with frontmatter.
///
/// Expects the format:
/// ```text
/// ---
/// autopsy:
///   field: value
///   ...
/// ---
/// (prose content)
/// ```
///
/// Returns `None` if no valid frontmatter or no `autopsy:` key found.
pub fn extract_frontmatter(content: &str) -> Option<AutopsyFrontmatter> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_open = &trimmed[3..];
    let close_idx = after_open.find("\n---")?;
    let yaml_block = &after_open[..close_idx];

    // Parse the outer YAML to get the autopsy key
    let outer: serde_json::Value = parse_yaml_to_json(yaml_block)?;
    let autopsy_val = outer.get("autopsy")?;

    // Deserialize the autopsy block
    serde_json::from_value(autopsy_val.clone()).ok()
}

/// Minimal YAML-subset parser that converts to serde_json::Value.
///
/// Handles the specific YAML structure used by autopsy frontmatter:
/// nested keys with string/number/null values. Does NOT handle full YAML spec.
/// This avoids adding a YAML parser dependency.
fn parse_yaml_to_json(yaml: &str) -> Option<serde_json::Value> {
    let mut root = serde_json::Map::new();
    let mut current_key: Option<String> = None;
    let mut current_obj = serde_json::Map::new();
    let mut sub_key: Option<String> = None;
    let mut sub_obj = serde_json::Map::new();

    for line in yaml.lines() {
        let stripped = line.trim_end();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        let trimmed = stripped.trim();

        if let Some((key, val)) = trimmed.split_once(':') {
            let key = key.trim();
            let val = val.trim();

            if indent == 0 {
                // Top-level key
                if let Some(sk) = sub_key.take() {
                    current_obj.insert(sk, serde_json::Value::Object(sub_obj.clone()));
                    sub_obj.clear();
                }
                if let Some(ck) = current_key.take() {
                    if current_obj.is_empty() {
                        // It was a leaf at level 0 — shouldn't happen in our format
                    } else {
                        root.insert(ck, serde_json::Value::Object(current_obj.clone()));
                        current_obj.clear();
                    }
                }
                current_key = Some(key.to_string());
                if !val.is_empty() {
                    root.insert(key.to_string(), parse_yaml_value(val));
                    current_key = None;
                }
            } else if indent <= 2 {
                // Second-level key (under autopsy:)
                if let Some(sk) = sub_key.take() {
                    current_obj.insert(sk, serde_json::Value::Object(sub_obj.clone()));
                    sub_obj.clear();
                }
                if val.is_empty() {
                    sub_key = Some(key.to_string());
                } else {
                    current_obj.insert(key.to_string(), parse_yaml_value(val));
                }
            } else {
                // Third-level key (under root_causes:, metrics:, compounding:)
                sub_obj.insert(key.to_string(), parse_yaml_value(val));
            }
        }
    }

    // Flush remaining state
    if let Some(sk) = sub_key.take() {
        current_obj.insert(sk, serde_json::Value::Object(sub_obj));
    }
    if let Some(ck) = current_key.take() {
        if current_obj.is_empty() {
            // Shouldn't happen
        } else {
            root.insert(ck, serde_json::Value::Object(current_obj));
        }
    }

    if root.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(root))
    }
}

/// Parse a YAML scalar value to JSON.
fn parse_yaml_value(s: &str) -> serde_json::Value {
    let s = s.trim();

    // Strip quotes
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        let inner = &s[1..s.len() - 1];
        return serde_json::Value::String(inner.to_string());
    }

    // null
    if s == "null" || s == "~" || s.is_empty() {
        return serde_json::Value::Null;
    }

    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return serde_json::Value::Number(n.into());
    }

    // Float
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(n);
        }
    }

    // Boolean
    if s == "true" {
        return serde_json::Value::Bool(true);
    }
    if s == "false" {
        return serde_json::Value::Bool(false);
    }

    // Default: string
    serde_json::Value::String(s.to_string())
}

// ── Reflection Prose Parser ───────────────────────────────────────────────

/// Extract outcome verdict from session reflection prose.
///
/// Looks for "fully demonstrated", "partially demonstrated", or
/// "not demonstrated" (case-insensitive) in the content.
pub fn parse_outcome_verdict(content: &str) -> Option<String> {
    let lower = content.to_lowercase();
    if lower.contains("fully demonstrated") {
        Some("fully_demonstrated".into())
    } else if lower.contains("partially demonstrated") {
        Some("partially_demonstrated".into())
    } else if lower.contains("not demonstrated") {
        Some("not_demonstrated".into())
    } else {
        None
    }
}

/// Count lesson entries in reflection prose.
///
/// Counts lines containing the pattern `When ... because ...` inside
/// blockquotes or quoted strings. Handles both straight and curly quotes.
pub fn count_lessons(content: &str) -> i64 {
    let mut count: i64 = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        // Strip leading `> ` for blockquotes
        let inner = if let Some(rest) = trimmed.strip_prefix("> ") {
            rest.trim()
        } else {
            trimmed
        };
        // Match "When ... because ..." pattern (lesson format)
        if inner.contains("When ") && inner.contains(" because ") {
            count = count.saturating_add(1);
        }
    }
    count
}

/// Count pattern entries in reflection prose.
///
/// Counts lines matching: `"This session revealed that [...]"`
pub fn count_patterns(content: &str) -> i64 {
    let mut count: i64 = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("This session revealed that")
            || trimmed.contains("this session revealed that")
        {
            count = count.saturating_add(1);
        }
    }
    count
}

// ── Handoff Metric Extraction ─────────────────────────────────────────────

/// Metrics extracted from handoff data (structured columns + content parsing).
#[derive(Debug, Default)]
pub struct HandoffMetrics {
    pub files_modified: i64,
    pub lines_written: i64,
    pub commits: i64,
    pub tool_calls_total: i64,
    pub mcp_calls: i64,
    pub hook_blocks: i64,
}

/// Extract quantitative metrics from handoff prose content.
///
/// Parses patterns like:
/// - "**Tool Usage:** 150 calls total"
/// - "Tool calls: 87"
/// - "150 tool calls"
/// - "MCP calls: 22" or "22 MCP"
/// - "Hook blocks: 3" or "3 hook blocks" or "blocked 3 times"
///
/// Returns extracted values; unmatched fields remain 0.
pub fn parse_handoff_content(content: &str) -> HandoffMetrics {
    let mut metrics = HandoffMetrics::default();

    for line in content.lines() {
        let lower = line.to_lowercase();

        // Tool calls total
        if metrics.tool_calls_total == 0 {
            if let Some(n) = extract_number_near(
                &lower,
                &["tool calls", "tool usage", "calls total", "total calls"],
            ) {
                metrics.tool_calls_total = n;
            }
        }

        // MCP calls
        if metrics.mcp_calls == 0 {
            if let Some(n) = extract_number_near(&lower, &["mcp calls", "mcp tool", "mcp:", " mcp"])
            {
                metrics.mcp_calls = n;
            }
        }

        // Hook blocks
        if metrics.hook_blocks == 0 {
            if let Some(n) = extract_number_near(&lower, &["hook block", "blocked", "hook denial"])
            {
                metrics.hook_blocks = n;
            }
        }

        // Files modified (fallback from prose if structured field is 0)
        if metrics.files_modified == 0 {
            if let Some(n) = extract_number_near(
                &lower,
                &["files modified", "files changed", "modified files"],
            ) {
                metrics.files_modified = n;
            }
        }

        // Lines written
        if metrics.lines_written == 0 {
            if let Some(n) =
                extract_number_near(&lower, &["lines written", "lines added", "lines of code"])
            {
                metrics.lines_written = n;
            }
        }

        // Commits
        if metrics.commits == 0 {
            if let Some(n) = extract_number_near(&lower, &["commit"]) {
                metrics.commits = n;
            }
        }
    }

    metrics
}

/// Extract the nearest number to any of the given keywords in a line.
///
/// When both a number before and after the keyword exist, returns the
/// one closest by character distance. This prevents cross-contamination
/// when multiple metrics appear on a single comma-separated line.
fn extract_number_near(line: &str, keywords: &[&str]) -> Option<i64> {
    for keyword in keywords {
        if let Some(pos) = line.find(keyword) {
            let before = &line[..pos];
            let after = &line[pos + keyword.len()..];

            let before_num = last_number_in(before);
            let after_num = first_number_in(after);

            match (before_num, after_num) {
                (Some(b), Some(a)) => {
                    // Pick the closer number by character distance
                    let before_dist = before
                        .chars()
                        .rev()
                        .position(|c| c.is_ascii_digit())
                        .unwrap_or(before.len());
                    let after_dist = after
                        .find(|c: char| c.is_ascii_digit())
                        .unwrap_or(after.len());
                    return Some(if before_dist <= after_dist { b } else { a });
                }
                (Some(b), None) => return Some(b),
                (None, Some(a)) => return Some(a),
                (None, None) => {}
            }
        }
    }
    None
}

/// Find the last integer in a string.
fn last_number_in(s: &str) -> Option<i64> {
    let mut result = None;
    let mut num_start = None;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() {
            if num_start.is_none() {
                num_start = Some(i);
            }
        } else if let Some(start) = num_start {
            if let Ok(n) = s[start..i].parse::<i64>() {
                result = Some(n);
            }
            num_start = None;
        }
    }

    // Check trailing number
    if let Some(start) = num_start {
        if let Ok(n) = s[start..].parse::<i64>() {
            result = Some(n);
        }
    }

    result
}

/// Find the first integer in a string.
fn first_number_in(s: &str) -> Option<i64> {
    let mut num_start = None;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() {
            if num_start.is_none() {
                num_start = Some(i);
            }
        } else if let Some(start) = num_start {
            return s[start..i].parse::<i64>().ok();
        }
    }

    if let Some(start) = num_start {
        return s[start..].parse::<i64>().ok();
    }

    None
}

// ── Retroactive Mode ──────────────────────────────────────────────────────

/// Result of a retroactive autopsy run.
#[derive(Debug)]
pub struct RetroactiveResult {
    /// Sessions processed.
    pub sessions_processed: usize,
    /// Autopsy records inserted (new records only).
    pub records_inserted: usize,
    /// Sessions skipped (already had autopsy records).
    pub records_skipped: usize,
    /// Errors encountered (non-fatal).
    pub errors: Vec<String>,
}

/// Run retroactive autopsy across all sessions in the database.
///
/// For each session:
/// 1. Check if autopsy already exists (skip if so)
/// 2. Look up handoffs for files_modified, lines_written, commits
/// 3. Look up session-reflection.md artifact for outcome_verdict
/// 4. Look up token_efficiency for tokens_total
/// 5. Aggregate tool_usage for tool_calls_total and mcp_calls
/// 6. Parse handoff content for any metrics not in structured columns
/// 7. Insert partial autopsy record (G1/G2/G3 = not_evaluated)
///
/// # Errors
///
/// Returns a fatal error only on database access failure.
/// Per-session errors are collected in [`RetroactiveResult::errors`].
pub fn run_retroactive(conn: &Connection) -> Result<RetroactiveResult> {
    let mut result = RetroactiveResult {
        sessions_processed: 0,
        records_inserted: 0,
        records_skipped: 0,
        errors: Vec::new(),
    };

    // Get all sessions
    let mut stmt = conn.prepare(
        "SELECT id, project, description, created_at FROM sessions ORDER BY created_at ASC",
    )?;
    let sessions: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (session_id, _project, description, created_at) in &sessions {
        result.sessions_processed = result.sessions_processed.saturating_add(1);

        let (inserted, errors) =
            build_and_insert_retroactive(conn, session_id, description, created_at);
        for e in errors {
            result.errors.push(e);
        }
        match inserted {
            Some(true) => {
                result.records_inserted = result.records_inserted.saturating_add(1);
            }
            Some(false) => {
                result.records_skipped = result.records_skipped.saturating_add(1);
            }
            None => {} // error already pushed
        }
    }

    Ok(result)
}

/// Run retroactive autopsy for a single session by ID.
///
/// Looks up the session metadata, builds an autopsy row from available data,
/// and inserts it (INSERT OR IGNORE for idempotency).
///
/// Returns the autopsy row and whether it was newly inserted.
///
/// # Errors
///
/// Returns an error if the session is not found in the database.
pub fn run_retroactive_single(conn: &Connection, session_id: &str) -> Result<(AutopsyRow, bool)> {
    // Look up session metadata
    let (description, created_at): (String, String) = conn
        .query_row(
            "SELECT description, created_at FROM sessions WHERE id = ?1",
            [session_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|_| crate::error::DbError::NotFound(["Session '", session_id, "'"].concat()))?;

    let mut errors = Vec::new();
    let mut row = build_retroactive_row(conn, session_id, &description, &created_at, &mut errors);

    let inserted = autopsy::insert_autopsy(conn, &row)?;

    // If already existed, load the existing record for return
    if inserted == 0 {
        row = autopsy::get_by_session(conn, session_id)?;
    }

    Ok((row, inserted > 0))
}

/// Shared per-session retroactive logic.
///
/// Returns (was_inserted_option, collected_errors).
/// `Some(true)` = newly inserted, `Some(false)` = duplicate skipped, `None` = insert failed.
fn build_and_insert_retroactive(
    conn: &Connection,
    session_id: &str,
    description: &str,
    created_at: &str,
) -> (Option<bool>, Vec<String>) {
    let mut errors = Vec::new();
    let row = build_retroactive_row(conn, session_id, description, created_at, &mut errors);

    match autopsy::insert_autopsy(conn, &row) {
        Ok(1) => (Some(true), errors),
        Ok(_) => (Some(false), errors),
        Err(e) => {
            errors.push(
                [
                    "Autopsy insert error for ",
                    session_id,
                    ": ",
                    &e.to_string(),
                ]
                .concat(),
            );
            (None, errors)
        }
    }
}

/// Build an autopsy row from retroactive session data.
fn build_retroactive_row(
    conn: &Connection,
    session_id: &str,
    description: &str,
    created_at: &str,
    errors: &mut Vec<String>,
) -> AutopsyRow {
    let mut row = AutopsyRow {
        session_id: session_id.to_string(),
        session_started_at: created_at.to_string(),
        ..AutopsyRow::default()
    };

    row.directive_id = extract_directive_id(description);
    row.phase = extract_phase(description);
    row.phase_type = extract_phase_type(description);

    aggregate_handoff_metrics(conn, session_id, &mut row, errors);
    parse_reflection_artifact(conn, session_id, &mut row);
    aggregate_token_metrics(conn, session_id, &mut row);
    // NOTE: aggregate_tool_usage intentionally NOT called in retroactive mode.
    // The tool_usage table is global (no session_id column), so querying it
    // would stamp every session with the same total. Per-session tool counts
    // come from handoff content parsing and aggregate_token_metrics instead.

    row
}

/// Extract directive ID from session description (e.g., "D008", "VDAG-CONSERVATION").
fn extract_directive_id(description: &str) -> Option<String> {
    // Match "D\d+" pattern
    let bytes = description.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'D' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let start = i;
            let mut end = i + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            return Some(description[start..end].to_string());
        }
    }
    None
}

/// Extract phase label from description (e.g., "Phase 1", "Phase 2").
fn extract_phase(description: &str) -> Option<String> {
    let lower = description.to_lowercase();
    if let Some(pos) = lower.find("phase") {
        let rest = &description[pos..];
        // Take "Phase N" or "Phase N+1"
        let end = rest
            .char_indices()
            .skip(6) // "Phase "
            .find(|&(_, c)| !c.is_ascii_digit() && c != '+')
            .map_or(rest.len(), |(i, _)| i);
        let phase = rest[..end].trim();
        if phase.len() > 5 {
            return Some(phase.to_string());
        }
    }
    None
}

/// Extract phase type from description.
fn extract_phase_type(description: &str) -> Option<String> {
    let lower = description.to_lowercase();
    for phase_type in ["audit", "design", "implementation", "migration"] {
        if lower.contains(phase_type) {
            return Some(phase_type.to_string());
        }
    }
    None
}

/// Aggregate handoff metrics for a session.
fn aggregate_handoff_metrics(
    conn: &Connection,
    session_id: &str,
    row: &mut AutopsyRow,
    errors: &mut Vec<String>,
) {
    let query_result = conn.prepare(
        "SELECT files_modified, lines_written, commits, content
         FROM handoffs WHERE session_id = ?1",
    );

    let mut stmt = match query_result {
        Ok(s) => s,
        Err(e) => {
            errors.push(format!("Handoff query for {session_id}: {e}"));
            return;
        }
    };

    let handoffs: Vec<(i32, i32, i32, String)> = stmt
        .query_map([session_id], |r| {
            Ok((
                r.get::<_, i32>(0)?,
                r.get::<_, i32>(1)?,
                r.get::<_, i32>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    for (fm, lw, cm, content) in &handoffs {
        row.files_modified = row.files_modified.saturating_add(i64::from(*fm));
        row.lines_written = row.lines_written.saturating_add(i64::from(*lw));
        row.commits = row.commits.saturating_add(i64::from(*cm));

        // Parse content for tool_calls_total, mcp_calls, hook_blocks
        let parsed = parse_handoff_content(content);
        if parsed.tool_calls_total > 0 {
            row.tool_calls_total = row.tool_calls_total.saturating_add(parsed.tool_calls_total);
        }
        if parsed.mcp_calls > 0 {
            row.mcp_calls = row.mcp_calls.saturating_add(parsed.mcp_calls);
        }
        if parsed.hook_blocks > 0 {
            row.hook_blocks = row.hook_blocks.saturating_add(parsed.hook_blocks);
        }
    }
}

/// Parse the session-reflection.md artifact for outcome verdict and counts.
fn parse_reflection_artifact(conn: &Connection, session_id: &str, row: &mut AutopsyRow) {
    // Try to find session-reflection.md artifact
    let art_result = conn.query_row(
        "SELECT content FROM artifacts
         WHERE session_id = ?1 AND name = 'session-reflection.md'",
        [session_id],
        |r| r.get::<_, String>(0),
    );

    if let Ok(content) = art_result {
        // First try YAML frontmatter (prospective-enriched artifacts)
        if let Some(fm) = extract_frontmatter(&content) {
            apply_frontmatter(row, &fm);
            return;
        }

        // Fall back to prose parsing (retroactive mode)
        row.outcome_verdict = parse_outcome_verdict(&content);
        row.lesson_count = count_lessons(&content);
        row.pattern_count = count_patterns(&content);
        row.reflection_artifact = Some("session-reflection.md".into());
    }
}

/// Apply frontmatter data to an autopsy row.
fn apply_frontmatter(row: &mut AutopsyRow, fm: &AutopsyFrontmatter) {
    if fm.directive_id.is_some() {
        row.directive_id = fm.directive_id.clone();
    }
    if fm.phase.is_some() {
        row.phase = fm.phase.clone();
    }
    if fm.phase_type.is_some() {
        row.phase_type = fm.phase_type.clone();
    }
    row.outcome_verdict = fm.outcome_verdict.clone();
    row.lesson_count = fm.lesson_count;
    row.pattern_count = fm.pattern_count;

    row.rc_pdp_proposition = fm.root_causes.pdp_proposition;
    row.rc_pdp_so_what = fm.root_causes.pdp_so_what;
    row.rc_pdp_why = fm.root_causes.pdp_why;
    row.rc_hook_gap = fm.root_causes.hook_gap;

    row.tool_calls_total = fm.metrics.tool_calls_total;
    row.mcp_calls = fm.metrics.mcp_calls;
    row.hook_blocks = fm.metrics.hook_blocks;
    row.files_modified = fm.metrics.files_modified;
    row.lines_written = fm.metrics.lines_written;
    row.commits = fm.metrics.commits;
    row.tokens_total = fm.metrics.tokens_total;

    row.rho_session = fm.compounding.rho_session;
    row.tools_sovereign = fm.compounding.tools_sovereign;
    row.tools_analysis = fm.compounding.tools_analysis;

    row.reflection_artifact = Some("session-reflection.md".into());
}

/// Look up token efficiency for this session.
fn aggregate_token_metrics(conn: &Connection, session_id: &str, row: &mut AutopsyRow) {
    let result = conn.query_row(
        "SELECT action_count, total_tokens FROM token_efficiency WHERE session_id = ?1",
        [session_id],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
    );

    if let Ok((action_count, total_tokens)) = result {
        if row.tool_calls_total == 0 {
            row.tool_calls_total = action_count;
        }
        row.tokens_total = total_tokens;
    }
}

// ── Prospective Mode ──────────────────────────────────────────────────────

/// Run prospective autopsy for a single session.
///
/// Reads enriched session-reflection.md artifact (with YAML frontmatter)
/// and PDP evaluation telemetry from the provided JSONL path.
///
/// # Arguments
///
/// * `conn` — Database connection with V4 schema
/// * `session_id` — Brain session ID to autopsy
/// * `pdp_jsonl_path` — Path to `pdp_evaluations.jsonl`
///
/// # Errors
///
/// Returns error on database failure.
pub fn run_prospective(
    conn: &Connection,
    session_id: &str,
    pdp_jsonl_path: &std::path::Path,
) -> Result<AutopsyRow> {
    let mut row = AutopsyRow {
        session_id: session_id.to_string(),
        ..AutopsyRow::default()
    };

    // Get session metadata
    if let Ok(session_created) = conn.query_row(
        "SELECT created_at FROM sessions WHERE id = ?1",
        [session_id],
        |r| r.get::<_, String>(0),
    ) {
        row.session_started_at = session_created;
    }

    // Parse enriched reflection artifact (YAML frontmatter + prose)
    parse_reflection_artifact(conn, session_id, &mut row);

    // Parse PDP evaluation telemetry
    let evals = pdp_telemetry::read_evaluations_from(pdp_jsonl_path).unwrap_or_default();

    let session_evals: Vec<_> = evals
        .iter()
        .filter(|e| e.session_id == session_id)
        .collect();

    // Foundation gate (G1/G2/G3)
    for eval in session_evals
        .iter()
        .filter(|e| e.gate == PdpGate::Foundation)
    {
        for check in &eval.checks {
            let val = if check.passed { "pass" } else { "fail" };
            match check.check_id.as_str() {
                "G1" => row.g1_proposition = val.into(),
                "G2" => row.g2_specificity = val.into(),
                "G3" => row.g3_singularity = val.into(),
                _ => {}
            }
        }
    }

    // Structural gate (S1-S5)
    for eval in session_evals
        .iter()
        .filter(|e| e.gate == PdpGate::Structural)
    {
        for check in &eval.checks {
            let val = if check.passed { 1 } else { 0 };
            match check.check_id.as_str() {
                "S1" => row.s1_badjective = i64::from(!check.passed),
                "S2" => row.s2_throat_clear = i64::from(!check.passed),
                "S3" => row.s3_hedging = i64::from(!check.passed),
                "S4" => row.s4_context = val,
                "S5" => row.s5_output_spec = val,
                _ => {}
            }
        }
    }

    // Calibration gate (C1-C5)
    for eval in session_evals
        .iter()
        .filter(|e| e.gate == PdpGate::Calibration)
    {
        for check in &eval.checks {
            let val = if check.passed { 1 } else { 0 };
            match check.check_id.as_str() {
                "C1" => row.c1_eval_criteria = val,
                "C2" => row.c2_outcome_focus = val,
                "C3" => row.c3_abstraction = val,
                "C4" => row.c4_decisive_ending = val,
                "C5" => row.c5_sell_mode = i64::from(!check.passed),
                _ => {}
            }
        }
    }

    // Count hook blocks from PDP evaluations
    let pdp_blocks: i64 = session_evals
        .iter()
        .filter(|e| e.blocked)
        .count()
        .try_into()
        .unwrap_or(0);
    if pdp_blocks > row.hook_blocks {
        row.hook_blocks = pdp_blocks;
    }

    // Aggregate handoff + token data from DB (fills gaps not covered by frontmatter)
    let mut errors = Vec::new();
    if row.files_modified == 0 && row.lines_written == 0 && row.commits == 0 {
        aggregate_handoff_metrics(conn, session_id, &mut row, &mut errors);
    }
    if row.tokens_total == 0 {
        aggregate_token_metrics(conn, session_id, &mut row);
    }
    // NOTE: aggregate_tool_usage NOT called — tool_usage table is global
    // (no session_id). Prospective mode gets tool counts from frontmatter
    // or handoff parsing. Zero is honest when no per-session data exists.

    // Insert (idempotent)
    autopsy::insert_autopsy(conn, &row)?;

    Ok(row)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool::DbPool;

    // ── YAML Frontmatter Tests ──

    #[test]
    fn test_yaml_frontmatter_round_trip() {
        let content = r#"---
autopsy:
  directive_id: "D008"
  phase: "Phase 2"
  phase_type: "implementation"
  outcome_verdict: "fully_demonstrated"
  lesson_count: 2
  pattern_count: 3
  root_causes:
    pdp_proposition: 0
    pdp_so_what: 1
    pdp_why: 0
    hook_gap: 1
  metrics:
    tool_calls_total: 87
    mcp_calls: 22
    hook_blocks: 3
    files_modified: 5
    lines_written: 340
    commits: 2
    tokens_total: 45000
  compounding:
    rho_session: 0.55
    tools_sovereign: 12
    tools_analysis: 22
---

### Lessons Learned

> "When [X occurred], [Y failed] because [Z root cause]."
> "When [A happened], [B broke] because [C reason]."

### Session Reflection

The session's proposition was **fully demonstrated**.

### Identified Patterns for Future Capability Enhancements

> "This session revealed that [gap 1]. A future capability is [mech 1]."
> "This session revealed that [gap 2]. A future capability is [mech 2]."
> "This session revealed that [gap 3]. A future capability is [mech 3]."
"#;

        let fm = extract_frontmatter(content).expect("Should parse frontmatter");

        assert_eq!(fm.directive_id.as_deref(), Some("D008"));
        assert_eq!(fm.phase.as_deref(), Some("Phase 2"));
        assert_eq!(fm.phase_type.as_deref(), Some("implementation"));
        assert_eq!(fm.outcome_verdict.as_deref(), Some("fully_demonstrated"));
        assert_eq!(fm.lesson_count, 2);
        assert_eq!(fm.pattern_count, 3);

        assert_eq!(fm.root_causes.pdp_proposition, 0);
        assert_eq!(fm.root_causes.pdp_so_what, 1);
        assert_eq!(fm.root_causes.pdp_why, 0);
        assert_eq!(fm.root_causes.hook_gap, 1);

        assert_eq!(fm.metrics.tool_calls_total, 87);
        assert_eq!(fm.metrics.mcp_calls, 22);
        assert_eq!(fm.metrics.hook_blocks, 3);
        assert_eq!(fm.metrics.files_modified, 5);
        assert_eq!(fm.metrics.lines_written, 340);
        assert_eq!(fm.metrics.commits, 2);
        assert_eq!(fm.metrics.tokens_total, 45000);

        assert!((fm.compounding.rho_session.unwrap() - 0.55).abs() < f64::EPSILON);
        assert_eq!(fm.compounding.tools_sovereign, 12);
        assert_eq!(fm.compounding.tools_analysis, 22);
    }

    #[test]
    fn test_frontmatter_null_fields() {
        let content = "---\nautopsy:\n  directive_id: null\n  outcome_verdict: \"not_demonstrated\"\n  lesson_count: 0\n  pattern_count: 0\n  root_causes:\n    pdp_proposition: 0\n    pdp_so_what: 0\n    pdp_why: 0\n    hook_gap: 0\n  metrics:\n    tool_calls_total: 0\n    mcp_calls: 0\n    hook_blocks: 0\n    files_modified: 0\n    lines_written: 0\n    commits: 0\n    tokens_total: 0\n  compounding:\n    rho_session: null\n    tools_sovereign: 0\n    tools_analysis: 0\n---\nContent here";

        let fm = extract_frontmatter(content).expect("Should parse");
        assert!(fm.directive_id.is_none());
        assert_eq!(fm.outcome_verdict.as_deref(), Some("not_demonstrated"));
        assert!(fm.compounding.rho_session.is_none());
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "### Lessons Learned\nNo failures.";
        assert!(extract_frontmatter(content).is_none());
    }

    // ── Handoff Parsing Tests ──

    #[test]
    fn test_parse_handoff_content_format_a() {
        let content = "## Session Summary\n\n**Tool Usage:** 150 calls total, 30 MCP, hook blocked 5 times\n**Files Modified:** 8\n**Lines Written:** 400\n**Commits:** 2";

        let m = parse_handoff_content(content);
        assert_eq!(m.tool_calls_total, 150);
        assert_eq!(m.mcp_calls, 30);
        assert_eq!(m.hook_blocks, 5);
        assert_eq!(m.files_modified, 8);
        assert_eq!(m.lines_written, 400);
        assert_eq!(m.commits, 2);
    }

    #[test]
    fn test_parse_handoff_content_format_b() {
        let content = "Tool calls: 87\nMCP calls: 22\nHook blocks: 3\nFiles modified: 5\nLines written: 340\nCommits: 2";

        let m = parse_handoff_content(content);
        assert_eq!(m.tool_calls_total, 87);
        assert_eq!(m.mcp_calls, 22);
        assert_eq!(m.hook_blocks, 3);
        assert_eq!(m.files_modified, 5);
        assert_eq!(m.lines_written, 340);
        assert_eq!(m.commits, 2);
    }

    #[test]
    fn test_parse_handoff_no_metrics() {
        let content = "Just some general notes about the session.";
        let m = parse_handoff_content(content);
        assert_eq!(m.tool_calls_total, 0);
        assert_eq!(m.mcp_calls, 0);
    }

    // ── Reflection Prose Parsing Tests ──

    #[test]
    fn test_outcome_verdict_parsing() {
        assert_eq!(
            parse_outcome_verdict("The proposition was **fully demonstrated**."),
            Some("fully_demonstrated".into())
        );
        assert_eq!(
            parse_outcome_verdict("Partially demonstrated — with caveats."),
            Some("partially_demonstrated".into())
        );
        assert_eq!(
            parse_outcome_verdict("Not demonstrated due to blockers."),
            Some("not_demonstrated".into())
        );
        assert_eq!(
            parse_outcome_verdict("Some random text with no verdict."),
            None
        );
    }

    #[test]
    fn test_count_lessons() {
        let content = r#"### Lessons Learned

> "When the hook fired, the build failed because the state file was stale."
> "When MCP tool was called, it returned error because the server was down."

No other lessons."#;

        assert_eq!(count_lessons(content), 2);
    }

    #[test]
    fn test_count_patterns() {
        let content = r#"### Identified Patterns

> "This session revealed that hook state files lack TTL. A future capability is a state-gc hook."
> "This session revealed that MCP error classification is inconsistent. A future capability is error-classifier crate."
"#;

        assert_eq!(count_patterns(content), 2);
    }

    // ── Retroactive Mode Tests ──

    fn setup_test_db() -> DbPool {
        let pool = DbPool::open_in_memory().expect("open");
        pool.with_conn(|conn| {
            // Insert sessions
            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-retro-1', 'nexcore', 'D008 Phase 1 audit', '2026-02-20T10:00:00Z')",
                [],
            )?;
            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-retro-2', 'nexcore', 'General coding session', '2026-02-21T14:00:00Z')",
                [],
            )?;

            // Insert handoff for session 1
            conn.execute(
                "INSERT INTO handoffs (project, handoff_number, session_id, generated_at, status, duration, files_modified, lines_written, commits, uncommitted, content)
                 VALUES ('nexcore', 1, 'sess-retro-1', '2026-02-20T16:00:00Z', 'Complete', '6h', 8, 400, 2, 0,
                 'Tool calls: 150\nMCP calls: 30\nHook blocks: 5')",
                [],
            )?;

            // Insert reflection artifact for session 1
            conn.execute(
                "INSERT INTO artifacts (session_id, name, artifact_type, content, summary, current_version, tags, custom_meta, created_at, updated_at)
                 VALUES ('sess-retro-1', 'session-reflection.md', 'custom',
                 '### Lessons Learned\n\n> \"When the hook fired, the build failed because state was stale.\"\n\n### Session Reflection\n\nThe proposition was **fully demonstrated**.\n\n### Identified Patterns\n\n> \"This session revealed that state files lack TTL. A future capability is state-gc.\"',
                 'Session reflection', 0, '[]', 'null', '2026-02-20T16:00:00Z', '2026-02-20T16:00:00Z')",
                [],
            )?;

            // Token efficiency for session 1
            conn.execute(
                "INSERT INTO token_efficiency (session_id, action_count, total_tokens, started_at)
                 VALUES ('sess-retro-1', 150, 45000, 1708419600)",
                [],
            )?;

            Ok(())
        })
        .expect("setup");
        pool
    }

    #[test]
    fn test_retroactive_basic() {
        let pool = setup_test_db();
        pool.with_conn(|conn| {
            let result = run_retroactive(conn)?;

            assert_eq!(result.sessions_processed, 2);
            assert_eq!(result.records_inserted, 2);
            assert_eq!(result.records_skipped, 0);

            // Check session 1 — has handoff + artifact data
            let a1 = autopsy::get_by_session(conn, "sess-retro-1")?;
            assert_eq!(a1.directive_id.as_deref(), Some("D008"));
            assert_eq!(a1.phase.as_deref(), Some("Phase 1"));
            assert_eq!(a1.phase_type.as_deref(), Some("audit"));
            assert_eq!(a1.outcome_verdict.as_deref(), Some("fully_demonstrated"));
            assert_eq!(a1.lesson_count, 1);
            assert_eq!(a1.pattern_count, 1);
            assert_eq!(a1.files_modified, 8);
            assert_eq!(a1.lines_written, 400);
            assert_eq!(a1.commits, 2);
            assert_eq!(a1.tool_calls_total, 150);
            assert_eq!(a1.mcp_calls, 30);
            assert_eq!(a1.hook_blocks, 5);
            assert_eq!(a1.tokens_total, 45000);
            // G1/G2/G3 should be not_evaluated for retroactive
            assert_eq!(a1.g1_proposition, "not_evaluated");
            assert_eq!(a1.g2_specificity, "not_evaluated");
            assert_eq!(a1.g3_singularity, "not_evaluated");

            // Check session 2 — minimal data (no handoff, no artifact)
            let a2 = autopsy::get_by_session(conn, "sess-retro-2")?;
            assert!(a2.directive_id.is_none());
            assert!(a2.outcome_verdict.is_none());
            assert_eq!(a2.files_modified, 0);

            Ok(())
        })
        .expect("test");
    }

    #[test]
    fn test_retroactive_idempotent() {
        let pool = setup_test_db();
        pool.with_conn(|conn| {
            let r1 = run_retroactive(conn)?;
            assert_eq!(r1.records_inserted, 2);

            let r2 = run_retroactive(conn)?;
            assert_eq!(r2.records_inserted, 0);
            assert_eq!(r2.records_skipped, 2);

            // Total count should still be 2
            assert_eq!(autopsy::count_autopsies(conn)?, 2);

            Ok(())
        })
        .expect("test");
    }

    // ── Prospective Mode Tests ──

    #[test]
    fn test_prospective_with_enriched_artifact() {
        let pool = DbPool::open_in_memory().expect("open");
        pool.with_conn(|conn| {
            // Insert session
            conn.execute(
                "INSERT INTO sessions (id, project, description, created_at)
                 VALUES ('sess-pro-1', 'nexcore', 'D009 Phase 2 implementation', '2026-02-28T10:00:00Z')",
                [],
            )?;

            // Insert enriched artifact with YAML frontmatter
            let enriched = r#"---
autopsy:
  directive_id: "D009"
  phase: "Phase 2"
  phase_type: "implementation"
  outcome_verdict: "fully_demonstrated"
  lesson_count: 1
  pattern_count: 2
  root_causes:
    pdp_proposition: 0
    pdp_so_what: 0
    pdp_why: 1
    hook_gap: 0
  metrics:
    tool_calls_total: 120
    mcp_calls: 40
    hook_blocks: 2
    files_modified: 6
    lines_written: 500
    commits: 3
    tokens_total: 60000
  compounding:
    rho_session: 0.65
    tools_sovereign: 15
    tools_analysis: 23
---

### Lessons Learned

> "When the validator ran, parsing failed because the YAML lacked quotes."

### Session Reflection

The proposition was **fully demonstrated**.

### Identified Patterns

> "This session revealed that YAML parsing is fragile. A future capability is yaml-validator hook."
> "This session revealed that frontmatter specs need tests. A future capability is frontmatter-roundtrip test suite."
"#;

            conn.execute(
                "INSERT INTO artifacts (session_id, name, artifact_type, content, summary, current_version, tags, custom_meta, created_at, updated_at)
                 VALUES ('sess-pro-1', 'session-reflection.md', 'custom', ?1, 'Enriched reflection', 0, '[]', 'null', '2026-02-28T14:00:00Z', '2026-02-28T14:00:00Z')",
                params![enriched],
            )?;

            // Create PDP evaluation telemetry
            let pdp_path = std::env::temp_dir().join("test_prospective_pdp.jsonl");
            let evals = vec![
                crate::pdp_telemetry::PdpEvaluation {
                    timestamp: "2026-02-28T10:05:00Z".into(),
                    session_id: "sess-pro-1".into(),
                    gate: PdpGate::Foundation,
                    checks: vec![
                        crate::pdp_telemetry::CheckResult { check_id: "G1".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "G2".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "G3".into(), passed: false, detail: Some("Multiple objectives".into()) },
                    ],
                    blocked: true,
                    prompt_hash: Some("abc123".into()),
                    reason: Some("G3 failed".into()),
                },
                crate::pdp_telemetry::PdpEvaluation {
                    timestamp: "2026-02-28T10:05:00Z".into(),
                    session_id: "sess-pro-1".into(),
                    gate: PdpGate::Structural,
                    checks: vec![
                        crate::pdp_telemetry::CheckResult { check_id: "S1".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "S2".into(), passed: false, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "S3".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "S4".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "S5".into(), passed: true, detail: None },
                    ],
                    blocked: false,
                    prompt_hash: None,
                    reason: None,
                },
                crate::pdp_telemetry::PdpEvaluation {
                    timestamp: "2026-02-28T10:10:00Z".into(),
                    session_id: "sess-pro-1".into(),
                    gate: PdpGate::Calibration,
                    checks: vec![
                        crate::pdp_telemetry::CheckResult { check_id: "C1".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "C2".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "C3".into(), passed: true, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "C4".into(), passed: false, detail: None },
                        crate::pdp_telemetry::CheckResult { check_id: "C5".into(), passed: true, detail: None },
                    ],
                    blocked: false,
                    prompt_hash: None,
                    reason: None,
                },
            ];

            for eval in &evals {
                crate::pdp_telemetry::write_evaluation_to(eval, &pdp_path).unwrap();
            }

            // Run prospective mode
            let row = run_prospective(conn, "sess-pro-1", &pdp_path)?;

            // Verify all fields populated from frontmatter
            assert_eq!(row.directive_id.as_deref(), Some("D009"));
            assert_eq!(row.phase.as_deref(), Some("Phase 2"));
            assert_eq!(row.phase_type.as_deref(), Some("implementation"));
            assert_eq!(
                row.outcome_verdict.as_deref(),
                Some("fully_demonstrated")
            );
            assert_eq!(row.lesson_count, 1);
            assert_eq!(row.pattern_count, 2);
            assert_eq!(row.rc_pdp_why, 1);
            assert_eq!(row.tool_calls_total, 120);
            assert_eq!(row.mcp_calls, 40);
            assert_eq!(row.files_modified, 6);
            assert_eq!(row.lines_written, 500);
            assert_eq!(row.commits, 3);
            assert_eq!(row.tokens_total, 60000);
            assert!((row.rho_session.unwrap() - 0.65).abs() < f64::EPSILON);
            assert_eq!(row.tools_sovereign, 15);
            assert_eq!(row.tools_analysis, 23);

            // Verify PDP gate results from telemetry
            assert_eq!(row.g1_proposition, "pass");
            assert_eq!(row.g2_specificity, "pass");
            assert_eq!(row.g3_singularity, "fail");
            assert_eq!(row.s1_badjective, 0); // S1 passed
            assert_eq!(row.s2_throat_clear, 1); // S2 failed
            assert_eq!(row.s3_hedging, 0); // S3 passed
            assert_eq!(row.s4_context, 1); // S4 passed
            assert_eq!(row.s5_output_spec, 1); // S5 passed
            assert_eq!(row.c1_eval_criteria, 1); // C1 passed
            assert_eq!(row.c4_decisive_ending, 0); // C4 failed
            assert_eq!(row.c5_sell_mode, 0); // C5 passed (no sell mode)

            // Hook blocks: 2 from frontmatter, but PDP shows 1 block
            // Frontmatter metrics win since they were set first
            assert_eq!(row.hook_blocks, 2);

            // Cleanup
            std::fs::remove_file(&pdp_path).ok();

            Ok(())
        })
        .expect("test");
    }

    // ── Helper Function Tests ──

    #[test]
    fn test_extract_directive_id() {
        assert_eq!(
            extract_directive_id("D008 Phase 1 audit"),
            Some("D008".into())
        );
        assert_eq!(
            extract_directive_id("Working on D012 implementation"),
            Some("D012".into())
        );
        assert_eq!(extract_directive_id("General coding session"), None);
        assert_eq!(extract_directive_id("VDAG-CONSERVATION exploration"), None);
    }

    #[test]
    fn test_extract_phase() {
        assert_eq!(extract_phase("D008 Phase 1 audit"), Some("Phase 1".into()));
        assert_eq!(
            extract_phase("Phase 2 implementation work"),
            Some("Phase 2".into())
        );
        assert_eq!(extract_phase("General coding"), None);
    }

    #[test]
    fn test_extract_phase_type() {
        assert_eq!(
            extract_phase_type("D008 Phase 1 audit"),
            Some("audit".into())
        );
        assert_eq!(
            extract_phase_type("implementation of feature X"),
            Some("implementation".into())
        );
        assert_eq!(extract_phase_type("General coding"), None);
    }

    #[test]
    fn test_number_extraction() {
        assert_eq!(first_number_in(": 42 things"), Some(42));
        assert_eq!(first_number_in(" no numbers "), None);
        assert_eq!(last_number_in("abc 10 def 20 ghi"), Some(20));
        assert_eq!(last_number_in("150 calls"), Some(150));
    }
}
