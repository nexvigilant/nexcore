//! PostToolUse hook: Cargo check gate after N lines written
//!
//! Tracks cumulative lines written by agent and triggers cargo check
//! after threshold is exceeded. Blocks further writes until check passes.
//!
//! Exit codes:
//! - 0: Allow (under threshold or cargo check passed)
//! - 2: Block (threshold exceeded and cargo check failed)

use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

/// Lines written threshold before requiring cargo check
const LINES_THRESHOLD: usize = 400;

/// State file for tracking lines written
fn state_file_path() -> PathBuf {
    let tmp = env::var("TMPDIR")
        .or_else(|_| env::var("TMP"))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(tmp).join("claude-cargo-check-gate-state.json")
}

#[derive(Debug, Default, Deserialize, serde::Serialize)]
struct GateState {
    lines_written: usize,
    last_check_passed: bool,
    workspace_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HookInput {
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    file_path: Option<String>,
    content: Option<String>,
    new_string: Option<String>,
}

fn load_state() -> GateState {
    let path = state_file_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        GateState::default()
    }
}

fn save_state(state: &GateState) {
    let path = state_file_path();
    if let Ok(json) = serde_json::to_string(state) {
        // Best-effort save - state loss is acceptable, will just recount
        if let Err(e) = fs::write(&path, json) {
            eprintln!("● cargo-check-gate: Warning: failed to save state: {e}");
        }
    }
}

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

fn find_workspace_root(file_path: &str) -> Option<String> {
    let mut path = PathBuf::from(file_path);
    while path.pop() {
        let cargo_toml = path.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if it's a workspace
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn run_cargo_check(workspace: &str) -> bool {
    eprintln!("● cargo-check-gate: Running cargo check in {workspace}...");

    let output = Command::new("cargo")
        .arg("check")
        .arg("--message-format=short")
        .current_dir(workspace)
        .output();

    match output {
        Ok(result) => {
            let success = result.status.success();
            if !success {
                let stderr = String::from_utf8_lossy(&result.stderr);
                // Extract just errors
                for line in stderr.lines() {
                    if line.contains("error") {
                        eprintln!("  {line}");
                    }
                }
            }
            success
        }
        Err(e) => {
            eprintln!("● cargo-check-gate: Failed to run cargo: {e}");
            false
        }
    }
}

fn main() {
    // Read hook input from stdin
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(0); // Allow on read error
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(h) => h,
        Err(_) => std::process::exit(0), // Allow on parse error
    };

    // Only process Write and Edit tools
    let tool_name = hook_input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        std::process::exit(0);
    }

    // Only process Rust files
    let file_path = hook_input
        .tool_input
        .as_ref()
        .and_then(|t| t.file_path.as_deref())
        .unwrap_or("");

    if !file_path.ends_with(".rs") {
        std::process::exit(0);
    }

    // Calculate lines written
    let lines = hook_input
        .tool_input
        .as_ref()
        .and_then(|t| t.content.as_ref().or(t.new_string.as_ref()))
        .map(|s| count_lines(s))
        .unwrap_or(0);

    // Load state
    let mut state = load_state();
    state.lines_written += lines;

    // Detect workspace
    if state.workspace_path.is_none() {
        state.workspace_path = find_workspace_root(file_path);
    }

    // Check if threshold exceeded
    if state.lines_written >= LINES_THRESHOLD {
        eprintln!(
            "● cargo-check-gate: {}/{} lines written - THRESHOLD EXCEEDED",
            state.lines_written, LINES_THRESHOLD
        );

        if let Some(ref workspace) = state.workspace_path {
            let passed = run_cargo_check(workspace);

            if passed {
                eprintln!("● cargo-check-gate: ✓ Cargo check PASSED - resetting counter");
                state.lines_written = 0;
                state.last_check_passed = true;
                save_state(&state);
                std::process::exit(0);
            } else {
                eprintln!("● cargo-check-gate: ✗ Cargo check FAILED - BLOCKING further writes");
                eprintln!("  Fix compilation errors before continuing");
                state.last_check_passed = false;
                save_state(&state);
                std::process::exit(2); // Block
            }
        } else {
            eprintln!("● cargo-check-gate: No workspace found, allowing");
            state.lines_written = 0;
            save_state(&state);
            std::process::exit(0);
        }
    } else {
        eprintln!(
            "● cargo-check-gate: {}/{} lines written",
            state.lines_written, LINES_THRESHOLD
        );
        save_state(&state);
        std::process::exit(0);
    }
}
