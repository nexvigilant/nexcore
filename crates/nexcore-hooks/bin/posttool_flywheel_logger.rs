//! Flywheel Logger - PostToolUse:Edit|Write
//!
//! After LLM fallbacks in skills, logs for flywheel analysis.
//! The flywheel loop is: log -> analyze -> improve corpus -> reduce LLM dependency.
//!
//! This hook:
//! 1. Detects when LLM fallback code is added/modified
//! 2. Reminds to log fallback patterns for analysis
//! 3. Suggests adding telemetry for flywheel improvement
//!
//! The goal is to continuously improve deterministic coverage by learning
//! from LLM fallback patterns and adding them to the deterministic corpus.

use chrono::{DateTime, Utc};
use nexcore_hooks::{exit_ok, exit_warn, get_content, get_file_path, is_rust_file, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Flywheel log entry for analysis
#[derive(Debug, Serialize, Deserialize)]
struct FlywheelLogEntry {
    timestamp: DateTime<Utc>,
    file_path: String,
    fallback_type: String,
    context: String,
    suggestion: String,
}

/// Patterns indicating LLM fallback usage
const FALLBACK_PATTERNS: &[(&str, &str)] = &[
    ("fallback", "generic_fallback"),
    ("Fallback", "generic_fallback"),
    ("delegate_to_llm", "llm_delegation"),
    ("llm_fallback", "llm_delegation"),
    ("ask_llm", "llm_query"),
    ("query_llm", "llm_query"),
    ("generate_response", "llm_generation"),
    ("ai_complete", "llm_completion"),
    ("model.generate", "llm_generation"),
    ("_ =>", "catch_all_fallback"),
    ("else {", "else_fallback"),
];

/// Patterns indicating flywheel logging is already in place
const FLYWHEEL_LOGGING_PATTERNS: &[&str] = &[
    "log_fallback",
    "record_fallback",
    "flywheel_log",
    "telemetry",
    "track_fallback",
    "emit_fallback_event",
    "fallback_counter",
    "increment_fallback",
    "tracing::info",
    "tracing::warn",
    "log::info",
    "log::warn",
];

/// Check if content has LLM fallback patterns
fn detect_fallback_patterns(content: &str) -> Vec<(&'static str, &'static str)> {
    FALLBACK_PATTERNS
        .iter()
        .filter(|(pattern, _)| content.contains(pattern))
        .copied()
        .collect()
}

/// Check if flywheel logging is already present
fn has_flywheel_logging(content: &str) -> bool {
    FLYWHEEL_LOGGING_PATTERNS
        .iter()
        .any(|p| content.contains(p))
}

/// Get the flywheel log path
fn flywheel_log_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{home}/.claude/flywheel/fallback_log.json")
}

/// Append entry to flywheel log (best-effort, non-critical)
fn append_to_flywheel_log(entry: &FlywheelLogEntry) {
    let log_path = flywheel_log_path();

    // Ensure directory exists (best-effort for logging)
    if let Some(parent) = Path::new(&log_path).parent() {
        if fs::create_dir_all(parent).is_err() {
            // Directory creation failed, skip logging
            return;
        }
    }

    // Read existing entries
    let mut entries: Vec<FlywheelLogEntry> = fs::read_to_string(&log_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    // Append new entry
    entries.push(FlywheelLogEntry {
        timestamp: entry.timestamp,
        file_path: entry.file_path.clone(),
        fallback_type: entry.fallback_type.clone(),
        context: entry.context.clone(),
        suggestion: entry.suggestion.clone(),
    });

    // Keep only last 1000 entries
    if entries.len() > 1000 {
        let skip_count = entries.len() - 1000;
        entries = entries.into_iter().skip(skip_count).collect();
    }

    // Write back (best-effort, log file is non-critical)
    if let Ok(json) = serde_json::to_string_pretty(&entries) {
        // Ignore write errors for telemetry logging
        drop(fs::write(&log_path, json));
    }
}

/// Generate suggestion based on fallback type
fn generate_suggestion(fallback_type: &str) -> String {
    match fallback_type {
        "catch_all_fallback" => {
            "Analyze common patterns hitting catch-all. Add explicit match arms for frequent cases."
                .to_string()
        }
        "llm_delegation" => {
            "Log the inputs triggering LLM delegation. Build lookup table for common inputs."
                .to_string()
        }
        "llm_query" => {
            "Cache LLM query results. Build pattern library for common questions.".to_string()
        }
        "llm_generation" => {
            "Template common generations. Use LLM only for novel content.".to_string()
        }
        "llm_completion" => {
            "Build completion cache. Most completions repeat; cache them.".to_string()
        }
        "else_fallback" => {
            "The else branch indicates uncovered cases. Add if-else for common patterns."
                .to_string()
        }
        _ => "Log fallback inputs and analyze for patterns to add to deterministic corpus."
            .to_string(),
    }
}

/// Extract context around fallback pattern
fn extract_context(content: &str, pattern: &str) -> String {
    if let Some(pos) = content.find(pattern) {
        let start = pos.saturating_sub(50);
        let end = (pos + pattern.len() + 50).min(content.len());
        let context = &content[start..end];
        // Clean up for single-line storage
        context
            .replace('\n', " ")
            .replace("  ", " ")
            .chars()
            .take(100)
            .collect()
    } else {
        String::new()
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only check Write and Edit tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" && tool_name != "Edit" {
        exit_ok();
    }

    // Get tool_input
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_ok(),
    };

    // Get file path
    let file_path = match get_file_path(tool_input) {
        Some(p) => p,
        None => exit_ok(),
    };

    // Only check Rust files
    if !is_rust_file(&file_path) {
        exit_ok();
    }

    // Skip test files
    if file_path.contains("/tests/") || file_path.contains("_test.rs") {
        exit_ok();
    }

    // Get content
    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Detect fallback patterns
    let fallbacks = detect_fallback_patterns(&content);

    if fallbacks.is_empty() {
        exit_ok();
    }

    // Check if flywheel logging is already in place
    let has_logging = has_flywheel_logging(&content);

    // Log to flywheel for analysis
    for (pattern, fallback_type) in &fallbacks {
        let entry = FlywheelLogEntry {
            timestamp: Utc::now(),
            file_path: file_path.clone(),
            fallback_type: fallback_type.to_string(),
            context: extract_context(&content, pattern),
            suggestion: generate_suggestion(fallback_type),
        };
        append_to_flywheel_log(&entry);
    }

    // Warn if no flywheel logging in code
    if !has_logging {
        let fallback_types: Vec<_> = fallbacks.iter().map(|(_, t)| *t).collect();
        let unique_types: std::collections::HashSet<_> = fallback_types.into_iter().collect();
        let types_str: Vec<_> = unique_types.into_iter().collect();

        exit_warn(&format!(
            "Fallback patterns detected ({}). Add flywheel logging: \
             log inputs triggering fallbacks for pattern analysis. \
             Goal: convert frequent fallbacks to deterministic handlers. \
             Log location: {}",
            types_str.join(", "),
            flywheel_log_path()
        ));
    }

    exit_ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_fallback_patterns() {
        let code_with_fallback = r#"
            match command {
                "help" => help(),
                _ => delegate_to_llm(command),
            }
        "#;
        let patterns = detect_fallback_patterns(code_with_fallback);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|(_, t)| *t == "llm_delegation"));
    }

    #[test]
    fn test_has_flywheel_logging() {
        assert!(has_flywheel_logging("log_fallback(input);"));
        assert!(has_flywheel_logging(
            "tracing::info!(\"fallback triggered\");"
        ));
        assert!(!has_flywheel_logging("fallback_handler();"));
    }

    #[test]
    fn test_generate_suggestion() {
        let suggestion = generate_suggestion("catch_all_fallback");
        assert!(suggestion.contains("match arms"));

        let suggestion = generate_suggestion("llm_delegation");
        assert!(suggestion.contains("lookup table"));
    }

    #[test]
    fn test_extract_context() {
        let content = "fn handle() { if complex { delegate_to_llm(x); } }";
        let context = extract_context(content, "delegate_to_llm");
        assert!(context.contains("delegate_to_llm"));
    }
}
