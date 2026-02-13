//! Async Overhead Analyzer (Hook 44)
//!
//! Detects async misuse that adds overhead without benefit.

use nexcore_hooks::parser::performance::detect_async_issues;
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

    let issues = detect_async_issues(content);
    if issues.is_empty() {
        exit_success_auto();
    }

    let critical: Vec<_> = issues.iter().filter(|i| i.severity == "critical").collect();
    let other: Vec<_> = issues.iter().filter(|i| i.severity != "critical").collect();

    if !critical.is_empty() {
        let mut msg = format!("ASYNC OVERHEAD - {} critical issue(s)\n\n", critical.len());
        for i in &critical {
            msg.push_str(&format!(
                "Line {}: {}\n  Issue: {}\n  Fix: {}\n\n",
                i.line, i.code, i.issue, i.fix
            ));
        }
        msg.push_str("Blocking calls in async contexts stall the executor.");
        exit_block(&msg);
    }

    if !other.is_empty() {
        let mut msg = format!("ASYNC OVERHEAD - {} issue(s) to review\n\n", other.len());
        for i in &other {
            msg.push_str(&format!(
                "Line {}: {}\n  Issue: {}\n  Fix: {}\n\n",
                i.line, i.code, i.issue, i.fix
            ));
        }
        exit_warn(&msg);
    }

    exit_success_auto();
}
