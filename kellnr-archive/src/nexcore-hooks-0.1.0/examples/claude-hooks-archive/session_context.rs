//! Session context loader hook.
//!
//! Loads project context at session start (git info, project structure, etc.).

use claude_hooks::{
    exit_success, read_input, write_text,
    input::SessionStartInput,
    HookResult,
};
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

fn main() -> HookResult<()> {
    let input: SessionStartInput = read_input()?;

    let mut context = Vec::new();

    // Add session info
    context.push(format!("Session: {} ({})",
        input.common.session_id.chars().take(8).collect::<String>(),
        format!("{:?}", input.source).to_lowercase()
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
    context.push(format!("CWD: {}", input.common.cwd));

    // Output context
    if !context.is_empty() {
        write_text(&format!("Session Context:\n  {}", context.join("\n  ")))?;
    }

    exit_success();
}
