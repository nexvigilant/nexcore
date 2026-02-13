//! PostToolUse hook: Creates contribution marker when editing skills/hooks/MCP
//!
//! Works with stop_contribution_reminder to enforce session contributions.
//!
//! Exit codes:
//! - 0: Always (marker created if qualifying edit, otherwise no-op)

use nexcore_hooks::{HookInput, read_input};
use std::fs;
use std::path::PathBuf;

fn main() {
    let input: HookInput = match read_input() {
        Some(i) => i,
        None => std::process::exit(0),
    };

    // Only process Write/Edit tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        std::process::exit(0);
    }

    // Check if path qualifies as a contribution
    if let Some(path) = input.get_file_path() {
        if is_contribution_path(path) {
            create_contribution_marker();
        }
    }

    std::process::exit(0);
}

fn is_contribution_path(path: &str) -> bool {
    path.contains("/skills/")
        || path.contains("/hooks/")
        || path.contains("nexcore-hooks")
        || path.contains("nexcore-mcp")
        || path.contains(".mcp.json")
        || path.contains("/mcp/")
}

fn create_contribution_marker() {
    let marker_path = contribution_marker_path();

    // Ensure parent directory exists
    if let Some(parent) = marker_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Could not create marker directory: {}", e);
            return;
        }
    }

    // Create empty marker file
    if let Err(e) = fs::write(&marker_path, "") {
        eprintln!("Warning: Could not create contribution marker: {}", e);
    }
}

fn contribution_marker_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join(".contribution_marker")
}
