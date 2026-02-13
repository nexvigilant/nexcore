//! CEP Pipeline Gate (Accelerator 1)
//!
//! Enforces CEP methodology awareness in PV/vigilance code.
//! Warns when PV code doesn't reference primitive tiers.
//!
//! # Purpose
//!
//! Ensures all pharmacovigilance code is grounded in the primitive
//! extraction methodology defined in Patent NV-2026-002.
//!
//! # Exit Codes
//!
//! - `0`: Code references primitives or is not PV-related
//! - `1`: Warning - PV code should reference tier methodology

use nexcore_hooks::{exit_block, exit_success_auto, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = input.get_file_path().unwrap_or("");
    let content = input.get_written_content().unwrap_or("");

    // Only check PV/vigilance files
    let is_pv_file = file_path.contains("vigilance")
        || file_path.contains("pv")
        || file_path.contains("signal")
        || file_path.contains("pharmacovigilance");

    if !is_pv_file || !is_rust_file(file_path) {
        exit_success_auto();
    }

    // Check for primitive tier references
    let has_tier_ref = content.contains("T1")
        || content.contains("T2")
        || content.contains("T3")
        || content.contains("primitive")
        || content.contains("Tier")
        || content.contains("tier_");

    // Check for CEP references
    let has_cep_ref = content.contains("CEP")
        || content.contains("cep")
        || content.contains("extraction")
        || content.contains("decompose");

    // Allow test files and doc comments
    let is_test = file_path.contains("/tests/") || file_path.contains("_test.rs");
    let is_doc = content.contains("//!") || content.contains("///");

    if !has_tier_ref && !has_cep_ref && !is_test && !is_doc {
        exit_block(
            "🛑 PV code MUST reference primitive tiers (T1/T2/T3) or CEP methodology (NV-2026-002)",
        );
    }

    exit_success_auto();
}
