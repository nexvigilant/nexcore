use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PostToolUseInput, ToolInput},
    output::PostToolUseOutput,
    HookResult,
};
use std::fs;
use regex::Regex;

fn main() -> HookResult<()> {
    let input: PostToolUseInput = read_input()?;

    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    let path = match tool_input.file_path() {
        Some(p) if p.ends_with(".rs") => p,
        _ => exit_success(),
    };

    if path.contains("/tests/") || path.ends_with("_test.rs") {
        exit_success();
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let issues = detect_loop_clones(&content);

    if issues.is_empty() {
        exit_success();
    }

    let mut msg = String::from("--- CLONE AUDIT (ADVISORY) ---\n");
    for (line, code) in &issues {
        msg.push_str(&format!("Line {}: Found .clone() in loop: {}\n", line, code.trim()));
    }
    msg.push_str("\nTip: Clones in loops can be expensive. Consider hoisting or using references.\n");
    msg.push_str("------------------------------\n");

    let output = PostToolUseOutput::with_context(msg);
    write_output(&output)?;

    Ok(())
}

fn detect_loop_clones(content: &str) -> Vec<(usize, String)> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let re_loop = Regex::new(r"\b(for|while|loop)\b").unwrap();
    let re_clone = Regex::new(r"\.clone\(\)").unwrap();

    let mut in_loop = false;
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if re_loop.is_match(line) {
            in_loop = true;
        }

        if in_loop {
            brace_count += line.chars().filter(|&c| c == '{').count() as i32 
                         - line.chars().filter(|&c| c == '}').count() as i32;

            if re_clone.is_match(line) {
                issues.push((i + 1, line.to_string()));
            }

            if brace_count <= 0 && line.contains('}') {
                in_loop = false;
            }
        }
    }

    issues
}
