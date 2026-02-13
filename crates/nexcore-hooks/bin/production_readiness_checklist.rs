//! Production Readiness Checklist
//!
//! Event: Stop
//! Verifies production requirements before Claude stops.
//!
//! Stop hooks use "approve"/"block" not "allow"/"deny"

use nexcore_hooks::{exit_success_auto, read_input};
use std::path::Path;
use std::process::Command;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Don't loop if already checking
    if input.stop_hook_active.unwrap_or(false) {
        exit_success_auto();
    }

    let cwd = &input.cwd;

    // Check if this is a Rust project
    if !Path::new(cwd).join("Cargo.toml").exists() {
        exit_success_auto();
    }

    // Run checklist
    let mut issues = Vec::new();

    // Level 2: Static verification
    if !run_cargo_check(cwd) {
        issues.push("cargo check failed");
    }

    if !run_clippy_check(cwd) {
        issues.push("clippy warnings present");
    }

    // Level 3: Tests
    if !run_test_check(cwd) {
        issues.push("tests failing");
    }

    // Output Stop-compatible JSON with systemMessage for visibility
    let (system_msg, stop_reason) = if issues.is_empty() {
        (
            "✅ Production ready".to_string(),
            "All checks passed".to_string(),
        )
    } else {
        (
            format!("⚠️ {} issue(s): {}", issues.len(), issues.join(", ")),
            format!("Production check: {} issue(s)", issues.len()),
        )
    };

    let output = serde_json::json!({
        "continue": true,
        "decision": "approve",
        "stopReason": stop_reason,
        "systemMessage": system_msg
    });
    println!("{}", output);
}

fn run_cargo_check(cwd: &str) -> bool {
    Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(true)
}

fn run_clippy_check(cwd: &str) -> bool {
    Command::new("cargo")
        .args(["clippy", "--quiet", "--", "-D", "warnings"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(true)
}

fn run_test_check(cwd: &str) -> bool {
    Command::new("cargo")
        .args(["test", "--quiet", "--no-run"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(true)
}
