//! Lock Contention Detector (Hook 47)
//!
//! Detects lock contention patterns and deadlock risks.
//!
//! Patterns detected:
//! - Lock held across `.await` (deadlock risk in async)
//! - Lock acquired inside loop (high contention)
//! - Nested locks on same line (deadlock risk)
//! - `std::sync::Mutex` in async without `tokio::sync::Mutex`

use nexcore_hooks::parser::performance::detect_lock_issues;
use nexcore_hooks::{
    exit_block, exit_success_auto, exit_warn, is_rust_file, is_test_file, read_input,
};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

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

    let issues = detect_lock_issues(content);
    if issues.is_empty() {
        exit_success_auto();
    }

    // Separate by severity
    let critical: Vec<_> = issues.iter().filter(|i| i.severity == "critical").collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").collect();

    // Block on critical issues (deadlock risks, lock in loop)
    if !critical.is_empty() {
        let mut msg = format!(
            "LOCK CONTENTION DETECTOR - {} critical issue(s)\n\n",
            critical.len()
        );
        for issue in &critical {
            msg.push_str(&format!(
                "Line {}: {}\n  Pattern: {}\n  Issue: {}\n  Fix: {}\n\n",
                issue.line, issue.code, issue.pattern, issue.issue, issue.fix
            ));
        }
        msg.push_str("Lock contention and deadlock risks are serious bugs.\n");
        msg.push_str("Justify with // LOCK: or restructure locking strategy.");
        exit_block(&msg);
    }

    // Warn on lesser issues
    if !warnings.is_empty() {
        let mut msg = format!(
            "LOCK CONTENTION DETECTOR - {} issue(s) to review\n\n",
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
