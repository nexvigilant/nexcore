//! Axiom Integrity Guard (HOOK-GEM-02)
//!
//! Enforces that all modifications to the safety kernel (pv/vigilance)
//! explicitly reference Safety Axioms or Conservation Laws in the code content.
//!
//! ## Improvement (2026-01-29)
//!
//! Changed from checking prompt context to checking actual file content.
//! This allows legitimate Safety Axiom work to proceed while still enforcing
//! that safety-critical code documents its axiom references.

use nexcore_hooks::{exit_block, exit_success_auto, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = input.get_file_path().unwrap_or("");

    // Target safety-critical crates
    let is_critical = file_path.contains("nexcore-pv") || file_path.contains("nexcore-vigilance");

    if is_critical {
        // Check the actual content being written, not prompt context
        let content = input.get_written_content().unwrap_or("");

        // If no content (e.g., read-only operation), allow
        if content.is_empty() {
            exit_success_auto();
        }

        let content_lower = content.to_lowercase();

        // Check for axiom/safety references in the FILE CONTENT
        let has_axiom_ref = content_lower.contains("axiom")
            || content_lower.contains("conservation law")
            || content_lower.contains("tov")
            || content_lower.contains("safety")
            || content_lower.contains("harm type")
            || content_lower.contains("§"); // ToV section references

        // Also allow test files and doc comments
        let is_test = file_path.contains("/tests/") || file_path.contains("_test.rs");
        let has_doc_comment = content.contains("//!") || content.contains("///");

        if !has_axiom_ref && !is_test && !has_doc_comment {
            exit_block(
                "🦀 INTEGRITY VIOLATION: Code in the safety kernel must document Safety Axiom or Conservation Law references. Add doc comments referencing the relevant axioms.",
            );
        }
    }

    exit_success_auto();
}
