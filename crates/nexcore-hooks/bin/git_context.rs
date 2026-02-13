//! Git Context Hook
//!
//! **Event:** `SessionStart`
//! **Action:** Injects current git branch and last commit into session context
//!
//! # Purpose
//!
//! Provides Claude Code with repository awareness at session start, enabling:
//! - Branch-aware suggestions (feature vs main branch behavior)
//! - Recent commit context for continuity
//! - Repository state visibility without manual queries
//!
//! # Behavior
//!
//! 1. Reads `SessionStart` event from stdin
//! 2. Executes `git rev-parse --abbrev-ref HEAD` for current branch
//! 3. Executes `git log -1 --oneline` for last commit summary
//! 4. Outputs combined context via `exit_with_session_context()`
//!
//! # Output Format
//!
//! ```text
//! Git context: Branch 'feature/my-feature', last commit: 'abc123 Fix bug in handler'
//! ```
//!
//! # Exit Codes
//!
//! - `0` (success): Context injected, or gracefully skipped if not a git repo
//! - Never blocks (always exits 0)
//!
//! # Configuration (settings.json)
//!
//! ```json
//! {
//!   "hooks": {
//!     "SessionStart": [{
//!       "command": "~/nexcore/crates/nexcore-hooks/target/release/git_context",
//!       "timeout": 5000
//!     }]
//!   }
//! }
//! ```

use nexcore_hooks::{exit_success_auto, exit_with_session_context, read_input};
use std::process::Command;

fn get_git_info() -> Option<String> {
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let last_commit = Command::new("git")
        .args(["log", "-1", "--oneline"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    Some(format!(
        "Git context: Branch '{}', last commit: '{}'",
        branch, last_commit
    ))
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    if let Some(git_info) = get_git_info() {
        exit_with_session_context(&git_info);
    }

    exit_success_auto();
}
