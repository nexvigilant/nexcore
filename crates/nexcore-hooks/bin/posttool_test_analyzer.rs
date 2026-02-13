//! Test Quality Analyzer
//!
//! Analyzes test quality after test file changes.
//! This is advisory only (exit 0).

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

    // Only analyze files with tests
    if !content.contains("#[test]") {
        exit_success_auto();
    }

    let issues = check_test_quality(&content);
    if issues.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("TEST QUALITY ISSUES\n");
    for issue in &issues {
        msg.push_str(&format!("  {issue}\n"));
    }
    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}

fn check_test_quality(content: &str) -> Vec<&'static str> {
    let mut issues = Vec::new();

    let test_count = content.matches("#[test]").count();
    let assert_count = content.matches("assert").count();

    // Check for assertion-free tests
    if test_count > 0 && assert_count == 0 {
        issues.push("Tests without assertions detected");
    }

    // Check for tautological assertions
    if content.contains("assert!(true)") {
        issues.push("Tautological assert!(true) found");
    }

    // Check for ignored tests without reason
    if content.contains("#[ignore]") && !content.contains("#[ignore = ") {
        issues.push("Ignored test without reason");
    }

    issues
}
