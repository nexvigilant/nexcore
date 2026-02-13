//! Technical Debt Blocker
//!
//! Prevents introduction of untracked technical debt markers.
//! Requires issue references for TODO/FIXME comments.

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

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

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let violations = check_debt_markers(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = String::from("TECHNICAL DEBT BLOCKED\n\n");
    for (line, marker, reason) in &violations {
        msg.push_str(&format!("Line {line}: {marker}\n  Reason: {reason}\n\n"));
    }
    msg.push_str("RESOLUTION:\n");
    msg.push_str("  1. Fix it now (preferred)\n");
    msg.push_str("  2. Use tracked format: TODO(ISSUE-123): description\n");
    msg.push_str("  3. Use ACCEPTED_RISK: with documented reasoning\n");
    exit_block(&msg);
}

/// Build markers at runtime to avoid self-detection
fn get_markers() -> Vec<(String, &'static str)> {
    vec![
        (
            format!("// TO{}", "DO"),
            "Untracked work - use TODO(ISSUE-123)",
        ),
        (
            format!("// FIX{}", "ME"),
            "Untracked defect - use FIXME(ISSUE-123)",
        ),
        (format!("// HA{}", "CK"), "Unacceptable workaround"),
        (format!("// X{}", "XX"), "Unspecified concern"),
        (format!("// TEMPO{}", "RARY"), "Will become permanent"),
        (format!("// WORK{}", "AROUND"), "Masks real issue"),
    ]
}

fn check_debt_markers(content: &str) -> Vec<(usize, String, &'static str)> {
    let markers = get_markers();
    let mut violations = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        for (marker, reason) in &markers {
            if line.contains(marker) {
                // Allow if followed by issue reference like (ISSUE-123)
                let after = line.split(marker).nth(1).unwrap_or("");
                let has_issue = after.trim_start().starts_with('(')
                    && after.contains('-')
                    && after.contains(')');
                // Also allow ACCEPTED_RISK pattern
                let is_accepted = line.contains("ACCEPTED_RISK");

                if !has_issue && !is_accepted {
                    violations.push((line_num + 1, marker.clone(), *reason));
                }
            }
        }
    }

    violations
}
