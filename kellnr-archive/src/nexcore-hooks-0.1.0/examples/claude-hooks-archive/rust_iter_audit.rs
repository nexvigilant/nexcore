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
    let issues = detect_iterator_issues(&content);

    if issues.is_empty() {
        exit_success();
    }

    let mut msg = format!("ITERATOR ANTI-PATTERNS DETECTED ({})\n\n", issues.len());
    for (line, pattern, fix) in &issues {
        msg.push_str(&format!("- Line {}: Found '{}'\n  Fix: {}\n", line, pattern, fix));
    }
    msg.push_str("\nRedundant .collect() calls allocate heap memory needlessly.\n");

    let output = PostToolUseOutput::block(msg);
    write_output(&output)?;

    Ok(())
}

fn detect_iterator_issues(content: &str) -> Vec<(usize, String, String)> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let re_collect_iter = Regex::new(r"\.collect::<.*?>\(\)\.iter\(\)").unwrap();
    let re_collect_into_iter = Regex::new(r"\.collect::<.*?>\(\)\.into_iter\(\)").unwrap();
    let re_simple_collect_iter = Regex::new(r"\.collect\(\)\.iter\(\)").unwrap();
    let re_simple_collect_into_iter = Regex::new(r"\.collect\(\)\.into_iter\(\)").unwrap();

    for (i, line) in lines.iter().enumerate() {
        if re_collect_iter.is_match(line) || re_simple_collect_iter.is_match(line) {
            issues.push((i + 1, ".collect().iter()".to_string(), "remove .collect() and .iter() (or just .collect())".to_string()));
        } else if re_collect_into_iter.is_match(line) || re_simple_collect_into_iter.is_match(line) {
            issues.push((i + 1, ".collect().into_iter()".to_string(), "remove .collect() and .into_iter()".to_string()));
        }
    }

    issues
}
