//! One-shot JSON-to-SQLite migration.
//!
//! Reads the existing file-based Brain storage (`~/.claude/brain/`,
//! `~/.claude/implicit/`, `~/.claude/code_tracker/`) and imports
//! all data into the SQLite database.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Deserialize;

use crate::error::{DbError, Result};
use crate::{artifacts, decisions, implicit, knowledge, sessions, telemetry, tracker};

/// Statistics from the migration.
#[derive(Debug, Default)]
pub struct MigrationStats {
    /// Sessions imported
    pub sessions: usize,
    /// Artifacts imported
    pub artifacts: usize,
    /// Artifact versions imported
    pub versions: usize,
    /// Tracked files imported
    pub tracked_files: usize,
    /// Preferences imported
    pub preferences: usize,
    /// Patterns imported
    pub patterns: usize,
    /// Corrections imported
    pub corrections: usize,
    /// Beliefs imported
    pub beliefs: usize,
    /// Trust accumulators imported
    pub trust_accumulators: usize,
    /// Belief implications imported
    pub implications: usize,
    // V2 fields
    /// Decisions ingested
    pub decisions: usize,
    /// Tool usage entries ingested
    pub tool_usage: usize,
    /// Token efficiency sessions ingested
    pub token_efficiency: usize,
    /// Task history records ingested
    pub tasks: usize,
    /// Handoff records ingested
    pub handoffs: usize,
    /// Antibody records ingested
    pub antibodies: usize,
    /// Errors encountered (non-fatal)
    pub errors: Vec<String>,
}

impl std::fmt::Display for MigrationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migration: {} sessions, {} artifacts, {} versions, {} tracked, \
             {} preferences, {} patterns, {} corrections, {} beliefs, {} trust, {} implications | \
             V2: {} decisions, {} tools, {} efficiency, {} tasks, {} handoffs, {} antibodies | \
             {} errors",
            self.sessions,
            self.artifacts,
            self.versions,
            self.tracked_files,
            self.preferences,
            self.patterns,
            self.corrections,
            self.beliefs,
            self.trust_accumulators,
            self.implications,
            self.decisions,
            self.tool_usage,
            self.token_efficiency,
            self.tasks,
            self.handoffs,
            self.antibodies,
            self.errors.len()
        )
    }
}

/// Run the full migration from JSON files into the database.
///
/// # Arguments
///
/// * `conn` - Database connection (schema must already be initialized)
/// * `brain_dir` - Path to `~/.claude/brain/`
/// * `implicit_dir` - Path to `~/.claude/implicit/`
/// * `tracker_dir` - Path to `~/.claude/code_tracker/`
///
/// # Errors
///
/// Returns an error only on critical failures (e.g., transaction failure).
/// Individual file errors are captured in `MigrationStats::errors`.
pub fn run(
    conn: &Connection,
    brain_dir: &Path,
    implicit_dir: &Path,
    tracker_dir: &Path,
) -> Result<MigrationStats> {
    let mut stats = MigrationStats::default();

    // Wrap everything in a transaction for atomicity
    conn.execute_batch("BEGIN TRANSACTION;")?;

    migrate_sessions(conn, brain_dir, &mut stats);
    migrate_implicit(conn, implicit_dir, &mut stats);
    migrate_tracker(conn, tracker_dir, &mut stats);

    conn.execute_batch("COMMIT;")?;

    Ok(stats)
}

/// Run V2 migration: ingest dotfile telemetry and accumulated knowledge.
///
/// # Arguments
///
/// * `conn` - Database connection (schema V2 must be initialized)
/// * `claude_dir` - Path to `~/.claude/`
///
/// # Errors
///
/// Returns an error only on critical failures.
pub fn run_v2(conn: &Connection, claude_dir: &Path) -> Result<MigrationStats> {
    let mut stats = MigrationStats::default();

    conn.execute_batch("BEGIN TRANSACTION;")?;

    migrate_decisions(conn, claude_dir, &mut stats);
    migrate_journal(conn, claude_dir, &mut stats);
    migrate_tool_usage(conn, claude_dir, &mut stats);
    migrate_token_efficiency(conn, claude_dir, &mut stats);
    migrate_tasks(conn, claude_dir, &mut stats);
    migrate_handoffs(conn, claude_dir, &mut stats);
    migrate_antibodies(conn, claude_dir, &mut stats);

    conn.execute_batch("COMMIT;")?;

    Ok(stats)
}

/// Migrate sessions and their artifacts from the Brain directory.
fn migrate_sessions(conn: &Connection, brain_dir: &Path, stats: &mut MigrationStats) {
    let sessions_dir = brain_dir.join("sessions");
    if !sessions_dir.exists() {
        return;
    }

    // Read the index
    let index_path = brain_dir.join("index.json");
    let entries: Vec<SessionEntry> = match read_json(&index_path) {
        Ok(v) => v,
        Err(e) => {
            stats
                .errors
                .push(format!("Failed to read session index: {e}"));
            return;
        }
    };

    for entry in &entries {
        let session = sessions::SessionRow {
            id: entry.id.clone(),
            project: entry.project.clone().unwrap_or_default(),
            git_commit: entry.git_commit.clone(),
            description: entry.description.clone().unwrap_or_default(),
            created_at: entry.created_at,
        };

        if let Err(e) = sessions::insert(conn, &session) {
            stats.errors.push(format!("Session {}: {e}", entry.id));
            continue;
        }
        stats.sessions += 1;

        // Migrate artifacts for this session
        let session_dir = sessions_dir.join(&entry.id);
        if session_dir.exists() {
            migrate_session_artifacts(conn, &entry.id, &session_dir, stats);
        }
    }
}

/// Migrate artifacts within a single session directory.
///
/// Brain stores artifacts as bare files (no `.md` extension):
/// - `<name>` — artifact content
/// - `<name>.metadata.json` — per-artifact metadata
/// - `<name>.resolved` — directory marker
/// - `<name>.resolved.<N>` — immutable resolved version N
fn migrate_session_artifacts(
    conn: &Connection,
    session_id: &str,
    session_dir: &Path,
    stats: &mut MigrationStats,
) {
    // Scan for artifact files
    let entries = match fs::read_dir(session_dir) {
        Ok(e) => e,
        Err(e) => {
            stats
                .errors
                .push(format!("Cannot read {}: {e}", session_dir.display()));
            return;
        }
    };

    // Collect filenames to identify artifacts vs metadata/resolved files
    let mut all_files: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let filename = entry.file_name().to_string_lossy().to_string();
        all_files.push(filename);
    }

    // Artifact names = files that are NOT .metadata.json, NOT .resolved*, NOT directories
    let artifact_names: Vec<String> = all_files
        .iter()
        .filter(|f| {
            !f.contains(".metadata.json") && !f.contains(".resolved") && !f.starts_with('.')
        })
        .cloned()
        .collect();

    for artifact_name in &artifact_names {
        let path = session_dir.join(artifact_name);

        // Skip directories
        if path.is_dir() {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                stats
                    .errors
                    .push(format!("Cannot read {}: {e}", path.display()));
                continue;
            }
        };

        // Read per-artifact metadata: <name>.metadata.json
        let meta_path = session_dir.join(format!("{artifact_name}.metadata.json"));
        let meta: Option<ArtifactMeta> = read_json(&meta_path).ok();
        let now = Utc::now();

        let art = artifacts::ArtifactRow {
            id: None,
            session_id: session_id.to_string(),
            name: artifact_name.clone(),
            artifact_type: meta
                .as_ref()
                .map(|m| m.artifact_type.clone())
                .unwrap_or_else(|| infer_type(artifact_name)),
            content,
            summary: meta
                .as_ref()
                .and_then(|m| m.summary.clone())
                .unwrap_or_default(),
            current_version: meta.as_ref().map(|m| m.current_version).unwrap_or(0),
            tags: meta
                .as_ref()
                .map(|m| serde_json::to_string(&m.tags).unwrap_or_else(|_| "[]".into()))
                .unwrap_or_else(|| "[]".into()),
            custom_meta: meta
                .as_ref()
                .and_then(|m| m.custom.as_ref())
                .map(|c| c.to_string())
                .unwrap_or_else(|| "null".into()),
            created_at: meta.as_ref().map(|m| m.created_at).unwrap_or(now),
            updated_at: meta.as_ref().map(|m| m.updated_at).unwrap_or(now),
        };

        if let Err(e) = artifacts::upsert(conn, &art) {
            stats
                .errors
                .push(format!("Artifact {session_id}/{artifact_name}: {e}"));
            continue;
        }
        stats.artifacts += 1;

        // Migrate resolved versions: <name>.resolved.<N>
        let current_ver = meta.as_ref().map(|m| m.current_version).unwrap_or(0);
        for v in 1..=current_ver {
            // Try both formats: `<name>.resolved.<N>` and `<name>.resolved.<N>.md`
            let resolved_path = session_dir.join(format!("{artifact_name}.resolved.{v}"));
            let resolved_md = session_dir.join(format!("{artifact_name}.resolved.{v}.md"));

            let resolved_content =
                fs::read_to_string(&resolved_path).or_else(|_| fs::read_to_string(&resolved_md));

            if let Ok(content) = resolved_content {
                let ver = artifacts::ArtifactVersionRow {
                    id: None,
                    session_id: session_id.to_string(),
                    artifact_name: artifact_name.clone(),
                    version: v,
                    content,
                    resolved_at: now,
                };
                if let Err(e) = artifacts::insert_version(conn, &ver) {
                    stats
                        .errors
                        .push(format!("Version {session_id}/{artifact_name}@{v}: {e}"));
                } else {
                    stats.versions += 1;
                }
            }
        }
    }
}

/// Migrate implicit knowledge (preferences, patterns, corrections, beliefs, trust).
fn migrate_implicit(conn: &Connection, implicit_dir: &Path, stats: &mut MigrationStats) {
    if !implicit_dir.exists() {
        return;
    }

    // Preferences
    let prefs_path = implicit_dir.join("preferences.json");
    if let Ok(prefs) = read_json::<HashMap<String, JsonPreference>>(&prefs_path) {
        for (key, p) in prefs {
            let row = implicit::PreferenceRow {
                key,
                value: serde_json::to_string(&p.value).unwrap_or_else(|_| "null".into()),
                description: p.description,
                confidence: p.confidence,
                reinforcement_count: p.reinforcement_count,
                updated_at: p.updated_at,
            };
            if let Err(e) = implicit::upsert_preference(conn, &row) {
                stats.errors.push(format!("Preference {}: {e}", row.key));
            } else {
                stats.preferences += 1;
            }
        }
    }

    // Patterns
    let patterns_path = implicit_dir.join("patterns.json");
    if let Ok(patterns) = read_json::<Vec<JsonPattern>>(&patterns_path) {
        for p in patterns {
            let row = implicit::PatternRow {
                id: p.id,
                pattern_type: p.pattern_type,
                description: p.description,
                examples: serde_json::to_string(&p.examples).unwrap_or_else(|_| "[]".into()),
                detected_at: p.detected_at,
                updated_at: p.updated_at,
                confidence: p.confidence,
                occurrence_count: p.occurrence_count,
                t1_grounding: p.t1_grounding.map(|g| format!("{g:?}").to_lowercase()),
            };
            if let Err(e) = implicit::upsert_pattern(conn, &row) {
                stats.errors.push(format!("Pattern {}: {e}", row.id));
            } else {
                stats.patterns += 1;
            }
        }
    }

    // Corrections
    let corrections_path = implicit_dir.join("corrections.json");
    if let Ok(corrections) = read_json::<Vec<JsonCorrection>>(&corrections_path) {
        for c in corrections {
            let row = implicit::CorrectionRow {
                id: None,
                mistake: c.mistake,
                correction: c.correction,
                context: c.context,
                learned_at: c.learned_at,
                application_count: c.application_count,
            };
            if let Err(e) = implicit::insert_correction(conn, &row) {
                stats.errors.push(format!("Correction: {e}"));
            } else {
                stats.corrections += 1;
            }
        }
    }

    // Beliefs
    let beliefs_path = implicit_dir.join("beliefs.json");
    if let Ok(beliefs) = read_json::<Vec<JsonBelief>>(&beliefs_path) {
        for b in beliefs {
            let row = implicit::BeliefRow {
                id: b.id,
                proposition: b.proposition,
                category: b.category,
                confidence: b.confidence,
                evidence: serde_json::to_string(&b.evidence).unwrap_or_else(|_| "[]".into()),
                t1_grounding: b.t1_grounding.map(|g| format!("{g:?}").to_lowercase()),
                formed_at: b.formed_at,
                updated_at: b.updated_at,
                validation_count: b.validation_count,
                user_confirmed: b.user_confirmed,
            };
            if let Err(e) = implicit::upsert_belief(conn, &row) {
                stats.errors.push(format!("Belief {}: {e}", row.id));
            } else {
                stats.beliefs += 1;
            }
        }
    }

    // Trust accumulators
    let trust_path = implicit_dir.join("trust.json");
    if let Ok(trust) = read_json::<HashMap<String, JsonTrust>>(&trust_path) {
        for (domain, t) in trust {
            let row = implicit::TrustRow {
                domain,
                demonstrations: t.demonstrations,
                failures: t.failures,
                created_at: t.created_at,
                updated_at: t.updated_at,
                t1_grounding: t.t1_grounding.map(|g| format!("{g:?}").to_lowercase()),
            };
            if let Err(e) = implicit::upsert_trust(conn, &row) {
                stats.errors.push(format!("Trust {}: {e}", row.domain));
            } else {
                stats.trust_accumulators += 1;
            }
        }
    }

    // Belief graph
    let graph_path = implicit_dir.join("belief_graph.json");
    if let Ok(graph) = read_json::<JsonBeliefGraph>(&graph_path) {
        for imp in graph.implications {
            let row = implicit::ImplicationRow {
                from_belief: imp.from,
                to_belief: imp.to,
                strength: format!("{:?}", imp.strength).to_lowercase(),
                established_at: imp.established_at,
            };
            if let Err(e) = implicit::insert_implication(conn, &row) {
                stats.errors.push(format!("Implication: {e}"));
            } else {
                stats.implications += 1;
            }
        }
    }
}

/// Migrate the code tracker.
fn migrate_tracker(conn: &Connection, tracker_dir: &Path, stats: &mut MigrationStats) {
    if !tracker_dir.exists() {
        return;
    }

    // The tracker stores an index.json with project snapshots
    let index_path = tracker_dir.join("index.json");
    let index: TrackerIndex = match read_json(&index_path) {
        Ok(i) => i,
        Err(_) => return,
    };

    for (project_name, snapshot) in &index.projects {
        for (file_path, tracked) in &snapshot.files {
            let row = tracker::TrackedFileRow {
                id: None,
                project: project_name.clone(),
                file_path: file_path.clone(),
                content_hash: tracked.content_hash.clone(),
                file_size: tracked.size,
                tracked_at: tracked.tracked_at,
                mtime: tracked.mtime,
            };
            if let Err(e) = tracker::upsert(conn, &row) {
                stats
                    .errors
                    .push(format!("Tracked {project_name}/{file_path}: {e}"));
            } else {
                stats.tracked_files += 1;
            }
        }
    }
}

// ========== V2 Migration Functions ==========

/// Migrate decision audit log from JSONL.
fn migrate_decisions(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let decisions_path = claude_dir.join("decision-audit/decisions.jsonl");
    if !decisions_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&decisions_path) {
        Ok(c) => c,
        Err(e) => {
            stats
                .errors
                .push(format!("Cannot read decisions.jsonl: {e}"));
            return;
        }
    };

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: std::result::Result<JsonDecision, _> = serde_json::from_str(line);
        match parsed {
            Ok(d) => {
                let row = decisions::DecisionRow {
                    id: None,
                    timestamp: d.timestamp,
                    session_id: d.session_id,
                    tool: d.tool,
                    action: d.action,
                    target: d.target,
                    risk_level: d.risk_level,
                    reversible: d.reversible,
                };
                if let Err(e) = decisions::insert(conn, &row) {
                    stats.errors.push(format!("Decision insert: {e}"));
                } else {
                    stats.decisions += 1;
                }
            }
            Err(e) => {
                stats.errors.push(format!("Decision parse: {e}"));
            }
        }
    }
}

/// Migrate journal entries (architecture-significant decisions) from JSONL.
///
/// Reads `~/.claude/decision-audit/journal.jsonl` and maps the hook's
/// classification field to the decision_audit schema.
fn migrate_journal(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let journal_path = claude_dir.join("decision-audit/journal.jsonl");
    if !journal_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&journal_path) {
        Ok(c) => c,
        Err(e) => {
            stats.errors.push(format!("Cannot read journal.jsonl: {e}"));
            return;
        }
    };

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: std::result::Result<JsonJournalEntry, _> = serde_json::from_str(line);
        match parsed {
            Ok(j) => {
                let risk = classify_risk(&j.classification);
                let row = decisions::DecisionRow {
                    id: None,
                    timestamp: j.timestamp,
                    session_id: j.session_id,
                    tool: j.tool,
                    action: j.classification,
                    target: j.target,
                    risk_level: risk,
                    reversible: true, // git-tracked files are reversible
                };
                if let Err(e) = decisions::insert(conn, &row) {
                    // UNIQUE constraint violations are expected (re-sync)
                    let msg = e.to_string();
                    if !msg.contains("UNIQUE") {
                        stats.errors.push(format!("Journal insert: {e}"));
                    }
                } else {
                    stats.decisions += 1;
                }
            }
            Err(e) => {
                stats.errors.push(format!("Journal parse: {e}"));
            }
        }
    }
}

/// Map a classification to a risk level.
fn classify_risk(classification: &str) -> String {
    match classification {
        "dependency" | "infrastructure" | "ci" => "MEDIUM".to_string(),
        "hook" | "configuration" | "mcp" => "MEDIUM".to_string(),
        _ => "LOW".to_string(),
    }
}

/// Migrate tool usage telemetry from JSON.
fn migrate_tool_usage(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let telem_path = claude_dir.join("metrics/usage_telemetry.json");
    if !telem_path.exists() {
        return;
    }

    let parsed: std::result::Result<JsonUsageTelemetry, _> = read_json(&telem_path);
    let telem = match parsed {
        Ok(t) => t,
        Err(e) => {
            stats
                .errors
                .push(format!("Cannot parse usage_telemetry.json: {e}"));
            return;
        }
    };

    for (tool_name, tool_data) in &telem.tools {
        if tool_name.is_empty() {
            continue; // Skip empty-string entries
        }
        let row = telemetry::ToolUsageRow {
            tool_name: tool_name.clone(),
            total_calls: tool_data.total_calls,
            success_count: tool_data.success_count,
            failure_count: tool_data.failure_count,
            last_used: tool_data.last_used,
        };
        if let Err(e) = telemetry::upsert_tool_usage(conn, &row) {
            stats.errors.push(format!("Tool usage {tool_name}: {e}"));
        } else {
            stats.tool_usage += 1;
        }
    }
}

/// Migrate token efficiency from JSON.
fn migrate_token_efficiency(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let eff_path = claude_dir.join("metrics/token_efficiency.json");
    if !eff_path.exists() {
        return;
    }

    let parsed: std::result::Result<JsonTokenEfficiency, _> = read_json(&eff_path);
    let eff = match parsed {
        Ok(e) => e,
        Err(e) => {
            stats
                .errors
                .push(format!("Cannot parse token_efficiency.json: {e}"));
            return;
        }
    };

    for (session_id, sess_data) in &eff.by_session {
        let row = telemetry::TokenEfficiencyRow {
            session_id: session_id.clone(),
            action_count: sess_data.action_count,
            total_tokens: sess_data.total_tokens,
            started_at: sess_data.started_at,
        };
        if let Err(e) = telemetry::upsert_token_efficiency(conn, &row) {
            stats
                .errors
                .push(format!("Token efficiency {session_id}: {e}"));
        } else {
            stats.token_efficiency += 1;
        }
    }
}

/// Migrate tasks from ~/.claude/tasks/ directories.
fn migrate_tasks(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let tasks_dir = claude_dir.join("tasks");
    if !tasks_dir.exists() {
        return;
    }

    let session_dirs = match fs::read_dir(&tasks_dir) {
        Ok(d) => d,
        Err(e) => {
            stats.errors.push(format!("Cannot read tasks dir: {e}"));
            return;
        }
    };

    for entry in session_dirs.flatten() {
        let session_id = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_dir() {
            continue;
        }

        let task_files = match fs::read_dir(entry.path()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for task_entry in task_files.flatten() {
            let fname = task_entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".json") {
                continue;
            }

            let parsed: std::result::Result<JsonTask, _> = read_json(&task_entry.path());
            match parsed {
                Ok(t) => {
                    let row = knowledge::TaskHistoryRow {
                        id: None,
                        session_id: session_id.clone(),
                        task_id: t.id,
                        subject: t.subject,
                        description: t.description,
                        active_form: t.active_form.unwrap_or_default(),
                        status: t.status,
                        blocks: serde_json::to_string(&t.blocks).unwrap_or_else(|_| "[]".into()),
                        blocked_by: serde_json::to_string(&t.blocked_by)
                            .unwrap_or_else(|_| "[]".into()),
                    };
                    if let Err(e) = knowledge::upsert_task(conn, &row) {
                        stats
                            .errors
                            .push(format!("Task {session_id}/{}: {e}", row.task_id));
                    } else {
                        stats.tasks += 1;
                    }
                }
                Err(e) => {
                    stats
                        .errors
                        .push(format!("Task parse {session_id}/{fname}: {e}"));
                }
            }
        }
    }
}

/// Migrate handoff summaries from ~/.claude/handoffs/ directories.
fn migrate_handoffs(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let handoffs_dir = claude_dir.join("handoffs");
    if !handoffs_dir.exists() {
        return;
    }

    let project_dirs = match fs::read_dir(&handoffs_dir) {
        Ok(d) => d,
        Err(e) => {
            stats.errors.push(format!("Cannot read handoffs dir: {e}"));
            return;
        }
    };

    for entry in project_dirs.flatten() {
        let project = entry.file_name().to_string_lossy().to_string();
        if !entry.path().is_dir() {
            continue;
        }

        let md_files = match fs::read_dir(entry.path()) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for md_entry in md_files.flatten() {
            let fname = md_entry.file_name().to_string_lossy().to_string();
            if !fname.ends_with(".md") {
                continue;
            }

            let content = match fs::read_to_string(md_entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Parse handoff number from filename (e.g., "00097.md" -> 97)
            let handoff_number = fname.trim_end_matches(".md").parse::<i32>().unwrap_or(0);

            // Extract structured fields from markdown headers
            let row = parse_handoff_markdown(&project, handoff_number, &content);
            if let Err(e) = knowledge::upsert_handoff(conn, &row) {
                stats.errors.push(format!("Handoff {project}/{fname}: {e}"));
            } else {
                stats.handoffs += 1;
            }
        }
    }
}

/// Parse structured data from a handoff markdown file.
fn parse_handoff_markdown(project: &str, number: i32, content: &str) -> knowledge::HandoffRow {
    let mut session_id = String::new();
    let mut generated_at = String::new();
    let mut status = String::new();
    let mut duration = String::new();
    let mut files_modified = 0i32;
    let mut lines_written = 0i32;
    let mut commits = 0i32;
    let mut uncommitted = 0i32;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Session:") {
            session_id = trimmed.trim_start_matches("Session:").trim().to_string();
        } else if trimmed.starts_with("Generated:") {
            generated_at = trimmed.trim_start_matches("Generated:").trim().to_string();
        } else if trimmed.starts_with("Status:") {
            status = trimmed
                .trim_start_matches("Status:")
                .trim()
                .trim_matches('*')
                .to_string();
        } else if trimmed.starts_with("| Duration") {
            if let Some(val) = extract_table_value(trimmed) {
                duration = val;
            }
        } else if trimmed.starts_with("| Files modified") {
            if let Some(val) = extract_table_value(trimmed) {
                files_modified = val.parse().unwrap_or(0);
            }
        } else if trimmed.starts_with("| Lines written") {
            if let Some(val) = extract_table_value(trimmed) {
                lines_written = val.parse().unwrap_or(0);
            }
        } else if trimmed.starts_with("| Commits") && !trimmed.contains("Uncommitted") {
            if let Some(val) = extract_table_value(trimmed) {
                commits = val.parse().unwrap_or(0);
            }
        } else if trimmed.starts_with("| Uncommitted") {
            if let Some(val) = extract_table_value(trimmed) {
                uncommitted = val.parse().unwrap_or(0);
            }
        }
    }

    knowledge::HandoffRow {
        id: None,
        project: project.to_string(),
        handoff_number: number,
        session_id,
        generated_at,
        status,
        duration,
        files_modified,
        lines_written,
        commits,
        uncommitted,
        content: content.to_string(),
    }
}

/// Extract the value column from a markdown table row like `| Key | Value |`.
fn extract_table_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 3 {
        Some(parts[2].trim().to_string())
    } else {
        None
    }
}

/// Migrate antibodies from YAML.
fn migrate_antibodies(conn: &Connection, claude_dir: &Path, stats: &mut MigrationStats) {
    let ab_path = claude_dir.join("immunity/antibodies.yaml");
    if !ab_path.exists() {
        return;
    }

    let content = match fs::read_to_string(&ab_path) {
        Ok(c) => c,
        Err(e) => {
            stats
                .errors
                .push(format!("Cannot read antibodies.yaml: {e}"));
            return;
        }
    };

    // Parse YAML manually since we don't have a YAML dep — extract key fields
    // using line-based parsing of the well-structured YAML format
    let mut current_ab: Option<AntibodyBuilder> = None;
    let mut in_description = false;
    let mut in_detection = false;
    let mut in_response = false;
    let mut desc_lines: Vec<String> = Vec::new();
    let mut detection_lines: Vec<String> = Vec::new();
    let mut response_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // New antibody entry
        if trimmed.starts_with("- id:") {
            // Flush previous
            if let Some(ab) = current_ab.take() {
                flush_antibody(
                    conn,
                    ab,
                    &desc_lines,
                    &detection_lines,
                    &response_lines,
                    stats,
                );
            }
            desc_lines.clear();
            detection_lines.clear();
            response_lines.clear();
            in_description = false;
            in_detection = false;
            in_response = false;

            let id = trimmed.trim_start_matches("- id:").trim().to_string();
            current_ab = Some(AntibodyBuilder {
                id,
                ..Default::default()
            });
            continue;
        }

        if let Some(ref mut ab) = current_ab {
            if trimmed.starts_with("name:") {
                ab.name = trimmed.trim_start_matches("name:").trim().to_string();
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("threat_type:") {
                ab.threat_type = trimmed
                    .trim_start_matches("threat_type:")
                    .trim()
                    .to_string();
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("severity:") {
                ab.severity = trimmed.trim_start_matches("severity:").trim().to_string();
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("confidence:") {
                ab.confidence = trimmed
                    .trim_start_matches("confidence:")
                    .trim()
                    .parse()
                    .unwrap_or(0.5);
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("applications:") {
                ab.applications = trimmed
                    .trim_start_matches("applications:")
                    .trim()
                    .parse()
                    .unwrap_or(0);
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("false_positives:") {
                ab.false_positives = trimmed
                    .trim_start_matches("false_positives:")
                    .trim()
                    .parse()
                    .unwrap_or(0);
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("learned_from:") {
                ab.learned_from = trimmed
                    .trim_start_matches("learned_from:")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if trimmed == "description: |" {
                in_description = true;
                in_detection = false;
                in_response = false;
            } else if trimmed.starts_with("detection:") && !trimmed.contains("_") {
                in_description = false;
                in_detection = true;
                in_response = false;
            } else if trimmed.starts_with("response:") && !trimmed.contains("_") {
                in_description = false;
                in_detection = false;
                in_response = true;
            } else if trimmed.starts_with("validation:")
                || trimmed.starts_with("examples:")
                || trimmed.starts_with("reference:")
                || trimmed.starts_with("promoted_from:")
                || trimmed.starts_with("promoted_at:")
            {
                in_description = false;
                in_detection = false;
                in_response = false;
            } else if in_description {
                desc_lines.push(line.to_string());
            } else if in_detection {
                detection_lines.push(line.to_string());
            } else if in_response {
                response_lines.push(line.to_string());
            }
        }
    }

    // Flush final
    if let Some(ab) = current_ab.take() {
        flush_antibody(
            conn,
            ab,
            &desc_lines,
            &detection_lines,
            &response_lines,
            stats,
        );
    }
}

#[derive(Default)]
struct AntibodyBuilder {
    id: String,
    name: String,
    threat_type: String,
    severity: String,
    confidence: f64,
    applications: i32,
    false_positives: i32,
    learned_from: String,
}

fn flush_antibody(
    conn: &Connection,
    ab: AntibodyBuilder,
    desc_lines: &[String],
    detection_lines: &[String],
    response_lines: &[String],
    stats: &mut MigrationStats,
) {
    let description = desc_lines
        .iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    let detection = detection_lines.join("\n");
    let response = response_lines.join("\n");

    // Extract T1 grounding from description
    let t1_grounding = if description.contains("Violates →") || description.contains("Causality")
    {
        Some("causality".to_string())
    } else if description.contains("Violates ∂") || description.contains("Boundary") {
        Some("boundary".to_string())
    } else if description.contains("Violates π") || description.contains("Persistence") {
        Some("persistence".to_string())
    } else if description.contains("Violates ς") || description.contains("State") {
        Some("state".to_string())
    } else {
        None
    };

    let row = knowledge::AntibodyRow {
        id: ab.id.clone(),
        name: ab.name,
        threat_type: ab.threat_type,
        severity: ab.severity,
        description,
        detection,
        response,
        confidence: ab.confidence,
        applications: ab.applications,
        false_positives: ab.false_positives,
        learned_from: ab.learned_from,
        t1_grounding,
    };

    if let Err(e) = knowledge::upsert_antibody(conn, &row) {
        stats.errors.push(format!("Antibody {}: {e}", ab.id));
    } else {
        stats.antibodies += 1;
    }
}

// ========== V2 JSON deserialization types ==========

#[derive(Debug, Deserialize)]
struct JsonDecision {
    timestamp: DateTime<Utc>,
    session_id: String,
    tool: String,
    action: String,
    target: String,
    risk_level: String,
    reversible: bool,
}

/// Journal entry from the decision-journal hook (journal.jsonl).
#[derive(Debug, Deserialize)]
struct JsonJournalEntry {
    #[serde(alias = "ts")]
    timestamp: DateTime<Utc>,
    session_id: String,
    tool: String,
    target: String,
    classification: String,
    #[allow(dead_code)]
    rationale_prompt: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct JsonUsageTelemetry {
    #[serde(default)]
    tools: HashMap<String, JsonToolStats>,
}

#[derive(Debug, Deserialize)]
struct JsonToolStats {
    #[serde(default)]
    total_calls: i64,
    #[serde(default)]
    success_count: i64,
    #[serde(default)]
    failure_count: i64,
    #[serde(default)]
    last_used: i64,
}

#[derive(Debug, Deserialize)]
struct JsonTokenEfficiency {
    #[serde(default)]
    by_session: HashMap<String, JsonSessionEfficiency>,
}

#[derive(Debug, Deserialize)]
struct JsonSessionEfficiency {
    #[serde(default)]
    action_count: i64,
    #[serde(default)]
    total_tokens: i64,
    #[serde(default)]
    started_at: i64,
}

#[derive(Debug, Deserialize)]
struct JsonTask {
    id: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    description: String,
    #[serde(default, rename = "activeForm")]
    active_form: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    blocks: Vec<String>,
    #[serde(default, rename = "blockedBy")]
    blocked_by: Vec<String>,
}

// ========== V1 JSON deserialization types (matching existing Brain format) ==========

#[derive(Debug, Deserialize)]
struct SessionEntry {
    id: String,
    created_at: DateTime<Utc>,
    project: Option<String>,
    git_commit: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArtifactMeta {
    artifact_type: String,
    summary: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    current_version: u32,
    #[serde(default)]
    tags: Vec<String>,
    custom: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JsonPreference {
    value: serde_json::Value,
    description: Option<String>,
    confidence: f64,
    reinforcement_count: u32,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct JsonPattern {
    id: String,
    pattern_type: String,
    description: String,
    #[serde(default)]
    examples: Vec<String>,
    detected_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    updated_at: DateTime<Utc>,
    confidence: f64,
    occurrence_count: u32,
    t1_grounding: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JsonCorrection {
    mistake: String,
    correction: String,
    context: Option<String>,
    learned_at: DateTime<Utc>,
    application_count: u32,
}

#[derive(Debug, Deserialize)]
struct JsonBelief {
    id: String,
    proposition: String,
    category: String,
    confidence: f64,
    #[serde(default)]
    evidence: Vec<serde_json::Value>,
    t1_grounding: Option<serde_json::Value>,
    formed_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    validation_count: u32,
    #[serde(default)]
    user_confirmed: bool,
}

#[derive(Debug, Deserialize)]
struct JsonTrust {
    demonstrations: u32,
    failures: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    t1_grounding: Option<serde_json::Value>,
}

#[derive(Debug, Default, Deserialize)]
struct JsonBeliefGraph {
    #[serde(default)]
    implications: Vec<JsonImplication>,
}

#[derive(Debug, Deserialize)]
struct JsonImplication {
    from: String,
    to: String,
    strength: serde_json::Value,
    established_at: DateTime<Utc>,
}

#[derive(Debug, Default, Deserialize)]
struct TrackerIndex {
    #[serde(default)]
    projects: HashMap<String, ProjectSnapshot>,
}

#[derive(Debug, Deserialize)]
struct ProjectSnapshot {
    #[serde(default)]
    files: HashMap<String, TrackedFileJson>,
}

#[derive(Debug, Deserialize)]
struct TrackedFileJson {
    content_hash: String,
    #[serde(default)]
    size: u64,
    tracked_at: DateTime<Utc>,
    mtime: DateTime<Utc>,
}

/// Read a JSON file and deserialize it.
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)
        .map_err(|e| DbError::Migration(format!("Cannot read {}: {e}", path.display())))?;
    serde_json::from_str(&content)
        .map_err(|e| DbError::Migration(format!("Cannot parse {}: {e}", path.display())))
}

fn infer_type(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("task") {
        "task".into()
    } else if lower.contains("plan") || lower.contains("implementation") {
        "implementation_plan".into()
    } else if lower.contains("walkthrough") || lower.contains("progress") {
        "walkthrough".into()
    } else if lower.contains("review") {
        "review".into()
    } else if lower.contains("research") {
        "research".into()
    } else if lower.contains("decision") {
        "decision".into()
    } else {
        "custom".into()
    }
}
