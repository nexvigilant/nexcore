//! Extraction Completeness Checker
//!
//! Stop hook that verifies primitive extraction followed the algorithm.
//! Returns `{"decision":"block"}` if extraction is incomplete, forcing completion.
//!
//! Checks:
//! - Source mode is declared (source_mode:)
//! - Test cards are shown for analyzed terms
//! - T3 items have confidence scores
//!
//! This hook only activates when the conversation contains primitive extraction markers.

use nexcore_hooks::{exit_ok, read_input};
use std::fs;

/// Markers indicating primitive extraction is happening
const EXTRACTION_MARKERS: &[&str] = &[
    "/extract-primitives",
    "/decompose",
    "/primitives",
    "what are the primitives",
    "break this down to foundations",
    "find the atomic concepts",
    "tier_1_universal:",
    "tier_2_primitives:",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Avoid infinite loops - if stop hook already active, allow stop
    if input.stop_hook_active == Some(true) {
        exit_ok();
    }

    // Read transcript to check for extraction context
    let transcript_content = match &input.transcript_path {
        Some(path) => fs::read_to_string(path).unwrap_or_default(),
        None => exit_ok(),
    };

    // Check if this session involves primitive extraction
    let has_extraction = EXTRACTION_MARKERS.iter().any(|marker| {
        transcript_content
            .to_lowercase()
            .contains(&marker.to_lowercase())
    });

    if !has_extraction {
        exit_ok();
    }

    // Check extraction completeness
    let issues = check_completeness(&transcript_content);

    if issues.is_empty() {
        exit_ok();
    }

    // Force continuation with feedback
    let feedback = format!(
        "Primitive extraction incomplete:\n{}",
        issues
            .iter()
            .map(|i| format!("  - {}", i))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Output block decision - force completion
    let output = serde_json::json!({
        "decision": "block",
        "reason": feedback
    });

    println!("{}", output);
    eprintln!(
        "\x1b[33m●\x1b[0m stop_extraction_completeness: {}",
        issues.first().unwrap_or(&"incomplete".to_string())
    );
    std::process::exit(0);
}

/// Check extraction completeness, return list of issues
fn check_completeness(content: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Check 1: Source mode declared
    let has_source_mode = content.contains("source_mode:")
        || content.contains("source_mode =")
        || content.contains("Mode:")
        || content.contains("**Full Corpus**")
        || content.contains("**Expert Generation**")
        || content.contains("**Hybrid");

    // Only require source_mode if there's actual extraction output
    let has_extraction_output = content.contains("primitives:")
        || content.contains("tier_1_universal:")
        || content.contains("PRIMITIVE TEST:");

    if has_extraction_output && !has_source_mode {
        issues.push("Missing source_mode declaration (full/partial/expert/hybrid)".to_string());
    }

    // Check 2: Test cards present
    let has_test_cards =
        content.contains("PRIMITIVE TEST:") || content.contains("│ PRIMITIVE TEST:");

    // Count extraction items
    let extraction_count = content.matches("- primitive:").count()
        + content.matches("- composite:").count()
        + content.matches("- item:").count();

    if has_extraction_output && extraction_count > 0 && !has_test_cards {
        issues.push(format!(
            "Missing PRIMITIVE TEST cards ({} items extracted without showing work)",
            extraction_count
        ));
    }

    // Check 3: T3 items have confidence
    let has_t3 = content.contains("tier_3_domain_specific:")
        || content.contains("T3")
        || content.contains("Domain-Specific");

    let has_confidence = content.contains("confidence:")
        || content.contains("structural:")
        || content.contains("functional:");

    // Only check confidence if T3 items with transfer mappings exist
    let has_transfer = content.contains("transfer_mappings:") || content.contains("equivalent:");

    if has_t3 && has_transfer && !has_confidence {
        issues.push("T3 items with transfer mappings missing confidence scores".to_string());
    }

    issues
}
