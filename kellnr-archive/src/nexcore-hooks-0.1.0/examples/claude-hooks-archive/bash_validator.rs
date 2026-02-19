use claude_hooks::{
    exit_success, read_input, write_output,
    input::PreToolUseInput,
    output::PreToolUseOutput,
    HookResult,
};

/// Validation rule: pattern to match and message to show.
struct ValidationRule {
    pattern: &'static str,
    message: &'static str,
    blocking: bool,
}

fn get_rules() -> Vec<ValidationRule> {
    vec![
        // Dangerous commands (blocking)
        ValidationRule {
            pattern: r"rm\s+(-[rf]+\s+)*(/|~|\$HOME)",
            message: "Blocked: rm on root, home, or $HOME directory",
            blocking: true,
        },
        ValidationRule {
            pattern: r">\s*/dev/sd[a-z]",
            message: "Blocked: direct write to block device",
            blocking: true,
        },
        ValidationRule {
            pattern: r"mkfs\.",
            message: "Blocked: filesystem creation commands",
            blocking: true,
        },
        ValidationRule {
            pattern: r"dd\s+.*of=/dev/",
            message: "Blocked: dd to block device",
            blocking: true,
        },
        
        // Style suggestions (non-blocking)
        ValidationRule {
            pattern: r"\bgrep\b",
            message: "Suggestion: Use 'rg' (ripgrep) for better performance",
            blocking: false,
        },
        ValidationRule {
            pattern: r"\bfind\s+\S+\s+-name\b",
            message: "Suggestion: Use 'fd' or 'rg --files' for faster file finding",
            blocking: false,
        },
    ]
}

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;
    
    let command = input.tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if command.is_empty() {
        exit_success();
    }
    
    let rules = get_rules();
    let mut blocking_issues = Vec::new();
    let mut suggestions = Vec::new();
    
    for rule in &rules {
        let re = regex::Regex::new(rule.pattern).unwrap_or_else(|_| {
            regex::Regex::new(&regex::escape(rule.pattern)).unwrap()
        });
        
        if re.is_match(command) {
            if rule.blocking {
                blocking_issues.push(rule.message);
            } else {
                suggestions.push(rule.message);
            }
        }
    }
    
    if !blocking_issues.is_empty() {
        let message = format!("Security Block:\n• {}", blocking_issues.join("\n• "));
        // Gemini Preferred: Exit 0 with JSON deny
        let output = PreToolUseOutput::deny(message);
        write_output(&output)?;
        return Ok(());
    }
    
    if !suggestions.is_empty() {
        let message = format!("Developer Suggestions:\n• {}", suggestions.join("\n• "));
        // Allow the tool but provide feedback to the model
        let mut output = PreToolUseOutput::allow("Command allowed with suggestions");
        if let Some(ref mut hook_specific) = output.hook_specific_output {
            hook_specific.additional_context = Some(message);
        }
        write_output(&output)?;
        return Ok(());
    }
    
    exit_success();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_root_is_blocked() {
        let rules = get_rules();
        let command = "rm -rf /";
        let mut blocked = false;
        for rule in &rules {
            if rule.blocking && regex::Regex::new(rule.pattern).unwrap().is_match(command) {
                blocked = true;
                break;
            }
        }
        assert!(blocked);
    }

    #[test]
    fn test_grep_suggestion() {
        let rules = get_rules();
        let command = "grep pattern file.txt";
        let mut suggested = false;
        for rule in &rules {
            if !rule.blocking && regex::Regex::new(rule.pattern).unwrap().is_match(command) {
                suggested = true;
                break;
            }
        }
        assert!(suggested);
    }

    #[test]
    fn test_safe_command_allowed() {
        let rules = get_rules();
        let command = "ls -la";
        let mut matched = false;
        for rule in &rules {
            if regex::Regex::new(rule.pattern).unwrap().is_match(command) {
                matched = true;
                break;
            }
        }
        assert!(!matched);
    }
}
