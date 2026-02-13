//! Immunity Learner - PostToolUse Hook
//!
//! Learns new antibodies from build/test failures.
//! When Bash commands fail, extracts error patterns and proposes antibodies.
//!
//! T1 Grounding: π (persistence) + μ (mapping) + → (causality)
//!
//! Action: Log failure patterns for antibody proposal
//! Exit: Always passes (learning is non-blocking)

use nexcore_hook_lib::cytokine::emit_hook_completed;

use chrono::Utc;
use regex::Regex;

const HOOK_NAME: &str = "immunity-learner";
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Known error signatures we want to learn from
const LEARNABLE_PATTERNS: &[(&str, &str)] = &[
    (r"error\[E\d+\]:", "rust-compiler"),
    (r"cannot find", "missing-dependency"),
    (r"unresolved import", "import-error"),
    (r"lifetime.*does not live long enough", "lifetime-error"),
    (r"borrowed.*cannot be.*while.*is borrowed", "borrow-error"),
    (r"mismatched types", "type-error"),
    (r"no method named", "method-not-found"),
    (r"trait.*is not implemented", "missing-trait-impl"),
    (r"thread.*panicked", "runtime-panic"),
    (r"assertion.*failed", "assertion-failure"),
];

/// PostToolUse hook input structure
#[derive(Deserialize)]
struct PostToolInput {
    tool_name: Option<String>,
    tool_result: Option<ToolResult>,
}

#[derive(Deserialize)]
struct ToolResult {
    #[allow(dead_code)]
    stdout: Option<String>,
    stderr: Option<String>,
    #[serde(default)]
    exit_code: Option<i32>,
}

fn pass() -> ! {
    println!("{{}}");
    std::process::exit(0);
}

fn main() {
    // Read JSON from stdin
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }

    let input: PostToolInput = match serde_json::from_str(&buffer) {
        Ok(i) => i,
        Err(_) => pass(),
    };

    // Only process Bash tool
    if input.tool_name.as_deref() != Some("Bash") {
        pass();
    }

    // Get tool result
    let result = match input.tool_result {
        Some(r) => r,
        None => pass(),
    };

    // Check if command failed
    let exit_code = result.exit_code.unwrap_or(0);
    if exit_code == 0 {
        pass(); // Success, nothing to learn
    }

    // Get stderr content
    let stderr = match &result.stderr {
        Some(s) if !s.is_empty() => s,
        _ => pass(),
    };

    // Extract learnable patterns
    let mut proposals = Vec::new();
    for (pattern, category) in LEARNABLE_PATTERNS {
        let re = match Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if let Some(m) = re.find(stderr) {
            // Extract context around match
            let start = m.start().saturating_sub(50);
            let end = (m.end() + 100).min(stderr.len());
            let context = &stderr[start..end];

            proposals.push(format!(
                "# Proposed: {}\n- pattern: \"{}\"\n  category: {}\n  context: \"{}\"\n  status: pending\n",
                Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
                pattern,
                category,
                context.replace('\n', "\\n").replace('"', "\\\"")
            ));
        }
    }

    // Write proposals if any
    if !proposals.is_empty() {
        let proposals_path =
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
                .join(".claude/immunity/learning_queue.yaml");

        // Ensure directory exists
        if let Some(parent) = proposals_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Append proposals
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&proposals_path)
        {
            for proposal in &proposals {
                let _ = file.write_all(proposal.as_bytes());
                let _ = file.write_all(b"\n");
            }
        }

        // Emit cytokine signal (TGF-beta = regulation, patterns learned)
        emit_hook_completed(
            HOOK_NAME,
            0,
            &format!("learned_{}_patterns", proposals.len()),
        );

        // Log to stderr for visibility (non-blocking)
        eprintln!(
            "🧬 Immunity: {} pattern(s) queued for learning",
            proposals.len()
        );
    }

    // Always pass - learning is non-blocking
    pass();
}
