//! Observability Checker Hook (HOOK-M5-53)
//!
//! PostToolUse hook that checks observability coverage after writes.
//! Suggests tracing instrumentation and logging improvements.
//! Non-blocking advisory only.
//!
//! Exit codes:
//! - 0: Allow (no issues found or non-applicable)
//! - 1: Warn (observability issues found - advisory)
//! - Never blocks

use nexcore_hooks::deployment::observability::{
    analyze_observability, format_observability_findings,
};
use nexcore_hooks::{HookOutput, exit_success_auto, is_rust_file, read_input};

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

    // Analyze observability patterns
    let findings = analyze_observability(file_path, content);

    if findings.is_empty() {
        exit_success_auto();
    }

    // Advisory only - warn but never block
    let report = format_observability_findings(file_path, &findings);
    HookOutput::warn(&report).emit();
    std::process::exit(0);
}
