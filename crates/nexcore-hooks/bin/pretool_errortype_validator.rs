//! Error Type Completeness Validator
//!
//! Validates that error types follow Rust best practices and the Primitive Codex
//! requirements. This hook enforces type safety by detecting common error handling
//! antipatterns and promoting typed error variants over stringly-typed or generic
//! error handling patterns.
//!
//! # Checks Performed
//!
//! - **Stringly-typed errors**: Detects `Err(String)` or `Error(String)` patterns
//! - **Generic variants**: Flags `Other(String)` or `Unknown(String)` variants
//! - **Source tracking**: Ensures wrapped errors have `#[source]` attribute
//! - **Library stability**: Requires `#[non_exhaustive]` on public error enums
//!
//! # Exit Behavior
//!
//! This hook is advisory only and always exits with 0 (allow). Issues are logged
//! as warnings via HookOutput. This supports incremental adoption without blocking.

use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};
use std::fs;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => exit_success_auto(),
    };

    // Only check files with error types
    if !content.contains("enum") || !content.contains("Error") {
        exit_success_auto();
    }

    let issues = check_error_types(&content);
    if issues.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("ERROR TYPE ISSUES\n");
    for issue in &issues {
        msg.push_str(&format!("  {issue}\n"));
    }
    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}

fn check_error_types(content: &str) -> Vec<&'static str> {
    let mut issues = Vec::new();

    // Check for stringly-typed errors
    if content.contains("Err(String)") || content.contains("Error(String)") {
        issues.push("Stringly-typed error - use typed variants");
    }

    // Check for generic Other variant
    if content.contains("Other(String)") || content.contains("Unknown(String)") {
        issues.push("Generic Other/Unknown variant - be specific");
    }

    // Check for missing #[source] on wrapped errors
    if content.contains("source:") && !content.contains("#[source]") {
        issues.push("Wrapped error missing #[source] attribute");
    }

    // Check for non-exhaustive on library errors
    if content.contains("pub enum")
        && content.contains("Error")
        && !content.contains("#[non_exhaustive]")
    {
        issues.push("Library error missing #[non_exhaustive]");
    }

    issues
}
