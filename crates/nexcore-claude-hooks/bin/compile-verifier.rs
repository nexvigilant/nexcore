//! Compile Verifier - Atomic Hook
//!
//! PostToolUse hook that runs `cargo check` after Rust file edits.
//! Blocks if compilation fails.
//!
//! Action: Run cargo check, block on failure
//! Exit: 0 = pass, 2 = block on compile error
//!
//! # Cytokine Integration
//! - **Check Failed**: Emits IL-6 (acute response) for compilation failures

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{
    Confidence, Measured, block, file_path_or_pass, find_cargo_toml, get_crate_name, pass,
    read_input, require_rust_file,
};
use std::path::Path;
use std::process::Command;

const HOOK_NAME: &str = "compile-verifier";

/// Tier: T2-C (cross-domain composite)
/// Grounds to: T1(bool) and T1(String) via Vec.
#[derive(Debug, Clone)]
struct CheckResult {
    success: bool,
    errors: Vec<String>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };
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

    // Run cargo check
    let result = run_check(cargo_dir, crate_name.as_deref());

    if result.value.success {
        pass();
    }

    // Emit IL-6 (acute response) cytokine for compilation failure
    let error_summary = result
        .value
        .errors
        .first()
        .map(|s| s.as_str())
        .unwrap_or("compilation failed");
    emit_check_failed(HOOK_NAME, error_summary);

    // Block with error message
    let msg = format_errors(&result.value.errors);
    block(&msg);
}

/// Run cargo check, return measured result
fn run_check(cargo_dir: &Path, crate_name: Option<&str>) -> Measured<CheckResult> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(cargo_dir);
    cmd.arg("check");

    if let Some(name) = crate_name {
        cmd.arg("-p").arg(name);
    }

    cmd.arg("--message-format=short");

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return Measured {
                value: CheckResult {
                    success: false,
                    errors: vec![format!("Failed to run cargo check: {e}")],
                },
                confidence: Confidence(1.0),
            };
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract error lines
    let errors: Vec<String> = stderr
        .lines()
        .filter(|line| line.contains("error[") || line.contains("error:"))
        .map(|s| s.trim().to_string())
        .collect();

    Measured {
        value: CheckResult {
            success: output.status.success(),
            errors,
        },
        confidence: Confidence(1.0),
    }
}

/// Format errors for display
fn format_errors(errors: &[String]) -> String {
    let mut msg = String::from("COMPILATION FAILED\n\n");

    if errors.is_empty() {
        msg.push_str("  cargo check failed (no specific errors captured)\n");
    } else {
        for (i, e) in errors.iter().take(10).enumerate() {
            msg.push_str(&format!("  {}. {}\n", i + 1, e));
        }
        if errors.len() > 10 {
            msg.push_str(&format!("\n  ... and {} more errors\n", errors.len() - 10));
        }
    }

    msg.push_str("\nFix compilation errors before proceeding.\n");
    msg
}
