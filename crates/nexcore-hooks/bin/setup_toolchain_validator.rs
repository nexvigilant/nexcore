//! Setup Toolchain Validator Hook
//!
//! Event: Setup
//! Validates Rust toolchain on `claude --init`.
//!
//! Checks:
//! - rustc version and edition
//! - cargo version
//! - clippy installed
//! - cargo-audit installed (optional)

use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use std::process::Command;

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let mut issues = Vec::new();
    let mut info = Vec::new();

    // Check rustc
    match get_rustc_version() {
        Some(version) => {
            info.push(format!("rustc: {}", version));
            // Warn if older than 1.75 (2023 edition support)
            if version.starts_with("1.7")
                && version.chars().nth(3).map(|c| c < '5').unwrap_or(false)
            {
                issues.push("rustc version < 1.75 - consider updating for Rust 2024 support");
            }
        }
        None => issues.push("rustc not found - install Rust toolchain"),
    }

    // Check cargo
    match get_cargo_version() {
        Some(version) => info.push(format!("cargo: {}", version)),
        None => issues.push("cargo not found"),
    }

    // Check clippy
    if !check_clippy_installed() {
        issues.push("clippy not installed - run `rustup component add clippy`");
    } else {
        info.push("clippy: installed".to_string());
    }

    // Check cargo-audit (optional)
    if check_cargo_audit_installed() {
        info.push("cargo-audit: installed".to_string());
    } else {
        info.push("cargo-audit: not installed (optional)".to_string());
    }

    // Output results
    if issues.is_empty() {
        HookOutput::allow()
            .with_system_message("🦀 Rust toolchain validated: all tools present")
            .emit();
    } else {
        let msg = format!("⚠️ Toolchain issues:\n  {}", issues.join("\n  "));
        HookOutput::warn(&msg)
            .with_system_message("⚠️ Rust toolchain has issues")
            .emit();
    }

    std::process::exit(0);
}

fn get_rustc_version() -> Option<String> {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn get_cargo_version() -> Option<String> {
    Command::new("cargo")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn check_clippy_installed() -> bool {
    Command::new("cargo")
        .args(["clippy", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_cargo_audit_installed() -> bool {
    Command::new("cargo")
        .args(["audit", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
