//! Stop Completion Checker
//!
//! Evaluates whether Claude should stop by reading the transcript
//! and checking for incomplete tasks, errors, or unresolved issues.
//!
//! Conservative approach: Only block on clear errors, not conversational patterns.
//!
//! Exit codes:
//! - 0: Allow stop (default - tasks assumed complete unless clear error)

use nexcore_hooks::{HookOutput, read_input};
use std::fs;
use std::process;

/// Critical patterns that MUST block stop (actual errors)
const CRITICAL_PATTERNS: &[&str] = &[
    "error[E",           // Rust compilation error
    "panicked at",       // Runtime panic
    "unresolved import", // Import error
];

/// Patterns with context exclusions (pattern, exclusion)
/// If exclusion appears in same context, it's a false positive
const CONTEXTUAL_PATTERNS: &[(&str, &str)] = &[
    ("FAILED", "0 failed"),                 // Test failure (exclude "0 failed")
    ("cannot find value", "did not match"), // Rust compiler: cannot find value X in this scope
    ("cannot find type", "did not match"),  // Rust compiler: cannot find type X in this scope
    ("cannot find macro", "did not match"), // Rust compiler: cannot find macro X in this scope
];

/// Patterns that suggest work is complete (overrides warnings)
const COMPLETE_PATTERNS: &[&str] = &[
    "Done.",
    "Complete.",
    "Finished.",
    "All tests pass",
    "Successfully",
    "PASS",
    "Compiled successfully",
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => {
            // No input - allow stop
            println!(r#"{{"decision":"approve"}}"#);
            process::exit(0);
        }
    };

    // Check if stop hook is already active (prevent infinite loop)
    if input.is_stop_hook_active() {
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Try to read transcript
    let transcript_path = match &input.transcript_path {
        Some(p) => p,
        None => {
            // No transcript - allow stop
            println!(r#"{{"decision":"approve"}}"#);
            process::exit(0);
        }
    };

    let transcript = match fs::read_to_string(transcript_path) {
        Ok(content) => content,
        Err(_) => {
            // Can't read transcript - allow stop
            println!(r#"{{"decision":"approve"}}"#);
            process::exit(0);
        }
    };

    // Get last ~50 lines for recent context
    let lines: Vec<&str> = transcript.lines().collect();
    let recent_lines = if lines.len() > 50 {
        &lines[lines.len() - 50..]
    } else {
        &lines[..]
    };
    let recent_content = recent_lines.join("\n");

    // Check for completion patterns first (if present, always allow stop)
    let has_completion = COMPLETE_PATTERNS.iter().any(|p| recent_content.contains(p));

    if has_completion {
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Check for critical error patterns only
    let mut critical_issues: Vec<&str> = Vec::new();

    // Simple patterns (no context exclusion needed)
    for pattern in CRITICAL_PATTERNS {
        if recent_content.contains(pattern) {
            critical_issues.push(pattern);
        }
    }

    // Contextual patterns (check exclusion)
    for (pattern, exclusion) in CONTEXTUAL_PATTERNS {
        if recent_content.contains(pattern) && !recent_content.contains(exclusion) {
            critical_issues.push(pattern);
        }
    }

    // Only block on critical errors
    if critical_issues.is_empty() {
        // No critical errors - allow stop
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Found critical issues - block stop
    let reason = format!(
        "Critical issue detected: {}. Fix before stopping.",
        critical_issues.join(", ")
    );

    let output = HookOutput {
        decision: Some(nexcore_hooks::HookDecision::Block),
        reason: Some(reason.clone()),
        ..Default::default()
    };

    output.emit();
    eprintln!("{reason}");
    process::exit(0);
}
