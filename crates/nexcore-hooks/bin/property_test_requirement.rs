//! # Property-Based Testing Requirement Hook
//!
//! **Event:** `PostToolUse:Write` (Rust files only)
//! **Exit Code:** Always 0 (advisory, never blocks)
//! **Tier:** All tiers (dev/review/deploy)
//!
//! ## Purpose
//!
//! Detects functions that are strong candidates for property-based testing
//! and suggests adding `proptest` coverage. Property tests verify invariants
//! hold across randomly generated inputs, catching edge cases unit tests miss.
//!
//! ## Detection Patterns
//!
//! | Pattern | Type | Suggested Property |
//! |---------|------|-------------------|
//! | `fn parse*` | Parser | Never panics on any input |
//! | `fn deserialize*` | Deserializer | Roundtrip preserves data |
//! | `fn serialize*` | Serializer | Roundtrip preserves data |
//! | `fn from_str*` | Parser | Never panics on any input |
//! | `fn try_from*` | Converter | Valid input always succeeds |
//! | `fn decode*` | Decoder | Roundtrip preserves data |
//! | `fn encode*` | Encoder | Roundtrip preserves data |
//! | `fn validate*` | Validator | Consistent accept/reject |
//!
//! ## Example Output
//!
//! ```text
//! PROPERTY TEST SUGGESTED
//!
//! Functions that would benefit from property-based tests:
//!   Line 42: Parser - Never panics on any input
//!   Line 87: Serializer - Roundtrip preserves data
//!
//! Add proptest to dev-dependencies and use:
//!   proptest! { #[test] fn name(input: Type) { ... } }
//! ```
//!
//! ## Why Property Tests Matter
//!
//! From the CTVP (Clinical Trial Validation Paradigm) perspective:
//! - **Phase 0 (Preclinical):** Unit tests validate happy paths
//! - **Phase 1 (Safety):** Property tests validate "never panics" invariants
//! - **Phase 2 (Efficacy):** Property tests verify roundtrip/idempotence
//!
//! Property tests bridge the gap between unit tests and real-world chaos.
//!
//! ## Integration
//!
//! ```toml
//! [dev-dependencies]
//! proptest = "1.0"
//! ```
//!
//! ```rust
//! use proptest::prelude::*;
//!
//! proptest! {
//!     #[test]
//!     fn parse_never_panics(input: String) {
//!         // Should not panic on any input - result is intentionally unused
//!         drop(parse(&input));
//!     }
//!
//!     #[test]
//!     fn roundtrip_preserves_data(data: MyStruct) {
//!         let encoded = serialize(&data);
//!         let decoded = deserialize(&encoded).expect("roundtrip decode");
//!         prop_assert_eq!(data, decoded);
//!     }
//! }
//! ```

use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let suggestions = check_property_test_candidates(content);
    if suggestions.is_empty() {
        exit_success_auto();
    }

    // This is advisory, not blocking
    let mut msg = String::from("PROPERTY TEST SUGGESTED\n\n");
    msg.push_str("Functions that would benefit from property-based tests:\n");
    for (line, fn_type, property) in &suggestions {
        msg.push_str(&format!("  Line {line}: {fn_type} - {property}\n"));
    }
    msg.push_str("\nAdd proptest to dev-dependencies and use:\n");
    msg.push_str("  proptest! { #[test] fn name(input: Type) { ... } }\n");
    // Advisory output
    HookOutput::warn(&msg).emit();
    std::process::exit(0);
}

/// Build patterns at runtime to avoid potential self-detection
fn get_patterns() -> Vec<(String, &'static str, &'static str)> {
    vec![
        (
            format!("fn par{}", "se"),
            "Parser",
            "Never panics on any input",
        ),
        (
            format!("fn deseri{}", "alize"),
            "Deserializer",
            "Roundtrip preserves data",
        ),
        (
            format!("fn seri{}", "alize"),
            "Serializer",
            "Roundtrip preserves data",
        ),
        (
            format!("fn from_{}", "str"),
            "Parser",
            "Never panics on any input",
        ),
        (
            format!("fn try_{}", "from"),
            "Converter",
            "Valid input always succeeds",
        ),
        (
            format!("fn dec{}", "ode"),
            "Decoder",
            "Roundtrip preserves data",
        ),
        (
            format!("fn enc{}", "ode"),
            "Encoder",
            "Roundtrip preserves data",
        ),
        (
            format!("fn vali{}", "date"),
            "Validator",
            "Consistent accept/reject",
        ),
    ]
}

fn check_property_test_candidates(content: &str) -> Vec<(usize, &'static str, &'static str)> {
    let patterns = get_patterns();
    let mut suggestions = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        // Only check function definitions
        if !trimmed.contains("fn ") {
            continue;
        }

        for (pattern, fn_type, property) in &patterns {
            if trimmed.contains(pattern) {
                suggestions.push((line_num + 1, *fn_type, *property));
                break; // One suggestion per line
            }
        }
    }

    suggestions
}
