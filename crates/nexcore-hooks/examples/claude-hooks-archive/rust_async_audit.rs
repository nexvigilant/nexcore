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
    let issues = detect_async_issues(&content);

    if issues.is_empty() {
        exit_success();
    }

    let mut msg = format!("ASYNC MISUSE DETECTED ({})\n\n", issues.len());
    for (line, issue, fix) in &issues {
        msg.push_str(&format!("- Line {}: {}\n  Fix: {}\n", line, issue, fix));
    }
    msg.push_str("\nBlocking calls in an async context stall the entire runtime.\n");

    let output = PostToolUseOutput::block(msg);
    write_output(&output)?;

    Ok(())
}

fn detect_async_issues(content: &str) -> Vec<(usize, String, String)> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let re_async_fn = Regex::new(r"async\s+fn").unwrap();
    let re_blocking_call = Regex::new(r"(std::thread::sleep|std::fs::|fs::read|fs::write)").unwrap();

    let mut in_async_fn = false;
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if re_async_fn.is_match(line) {
            in_async_fn = true;
        }

        if in_async_fn {
            brace_count += line.chars().filter(|&c| c == '{').count() as i32 
                         - line.chars().filter(|&c| c == '}').count() as i32;

            if let Some(mat) = re_blocking_call.find(line) {
                let call = mat.as_str();
                let fix = match call {
                    "std::thread::sleep" => "tokio::time::sleep",
                    c if c.contains("fs::") => "tokio::fs",
                    _ => "use an async alternative",
                };
                issues.push((i + 1, format!("Found blocking call '{}' inside async fn", call), fix.to_string()));
            }

            if brace_count <= 0 && line.contains('}') {
                in_async_fn = false;
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_blocking() {
        let content = "async fn do_work() {\n    std::thread::sleep(d);\n}";
        let issues = detect_async_issues(content);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].1.contains("std::thread::sleep"));
    }
}
