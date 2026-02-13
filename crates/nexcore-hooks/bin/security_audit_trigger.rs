//! # Security Audit Trigger
//!
//! Runs security checks at session start for Rust projects.
//!
//! ## Event
//! - **Hook Type**: `SessionStart`
//! - **Tier**: `[]` (inactive by default - enable in settings.json)
//! - **Timeout**: 5 seconds
//!
//! ## Behavior
//! 1. **Detects Rust projects** by checking for `Cargo.toml` in cwd
//! 2. **Runs `cargo audit`** (if installed) to check for known vulnerabilities
//! 3. **Checks for `#![forbid(unsafe_code)]`** in lib.rs/main.rs
//! 4. **Emits system message** with scan summary
//!
//! ## Exit Codes
//! - `0` (allow): Always allows session to proceed; issues are informational
//!
//! ## Prerequisites
//! - `cargo-audit` should be installed: `cargo install cargo-audit`
//! - RustSec Advisory Database is fetched automatically by cargo-audit
//!
//! ## Example Output
//! ```text
//! 🔒 Security scan: no vulnerabilities detected
//! ```
//!
//! ## Integration
//! Enable in `~/.claude/settings.json`:
//! ```json
//! {
//!   "hooks": {
//!     "SessionStart": [{
//!       "command": "~/nexcore/target/release/security_audit_trigger",
//!       "timeout": 5000
//!     }]
//!   }
//! }
//! ```

use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use std::path::Path;
use std::process::Command;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let cwd = &input.cwd;

    // Check if this is a Rust project
    if !Path::new(cwd).join("Cargo.toml").exists() {
        exit_success_auto();
    }

    let mut report = String::from("RUST SECURITY AUDIT\n\n");
    let mut has_issues = false;

    // Check for cargo-audit
    if check_cargo_audit(cwd) {
        report.push_str("[OK] cargo audit: No vulnerabilities\n");
    } else {
        report.push_str("[!] cargo audit: Run `cargo audit` to check\n");
        has_issues = true;
    }

    // Check for unsafe usage in Cargo.toml
    if check_forbid_unsafe(cwd) {
        report.push_str("[OK] #![forbid(unsafe_code)] or documented unsafe\n");
    } else {
        report.push_str("[?] Unsafe code: Review unsafe blocks\n");
    }

    // Summary
    if has_issues {
        report.push_str("\nRun `cargo audit` for full security report.");
    }

    // Output with system message for visibility
    HookOutput::allow()
        .with_system_message(if has_issues {
            "🔒 Security scan: issues found - run `cargo audit`"
        } else {
            "🔒 Security scan: no vulnerabilities detected"
        })
        .emit();
    std::process::exit(0);
}

/// Runs `cargo audit --quiet` to check for known vulnerabilities.
///
/// Returns `true` if audit passes (no vulnerabilities), `false` if:
/// - Vulnerabilities found
/// - cargo-audit not installed
/// - Command execution fails
fn check_cargo_audit(cwd: &str) -> bool {
    Command::new("cargo")
        .args(["audit", "--quiet"])
        .current_dir(cwd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Checks if `#![forbid(unsafe_code)]` is declared in lib.rs or main.rs.
///
/// Returns `true` if the crate explicitly forbids unsafe code at the crate level.
fn check_forbid_unsafe(cwd: &str) -> bool {
    let lib_path = Path::new(cwd).join("src/lib.rs");
    let main_path = Path::new(cwd).join("src/main.rs");

    for path in [lib_path, main_path] {
        if let Ok(content) = std::fs::read_to_string(path) {
            if content.contains("#![forbid(unsafe_code)]") {
                return true;
            }
        }
    }
    false
}
