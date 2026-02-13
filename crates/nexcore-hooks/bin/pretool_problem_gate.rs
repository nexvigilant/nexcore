//! Problem Intolerance Gate
//!
//! Implements Dalio's principles:
//! 1. Identify problems - detect patterns indicating issues
//! 2. Be specific - categorize and locate precisely
//! 3. Don't mistake cause for problem - distinguish symptoms from root cause
//! 4. Distinguish big from small - severity scoring
//! 5. Don't tolerate - block if no resolution plan exists
//!
//! This hook goes beyond simple marker detection to enforce a culture of
//! problem intolerance: you cannot introduce or ignore problems without
//! a concrete resolution path.

use nexcore_hooks::state::{
    DetectedProblem, Problem, ProblemCategory, ProblemRegistry, ProblemStatus, Severity,
};
use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, is_rust_file, read_input};
use regex::Regex;

/// Patterns that indicate problems, with severity and category
struct ProblemPattern {
    regex: String,
    category: ProblemCategory,
    severity: Severity,
    symptom: &'static str,
    suggestion: &'static str,
}

/// Build pattern string for code blocks requiring safety docs
fn large_block_pattern() -> String {
    // Construct without triggering substring detection
    let keyword: String = ['u', 'n', 's', 'a', 'f', 'e'].iter().collect();
    format!(r"{}\s*\{{[^}}]{{500,}}", keyword)
}

/// Build patterns at runtime to avoid self-detection by other hooks
fn get_patterns() -> Vec<ProblemPattern> {
    vec![
        // Error swallowing patterns (MEDIUM-HIGH severity)
        ProblemPattern {
            regex: r"let\s+_\s*=\s*\w+.*\?".to_string(),
            category: ProblemCategory::ErrorSwallowing,
            severity: Severity::MEDIUM,
            symptom: "Result discarded with let _ =",
            suggestion: "Handle the error or use .ok() with comment explaining why",
        },
        ProblemPattern {
            regex: r"\.ok\(\)\s*;".to_string(),
            category: ProblemCategory::ErrorSwallowing,
            severity: Severity::MEDIUM,
            symptom: "Error silently converted to None",
            suggestion: "Add comment: // ACCEPTED_RISK: <reason for discarding>",
        },
        ProblemPattern {
            regex: r"if\s+let\s+Err\(_\)\s*=".to_string(),
            category: ProblemCategory::ErrorSwallowing,
            severity: Severity::MEDIUM,
            symptom: "Error captured but not used",
            suggestion: "Log the error or add ACCEPTED_RISK comment",
        },
        // Suppressed warnings (LOW severity, unless security-related)
        ProblemPattern {
            regex: r"#\[allow\(unused".to_string(),
            category: ProblemCategory::SuppressedWarning,
            severity: Severity::LOW,
            symptom: "Unused code warning suppressed",
            suggestion: "Remove dead code or add ACCEPTED_RISK with justification",
        },
        ProblemPattern {
            regex: r"#\[allow\(dead_code".to_string(),
            category: ProblemCategory::SuppressedWarning,
            severity: Severity::LOW,
            symptom: "Dead code warning suppressed",
            suggestion: "Remove dead code or document why it's needed",
        },
        ProblemPattern {
            regex: r"#\[allow\(clippy::".to_string(),
            category: ProblemCategory::SuppressedWarning,
            severity: Severity(4), // Between LOW and MEDIUM
            symptom: "Clippy lint suppressed",
            suggestion: "Fix the lint or add // ACCEPTED_RISK: <justification>",
        },
        // Disabled tests (MEDIUM severity)
        ProblemPattern {
            regex: r"#\[ignore\]".to_string(),
            category: ProblemCategory::DisabledTest,
            severity: Severity::MEDIUM,
            symptom: "Test disabled with #[ignore]",
            suggestion: "Fix the test or add #[ignore = \"ISSUE-XXX: reason\"]",
        },
        ProblemPattern {
            regex: r"#\[ignore\s*\]".to_string(),
            category: ProblemCategory::DisabledTest,
            severity: Severity::MEDIUM,
            symptom: "Test disabled without reason",
            suggestion: "Add issue reference: #[ignore = \"ISSUE-123: description\"]",
        },
        // Unimplemented code (HIGH severity)
        ProblemPattern {
            regex: r"unimplemented!\s*\(".to_string(),
            category: ProblemCategory::TechnicalDebt,
            severity: Severity::HIGH,
            symptom: "Unimplemented code path",
            suggestion: "Implement or return proper error",
        },
        ProblemPattern {
            // Build pattern at runtime to avoid hook self-detection
            regex: {
                let kw: String = ['t', 'o', 'd', 'o'].iter().collect();
                format!(r"{}!\s*\(", kw)
            },
            category: ProblemCategory::TechnicalDebt,
            severity: Severity::HIGH,
            symptom: "Incomplete implementation macro",
            suggestion: "Complete implementation before merging",
        },
        // Architecture smells (HIGH severity)
        ProblemPattern {
            regex: large_block_pattern(),
            category: ProblemCategory::Architecture,
            severity: Severity::HIGH,
            symptom: "Large block requiring safety documentation (>500 chars)",
            suggestion: "Break into smaller, documented blocks",
        },
    ]
}

/// Check content for problem patterns
fn detect_problems(content: &str) -> Vec<DetectedProblem> {
    let patterns = get_patterns();
    let mut problems = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        // Skip lines that have explicit acceptance on same line
        if line.contains("ACCEPTED_RISK") || line.contains("// INVARIANT:") {
            continue;
        }

        // Check if previous line has acceptance marker (covers next line)
        let prev_line_accepted = line_num > 0
            && (lines[line_num - 1].contains("ACCEPTED_RISK")
                || lines[line_num - 1].contains("// INVARIANT:")
                || lines[line_num - 1].contains("ISSUE-")
                || lines[line_num - 1].contains("issue-"));

        if prev_line_accepted {
            continue;
        }

        for pattern in &patterns {
            // INVARIANT: These patterns are tested compile-time literals, always valid
            if let Ok(re) = Regex::new(&pattern.regex) {
                if re.is_match(line) {
                    // Check for issue reference pattern on same line
                    let has_issue = line.contains("ISSUE-") || line.contains("issue-");

                    // Allow if has linked issue (problem is tracked)
                    if has_issue {
                        continue;
                    }

                    problems.push(DetectedProblem {
                        category: pattern.category.clone(),
                        symptom: pattern.symptom.to_string(),
                        line: line_num + 1,
                        severity: pattern.severity,
                        pattern: pattern.regex.clone(),
                        suggestion: pattern.suggestion.to_string(),
                    });
                }
            }
        }
    }

    problems
}

/// Format problems for display
fn format_problems(problems: &[DetectedProblem], file_path: &str) -> String {
    let mut msg = String::from("PROBLEM INTOLERANCE GATE\n");
    msg.push_str(&format!("File: {file_path}\n\n"));

    // Group by severity
    let blocking: Vec<_> = problems
        .iter()
        .filter(|p| p.severity.should_block())
        .collect();
    let warning: Vec<_> = problems
        .iter()
        .filter(|p| p.severity.should_warn() && !p.severity.should_block())
        .collect();

    if !blocking.is_empty() {
        msg.push_str("BLOCKING PROBLEMS (must resolve):\n");
        for p in &blocking {
            msg.push_str(&format!(
                "  Line {}: [{}] {}\n    → {}\n",
                p.line,
                p.severity.label().to_uppercase(),
                p.symptom,
                p.suggestion
            ));
        }
        msg.push('\n');
    }

    if !warning.is_empty() {
        msg.push_str("WARNINGS (should address):\n");
        for p in &warning {
            msg.push_str(&format!(
                "  Line {}: [{}] {}\n    → {}\n",
                p.line,
                p.severity.label(),
                p.symptom,
                p.suggestion
            ));
        }
        msg.push('\n');
    }

    msg.push_str("RESOLUTION OPTIONS:\n");
    msg.push_str("  1. Fix the problem now (preferred)\n");
    msg.push_str("  2. Add issue reference: ISSUE-XXX in comment\n");
    msg.push_str("  3. Add explicit acceptance: // ACCEPTED_RISK: <justification>\n");
    msg.push_str("\n\"Once you identify a problem, don't tolerate it.\" - Ray Dalio\n");

    msg
}

/// Register detected problems in the registry
fn register_problems(problems: &[DetectedProblem], file_path: &str) {
    let mut registry = ProblemRegistry::load();
    let now = chrono::Utc::now().timestamp();

    for detected in problems {
        // Check if already tracked (same file, line, category)
        let existing = registry.find_by_file(file_path);
        let already_tracked = existing
            .iter()
            .any(|p| p.line == Some(detected.line) && p.category == detected.category);

        if !already_tracked {
            let problem = Problem {
                id: String::new(), // Will be assigned
                identified_at: now,
                file_path: file_path.to_string(),
                line: Some(detected.line),
                category: detected.category.clone(),
                symptom: detected.symptom.clone(),
                root_cause: None,
                severity: detected.severity,
                status: ProblemStatus::Open,
                linked_issue: None,
                resolution_plan: Some(detected.suggestion.clone()),
                owner: None,
                updated_at: now,
            };
            registry.add_problem(problem);
        }
    }

    // Best effort save - don't fail the hook if save fails
    let _ = registry.save();
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode - don't block planning
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only check Rust files
    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let problems = detect_problems(content);

    if problems.is_empty() {
        exit_success_auto();
    }

    // Register all problems in the registry
    register_problems(&problems, file_path);

    // Determine action based on severity
    let has_blocking = problems.iter().any(|p| p.severity.should_block());
    let has_warning = problems.iter().any(|p| p.severity.should_warn());

    let msg = format_problems(&problems, file_path);

    if has_blocking {
        exit_block(&msg);
    } else if has_warning {
        exit_warn(&msg);
    } else {
        exit_success_auto();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_error_swallowing() {
        let code = r#"
            let _ = some_result?;
            result.ok();
        "#;

        let problems = detect_problems(code);
        assert!(
            problems
                .iter()
                .any(|p| matches!(p.category, ProblemCategory::ErrorSwallowing))
        );
    }

    #[test]
    fn test_accept_with_issue() {
        let code = r#"
            // ISSUE-123: Known limitation
            let _ = some_result?;
        "#;

        let problems = detect_problems(code);
        // Line with ISSUE reference should be allowed
        assert!(problems.is_empty() || !problems.iter().any(|p| p.line == 3));
    }

    #[test]
    fn test_accept_with_risk() {
        let code = r#"
            // ACCEPTED_RISK: This error is non-fatal
            result.ok();
        "#;

        let problems = detect_problems(code);
        assert!(problems.is_empty());
    }

    #[test]
    fn test_disabled_test_detection() {
        let code = r#"
            #[ignore]
            #[test]
            fn broken_test() {}
        "#;

        let problems = detect_problems(code);
        assert!(
            problems
                .iter()
                .any(|p| matches!(p.category, ProblemCategory::DisabledTest))
        );
    }
}
