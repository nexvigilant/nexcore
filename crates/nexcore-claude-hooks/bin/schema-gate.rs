// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Schema Gate — PreToolUse Hook
//!
//! Detects schema drift when writing JSON files. Uses the existing file
//! as a baseline contract and warns if the new content's schema diverges.
//!
//! Action: Compare old → new JSON schema, warn on drift
//! Exit: 0 = pass, 1 = warn on drift detected
//!
//! # Pipeline
//!
//! ```text
//! existing.json → transcriptase::infer → Ribosome::store_contract
//!     new content → transcriptase::infer → Ribosome::validate → drift?
//! ```
//!
//! # Cytokine Integration
//! - **Drift Detected**: Emits IL-6 (acute response) for schema drift warnings

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{content_or_pass, file_path_or_pass, pass, read_input, warn};
use std::path::Path;

const HOOK_NAME: &str = "schema-gate";

/// Drift threshold: warn if score exceeds this.
/// 0.25 matches the ribosome default.
const DRIFT_THRESHOLD: f64 = 0.25;

/// Minimum file size (bytes) to bother checking.
/// Tiny files (empty, `{}`, `[]`) aren't worth gating.
const MIN_BASELINE_BYTES: usize = 10;

/// File extensions we gate.
const GATED_EXTENSIONS: &[&str] = &[".json"];

/// Paths we skip (telemetry, generated, transient state).
const SKIP_PATTERNS: &[&str] = &[
    "/telemetry/",
    "/todos/",
    "/subagent_tracker/",
    "/projects/",
    "/tasks/",
    ".jsonl",
    "signals.json",
    "cytokine_metrics.json",
    "session_learnings.jsonl",
    "skill_invocations.jsonl",
    "verification_chain.jsonl",
    "events.jsonl",
    "tracking_registry.json",
    "mcp_efficacy.json",
    "prompt_kinetics.json",
    "token_efficiency.json",
    "usage_telemetry.json",
    "compound_growth.json",
    "pattern_tracker.json",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    let file_path = file_path_or_pass(&input);

    // Only gate JSON files
    if !is_gated_file(file_path) {
        pass();
    }

    // Skip transient/telemetry paths
    if should_skip(file_path) {
        pass();
    }

    // Get the new content being written
    let new_content = content_or_pass(&input);

    // Parse new content as JSON
    let new_json: serde_json::Value = match serde_json::from_str(new_content) {
        Ok(v) => v,
        Err(_) => pass(), // Not valid JSON — let it through, other hooks can catch this
    };

    // Read existing file as baseline
    let existing_content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => pass(), // New file, no baseline to compare against
    };

    // Skip if baseline is too small
    if existing_content.len() < MIN_BASELINE_BYTES {
        pass();
    }

    // Parse existing content as JSON
    let existing_json: serde_json::Value = match serde_json::from_str(&existing_content) {
        Ok(v) => v,
        Err(_) => pass(), // Existing file isn't valid JSON
    };

    // Infer schemas
    let baseline_schema = nexcore_transcriptase::infer(&existing_json);
    let new_schema = nexcore_transcriptase::infer(&new_json);

    // Create a temporary ribosome and store the baseline contract
    let mut ribosome = nexcore_ribosome::Ribosome::with_config(nexcore_ribosome::RibosomeConfig {
        drift_threshold: DRIFT_THRESHOLD,
        auto_update: false,
    });

    let contract_id = "pretool-gate";
    if ribosome
        .store_contract(contract_id, baseline_schema)
        .is_err()
    {
        pass(); // Can't store contract — fail open
    }

    // Validate new content against baseline
    let drift_result = match ribosome.validate(contract_id, &new_json) {
        Some(r) => r,
        None => pass(), // Contract not found — fail open
    };

    if !drift_result.drift_detected {
        pass(); // No drift — all clear
    }

    // Drift detected! Build warning message.
    let file_name = Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let mut msg = format!(
        "SCHEMA DRIFT DETECTED in {}\n\n  Drift score: {:.2} (threshold: {:.2})\n",
        file_name, drift_result.drift_score, DRIFT_THRESHOLD,
    );

    // List violations (up to 5)
    for (i, violation) in drift_result.violations.iter().take(5).enumerate() {
        msg.push_str(&format!(
            "  {}. {} — {} (field: {})\n",
            i + 1,
            violation.drift_type,
            violation.severity,
            violation.field,
        ));
    }

    if drift_result.violations.len() > 5 {
        msg.push_str(&format!(
            "  ... and {} more violations\n",
            drift_result.violations.len() - 5,
        ));
    }

    msg.push_str(&format!(
        "\n  Schema of '{}' changed from baseline.\n  New schema: {}\n",
        file_name,
        describe_schema(&new_schema),
    ));

    // Emit cytokine for drift detection
    emit_check_failed(
        HOOK_NAME,
        &format!(
            "schema drift {:.2} in {}",
            drift_result.drift_score, file_name
        ),
    );

    // Warn (don't block — schema changes may be intentional)
    warn(&msg);
}

/// Check if file has a gated extension.
fn is_gated_file(path: &str) -> bool {
    GATED_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

/// Check if path matches any skip pattern.
fn should_skip(path: &str) -> bool {
    SKIP_PATTERNS.iter().any(|pattern| path.contains(pattern))
}

/// Brief schema description for the warning message.
fn describe_schema(schema: &nexcore_transcriptase::Schema) -> String {
    format!("{:?} ({}obs)", schema.kind, schema.observations)
}
