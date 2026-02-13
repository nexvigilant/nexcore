//! Session Restore - SessionStart Hook
//!
//! Restores previous session context from the brain system on session start.
//! Injects context via stderr to provide Claude with continuity information.
//!
//! Hook Protocol:
//! - Input: JSON on stdin with session_id, cwd, etc.
//! - Output: Empty JSON `{}` on stdout
//! - Context injection: stderr (displayed to Claude)
//! - Exit: 0 = pass (always, never blocks)

use chrono::{DateTime, Utc};
use nexcore_brain::{Artifact, BrainSession, CodeTracker, ImplicitKnowledge};
use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::pass;

const HOOK_NAME: &str = "session-restore";
use serde::Deserialize;
use std::io::{self, Read};

/// SessionStart input structure from Claude Code
/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(String) via Option.
#[derive(Debug, Deserialize)]
struct SessionStartInput {
    #[serde(default)]
    #[allow(dead_code)]
    session_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

/// Context to be restored and injected
/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(String/bool) via Vec/Option/tuple and T3(chrono::DateTime<Utc>).
struct RestoredContext {
    session_id: String,
    resumed_at: DateTime<Utc>,
    task: Option<String>,
    plan: Option<String>,
    tracked_files: Vec<TrackedFileStatus>,
    preferences: Vec<(String, String)>,
}

/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(String, bool).
struct TrackedFileStatus {
    path: String,
    modified: bool,
}

fn main() {
    // Read stdin (SessionStart event data)
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }

    // Parse input (gracefully handle missing fields)
    let input: SessionStartInput = serde_json::from_str(&buffer).unwrap_or(SessionStartInput {
        session_id: None,
        cwd: None,
    });

    // Try to restore context from brain system
    let context = match restore_context(&input) {
        Some(ctx) => ctx,
        None => {
            // No previous session or failed to load - start fresh
            eprintln!("\n--- Session Context: Fresh Start ---\n");
            pass();
        }
    };

    // Inject context via stderr
    inject_context(&context);

    // Emit cytokine signal (TGF-beta = regulation, context restored)
    emit_hook_completed(HOOK_NAME, 0, "context_restored");

    pass();
}

/// Attempt to restore context from the brain system
fn restore_context(input: &SessionStartInput) -> Option<RestoredContext> {
    // Get the latest brain session
    let session = BrainSession::load_latest().ok()?;
    let session_id = session.id.to_string();

    // Load task.md artifact
    let task = session
        .get_artifact("task.md", None)
        .ok()
        .map(|a| summarize_artifact(&a));

    // Load plan.md artifact
    let plan = session
        .get_artifact("plan.md", None)
        .ok()
        .map(|a| summarize_artifact(&a));

    // If neither task nor plan exists, this is effectively a fresh session
    if task.is_none() && plan.is_none() {
        return None;
    }

    // Load tracked files status
    let tracked_files = load_tracked_files(input.cwd.as_deref());

    // Load implicit preferences (top 5 highest confidence)
    let preferences = load_top_preferences();

    Some(RestoredContext {
        session_id,
        resumed_at: Utc::now(),
        task,
        plan,
        tracked_files,
        preferences,
    })
}

/// Summarize an artifact for context injection
fn summarize_artifact(artifact: &Artifact) -> String {
    let content = &artifact.content;

    // If content is short, return as-is
    if content.len() <= 500 {
        return content.clone();
    }

    // Otherwise, return first 500 chars with truncation indicator
    let truncated: String = content.chars().take(500).collect();
    format!(
        "{truncated}\n\n[... truncated, {} total chars ...]",
        content.len()
    )
}

/// Load tracked files and check modification status
fn load_tracked_files(cwd: Option<&str>) -> Vec<TrackedFileStatus> {
    let project = cwd.unwrap_or("default");

    let tracker = match CodeTracker::load(project) {
        Ok(t) => t,
        Err(_err) => return Vec::new(),
    };

    tracker
        .list_files()
        .iter()
        .take(10) // Limit to 10 most relevant files
        .map(|tf| {
            let modified = tracker.has_changed(&tf.path).unwrap_or(false);
            TrackedFileStatus {
                path: tf.path.display().to_string(),
                modified,
            }
        })
        .collect()
}

/// Load top 5 preferences by confidence
fn load_top_preferences() -> Vec<(String, String)> {
    let implicit = match ImplicitKnowledge::load() {
        Ok(i) => i,
        Err(_err) => return Vec::new(),
    };

    let mut prefs: Vec<_> = implicit
        .list_preferences()
        .iter()
        .map(|p| (p.key.clone(), p.value.clone(), p.confidence))
        .collect();

    // Sort by confidence descending
    prefs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Take top 5 and format values
    prefs
        .into_iter()
        .take(5)
        .map(|(k, v, _)| {
            let value_str = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            (k, value_str)
        })
        .collect()
}

/// Inject restored context via stderr
fn inject_context(ctx: &RestoredContext) {
    eprintln!();
    eprintln!(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
    );
    eprintln!("\u{1F4CB} SESSION CONTINUITY CONTEXT");
    eprintln!(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
    );
    eprintln!("Session: {}", ctx.session_id);
    eprintln!(
        "Resumed: {}",
        ctx.resumed_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    eprintln!();

    // Active task
    eprintln!("\u{1F4CC} ACTIVE TASK");
    match &ctx.task {
        Some(task) => eprintln!("{task}"),
        None => eprintln!("No active task"),
    }
    eprintln!();

    // Current plan
    eprintln!("\u{1F4DD} CURRENT PLAN");
    match &ctx.plan {
        Some(plan) => eprintln!("{plan}"),
        None => eprintln!("No active plan"),
    }
    eprintln!();

    // Tracked files (if any)
    if !ctx.tracked_files.is_empty() {
        eprintln!("\u{1F50D} TRACKED FILES");
        for tf in &ctx.tracked_files {
            let status = if tf.modified { "modified" } else { "unchanged" };
            eprintln!("- {} ({})", tf.path, status);
        }
        eprintln!();
    }

    // Preferences (if any)
    if !ctx.preferences.is_empty() {
        eprintln!("\u{2699}\u{FE0F} PREFERENCES");
        for (key, value) in &ctx.preferences {
            eprintln!("- {key}: {value}");
        }
        eprintln!();
    }

    eprintln!(
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
    );
    eprintln!();
}
