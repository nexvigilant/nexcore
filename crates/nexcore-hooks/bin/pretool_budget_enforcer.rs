//! Complexity Budget Enforcer
//!
//! Measures and reports complexity metrics after file changes.
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

    let issues = check_complexity(&content);
    if issues.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("🛑 COMPLEXITY EXCEEDED - BLOCKED\n");
    for issue in &issues {
        msg.push_str(&format!("  {issue}\n"));
    }
    msg.push_str("Refactor to reduce file size (<500 lines) or nesting (<4 levels)");
    HookOutput::block(&msg).emit();
    std::process::exit(2);
}

fn check_complexity(content: &str) -> Vec<String> {
    let mut issues = Vec::new();
    let lines = content.lines().count();

    if lines > 500 {
        issues.push(format!("File: {lines} lines (max 500)"));
    }

    let depth = max_nesting(content);
    if depth > 4 {
        issues.push(format!("Nesting: {depth} (max 4)"));
    }

    issues
}

fn max_nesting(content: &str) -> usize {
    let (mut max, mut cur): (usize, usize) = (0, 0);
    for ch in content.chars() {
        match ch {
            '{' => {
                cur += 1;
                max = max.max(cur);
            }
            '}' => cur = cur.saturating_sub(1),
            _ => {}
        }
    }
    max
}
