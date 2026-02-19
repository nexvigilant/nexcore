use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PreToolUseInput, ToolInput},
    output::PreToolUseOutput,
    HookResult,
};
use std::fs;

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    // Parse the raw tool_input JSON into the structured ToolInput enum
    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    // Only process file-related tools
    let path = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

    // Get the content as it would be after the tool executes
    let content = match &tool_input {
        ToolInput::Write(w) => w.content.clone(),
        ToolInput::Edit(e) => {
            // Read current file and apply edit to simulate the result
            let current = fs::read_to_string(path).unwrap_or_default();
            if e.replace_all.unwrap_or(false) {
                current.replace(&e.old_string, &e.new_string)
            } else {
                current.replacen(&e.old_string, &e.new_string, 1)
            }
        }
        _ => exit_success(),
    };

    let violations = check_unsafe_blocks(&content);
    if violations.is_empty() {
        exit_success();
    }

    let mut msg = String::from("UNSAFE BLOCK REQUIRES DOCUMENTATION\n\n");
    for line_num in &violations {
        msg.push_str(&format!(
            "- Line {line_num}: unsafe block without // SAFETY: comment\n"
        ));
    }
    msg.push_str("\nREQUIRED structure:\n");
    msg.push_str("  // SAFETY: This unsafe block is sound because:\n");
    msg.push_str("  // 1. [Invariant]: explanation\n");
    msg.push_str("  unsafe { ... }\n");

    let output = PreToolUseOutput::deny(msg);
    write_output(&output)?;

    Ok(())
}

fn check_unsafe_blocks(content: &str) -> Vec<usize> {
    let lines: Vec<&str> = content.lines().collect();
    let mut violations = Vec::new();
    
    // Pattern to look for
    let marker = "unsafe {";

    for (i, line) in lines.iter().enumerate() {
        if line.contains(marker) {
            // Check previous 2 lines for SAFETY comment
            let has_safety = (i > 0 && lines[i-1].contains("// SAFETY:"))
                || (i > 1 && lines[i-2].contains("// SAFETY:"));

            if !has_safety {
                violations.push(i + 1);
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_unsafe() {
        let content = "// SAFETY: trust me\nunsafe { core::mem::forget(x); }";
        assert!(check_unsafe_blocks(content).is_empty());
    }

    #[test]
    fn test_invalid_unsafe() {
        let content = "fn main() {\n    unsafe { panic!(); }\n}";
        assert_eq!(check_unsafe_blocks(content), vec![2]);
    }

    #[test]
    fn test_remote_safety() {
        let content = "// SAFETY: okay\n\nunsafe { }";
        assert!(check_unsafe_blocks(content).is_empty());
    }
}
