//! Clippy Enforcer - Atomic Hook
//!
//! PostToolUse hook that runs `cargo clippy` after Rust file edits.
//! Warns on any clippy warnings (does not block).
//!
//! # Codex Compliance
//! - **Tier**: T3 (Development Quality Hook)
//! - **Commandments**: VI (Match), VII (Type)
//!
//! # Cytokine Integration
//! - **Warn**: Emits IL-6 (acute response) via cytokine bridge
//! - **Pass**: No emission (homeostasis maintained)

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{
    Confidence, Measured, file_path_or_pass, find_cargo_toml, get_crate_name, pass, read_input,
    require_edit_tool, require_rust_file, warn,
};
use std::path::Path;
use std::process::Command;

const HOOK_NAME: &str = "clippy-enforcer";

/// Result of a clippy check.
///
/// # Tier: T2-C
/// Grounds to: T1(bool) and T1(String) via Vec.
#[derive(Debug, Clone)]
struct ClippyResult {
    success: bool,
    warnings: Vec<String>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
    // Check tool type - Commandment VI (Match)
    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);
    // Only check Rust files
    require_rust_file(file_path);

    // Find Cargo.toml
    let cargo_toml = match find_cargo_toml(file_path) {
        Some(c) => c,
        None => pass(), // Not in a Cargo project
    };

    let cargo_dir = match cargo_toml.parent() {
        Some(d) => d,
        None => pass(),
    };

    // Get crate name for targeted check
    let crate_name = get_crate_name(&cargo_toml);

    // Run clippy
    let result = run_clippy(cargo_dir, crate_name.as_deref());

    if result.value.success && result.value.warnings.is_empty() {
        pass();
    }

    // Format warning message
    let msg = format_warnings(&result.value.warnings);
    if !msg.is_empty() {
        // Emit cytokine signal before warning (IL-6 = acute response)
        emit_check_failed(HOOK_NAME, &msg);
        warn(&msg);
    }

    pass();
}

/// Run cargo clippy, return measured result.
fn run_clippy(cargo_dir: &Path, crate_name: Option<&str>) -> Measured<ClippyResult> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(cargo_dir);
    cmd.arg("clippy");

    if let Some(name) = crate_name {
        cmd.arg("-p").arg(name);
    }

    cmd.arg("--message-format=short");
    cmd.arg("--").arg("-D").arg("warnings");

    let output = match cmd.output() {
        Ok(o) => o,
        Err(_) => {
            return Measured {
                value: ClippyResult {
                    success: false,
                    warnings: Vec::new(),
                },
                confidence: Confidence(0.5), // Could not run — uncertain
            };
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract warning/error lines
    // With -D warnings, warnings become "error:" so we catch both
    // Filter out meta-lines like "error: could not compile"
    let warnings: Vec<String> = stderr
        .lines()
        .filter(|line| {
            (line.contains("warning:") || line.contains("error:") || line.contains("error["))
                && !line.contains("could not compile")
                && !line.contains("aborting due to")
        })
        .map(|s| s.trim().to_string())
        .collect();

    Measured {
        value: ClippyResult {
            success: output.status.success(),
            warnings,
        },
        confidence: Confidence::certain(),
    }
}

/// Format warnings for display
fn format_warnings(warnings: &[String]) -> String {
    if warnings.is_empty() {
        return String::new();
    }

    let mut msg = String::from("CLIPPY WARNINGS\n");
    for (i, w) in warnings.iter().take(10).enumerate() {
        msg.push_str(&format!("  {}. {}\n", i + 1, w));
    }
    if warnings.len() > 10 {
        msg.push_str(&format!("  ... and {} more\n", warnings.len() - 10));
    }
    msg
}
