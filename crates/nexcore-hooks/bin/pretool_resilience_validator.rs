//! Resilience Validator Hook (HOOK-M5-51)
//!
//! PreToolUse hook that detects external calls without proper timeout handling.
//! Warns on HTTP/gRPC without timeout, blocks on database connections without pool timeout.
//!
//! ## Trigger
//!
//! - **Event:** PreToolUse (Write only — Edit lacks full content)
//! - **Scope:** Rust files (`*.rs`)
//! - **Tier:** deploy (M5 molecule)
//!
//! ## Patterns Detected
//!
//! - `reqwest::Client` / `hyper` calls without `.timeout()`
//! - gRPC channel creation without deadline configuration
//! - `sqlx::Pool` / `diesel` connections without `connect_timeout` or pool idle limits
//!
//! ## Exit Codes
//!
//! - `0` — Allow: no resilience issues found
//! - `1` — Warn: medium/low severity (HTTP/gRPC without timeout)
//! - `2` — Block: high severity (database connection without pool timeout)

use nexcore_hooks::deployment::resilience::{
    analyze_resilience_patterns, format_resilience_issues,
};
use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only scan on Write tool (Edit doesn't have full content)
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" {
        exit_success_auto();
    }

    // Get file path
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only scan Rust files
    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    // Get content from tool input
    let content = match input.get_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    // Analyze resilience patterns
    let issues = analyze_resilience_patterns(file_path, content);

    if issues.is_empty() {
        exit_success_auto();
    }

    // Check severity levels
    let has_high_severity = issues.iter().any(|i| i.severity.should_block());

    if has_high_severity {
        let report = format_resilience_issues(file_path, &issues);
        exit_block(&report);
    } else {
        // Medium/low findings - warn but allow
        let count = issues.len();
        let msg = format!(
            "⚠️ Found {} resilience issue(s) in {}\n\
             Consider adding timeout configuration to external calls.",
            count, file_path
        );
        exit_warn(&msg);
    }
}
