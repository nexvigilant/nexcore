//! Stop hook: Requires one contribution to skill/hook/MCP before ending
//!
//! Uses marker file to track contributions across the session.
//! PostToolUse hooks should create the marker when editing skills/hooks/MCP.
//! SessionStart hook should clean up stale markers from previous sessions.
//!
//! Exit codes:
//! - 0: Allow (contribution detected via marker)
//! - 2: Block (no contribution made)

use std::path::PathBuf;

fn main() {
    // Check for contribution marker file
    let marker_path = contribution_marker_path();

    if marker_path.exists() {
        // Contribution was made this session - allow
        // NOTE: Don't remove marker here! Stop fires every turn, not just session end.
        // Marker cleanup happens on next SessionStart.
        std::process::exit(0);
    } else {
        // No contribution detected
        eprintln!(
            "🛑 Contribution required: Update a skill, hook, or MCP tool before ending session."
        );
        std::process::exit(2);
    }
}

fn contribution_marker_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join(".contribution_marker")
}
