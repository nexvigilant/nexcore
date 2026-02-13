//! Rust Toolchain Validator
//!
//! Event: SessionStart
//! Verifies Rust toolchain is properly configured.

use std::process::Command;

fn get_version(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "not found".to_string())
}

fn main() {
    let rustc = get_version("rustc", &["--version"]);
    let cargo = get_version("cargo", &["--version"]);
    let clippy = get_version("cargo", &["clippy", "--version"]);
    let _toolchain = get_version("rustup", &["show", "active-toolchain"]);

    let mut warnings = Vec::new();

    if rustc.contains("not found") {
        warnings.push("rustc not found");
    }
    if cargo.contains("not found") {
        warnings.push("cargo not found");
    }
    if clippy.contains("not found") {
        warnings.push("clippy not found");
    }

    let status = if warnings.is_empty() {
        format!(
            "Rust toolchain OK: {}",
            rustc
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        format!("Rust toolchain issues: {}", warnings.join(", "))
    };

    let output = serde_json::json!({
        "continue": true,
        "stopReason": status
    });

    println!("{}", output);
}
