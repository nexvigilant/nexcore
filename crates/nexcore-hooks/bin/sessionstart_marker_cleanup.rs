//! SessionStart Marker Cleanup Hook
//!
//! SessionStart hook that cleans up stale contribution markers from previous sessions.
//!
//! # Event
//! SessionStart
//!
//! # Purpose
//! Removes leftover .contribution_marker files to ensure fresh session state.
//!
//! # Related Hooks
//! - stop_contribution_reminder: Reminds to commit at session end
//! - posttool_contribution_marker: Sets marker after significant edits
//!
//! # Exit Codes
//! - 0: Always (cleanup is best-effort, never blocks)

use std::fs;
use std::path::PathBuf;

fn main() {
    let marker_path = contribution_marker_path();

    // Remove stale marker if it exists (from previous session)
    // Best-effort: if removal fails, we proceed anyway
    if marker_path.exists() {
        if let Err(e) = fs::remove_file(&marker_path) {
            eprintln!("Note: Could not remove stale marker: {}", e);
        }
    }

    std::process::exit(0);
}

fn contribution_marker_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join(".contribution_marker")
}
