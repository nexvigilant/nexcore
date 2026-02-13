//! # PreToolUse: Magic Number Detector
//!
//! Detects magic numbers in Rust code and suggests extraction to named constants.
//! Part of the capability acceleration infrastructure learned from guardian/response.rs extraction.
//!
//! ## Hook Event
//! - **Event**: `PreToolUse:Write` or `PreToolUse:Edit`
//! - **Action**: Warn when magic numbers detected in Rust files
//!
//! ## Detection Patterns
//! - Numeric literals in match arms (except 0, 1)
//! - Float literals outside of obvious contexts (0.0, 1.0)
//! - Integer literals > 2 in business logic
//! - Priority values, thresholds, limits
//!
//! ## Exit Codes
//! - 0: No magic numbers or non-Rust file
//! - 1: Magic numbers detected (warning, allow proceed)
//! - 2: Block (reserved for future strict mode)

use regex::Regex;
use serde::Deserialize;
use std::io::{self, Read};

/// Minimum count of magic numbers to trigger warning
const WARNING_THRESHOLD: usize = 3;

/// Numbers that are typically not "magic" (contextually obvious)
const ALLOWED_INTEGERS: &[i64] = &[0, 1, 2, -1];
const ALLOWED_FLOATS: &[f64] = &[0.0, 1.0, 0.5, 2.0];

#[derive(Debug, Deserialize)]
struct HookInput {
    tool_name: String,
    tool_input: ToolInput,
}

#[derive(Debug, Deserialize)]
struct ToolInput {
    file_path: Option<String>,
    content: Option<String>,
    new_string: Option<String>,
}

#[derive(Debug)]
struct MagicNumber {
    line: usize,
    value: String,
    context: String,
}

struct Patterns {
    int_pattern: Regex,
    float_pattern: Regex,
    priority_pattern: Regex,
    threshold_pattern: Regex,
}

impl Patterns {
    fn new() -> Option<Self> {
        Some(Self {
            int_pattern: Regex::new(r"\b(\d{2,})\b").ok()?,
            float_pattern: Regex::new(r"\b(\d+\.\d+)\b").ok()?,
            priority_pattern: Regex::new(r"priority.*?(\d+)").ok()?,
            threshold_pattern: Regex::new(r"(max|min|limit|threshold|ceiling|floor).*?(\d+)")
                .ok()?,
        })
    }
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(0);
    }

    let hook_input: HookInput = match serde_json::from_str(&input) {
        Ok(h) => h,
        Err(_) => std::process::exit(0),
    };

    // Only check Write and Edit tools
    if !matches!(hook_input.tool_name.as_str(), "Write" | "Edit") {
        std::process::exit(0);
    }

    // Only check Rust files
    let file_path = hook_input.tool_input.file_path.unwrap_or_default();
    if !file_path.ends_with(".rs") {
        std::process::exit(0);
    }

    // Get content to analyze
    let content = hook_input
        .tool_input
        .content
        .or(hook_input.tool_input.new_string)
        .unwrap_or_default();

    if content.is_empty() {
        std::process::exit(0);
    }

    let patterns = match Patterns::new() {
        Some(p) => p,
        None => std::process::exit(0),
    };

    let magic_numbers = detect_magic_numbers(&content, &patterns);

    if magic_numbers.len() >= WARNING_THRESHOLD {
        eprintln!(
            "⚠️  Magic Number Detection: {} potential magic numbers found",
            magic_numbers.len()
        );
        eprintln!("   Consider extracting to named constants for maintainability.\n");

        for mn in magic_numbers.iter().take(5) {
            eprintln!(
                "   Line {}: {} in \"{}\"",
                mn.line,
                mn.value,
                truncate(&mn.context, 50)
            );
        }

        if magic_numbers.len() > 5 {
            eprintln!("   ... and {} more", magic_numbers.len() - 5);
        }

        eprintln!(
            "\n   Pattern: pub mod constants {{ pub const NAME: Type = {}; }}",
            magic_numbers
                .first()
                .map(|m| m.value.as_str())
                .unwrap_or("value")
        );

        // Exit 1 = warning (allow proceed)
        std::process::exit(1);
    }

    std::process::exit(0);
}

fn detect_magic_numbers(content: &str, patterns: &Patterns) -> Vec<MagicNumber> {
    let mut results = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        // Skip comments, doc comments, and const definitions
        if trimmed.starts_with("//")
            || trimmed.starts_with("///")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("pub const ")
        {
            continue;
        }

        // Skip test code
        if trimmed.contains("#[test]") || trimmed.contains("assert") {
            continue;
        }

        // Check for integers (2+ digits)
        for cap in patterns.int_pattern.captures_iter(line) {
            let value: i64 = cap[1].parse().unwrap_or(0);
            if !ALLOWED_INTEGERS.contains(&value) && value > 2 {
                results.push(MagicNumber {
                    line: line_num,
                    value: cap[1].to_string(),
                    context: trimmed.to_string(),
                });
            }
        }

        // Check for floats
        for cap in patterns.float_pattern.captures_iter(line) {
            let value: f64 = cap[1].parse().unwrap_or(0.0);
            if !ALLOWED_FLOATS.contains(&value) {
                results.push(MagicNumber {
                    line: line_num,
                    value: cap[1].to_string(),
                    context: trimmed.to_string(),
                });
            }
        }

        // Priority values (any numeric in priority context)
        for cap in patterns.priority_pattern.captures_iter(line) {
            let value: i64 = cap[1].parse().unwrap_or(0);
            if value > 1
                && !results
                    .iter()
                    .any(|r| r.line == line_num && r.value == cap[1])
            {
                results.push(MagicNumber {
                    line: line_num,
                    value: cap[1].to_string(),
                    context: format!("Priority value: {}", trimmed),
                });
            }
        }

        // Threshold/limit values
        for cap in patterns.threshold_pattern.captures_iter(line) {
            let value: i64 = cap[2].parse().unwrap_or(0);
            if value > 1
                && !results
                    .iter()
                    .any(|r| r.line == line_num && r.value == cap[2])
            {
                results.push(MagicNumber {
                    line: line_num,
                    value: cap[2].to_string(),
                    context: format!("Threshold/limit: {}", trimmed),
                });
            }
        }
    }

    // Deduplicate by line and value
    results.sort_by(|a, b| a.line.cmp(&b.line));
    results.dedup_by(|a, b| a.line == b.line && a.value == b.value);

    results
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_magic_numbers() {
        let code = r#"
            fn priority(&self) -> u8 {
                80 // High priority
            }

            let threshold = 100;
            let factor = 1.2;
        "#;

        let patterns = Patterns::new().unwrap();
        let results = detect_magic_numbers(code, &patterns);
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.value == "80"));
        assert!(results.iter().any(|r| r.value == "100"));
        assert!(results.iter().any(|r| r.value == "1.2"));
    }

    #[test]
    fn test_ignores_allowed_values() {
        let code = r#"
            let x = 0;
            let y = 1;
            let z = 1.0;
        "#;

        let patterns = Patterns::new().unwrap();
        let results = detect_magic_numbers(code, &patterns);
        assert!(results.is_empty());
    }

    #[test]
    fn test_ignores_constants() {
        let code = r#"
            pub const MAX_VALUE: u32 = 100;
            const THRESHOLD: f64 = 1.5;
        "#;

        let patterns = Patterns::new().unwrap();
        let results = detect_magic_numbers(code, &patterns);
        assert!(results.is_empty());
    }
}
