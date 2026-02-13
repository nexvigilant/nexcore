//! Bash Validator Hook (from claude-hooks library)
//!
//! PreToolUse:Bash hook that blocks dangerous commands and suggests alternatives.

use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, read_input};

/// Validation rule
struct Rule {
    pattern: &'static str,
    message: &'static str,
    blocking: bool,
}

fn get_rules() -> Vec<Rule> {
    vec![
        // Dangerous commands (blocking)
        Rule {
            pattern: r"rm\s+(-[rf]+\s+)*(/|~|\$HOME)",
            message: "Blocked: rm on root, home, or $HOME directory",
            blocking: true,
        },
        Rule {
            pattern: r">\s*/dev/sd[a-z]",
            message: "Blocked: direct write to block device",
            blocking: true,
        },
        Rule {
            pattern: r"mkfs\.",
            message: "Blocked: filesystem creation commands",
            blocking: true,
        },
        Rule {
            pattern: r"dd\s+.*of=/dev/",
            message: "Blocked: dd to block device",
            blocking: true,
        },
        Rule {
            pattern: r":(){:|:&};:",
            message: "Blocked: fork bomb detected",
            blocking: true,
        },
        Rule {
            pattern: r"chmod\s+(-R\s+)?777\s+/",
            message: "Blocked: chmod 777 on root",
            blocking: true,
        },
        // Suggestions (non-blocking)
        Rule {
            pattern: r"\bgrep\b(?!.*\|)",
            message: "Suggestion: Use 'rg' (ripgrep) for better performance",
            blocking: false,
        },
        Rule {
            pattern: r"\bfind\s+\S+\s+-name\b",
            message: "Suggestion: Use 'fd' for faster file finding",
            blocking: false,
        },
        Rule {
            pattern: r"\bcat\b.*\|\s*(head|tail|grep)",
            message: "Suggestion: Useless use of cat - pipe directly from file",
            blocking: false,
        },
    ]
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only check Bash commands
    if input.tool_name.as_deref() != Some("Bash") {
        exit_success_auto();
    }

    let command = input.get_command().unwrap_or_default();

    if command.is_empty() {
        exit_success_auto();
    }

    let rules = get_rules();
    let mut blocking_issues = Vec::new();
    let mut suggestions = Vec::new();

    for rule in &rules {
        let re = match regex::Regex::new(rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if re.is_match(&command) {
            if rule.blocking {
                blocking_issues.push(rule.message);
            } else {
                suggestions.push(rule.message);
            }
        }
    }

    if !blocking_issues.is_empty() {
        let message = blocking_issues.join("\n• ");
        exit_block(&format!("• {}", message));
    }

    if !suggestions.is_empty() {
        let message = suggestions.join("\n• ");
        exit_warn(&format!("• {}", message));
    }

    exit_success_auto();
}
