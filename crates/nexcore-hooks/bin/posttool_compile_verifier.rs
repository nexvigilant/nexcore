//! Compilation Verification Hook
//!
//! Event: PostToolUse (Bash - cargo check/build/test/clippy)
//! Treats compilation results as evidence for assumption verification.

use nexcore_hooks::state::SessionState;
use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use regex::Regex;
use std::collections::HashSet;

/// Error codes indicating hallucination (module/path not found)
const HALLUCINATION_ERRORS: &[&str] = &["E0432", "E0433", "E0463"];

/// Error codes for borrow checker issues
const BORROW_ERRORS: &[&str] = &[
    "E0382", "E0499", "E0502", "E0503", "E0505", "E0506", "E0507", "E0515", "E0597", "E0716",
];

/// Error codes for type errors
const TYPE_ERRORS: &[&str] = &["E0277", "E0308", "E0412", "E0433"];

fn is_cargo_command(cmd: &str) -> bool {
    let patterns = ["cargo check", "cargo build", "cargo test", "cargo clippy"];
    patterns.iter().any(|p| cmd.contains(p))
}

fn extract_error_codes(output: &str) -> HashSet<String> {
    let mut codes = HashSet::new();
    if let Ok(re) = Regex::new(r"error\[E(\d{4})\]") {
        for cap in re.captures_iter(output) {
            codes.insert(format!("E{}", &cap[1]));
        }
    }
    codes
}

fn classify_errors(codes: &HashSet<String>) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut hallucination = Vec::new();
    let mut borrow = Vec::new();
    let mut type_err = Vec::new();

    for code in codes {
        if HALLUCINATION_ERRORS.contains(&code.as_str()) {
            hallucination.push(code.clone());
        } else if BORROW_ERRORS.contains(&code.as_str()) {
            borrow.push(code.clone());
        } else if TYPE_ERRORS.contains(&code.as_str()) {
            type_err.push(code.clone());
        }
    }

    (hallucination, borrow, type_err)
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Check if this was a cargo command
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_success_auto(),
    };

    let command = tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !is_cargo_command(command) {
        exit_success_auto();
    }

    // Get the tool response (stdout/stderr)
    let response = match &input.tool_response {
        Some(v) => v,
        None => exit_success_auto(),
    };

    let output = response
        .get("output")
        .or_else(|| response.get("stdout"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let stderr = response
        .get("stderr")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let combined = format!("{}\n{}", output, stderr);

    // Load session state
    let mut state = SessionState::load();

    // Determine success/failure
    let success = response
        .get("exit_code")
        .or_else(|| response.get("exitCode"))
        .and_then(|v| v.as_i64())
        .map(|c| c == 0)
        .unwrap_or_else(|| !combined.contains("error[E"));

    if success {
        // Compilation succeeded - record verification
        state.record_verification("success");
        let _ = state.save();
        exit_success_auto();
    }

    // Compilation failed - classify errors
    let error_codes = extract_error_codes(&combined);
    let (hallucination, borrow, type_err) = classify_errors(&error_codes);

    // Record failure
    state.record_verification("failed");

    // If hallucination errors, disprove related assumptions
    if !hallucination.is_empty() {
        let evidence = format!(
            "Compilation failed with hallucination errors: {:?}",
            hallucination
        );
        // Collect first to avoid borrow conflict
        let to_disprove: Vec<String> = state
            .get_unverified_assumptions()
            .iter()
            .filter(|a| a.assumption.contains("crate") || a.assumption.contains("module"))
            .map(|a| a.assumption.clone())
            .collect();
        for assumption in to_disprove {
            state.disprove_assumption(&assumption, &evidence);
        }
    }

    let _ = state.save();

    // Generate audit prompt
    let mut audit = Vec::new();

    if !hallucination.is_empty() {
        audit.push(format!(
            "HALLUCINATION DETECTED: {} (module/crate not found)",
            hallucination.join(", ")
        ));
        audit.push("→ Check: Did you reference a non-existent crate or module?".to_string());
    }

    if !borrow.is_empty() {
        audit.push(format!("BORROW CHECKER: {}", borrow.join(", ")));
        audit.push("→ Review ownership and lifetime assumptions".to_string());
    }

    if !type_err.is_empty() {
        audit.push(format!("TYPE ERRORS: {}", type_err.join(", ")));
        audit.push("→ Verify type assumptions match actual signatures".to_string());
    }

    if audit.is_empty() {
        // Generic compilation failure
        exit_warn(&format!(
            "Compilation failed with {} error code(s)",
            error_codes.len()
        ));
    }

    exit_warn(&format!(
        "COMPILATION VERIFICATION FAILED\n{}",
        audit.join("\n")
    ));
}
