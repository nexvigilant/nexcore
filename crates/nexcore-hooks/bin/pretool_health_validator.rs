//! Health Check Validator Hook (HOOK-M5-52)
//!
//! PreToolUse hook that validates health endpoints in router/handler files.
//! Non-blocking advisory only - warns about missing health endpoints.
//!
//! Exit codes:
//! - 0: Allow (no issues found or non-applicable)
//! - 1: Warn (health check issues found - advisory)
//! - Never blocks

use nexcore_hooks::deployment::health::{analyze_health_and_shutdown, format_health_analysis};
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

    // Analyze health and shutdown patterns
    let analysis = analyze_health_and_shutdown(file_path, content);

    if !analysis.has_issues() {
        exit_success_auto();
    }

    // Advisory only - warn but never block
    let report = format_health_analysis(file_path, &analysis);
    HookOutput::warn(&report).emit();
    std::process::exit(0);
}
