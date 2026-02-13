//! Agent Error Detector Hook
//!
//! Event: PostToolUse (matcher: Bash)
//!
//! Detects compiler errors from cargo commands and auto-triggers
//! the appropriate diagnostic agent:
//! - Borrow errors (E0382, E0499, etc.) → rust-borrow-doctor
//! - Lifetime errors (E0597, E0621, etc.) → rust-compiler-doctor
//! - Async errors (E0728, E0732) → rust-async-expert
//! - General errors → rust-compiler-doctor

use nexcore_hooks::agent_triggers::errors::detect_errors;
use nexcore_hooks::{exit_success_auto, exit_warn, read_input};

/// Check if the command is a cargo compilation command
fn is_cargo_command(cmd: &str) -> bool {
    let patterns = [
        "cargo check",
        "cargo build",
        "cargo test",
        "cargo clippy",
        "cargo run",
    ];
    patterns.iter().any(|p| cmd.contains(p))
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get the command that was run
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_success_auto(),
    };

    let command = tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Only analyze cargo commands
    if !is_cargo_command(command) {
        exit_success_auto();
    }

    // Get the command output
    let response = match &input.tool_response {
        Some(v) => v,
        None => exit_success_auto(),
    };

    // Combine stdout and stderr
    let stdout = response
        .get("output")
        .or_else(|| response.get("stdout"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let stderr = response
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let combined = format!("{}\n{}", stdout, stderr);

    // Check exit code - only trigger on failures
    let exit_code = response
        .get("exit_code")
        .or_else(|| response.get("exitCode"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if exit_code == 0 && !combined.contains("error[E") {
        exit_success_auto();
    }

    // Detect and classify errors
    if let Some(detection) = detect_errors(&combined) {
        exit_warn(&detection.to_context());
    }

    exit_success_auto();
}
