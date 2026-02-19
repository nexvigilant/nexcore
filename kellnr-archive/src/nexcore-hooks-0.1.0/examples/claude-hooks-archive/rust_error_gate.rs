use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PreToolUseInput, ToolInput},
    output::PreToolUseOutput,
    HookResult,
};
use std::fs;
use regex::Regex;

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    // Only process Rust files
    let path = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

    // Skip test files
    if path.contains("/tests/") || path.ends_with("_test.rs") {
        exit_success();
    }

    // Get final content
    let content = match &tool_input {
        ToolInput::Write(w) => w.content.clone(),
        ToolInput::Edit(e) => {
            let current = fs::read_to_string(path).unwrap_or_default();
            if e.replace_all.unwrap_or(false) {
                current.replace(&e.old_string, &e.new_string)
            } else {
                current.replacen(&e.old_string, &e.new_string, 1)
            }
        }
        _ => exit_success(),
    };

    let violations = check_error_handling(&content);
    if violations.is_empty() {
        exit_success();
    }

    let mut msg = format!("PROD RUST ERROR HANDLING VIOLATIONS ({})\n\n", violations.len());
    for (line, pattern) in &violations {
        msg.push_str(&format!("- Line {}: contains '{}'\n", line, pattern));
    }
    msg.push_str("\nRULE: Do not unwrap, panic, or leave todos in production code.\n");
    msg.push_str("FIX: Use ? or proper error handling, OR add a '// SAFETY:' or '// INVARIANT:' comment.\n");

    let output = PreToolUseOutput::deny(msg);
    write_output(&output)?;

    Ok(())
}

fn check_error_handling(content: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();

    // Patterns that indicate dangerous error handling
    let patterns = [
        (Regex::new(r"\.unwrap\(").unwrap(), ".unwrap()"),
        (Regex::new(r"\.expect\(").unwrap(), ".expect()"),
        (Regex::new(r"panic!\(").unwrap(), "panic!()"),
        (Regex::new(r"todo!\(").unwrap(), "todo!()"),
    ];

    for (i, line) in lines.iter().enumerate() {
        // Skip comment lines
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        for (re, label) in &patterns {
            if re.is_match(line) {
                // Check for safety override
                let has_safety = (i > 0 && is_safety_comment(lines[i-1]))
                    || is_safety_comment(line);

                if !has_safety {
                    violations.push((i + 1, *label));
                }
            }
        }
    }

    violations
}

fn is_safety_comment(line: &str) -> bool {
    let upper = line.to_uppercase();
    upper.contains("// SAFETY:") || upper.contains("// INVARIANT:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_error_handling() {
        let content = "let x = val?;";
        assert!(check_error_handling(content).is_empty());
    }

    #[test]
    fn test_invalid_unwrap() {
        let content = "let x = val.unwrap();";
        assert_eq!(check_error_handling(content).len(), 1);
    }

    #[test]
    fn test_safety_override() {
        let content = "// SAFETY: guaranteed valid\nlet x = val.unwrap();";
        assert!(check_error_handling(content).is_empty());
    }

    #[test]
    fn test_todo_blocked() {
        let content = "fn fix_me() { todo!(); }";
        assert_eq!(check_error_handling(content).len(), 1);
    }
}
