//! Agent File Detector Hook
//!
//! Event: PostToolUse (matcher: Edit|Write)
//!
//! Detects patterns in Rust files that warrant specialized agent attention:
//! - Unsafe blocks/functions → rust-unsafe-specialist
//! - Async functions/await → rust-async-expert
//! - Macros → rust-macro-engineer
//! - FFI boundaries → rust-ffi-bridge
//! - Test additions → rust-test-architect
//! - Large refactors → rust-reviewer
//!
//! ## Exit Codes
//!
//! - 0: No agent-worthy patterns found, or non-Rust file (skip)
//! - 1: Warning — agent recommendation emitted to stderr
//!
//! ## Integration
//!
//! Runs after Edit/Write completes. Reads written content via stdin JSON.
//! Recommendations are informational (warn, not block).

use nexcore_hooks::agent_triggers::patterns::{detect_patterns, is_large_refactor};
use nexcore_hooks::{exit_success_auto, exit_warn, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only trigger on Rust files
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if !is_rust_file(file_path) {
        exit_success_auto();
    }

    // Get the content that was written
    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    // Check for large refactor first
    if is_large_refactor(content) {
        exit_warn(
            r#"
🤖 AUTO-TRIGGERING RUST AGENT ─────────────────────────────
   Detected: Large refactor (>100 lines)
   Agent: rust-reviewer
   Reason: Significant code changes warrant review

   ⚡ AUTOMATIC ACTION: Invoking rust-reviewer subagent.

   Use Task tool with:
     subagent_type: "rust-reviewer"
     prompt: "Review this large refactor for ownership, error handling, and idiomatic patterns"
───────────────────────────────────────────────────────────"#,
        );
    }

    // Detect specific patterns
    if let Some(detection) = detect_patterns(content) {
        exit_warn(&detection.to_context());
    }

    exit_success_auto();
}
