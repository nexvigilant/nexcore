use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PreToolUseInput, ToolInput},
    output::PreToolUseOutput,
    HookResult,
};
use std::fs;

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    // Only process Rust files
    let path = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

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

    // Only analyze files with tests
    if !content.contains("#[test]") {
        exit_success();
    }

    let issues = check_test_quality(&content);
    if issues.is_empty() {
        exit_success();
    }

    let mut msg = String::from("--- TEST QUALITY AUDIT ---\n");
    for (line, issue) in &issues {
        msg.push_str(&format!("Line {}: {}\n", line, issue));
    }
    msg.push_str("--------------------------\n");

    // Advisory only - permit the write but inject context for the model
    let output = PreToolUseOutput::allow("Testing quality audit performed").with_context(msg);
    write_output(&output)?;

    Ok(())
}

fn check_test_quality(content: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut issues = Vec::new();

    // Patterns
    let test_marker = "#[test]";
    let assert_marker = "assert";
    let ignore_marker = "#[ignore]";
    let ignore_reason = "#[ignore = ";

    let mut current_test_line = None;
    let mut current_test_has_assert = false;

    for (i, line) in lines.iter().enumerate() {
        if line.contains(test_marker) {
            // Check previous test before starting new one
            if let Some(l) = current_test_line {
                if !current_test_has_assert {
                    issues.push((l, "Test appeared to have no assertions"));
                }
            }
            current_test_line = Some(i + 1);
            current_test_has_assert = false;
        }

        if current_test_line.is_some() && line.contains(assert_marker) {
            current_test_has_assert = true;
            if line.contains("assert!(true)") {
                issues.push((i + 1, "Tautological assert!(true) found"));
            }
        }

        if line.contains(ignore_marker) && !line.contains(ignore_reason) {
            issues.push((i + 1, "Ignored test without reason"));
        }
    }

    // Check last test
    if let Some(l) = current_test_line {
        if !current_test_has_assert {
            issues.push((l, "Test appeared to have no assertions"));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_test() {
        let content = "#[test]\nfn t() { assert!(1 == 1); }";
        assert!(check_test_quality(content).is_empty());
    }

    #[test]
    fn test_no_assert() {
        let content = "#[test]\nfn t() { let x = 1; }";
        assert_eq!(check_test_quality(content).len(), 1);
    }

    #[test]
    fn test_tautological() {
        let content = "#[test]\nfn t() { assert!(true); }";
        assert_eq!(check_test_quality(content).len(), 1);
    }

    #[test]
    fn test_ignore_no_reason() {
        let content = "#[ignore]\n#[test]\nfn t() {}";
        assert_eq!(check_test_quality(content).len(), 2); // No reason + No assert
    }
}
