//! Passive primitive extraction hook.
//!
//! Runs on PostToolUse:Write/Edit to extract T3 candidates from modified files.

use std::io::{self, Read};

fn main() {
    // Read tool input from stdin
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    // Parse JSON input
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(&input) else {
        return;
    };

    // Only process Write/Edit tools
    let tool_name = json.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        return;
    }

    // Get file path
    let file_path = json
        .get("tool_input")
        .and_then(|v| v.get("file_path"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Only process .rs files for now
    if !file_path.ends_with(".rs") {
        return;
    }

    // Extract potential T3 candidates (struct/enum/trait names)
    if let Some(content) = json
        .get("tool_input")
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_str())
    {
        let candidates: Vec<&str> = content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("pub enum ")
                    || trimmed.starts_with("pub trait ")
                {
                    // Extract name (word after keyword)
                    trimmed
                        .split_whitespace()
                        .nth(2)
                        .map(|s| s.trim_end_matches(|c| c == '{' || c == '<' || c == '('))
                } else {
                    None
                }
            })
            .collect();

        if !candidates.is_empty() {
            eprintln!(
                "● primitive-extractor: T3 candidates in {}: {:?}",
                file_path, candidates
            );
        }
    }
}
