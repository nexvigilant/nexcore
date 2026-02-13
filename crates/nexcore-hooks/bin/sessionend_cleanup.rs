//! Session End Cleanup Hook
//!
//! Event: SessionEnd
//! Performs final cleanup and logs session metrics.

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use std::fs;
use std::path::Path;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Load session state for metrics
    let state = SessionState::load();

    // Calculate session metrics
    let verified_crates = state.verified_crates.len();
    let unverified_constructs = state.unverified_constructs.len();
    let lines_written = state.lines_since_verification;
    let files_modified = state.files_since_verification;

    // Clean up session state backup file
    let claude_dir = Path::new(&input.cwd).join(".claude");
    let _ = fs::remove_file(claude_dir.join("session_state_backup.json"));

    // Reset session state for next session
    let mut state_mut = SessionState::load();
    state_mut.reset();

    // Generate session summary
    let summary = format!(
        "Session: {} crates, {} lines, {} files",
        verified_crates, lines_written, files_modified
    );

    // Log any remaining unverified constructs
    if unverified_constructs > 0 {
        eprintln!("⚠️ {} unverified constructs", unverified_constructs);
    }

    HookOutput::allow()
        .with_system_message(format!("📊 {}", summary))
        .emit();
    std::process::exit(0);
}
