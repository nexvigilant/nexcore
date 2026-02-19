use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PostToolUseInput, ToolInput},
    output::PostToolUseOutput,
    HookResult,
};
use std::fs;

fn main() -> HookResult<()> {
    let input: PostToolUseInput = read_input()?;

    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    let path = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

    let content = fs::read_to_string(path).unwrap_or_default();
    let issues = check_doc_quality(&content);

    if issues.is_empty() {
        exit_success();
    }

    let mut msg = String::from("--- DOC QUALITY AUDIT (ADVISORY) ---\n");
    for (line, issue) in &issues {
        msg.push_str(&format!("Line {}: {}\n", line, issue));
    }
    msg.push_str("\nTip: Good docs describe WHY and HOW, not just WHAT (avoid restating the name).\n");
    msg.push_str("------------------------------------\n");

    let output = PostToolUseOutput::with_context(msg);
    write_output(&output)?;

    Ok(())
}

fn check_doc_quality(content: &str) -> Vec<(usize, &'static str)> {
    let anti_patterns: &[(&str, &'static str)] = &[
        ("/// Creates a new", "Vague - describe what it represents or its initial state"),
        ("/// Returns a new", "Vague - describe the purpose of the return value"),
        ("/// Gets the", "Restates the name - add more context about the value"),
        ("/// Sets the", "Restates the name - describe the effect or side-effects"),
        ("/// Represents a", "Vague - describe the role or invariant"),
    ];

    let mut issues = Vec::new();
    for (i, line) in content.lines().enumerate() {
        for (pattern, issue) in anti_patterns {
            if line.trim().starts_with(pattern) {
                issues.push((i + 1, *issue));
            }
        }
    }
    issues
}
