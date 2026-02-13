//! String Allocation Optimizer (Hook 45)
//!
//! Detects string allocation anti-patterns that waste memory and CPU.
//! Part of the nexcore performance enforcement layer.
//!
//! ## Patterns Detected
//!
//! | Pattern | Issue | Fix |
//! |---------|-------|-----|
//! | `+` in loops | O(n²) allocation | Use `String::with_capacity` + `push_str` |
//! | `format!()` + `push_str()` | Extra allocation | Use `write!()` directly |
//! | Repeated `.to_string()` | Redundant clones | Store in variable |
//! | `&String` params | Unnecessary indirection | Use `&str` |
//!
//! ## Exit Codes
//!
//! - 0: No issues or non-Rust file
//! - 1: Warning (issues found but not blocking)
//! - 2: Block (severe anti-patterns in hot paths)

use nexcore_hooks::parser::performance::detect_string_issues;
use nexcore_hooks::{
    exit_block, exit_success_auto, exit_warn, is_rust_file, is_test_file, read_input,
};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) || is_test_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let issues = detect_string_issues(content);
    if issues.is_empty() {
        exit_success_auto();
    }

    // Separate by severity
    let critical: Vec<_> = issues.iter().filter(|i| i.severity == "critical").collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").collect();

    // Block on critical issues (e.g., string concat in loops)
    if !critical.is_empty() {
        let mut msg = format!(
            "STRING ALLOCATION OPTIMIZER - {} critical issue(s)\n\n",
            critical.len()
        );
        for issue in &critical {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Fix: {}\n\n",
                issue.line, issue.code, issue.pattern, issue.note
            ));
        }
        msg.push_str("String concatenation in loops causes O(n²) allocations.\n");
        msg.push_str("Justify with // STRING: or refactor to avoid allocations.");
        exit_block(&msg);
    }

    // Warn on lesser issues
    if !warnings.is_empty() {
        let mut msg = format!(
            "STRING ALLOCATION OPTIMIZER - {} issue(s) to review\n\n",
            warnings.len()
        );
        for issue in &warnings {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Suggestion: {}\n\n",
                issue.line, issue.code, issue.pattern, issue.note
            ));
        }
        exit_warn(&msg);
    }

    exit_success_auto();
}
