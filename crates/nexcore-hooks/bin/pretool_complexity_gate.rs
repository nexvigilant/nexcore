//! # Complexity Gate (Hook 15 - Lex Primitiva)
//!
//! PreToolUse hook that enforces architectural grounding (σ, ρ primitives).
//! Prevents deep nesting (>5 levels) and god functions (>50 lines).
//!
//! 2026-02-09: Updated to be non-blocking (warning only) and skip frontend.

use nexcore_hooks::{
    HookInput, exit_success_auto, exit_warn, is_rust_file, is_test_file, read_input,
};
use std::collections::HashMap;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => return,
    };
    if input.is_plan_mode() {
        exit_success_auto();
    }
    run_gate(&input);
}

fn run_gate(input: &HookInput) {
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Skip if not Rust, or if it's a test file, or if it's a frontend file
    if !is_rust_file(&file_path) || is_test_file(&file_path) || is_frontend_file(&file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };
    let mut violations = Vec::new();

    check_violations(&content, &mut violations);

    if !violations.is_empty() {
        report_violations(violations);
    }
    exit_success_auto();
}

fn is_frontend_file(path: &str) -> bool {
    path.contains("/apps/")
        || path.contains("/nucleus/")
        || path.contains("/studio/")
        || path.contains("/control-center/")
        || path.contains("/ncos/")
        || path.contains("frontend")
}

fn check_violations(content: &str, violations: &mut Vec<String>) {
    let max_indent = check_nesting(content);
    if max_indent > 5 {
        violations.push(format!(
            "Deep nesting detected ({} levels) - Lex Primitiva limit is 5.",
            max_indent
        ));
    }

    let long_functions = check_function_lengths(content);
    for (name, len) in long_functions {
        if len > 50 {
            violations.push(format!(
                "God function '{}' detected ({} lines) - Lex Primitiva limit is 50.",
                name, len
            ));
        }
    }
}

fn report_violations(violations: Vec<String>) {
    let mut msg = String::from("LEX PRIMITIVA COMPLEXITY VIOLATION\n\n");
    for v in violations {
        msg.push_str(&format!("  ⚠️  {}\n", v));
    }
    msg.push_str("\nRefactor logic to pure primitives (σ/ρ) by extracting helper functions.");

    // 2026-02-09: Changed from exit_block to exit_warn to make it non-blocking
    exit_warn(&msg);
}

fn check_nesting(content: &str) -> usize {
    let mut max_indent = 0;
    for line in content.lines() {
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        let level = indent / 4;
        if level > max_indent {
            max_indent = level;
        }
    }
    max_indent
}

fn check_function_lengths(content: &str) -> HashMap<String, usize> {
    let mut functions = HashMap::new();
    let mut current_fn = None;
    let mut current_len = 0;
    let mut brace_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            if let Some(name) = extract_fn_name(trimmed) {
                current_fn = Some(name);
                current_len = 0;
                brace_count = 0;
            }
        }
        if current_fn.is_some() {
            current_len += 1;
            brace_count += line.chars().filter(|&c| c == '{').count();
            brace_count = brace_count.saturating_sub(line.chars().filter(|&c| c == '}').count());
            if brace_count == 0 && current_len > 1 {
                if let Some(name) = current_fn.take() {
                    functions.insert(name, current_len);
                }
            }
        }
    }
    functions
}

fn extract_fn_name(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, &part) in parts.iter().enumerate() {
        if part == "fn" && i + 1 < parts.len() {
            let name_part = parts[i + 1];
            return Some(name_part.split('(').next().unwrap_or(name_part).to_string());
        }
    }
    None
}
