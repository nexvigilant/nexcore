//! Task Completion Auditor Hook
//!
//! Stop hook that audits incomplete tasks before session ends.
//!
//! # Event
//! Stop
//!
//! # Purpose
//! Warns about orphaned tasks to prevent work from being forgotten.
//!
//! # Checks
//! - Tasks left in_progress
//! - Tasks created but never started
//! - Blocked tasks with no resolution plan
//!
//! # Note
//! Tracks task-related tool uses via session state or transcript analysis.
//!
//! # Exit Codes
//! - 0: Always (warning only, non-blocking)

use nexcore_hooks::state::SessionState;
use std::fs;
use std::path::PathBuf;

/// Get task state file path
fn task_state_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("task_state.json")
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct TaskState {
    tasks_created: u32,
    tasks_completed: u32,
    tasks_in_progress: u32,
    last_updated: f64,
}

impl TaskState {
    fn load() -> Self {
        let path = task_state_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }
}

/// Count task operations in a single transcript line
/// O(1) per line - fixed number of string comparisons
/// Only counts actual tool invocations, not mentions in conversation or system reminders
fn count_task_ops_in_line(line: &str) -> (bool, bool, bool) {
    // Skip system reminders and conversation - they often mention tool names
    if line.contains("system-reminder") || line.contains("consider using") {
        return (false, false, false);
    }

    // Look for exact tool invocation patterns only:
    // - XML: name="TaskCreate" (antml:invoke format)
    // - JSON: "name": "TaskCreate" (API format)
    let has_create =
        line.contains("name=\"TaskCreate\"") || line.contains("\"name\": \"TaskCreate\"");
    let has_update_completed = (line.contains("name=\"TaskUpdate\"")
        || line.contains("\"name\": \"TaskUpdate\""))
        && line.contains("\"completed\"");
    let has_update_in_progress = (line.contains("name=\"TaskUpdate\"")
        || line.contains("\"name\": \"TaskUpdate\""))
        && line.contains("\"in_progress\"");

    (has_create, has_update_completed, has_update_in_progress)
}

/// Analyze transcript for task-related tool uses
/// O(n) where n is number of lines - single pass with O(1) work per line
fn analyze_transcript_for_tasks(transcript_path: &Option<String>) -> (u32, u32, u32) {
    let path = match transcript_path {
        Some(p) => p,
        None => return (0, 0, 0),
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0, 0),
    };

    let mut created = 0u32;
    let mut completed = 0u32;
    let mut in_progress = 0u32;

    for line in content.lines() {
        let (is_create, is_completed, is_in_progress) = count_task_ops_in_line(line);
        if is_create {
            created += 1;
        }
        if is_completed {
            completed += 1;
        }
        if is_in_progress {
            in_progress += 1;
        }
    }

    // Rough estimate: in_progress tasks that weren't completed
    let orphaned = in_progress.saturating_sub(completed);
    (created, completed, orphaned)
}

fn main() {
    let _state = SessionState::load();
    let task_state = TaskState::load();

    // Read stdin for hook input
    let mut buffer = String::new();
    let _ = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer);
    let input: serde_json::Value = serde_json::from_str(&buffer).unwrap_or_default();

    let transcript_path = input
        .get("transcript_path")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Analyze transcript for task activity
    let (created, completed, orphaned) = analyze_transcript_for_tasks(&transcript_path);

    // Merge with persisted task state
    let total_created = created + task_state.tasks_created;
    let total_completed = completed + task_state.tasks_completed;
    let total_orphaned = orphaned + task_state.tasks_in_progress;

    let has_issues = total_orphaned > 0 || (total_created > 0 && total_completed == 0);

    let message = if !has_issues && total_created == 0 {
        "📋 No task activity this session".to_string()
    } else if !has_issues {
        format!(
            "✅ Task audit passed\n\
             - Created: {}\n\
             - Completed: {}",
            total_created, total_completed
        )
    } else {
        let mut msg = format!(
            "⚠️ TASK AUDIT WARNING\n\n\
             - Tasks created: {}\n\
             - Tasks completed: {}\n\
             - Tasks potentially orphaned: {}\n\n",
            total_created, total_completed, total_orphaned
        );

        if total_orphaned > 0 {
            msg.push_str(
                "Orphaned tasks may indicate incomplete work.\n\
                 Consider:\n\
                 1. Completing in-progress tasks\n\
                 2. Marking tasks as completed if done\n\
                 3. Documenting blockers in the handoff\n",
            );
        }

        if total_created > 0 && total_completed == 0 {
            msg.push_str(
                "\nNo tasks were completed this session.\n\
                 Ensure task statuses are updated as work progresses.\n",
            );
        }

        msg
    };

    let output = serde_json::json!({
        "continue": true,
        "decision": "approve", // Never block, just warn
        "stopReason": format!("Task audit: {} created, {} completed, {} orphaned",
            total_created, total_completed, total_orphaned),
        "systemMessage": message
    });
    println!("{}", output);
}
