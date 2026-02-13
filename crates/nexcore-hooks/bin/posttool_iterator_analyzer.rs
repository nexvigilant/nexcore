//! Iterator Collection Analyzer (Hook 46)
//!
//! Detects iterator anti-patterns that cause unnecessary allocations.
//! Runs as PostToolUse hook on Write/Edit of Rust files.
//!
//! ## Patterns Detected
//!
//! - `.collect()` immediately followed by `.iter()` (redundant allocation)
//! - `.collect::<Vec<_>>()` when only `.first()` or `.next()` needed
//! - Multiple passes over collected data that could be single pass
//!
//! ## Exit Codes
//!
//! - 0: No anti-patterns found, or non-Rust file (skip)
//! - 1: Warning — anti-pattern detected but not blocking
//! - 2: Block — severe allocation waste in hot path
//!
//! ## Integration
//!
//! Event: `PostToolUse` (Write, Edit)
//! Tier: `dev` (default), `review` (stricter thresholds)
//! Parser: `nexcore_hooks::parser::performance::detect_iterator_issues`

use nexcore_hooks::parser::performance::detect_iterator_issues;
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

    let issues = detect_iterator_issues(content);
    if issues.is_empty() {
        exit_success_auto();
    }

    // Separate by severity
    let critical: Vec<_> = issues.iter().filter(|i| i.severity == "critical").collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").collect();

    // Block on critical issues (collect().iter(), collect for single element)
    if !critical.is_empty() {
        let mut msg = format!(
            "ITERATOR COLLECTION ANALYZER - {} critical issue(s)\n\n",
            critical.len()
        );
        for issue in &critical {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Issue: {}\n  Fix: {}\n\n",
                issue.line, issue.code, issue.pattern, issue.issue, issue.fix
            ));
        }
        msg.push_str("Unnecessary collect() allocates heap memory wastefully.\n");
        msg.push_str("Justify with // ITER: or refactor to use lazy iterators.");
        exit_block(&msg);
    }

    // Warn on lesser issues
    if !warnings.is_empty() {
        let mut msg = format!(
            "ITERATOR COLLECTION ANALYZER - {} issue(s) to review\n\n",
            warnings.len()
        );
        for issue in &warnings {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Issue: {}\n  Suggestion: {}\n\n",
                issue.line, issue.code, issue.pattern, issue.issue, issue.fix
            ));
        }
        exit_warn(&msg);
    }

    exit_success_auto();
}
