//! Numeric Premise Validator Hook
//!
//! Validates that numeric output is grounded in verifiable sources.
//! Warns on potential hallucinated statistics, percentages, or counts.
//!
//! # Tier: T2-C
//! Grounds to: T1(String, u64, bool) via regex pattern matching.
//!
//! # Hook Protocol
//! - Event: Stop or PostToolUse
//! - Exit 0: No ungrounded numbers
//! - Exit 1: Warn - numbers need verification

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{pass, warn};
use regex::Regex;

const HOOK_NAME: &str = "numeric-premise-validator";
use serde::Deserialize;
use std::io::Read;

/// Numeric source classification.
/// Tier: T2-P, grounds to T1(u8).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum NumericSource {
    LineNumber = 0,
    CodeLiteral = 1,
    ToolResult = 2,
    Ungrounded = 3,
}

/// Detected numeric instance. Tier: T2-C.
#[derive(Debug)]
struct NumericInstance {
    value: String,
    source: NumericSource,
    context: String,
    position: usize,
}

/// Extended hook input. Tier: T2-C.
#[derive(Debug, Deserialize)]
struct StopEventInput {
    #[serde(default)]
    assistant_response: Option<String>,
    #[serde(default)]
    tool_input: Option<ToolInputExt>,
}

#[derive(Debug, Deserialize)]
struct ToolInputExt {
    content: Option<String>,
    new_string: Option<String>,
}

fn main() {
    let content = read_content();
    if content.is_empty() {
        pass();
    }

    let instances = detect_numerics(&content);
    let ungrounded = filter_ungrounded(&instances);

    if ungrounded.is_empty() {
        pass();
    }

    let msg = format_warning(&ungrounded);
    // Emit cytokine signal before warning (IL-6 = acute response)
    emit_check_failed(
        HOOK_NAME,
        &format!("{} ungrounded numbers", ungrounded.len()),
    );
    warn(&msg);
}

/// Read content from stdin.
fn read_content() -> String {
    let mut buffer = String::new();
    if std::io::stdin().read_to_string(&mut buffer).is_err() {
        return String::new();
    }
    parse_content(&buffer)
}

/// Parse content from JSON input.
fn parse_content(buffer: &str) -> String {
    if buffer.trim().is_empty() {
        return String::new();
    }
    let input: StopEventInput = match serde_json::from_str(buffer) {
        Ok(i) => i,
        Err(_) => return String::new(),
    };
    // Extract tool content first to avoid partial move
    let tool_content = extract_tool_content(&input);
    input
        .assistant_response
        .or(tool_content)
        .unwrap_or_default()
}

fn extract_tool_content(input: &StopEventInput) -> Option<String> {
    input
        .tool_input
        .as_ref()
        .and_then(|t| t.content.clone().or_else(|| t.new_string.clone()))
}

/// Filter to ungrounded instances.
fn filter_ungrounded(instances: &[NumericInstance]) -> Vec<&NumericInstance> {
    instances
        .iter()
        .filter(|n| n.source == NumericSource::Ungrounded)
        .collect()
}

/// Format warning message.
fn format_warning(ungrounded: &[&NumericInstance]) -> String {
    let mut msg = format!(
        "NUMERIC PREMISE CHECK: {} ungrounded number(s)\n\n",
        ungrounded.len()
    );
    for (i, inst) in ungrounded.iter().take(5).enumerate() {
        msg.push_str(&format!(
            "  {}. \"{}\" - {}\n",
            i + 1,
            inst.value,
            truncate(&inst.context, 40)
        ));
    }
    if ungrounded.len() > 5 {
        msg.push_str(&format!("  ... and {} more\n", ungrounded.len() - 5));
    }
    msg
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max - 3).collect::<String>())
    }
}

/// Detect all numeric instances.
fn detect_numerics(content: &str) -> Vec<NumericInstance> {
    let code_ranges = find_code_ranges(content);
    let mut instances = Vec::new();

    instances.extend(detect_percentages(content, &code_ranges));
    instances.extend(detect_hedged_stats(content, &code_ranges));
    instances.extend(detect_speedups(content, &code_ranges));

    instances.sort_by_key(|i| i.position);
    instances.dedup_by_key(|i| i.position);
    instances
}

/// Find code block ranges.
fn find_code_ranges(content: &str) -> Vec<(usize, usize)> {
    let re = Regex::new(r"```[\s\S]*?```|`[^`]+`").unwrap();
    re.find_iter(content)
        .map(|m| (m.start(), m.end()))
        .collect()
}

fn in_code_block(pos: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|(s, e)| pos >= *s && pos < *e)
}

/// Detect percentage claims.
fn detect_percentages(content: &str, code_ranges: &[(usize, usize)]) -> Vec<NumericInstance> {
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*%").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .filter(|m| !in_code_block(m.start(), code_ranges))
        .map(|m| NumericInstance {
            value: format!("{}%", m.as_str()),
            source: NumericSource::Ungrounded,
            context: extract_context(content, m.start()),
            position: m.start(),
        })
        .collect()
}

/// Detect hedged statistics.
fn detect_hedged_stats(content: &str, code_ranges: &[(usize, usize)]) -> Vec<NumericInstance> {
    let re = Regex::new(r"(?i)(?:approximately|about|around|roughly|nearly|over|under)\s+(\d+(?:,\d{3})*(?:\.\d+)?)").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .filter(|m| !in_code_block(m.start(), code_ranges))
        .map(|m| NumericInstance {
            value: m.as_str().to_string(),
            source: NumericSource::Ungrounded,
            context: extract_context(content, m.start()),
            position: m.start(),
        })
        .collect()
}

/// Detect speedup claims.
fn detect_speedups(content: &str, code_ranges: &[(usize, usize)]) -> Vec<NumericInstance> {
    let re = Regex::new(r"(\d+(?:\.\d+)?)\s*x\b").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .filter(|m| !in_code_block(m.start(), code_ranges))
        .map(|m| {
            let ctx = extract_context(content, m.start());
            let src = if ctx.contains("benchmark") {
                NumericSource::ToolResult
            } else {
                NumericSource::Ungrounded
            };
            NumericInstance {
                value: format!("{}x", m.as_str()),
                source: src,
                context: ctx,
                position: m.start(),
            }
        })
        .collect()
}

/// Extract context around position.
fn extract_context(content: &str, pos: usize) -> String {
    let start = content[..pos]
        .char_indices()
        .rev()
        .nth(25)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let end = content[pos..]
        .char_indices()
        .nth(25)
        .map(|(i, _)| pos + i)
        .unwrap_or(content.len());
    content[start..end].replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_literals_grounded() {
        let content = "Use `timeout: 5000` in config";
        let instances = detect_numerics(content);
        assert!(filter_ungrounded(&instances).is_empty());
    }

    #[test]
    fn test_percentages_flagged() {
        let content = "This improves by 50%";
        let instances = detect_numerics(content);
        assert!(!filter_ungrounded(&instances).is_empty());
    }

    #[test]
    fn test_hedged_stats_flagged() {
        let content = "approximately 10,000 users";
        let instances = detect_numerics(content);
        assert!(!filter_ungrounded(&instances).is_empty());
    }
}
