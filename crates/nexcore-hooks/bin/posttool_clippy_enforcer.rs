//! Clippy Pedantic Enforcer
//!
//! Runs `cargo clippy` after Rust file changes and BLOCKS on any warnings.
//!
//! "Warnings are problems. Problems are intolerable." - Core principle
//!
//! 2026-02-09: Updated to skip frontend and be non-blocking for warnings.

use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, is_rust_file, read_input};
use std::path::Path;
use std::process::Command;

/// Find the Cargo.toml for a given file path (walk up the directory tree)
fn find_cargo_toml(file_path: &str) -> Option<std::path::PathBuf> {
    let path = Path::new(file_path);
    let mut current = path.parent()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(cargo_toml);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

fn is_frontend_file(path: &str) -> bool {
    path.contains("/apps/")
        || path.contains("/nucleus/")
        || path.contains("/studio/")
        || path.contains("/control-center/")
        || path.contains("/ncos/")
        || path.contains("frontend")
}

/// Extract crate name from Cargo.toml
fn get_crate_name(cargo_toml: &Path) -> Option<String> {
    let content = std::fs::read_to_string(cargo_toml).ok()?;

    let mut in_package = false;
    for line in content.lines() {
        if line.trim() == "[package]" {
            in_package = true;
            continue;
        }
        if line.trim().starts_with('[') && line.trim() != "[package]" {
            in_package = false;
            continue;
        }
        if in_package && line.trim().starts_with("name") {
            if let Some(name) = line.split('=').nth(1) {
                let name = name.trim().trim_matches('"').trim_matches('\'');
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Run cargo clippy and return (success, warnings_count, output)
fn run_clippy(cargo_dir: &Path, crate_name: Option<&str>) -> (bool, usize, String) {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(cargo_dir);
    cmd.arg("clippy");

    if let Some(name) = crate_name {
        cmd.arg("-p").arg(name);
    }

    cmd.arg("--message-format=short");
    cmd.arg("--").arg("-D").arg("warnings");
    cmd.arg("-W").arg("clippy::pedantic");
    cmd.arg("-A").arg("clippy::module_name_repetitions");
    cmd.arg("-A").arg("clippy::must_use_candidate");
    cmd.arg("-A").arg("clippy::missing_errors_doc");

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return (false, 0, format!("Failed to run cargo clippy: {e}"));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let warning_count = stderr
        .lines()
        .filter(|line| line.contains("warning:"))
        .count();

    let combined = format!("{stdout}\n{stderr}");

    (output.status.success(), warning_count, combined)
}

/// Parse clippy output to extract actionable issues
fn format_issues(output: &str) -> String {
    let mut issues = Vec::new();

    for line in output.lines() {
        if line.contains("warning:") || line.contains("error[") {
            issues.push(line.trim().to_string());
        }
    }

    if issues.is_empty() {
        return String::new();
    }

    let mut msg = String::from("CLIPPY/RUSTC ISSUES DETECTED\n\n");
    for issue in issues.iter().take(20) {
        msg.push_str(&format!("  {issue}\n"));
    }

    if issues.len() > 20 {
        msg.push_str(&format!("\n  ... and {} more issues\n", issues.len() - 20));
    }

    msg.push_str("\nWARNINGS ARE PROBLEMS. PROBLEMS ARE INTOLERABLE.\n");
    msg.push_str("Fix all warnings before proceeding.\n");

    msg
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(&file_path) || is_frontend_file(&file_path) {
        exit_success_auto();
    }

    let cargo_toml = match find_cargo_toml(&file_path) {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let cargo_dir = match cargo_toml.parent() {
        Some(d) => d,
        None => exit_success_auto(),
    };

    let crate_name = get_crate_name(&cargo_toml);
    let (success, warning_count, output) = run_clippy(cargo_dir, crate_name.as_deref());

    if success && warning_count == 0 {
        exit_success_auto();
    }

    let msg = format_issues(&output);

    if !msg.is_empty() {
        // 2026-02-09: Changed from exit_block to exit_warn to make it non-blocking
        exit_warn(&msg);
    } else if !success {
        exit_warn(&format!("Clippy exited with error:\n{output}"));
    }

    exit_success_auto();
}
