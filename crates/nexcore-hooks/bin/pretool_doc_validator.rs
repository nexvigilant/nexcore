//! # Documentation Completeness Validator
//!
//! A PreToolUse hook that validates Rust documentation quality after file edits.
//!
//! ## Purpose
//!
//! Detects common documentation anti-patterns that reduce code comprehension:
//! - **Vague descriptions**: "Creates a new X" without explaining what X represents
//! - **Name restating**: "Gets the value" when `get_value()` already conveys that
//! - **Missing context**: Documentation that doesn't add information beyond the signature
//!
//! ## Hook Behavior
//!
//! | Condition | Exit Code | Effect |
//! |-----------|-----------|--------|
//! | No issues found | 0 | Silent pass |
//! | Issues detected | 0 | Warning emitted (advisory) |
//! | Non-Rust file | 0 | Skip silently |
//!
//! ## Anti-Patterns Detected
//!
//! | Pattern | Issue | Better Alternative |
//! |---------|-------|-------------------|
//! | `/// Creates a new` | Vague | Describe what it represents and why |
//! | `/// Returns a new` | Vague | Explain the purpose and guarantees |
//! | `/// Gets the` | Restates name | Add context: when to use, invariants |
//! | `/// Sets the` | Restates name | Describe side effects and validation |
//! | `/// This function` | Filler | Start with action verb directly |
//! | `/// This method` | Filler | Start with action verb directly |
//!
//! ## Integration
//!
//! Triggered by: `PreToolUse:Edit`, `PreToolUse:Write` on `*.rs` files
//!
//! ## Example
//!
//! ```text
//! // BAD: Restates the function name
//! /// Gets the value
//! fn get_value(&self) -> i32
//!
//! // GOOD: Adds context and invariants
//! /// Returns the cached computation result, or 0 if not yet computed.
//! fn get_value(&self) -> i32
//! ```

use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};
use std::fs;

/// Entry point for the documentation validator hook.
///
/// Reads stdin for hook input, validates Rust file documentation,
/// and emits advisory warnings for detected anti-patterns.
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

    // Read the file content after edit
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => exit_success_auto(),
    };

    let issues = check_doc_quality(&content);
    if issues.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("DOC QUALITY ISSUES\n");
    for (line, issue) in &issues {
        msg.push_str(&format!("  L{line}: {issue}\n"));
    }
    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}

/// Analyzes content for documentation anti-patterns.
///
/// Returns a list of (line_number, issue_description) tuples for each
/// detected anti-pattern. Line numbers are 1-indexed.
fn check_doc_quality(content: &str) -> Vec<(usize, &'static str)> {
    /// Documentation anti-patterns: (pattern, issue description)
    const ANTI_PATTERNS: &[(&str, &str)] = &[
        ("/// Creates a new", "Vague - describe what it represents"),
        ("/// Returns a new", "Vague - describe purpose"),
        ("/// Gets the", "Restates name - add context"),
        ("/// Sets the", "Restates name - describe effect"),
        ("/// This function", "Filler - start with action verb"),
        ("/// This method", "Filler - start with action verb"),
    ];

    let mut issues = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for (pattern, issue) in ANTI_PATTERNS {
            if line.contains(pattern) {
                issues.push((line_num + 1, *issue));
            }
        }
    }
    issues
}
