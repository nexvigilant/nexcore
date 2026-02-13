//! Model Delegation Loop Hook
//!
//! Event: UserPromptSubmit - Detects bulk task patterns and suggests Gemini delegation
//!
//! Triggers when:
//! - User prompt contains bulk patterns (N > 10 items, "all", "every", "generate tests for")
//! - Task involves repetitive, pattern-heavy work suitable for Flash
//!
//! Actions:
//! - Injects context with delegation recommendation
//! - Suggests template from model-delegation skill

use nexcore_hooks::protocol::HookOutput;
use nexcore_hooks::{exit_success_auto, read_input};

/// Patterns that indicate bulk/repetitive tasks suitable for Gemini
const BULK_PATTERNS: &[&str] = &[
    "generate tests for all",
    "generate tests for every",
    "add tests to all",
    "document all",
    "docstrings for all",
    "create tests for",
    "generate documentation",
    "extract types from",
    "convert all",
    "migrate all",
    "for each file",
    "for every module",
    "bulk",
    "112 tools",
    "all mcp tools",
    "comprehensive tests",
];

/// Patterns that indicate high-stakes work NOT suitable for delegation
const KEEP_LOCAL_PATTERNS: &[&str] = &[
    "security",
    "authentication",
    "credentials",
    "production",
    "deploy",
    "delete",
    "drop",
    "sensitive",
    "secret",
    "api key",
    "password",
];

/// Minimum estimated item count to trigger delegation suggestion
const MIN_BULK_ITEMS: usize = 10;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get the user prompt (UserPromptSubmit uses 'prompt' field)
    let text_to_analyze = input
        .prompt
        .as_deref()
        .or(input.message.as_deref())
        .unwrap_or("");

    // Skip if empty or short
    if text_to_analyze.len() < 20 {
        exit_success_auto();
    }

    let text_lower = text_to_analyze.to_lowercase();

    // Check for high-stakes patterns first (abort delegation)
    for pattern in KEEP_LOCAL_PATTERNS {
        if text_lower.contains(pattern) {
            exit_success_auto();
        }
    }

    // Check for bulk task patterns
    let mut bulk_score = 0;
    let mut matched_patterns = Vec::new();

    for pattern in BULK_PATTERNS {
        if text_lower.contains(pattern) {
            bulk_score += 1;
            matched_patterns.push(*pattern);
        }
    }

    // Estimate item count from numbers in text
    let estimated_items = extract_item_count(&text_lower);

    // Decision: suggest delegation if bulk patterns found or high item count
    let should_delegate = bulk_score >= 1 || estimated_items >= MIN_BULK_ITEMS;

    if !should_delegate {
        exit_success_auto();
    }

    // Generate delegation recommendation
    let task_type = infer_task_type(&text_lower);
    let template_path = get_template_path(&task_type);
    let confidence = calculate_confidence(bulk_score, estimated_items);

    let context = format!(
        "🔀 **Delegation Candidate Detected**\n\n\
        | Attribute | Value |\n\
        |-----------|-------|\n\
        | Task type | {} |\n\
        | Estimated items | {} |\n\
        | Confidence | {:.0}% |\n\
        | Matched patterns | {} |\n\n\
        **Recommendation:** Delegate to Gemini Flash using `/delegate` skill.\n\
        Template: `{}`",
        task_type,
        if estimated_items > 0 {
            estimated_items.to_string()
        } else {
            "bulk".to_string()
        },
        confidence * 100.0,
        matched_patterns.join(", "),
        template_path
    );

    // Output context injection
    let output = HookOutput::with_context(context);
    println!("{}", serde_json::to_string(&output).unwrap_or_default());
    std::process::exit(0);
}

/// Extract estimated item count from text
fn extract_item_count(text: &str) -> usize {
    // Look for explicit numbers
    let numbers: Vec<usize> = text
        .split_whitespace()
        .filter_map(|word| word.trim_matches(|c: char| !c.is_numeric()).parse().ok())
        .filter(|&n| n >= 5 && n <= 1000) // Reasonable range for bulk tasks
        .collect();

    // Return largest number found, or 0 if none
    numbers.into_iter().max().unwrap_or(0)
}

/// Infer task type from text
fn infer_task_type(text: &str) -> &'static str {
    if text.contains("test") {
        "test_generation"
    } else if text.contains("doc") || text.contains("comment") {
        "doc_generation"
    } else if text.contains("type") || text.contains("schema") || text.contains("interface") {
        "schema_extraction"
    } else if text.contains("convert") || text.contains("transform") || text.contains("migrate") {
        "data_transformation"
    } else if text.contains("audit") || text.contains("check") || text.contains("compliance") {
        "compliance_audit"
    } else {
        "bulk_generation"
    }
}

/// Get template path for task type
fn get_template_path(task_type: &str) -> String {
    let base = "~/.claude/skills/model-delegation/templates";
    match task_type {
        "test_generation" => format!("{}/test-generation.md", base),
        "doc_generation" => format!("{}/doc-generation.md", base),
        "schema_extraction" => format!("{}/schema-extraction.md", base),
        _ => format!("{}/test-generation.md", base), // Default
    }
}

/// Calculate confidence score for delegation
fn calculate_confidence(bulk_score: usize, estimated_items: usize) -> f64 {
    let pattern_factor = (bulk_score as f64 * 0.2).min(0.6);
    let count_factor = if estimated_items >= 100 {
        0.4
    } else if estimated_items >= 50 {
        0.3
    } else if estimated_items >= 10 {
        0.2
    } else {
        0.1
    };

    (pattern_factor + count_factor).min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_detection() {
        assert!(extract_item_count("generate tests for 112 tools") >= 100);
        assert_eq!(
            infer_task_type("generate tests for all modules"),
            "test_generation"
        );
        assert_eq!(infer_task_type("document all functions"), "doc_generation");
    }

    #[test]
    fn test_confidence() {
        assert!(calculate_confidence(2, 100) > 0.5);
        assert!(calculate_confidence(1, 10) < 0.5);
    }
}
