//! # Session Context Hook
//!
//! `SessionStart` hook that assembles project context for Claude Code sessions.
//!
//! ## Purpose
//!
//! Provides immediate situational awareness at session start by collecting:
//! - **Session ID**: Truncated identifier for tracking
//! - **Git state**: Current branch and change count
//! - **Project type**: Auto-detected from manifest files
//! - **Working directory**: Current path context
//!
//! ## Event
//!
//! - **Trigger**: `SessionStart`
//! - **Output**: Context string via `exit_with_session_context()`
//!
//! ## Detection Logic
//!
//! | File Present | Project Type |
//! |--------------|--------------|
//! | `Cargo.toml` | Rust |
//! | `package.json` | Node.js |
//! | `pyproject.toml` | Python |
//! | `go.mod` | Go |
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success (context added to session) |
//!
//! ## Example Output
//!
//! ```text
//! Session Context:
//!   Session: a1b2c3d4 (cli)
//!   Git: main (3 changes)
//!   Project: Rust
//!   CWD: /home/user/project
//! ```
//!
//! ## Configuration
//!
//! Add to `~/.claude/settings.json`:
//!
//! ```json
//! {
//!   "hooks": {
//!     "SessionStart": [{
//!       "command": "session_context",
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
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())?;

    let status = Command::new("git")
        .args(["status", "--short"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())?;

    let changes = if status.is_empty() {
        "clean".to_string()
    } else {
        let count = status.lines().count();
        format!("{} changes", count)
    };

    Some(format!("Git: {} ({})", branch, changes))
}

fn get_project_type() -> Option<&'static str> {
    if std::path::Path::new("Cargo.toml").exists() {
        Some("Rust")
    } else if std::path::Path::new("package.json").exists() {
        Some("Node.js")
    } else if std::path::Path::new("pyproject.toml").exists() {
        Some("Python")
    } else if std::path::Path::new("go.mod").exists() {
        Some("Go")
    } else {
        None
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let mut context = Vec::new();

    // Add session info
    context.push(format!(
        "Session: {} (cli)",
        input.session_id.chars().take(8).collect::<String>()
    ));

    // Add git info
    if let Some(git) = get_git_info() {
        context.push(git);
    }

    // Add project type
    if let Some(project_type) = get_project_type() {
        context.push(format!("Project: {}", project_type));
    }

    // Add CWD
    context.push(format!("CWD: {}", input.cwd));

    // Output context
    if !context.is_empty() {
        let ctx = format!("Session Context:\n  {}", context.join("\n  "));
        exit_with_session_context(&ctx);
    }

    exit_success_auto();
}
