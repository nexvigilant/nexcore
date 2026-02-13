//! # Context Integrity Hook
//!
//! **Event:** `SessionStart`
//! **Blocking:** No (always continues)
//! **Purpose:** Verify project state and detect context drift on session start.
//!
//! ## What It Does
//!
//! 1. **Git State Check** - Detects repo, branch, uncommitted changes
//! 2. **Compilation Check** - Runs `cargo check` to verify project compiles
//! 3. **Drift Detection** - Flags if code that passed last session now fails
//! 4. **State Persistence** - Updates `~/.claude/session_state.json`
//!
//! ## Output Format
//!
//! ```json
//! {
//!   "continue": true,
//!   "stopReason": "Context: ✓ Passes | 3 uncommitted | Branch: main"
//! }
//! ```
//!
//! ## Drift Detection
//!
//! If `last_verification_result == "success"` but current compilation fails,
//! adds a high-confidence assumption requiring review.
//!
//! ## Dependencies
//!
//! - `git` (for repo state)
//! - `cargo` (for compilation check)
//! - `nexcore_hooks::state::SessionState`

use nexcore_hooks::state::SessionState;
use std::process::Command;

fn get_git_info() -> (bool, String, usize) {
    let is_repo = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !is_repo {
        return (false, String::new(), 0);
    }

    let branch = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let changes = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    (true, branch, changes)
}

fn check_compilation() -> (bool, usize, usize) {
    let output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .output();

    match output {
        Ok(o) => {
            let compiles = o.status.success();
            let stderr = String::from_utf8_lossy(&o.stderr);
            let errors = stderr.matches("error").count();
            let warnings = stderr.matches("warning").count();
            (compiles, errors, warnings)
        }
        Err(_) => (false, 0, 0),
    }
}

fn main() {
    let (is_repo, branch, uncommitted) = get_git_info();
    let (compiles, errors, _warnings) = check_compilation();

    // Load or initialize session state
    let mut state = SessionState::load();

    // Check for context drift
    let drift_detected = if state.last_verification_result == "success" && !compiles {
        // Code that was passing now fails - significant drift
        state.add_assumption(
            "Project state has drifted since last session",
            "high",
            "Review changes and run cargo check",
        );
        true
    } else {
        false
    };

    // Update state based on compilation result
    if compiles {
        state.record_verification("success");
    }

    let _ = state.save();

    let comp_status = if compiles {
        "✓ Passes".to_string()
    } else {
        format!("✗ {} errors", errors)
    };

    let drift_note = if drift_detected {
        " [DRIFT DETECTED]"
    } else {
        ""
    };

    let output = serde_json::json!({
        "continue": true,
        "stopReason": format!(
            "Context: {} | {} uncommitted | Branch: {}{}",
            comp_status,
            uncommitted,
            if is_repo { &branch } else { "N/A" },
            drift_note
        )
    });

    println!("{}", output);
}
