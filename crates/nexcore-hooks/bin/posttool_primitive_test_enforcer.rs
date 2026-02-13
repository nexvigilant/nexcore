//! Primitive Test Enforcer
//!
//! PostToolUse hook that warns when primitive extraction output lacks test cards.
//!
//! The primitive-extractor skill requires showing work via "PRIMITIVE TEST:" cards
//! for every term analyzed. This hook enforces that requirement.
//!
//! # Event
//! PostToolUse (Write, Edit)
//!
//! # Purpose
//! Ensures primitive extraction includes validation proof (test cards)
//!
//! # Exit Codes
//! - 0: OK (pass)
//! - 1: Warning (missing test cards)
//!
//! Triggers on: Write|Edit of files containing primitive extraction YAML

use nexcore_hooks::{exit_block, exit_ok, read_input};

/// Patterns that indicate primitive extraction output
const EXTRACTION_MARKERS: &[&str] = &[
    "tier_1_universal:",
    "tier_2_primitives:",
    "tier_2_composites:",
    "tier_3_domain_specific:",
    "primitives:",
];

/// Required test card marker
const TEST_CARD_MARKER: &str = "PRIMITIVE TEST:";

/// Alternative test card markers (box drawing)
const ALT_TEST_MARKERS: &[&str] = &[
    "┌────────────────────────────────────────────────┐",
    "PRIMITIVE TEST:",
    "│ PRIMITIVE TEST:",
];

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

    // Get the content being written
    let content = extract_content(&input);
    if content.is_empty() {
        exit_ok();
    }

    // Check if this is primitive extraction output
    let is_extraction = EXTRACTION_MARKERS
        .iter()
        .any(|marker| content.contains(marker));

    if !is_extraction {
        exit_ok();
    }

    // Count extraction items and test cards
    let extraction_count = count_extractions(&content);
    let test_card_count = count_test_cards(&content);

    // If there are extractions but no test cards, BLOCK
    if extraction_count > 0 && test_card_count == 0 {
        exit_block(&format!(
            "🛑 Primitive extraction detected ({} items) but NO test cards shown. \
             The primitive-extractor skill REQUIRES a PRIMITIVE TEST card for each term. \
             Missing test output = INVALID extraction.",
            extraction_count
        ));
    }

    // If test cards are fewer than extractions, BLOCK
    if test_card_count > 0 && test_card_count < extraction_count {
        exit_block(&format!(
            "🛑 Primitive extraction has {} items but only {} test cards. \
             Each analyzed term REQUIRES a PRIMITIVE TEST card.",
            extraction_count, test_card_count
        ));
    }

    exit_ok();
}

/// Extract content from tool input
fn extract_content(input: &nexcore_hooks::HookInput) -> String {
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => return String::new(),
    };

    // For Write tool: content field
    if let Some(content) = tool_input.get("content").and_then(|v| v.as_str()) {
        return content.to_string();
    }

    // For Edit tool: new_string field
    if let Some(new_string) = tool_input.get("new_string").and_then(|v| v.as_str()) {
        return new_string.to_string();
    }

    String::new()
}

/// Count primitive/composite items in extraction
fn count_extractions(content: &str) -> usize {
    let mut count = 0;

    // Count YAML list items under extraction sections
    let patterns = [
        "- primitive:",
        "- composite:",
        "- item:",
        "primitive: \"",
        "composite: \"",
    ];

    for pattern in patterns {
        count += content.matches(pattern).count();
    }

    count
}

/// Count test cards in content
fn count_test_cards(content: &str) -> usize {
    let mut count = 0;

    // Count primary marker
    count += content.matches(TEST_CARD_MARKER).count();

    // Count box-style test cards (only if primary not found)
    if count == 0 {
        for marker in ALT_TEST_MARKERS {
            count += content.matches(marker).count();
        }
        // Divide by markers per card (roughly)
        count /= 2;
    }

    count
}
