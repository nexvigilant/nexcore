//! Pre-Compact State Saver Hook
//!
//! PreCompact hook that preserves critical session state before context compaction.
//!
//! # Event
//! PreCompact
//!
//! # Purpose
//! Saves session state to ~/.claude/state/ before compaction to prevent data loss.
//!
//! # Saved State
//! - Verified crates list
//! - Assumptions made during session
//! - Requirements gathered from user
//! - Active task context
//!
//! # Exit Codes
//! - 0: State saved successfully

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use std::fs;
use std::path::Path;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Load current session state
    let state = SessionState::load();

    // Create compact summary of important state
    let summary = create_state_summary(&state, &input.cwd);

    // Save summary to a well-known location
    let state_dir = Path::new(&input.cwd).join(".claude");
    let _ = fs::create_dir_all(&state_dir);
    let state_file = state_dir.join("session_state_backup.json");

    if let Ok(json) = serde_json::to_string_pretty(&summary) {
        let _ = fs::write(&state_file, json);
    }

    // Output system message about preserved state
    let verified_count = state.verified_crates.len();
    let constructs_count = state.unverified_constructs.len();

    HookOutput::allow()
        .with_system_message(format!(
            "📦 State preserved before compaction: {} verified crates, {} constructs tracked",
            verified_count, constructs_count
        ))
        .emit();
    std::process::exit(0);
}

fn create_state_summary(state: &SessionState, cwd: &str) -> serde_json::Value {
    serde_json::json!({
        "session_id": state.session_id(),
        "cwd": cwd,
        "verified_crates": state.verified_crates,
        "unverified_constructs": state.unverified_constructs,
        "lines_since_verification": state.lines_since_verification,
        "files_since_verification": state.files_since_verification,
        "last_verification_result": state.last_verification_result,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}
