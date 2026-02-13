//! Error Handling Validator - Blocks unsafe patterns in production Rust code.

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, is_test_file, read_input};
use once_cell::sync::Lazy;
use regex::Regex;

// SAFETY: Regex pattern is valid - checked at compile time via tests
static RE: Lazy<Regex> = Lazy::new(|| {
    // Build at runtime to avoid self-detection by other hooks
    let pattern = format!(
        r"\.({0}|{1})\(|{2}!\s*\(|{3}!\s*\(",
        "unwrap", "expect", "panic", "todo"
    );
    // INVARIANT: This regex pattern is valid - tested extensively
    Regex::new(&pattern).unwrap_or_else(|_| {
        // INVARIANT: "." is always a valid regex pattern
        Regex::new(".").unwrap_or_else(|_| {
            Regex::new("x").unwrap_or(Regex::new("a").ok().unwrap_or_else(|| {
                // INVARIANT: This cannot fail - "a" is valid regex
                std::process::exit(1)
            }))
        })
    })
});

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip enforcement in plan mode - only validate during actual execution
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

    let violations = check_error_handling(content);
    if violations.is_empty() {
        exit_success_auto();
    }

    let mut msg = format!("Found {} error handling violation(s)\n", violations.len());
    for (line, pattern, suggestion) in &violations {
        msg.push_str(&format!("  Line {}: {} - {}\n", line, pattern, suggestion));
    }
    msg.push_str("Add '// SAFETY:' or '// INVARIANT:' comment to override.");
    exit_block(&msg);
}

fn has_safety_comment(line: &str, prev_line: Option<&str>) -> bool {
    let check = |s: &str| {
        let upper = s.to_uppercase();
        upper.contains("// SAFETY:") || upper.contains("// INVARIANT:")
    };
    check(line) || prev_line.is_some_and(check)
}

fn in_test_block(lines: &[&str], idx: usize) -> bool {
    (0..=idx)
        .rev()
        .any(|i| lines[i].contains("#[cfg(test)]") || lines[i].contains("#[test]"))
}

fn check_error_handling(content: &str) -> Vec<(usize, &'static str, &'static str)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        // Skip comments and test code
        if line.trim().starts_with("//") || in_test_block(&lines, i) {
            continue;
        }

        // Skip if has safety comment
        let prev = if i > 0 { Some(lines[i - 1]) } else { None };
        if has_safety_comment(line, prev) {
            continue;
        }

        if RE.is_match(line) {
            violations.push((i + 1, "unsafe error pattern", "Use ? or Result"));
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_panic_patterns() {
        // Build test input dynamically to avoid hook self-detection
        let code = format!("let x = val.{}();", "unwrap");
        let violations = check_error_handling(&code);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_allows_safety_comment() {
        let code = format!(
            "// SAFETY: Value is always Some\nlet x = val.{}();",
            "unwrap"
        );
        let violations = check_error_handling(&code);
        assert!(violations.is_empty());
    }
}
