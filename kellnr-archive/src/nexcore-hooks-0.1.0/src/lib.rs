//! # NexVigilant Core — Hooks
//!
//! Claude Code cognitive hooks for code quality and patient safety enforcement.
//!
//! This crate provides the hook infrastructure for Claude Code, implementing
//! cognitive integrity checks that run on every tool use.

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

// --- External Imports ---
use std::io::{self, Read};
use std::path::Path;
use std::process;

// --- Internal Modules ---
pub mod agent_triggers;
pub mod bonding;
pub mod chemistry_scoring;
pub mod constants;
pub mod ctvp;
pub mod deployment;
pub mod error;
pub mod experiment;
pub mod fluent_dsl;
pub mod homeostasis;
pub mod mcp;
pub mod mcp_config;
pub mod mcp_efficacy;
pub mod metrics;
pub mod output;
pub mod parser;
pub mod paths;
pub mod patterns;
pub mod primitives;
pub mod protocol;
pub mod results;
pub mod state;
pub mod telemetry;
pub mod tracking;
pub mod transcript;
pub mod types;

// --- Re-exports ---
pub use output::format_checkpoint;
pub use protocol::{
    AggregatedResult, Decision, FileContext, Finding, GroupResult, HookDecision, HookGroup,
    HookInput, HookOutput, HookResult, HookSpecificOutput, Language, Location, Severity,
};
pub use results::{HookResultRegistry, with_result_tracking};
pub use state::SessionState;
pub use telemetry::{HookTelemetry, SubagentTelemetry, emit_telemetry, telemetry_log_path};
pub use tracking::{ArtifactType, TrackedArtifact, TrackingRegistry, with_registry};

// --- Protocol Functions ---

/// Read hook input from stdin
///
/// Returns None if stdin is empty or invalid JSON
pub fn read_input() -> Option<HookInput> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).ok()?;

    if buffer.trim().is_empty() {
        return None;
    }

    serde_json::from_str(&buffer).ok()
}

/// Exit successfully (code 0) - allow tool execution
///
/// Emits JSON output: `{"decision":"approve"}`
pub fn exit_success() -> ! {
    // Emit minimal approve JSON for Claude Code compatibility
    println!(r#"{{"decision":"approve"}}"#);
    process::exit(0)
}

/// Get the current hook name from the binary path
///
/// Extracts the binary name (without path) from `argv[0]`.
/// Returns "unknown_hook" if detection fails.
pub fn current_hook_name() -> String {
    std::env::args()
        .next()
        .and_then(|path| {
            Path::new(&path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "unknown_hook".to_string())
}

// --- Internal Support ---

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

/// Check if hooks should run in silent/minimal mode
#[inline]
fn is_minimal_mode() -> bool {
    std::env::var("HOOKS_MINIMAL").is_ok()
}

// --- Exit Helpers ---

/// Exit successfully (code 0) - SILENT, absolute minimum output
///
/// Output: `{}` (just empty JSON)
#[inline]
pub fn exit_ok() -> ! {
    println!("{{}}");
    process::exit(0)
}

/// Exit successfully with green dot (code 0)
///
/// Output: `● hook_name` (green) - unless HOOKS_MINIMAL is set
pub fn exit_success_auto() -> ! {
    if is_minimal_mode() {
        println!("{{}}");
    } else {
        let hook_name = current_hook_name();
        eprintln!("{GREEN}●{RESET} {hook_name}");
        println!(r#"{{"decision":"approve"}}"#);
    }
    process::exit(0)
}

/// Exit successfully with green dot and detail (code 0)
///
/// Output: `● hook_name: detail` (green) - unless HOOKS_MINIMAL is set
pub fn exit_success_auto_with(detail: &str) -> ! {
    if is_minimal_mode() {
        println!("{{}}");
    } else {
        let hook_name = current_hook_name();
        eprintln!("{GREEN}●{RESET} {hook_name}: {detail}");
        println!(r#"{{"decision":"approve"}}"#);
    }
    process::exit(0)
}

/// Exit with warning (code 1) - allow but show feedback
///
/// Output: `● hook_name: message` (yellow) - unless HOOKS_MINIMAL is set
pub fn exit_warn(msg: &str) -> ! {
    if is_minimal_mode() {
        eprintln!("{msg}");
    } else {
        let hook_name = current_hook_name();
        eprintln!("{YELLOW}●{RESET} {hook_name}: {msg}");
    }
    process::exit(1)
}

/// Exit with warning, raw message only (no hook name prefix)
pub fn exit_warn_raw(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(1)
}

/// Exit with block (code 2) - prevent tool execution
///
/// Output: `● hook_name: message` (red) - unless HOOKS_MINIMAL is set
pub fn exit_block(msg: &str) -> ! {
    if is_minimal_mode() {
        eprintln!("{msg}");
    } else {
        let hook_name = current_hook_name();
        eprintln!("{RED}●{RESET} {hook_name}: {msg}");
    }
    process::exit(2)
}

/// Exit with block, raw message only (no hook name prefix)
pub fn exit_block_raw(msg: &str) -> ! {
    eprintln!("{msg}");
    process::exit(2)
}

/// Exit with JSON output for UserPromptSubmit hooks
pub fn exit_with_context(context: &str) -> ! {
    HookOutput::with_context(context).emit();
    process::exit(0)
}

/// Exit with JSON output for SessionStart hooks
pub fn exit_with_session_context(context: &str) -> ! {
    HookOutput::with_session_context(context).emit();
    process::exit(0)
}

/// Exit with empty context (skip for UserPromptSubmit)
pub fn exit_skip_prompt() -> ! {
    HookOutput::skip_prompt().emit();
    process::exit(0)
}

/// Exit with empty context for SessionStart hooks
pub fn exit_skip_session() -> ! {
    HookOutput::skip_session().emit();
    process::exit(0)
}

// --- Path Helpers ---

/// Check if path is a Rust file
pub fn is_rust_file(path: &str) -> bool {
    path.ends_with(".rs")
}

/// Check if path is a test file
pub fn is_test_file(path: &str) -> bool {
    path.contains("/tests/")
        || path.contains("/test/")
        || path.contains("_test.rs")
        || path.contains("/benches/")
        || path.starts_with("tests/")
        || path.starts_with("benches/")
}

/// Get file_path from tool_input
pub fn get_file_path(input: &serde_json::Value) -> Option<String> {
    input
        .get("file_path")
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Get content from tool_input (supports both Write and Edit tools)
pub fn get_content(input: &serde_json::Value) -> Option<String> {
    // Write tool uses "content", Edit tool uses "new_string"
    input
        .get("content")
        .or_else(|| input.get("new_string"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Library version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rust_file() {
        assert!(is_rust_file("src/main.rs"));
        assert!(is_rust_file("/home/user/project/lib.rs"));
        assert!(!is_rust_file("main.py"));
        assert!(!is_rust_file("Cargo.toml"));
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file("tests/integration.rs"));
        assert!(is_test_file("src/lib_test.rs"));
        assert!(is_test_file("/home/user/project/tests/unit.rs"));
        assert!(is_test_file("benches/benchmark.rs"));
        assert!(!is_test_file("src/main.rs"));
        assert!(!is_test_file("src/lib.rs"));
    }

    #[test]
    fn test_get_file_path() {
        let input = serde_json::json!({
            "file_path": "/home/user/test.rs",
            "content": "fn main() {}"
        });
        assert_eq!(
            get_file_path(&input),
            Some("/home/user/test.rs".to_string())
        );

        let empty = serde_json::json!({});
        assert_eq!(get_file_path(&empty), None);
    }

    #[test]
    fn test_get_content_write() {
        let input = serde_json::json!({
            "file_path": "test.rs",
            "content": "fn main() {}"
        });
        assert_eq!(get_content(&input), Some("fn main() {}".to_string()));
    }

    #[test]
    fn test_get_content_edit() {
        let input = serde_json::json!({
            "file_path": "test.rs",
            "new_string": "fn updated() {}"
        });
        assert_eq!(get_content(&input), Some("fn updated() {}".to_string()));
    }
}
