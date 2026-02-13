//! Phase 1 Fault Injection Tests for Hooks
//!
//! CTVP Phase 1: Safety - Failure Mode Validation
//!
//! Tests that hooks fail gracefully under:
//! - Malformed JSON input
//! - Missing directories
//! - Empty input
//! - Invalid data types
//!
//! All hooks should exit 0 (skip) rather than crash on bad input.

use std::io::Write;
use std::process::{Command, Stdio};

/// Run a hook binary with given stdin input
fn run_hook_with_input(hook_name: &str, input: &str) -> (i32, String, String) {
    let hook_path = format!(
        "{}/nexcore/target/release/{}",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string()),
        hook_name
    );

    let mut child = Command::new(&hook_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to spawn {}", hook_name));

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).ok();
    }

    let output = child.wait_with_output().expect("Failed to wait on child");
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (exit_code, stdout, stderr)
}

// ============================================================================
// handoff_completion_reviewer tests
// ============================================================================

#[test]
fn handoff_reviewer_handles_empty_input() {
    let (exit_code, _, _) = run_hook_with_input("handoff_completion_reviewer", "");
    // Should gracefully skip (exit 0) on empty input
    assert_eq!(exit_code, 0, "Hook should exit 0 on empty input");
}

#[test]
fn handoff_reviewer_handles_malformed_json() {
    let (exit_code, _, _) =
        run_hook_with_input("handoff_completion_reviewer", "{{not valid json at all");
    assert_eq!(exit_code, 0, "Hook should exit 0 on malformed JSON");
}

#[test]
fn handoff_reviewer_handles_wrong_json_type() {
    let (exit_code, _, _) = run_hook_with_input(
        "handoff_completion_reviewer",
        r#"["array", "instead", "of", "object"]"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 on wrong JSON type");
}

#[test]
fn handoff_reviewer_handles_missing_fields() {
    let (exit_code, _, _) = run_hook_with_input(
        "handoff_completion_reviewer",
        r#"{"some_random_field": "value"}"#,
    );
    assert_eq!(
        exit_code, 0,
        "Hook should exit 0 on missing required fields"
    );
}

#[test]
fn handoff_reviewer_handles_null_values() {
    let (exit_code, _, _) = run_hook_with_input(
        "handoff_completion_reviewer",
        r#"{"session_id": null, "cwd": null, "hook_event_name": null}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 on null values");
}

// ============================================================================
// mcp_tool_suggester tests
// ============================================================================

#[test]
fn mcp_suggester_handles_empty_input() {
    let (exit_code, _, _) = run_hook_with_input("mcp_tool_suggester", "");
    assert_eq!(exit_code, 0, "Hook should exit 0 on empty input");
}

#[test]
fn mcp_suggester_handles_malformed_json() {
    let (exit_code, _, _) = run_hook_with_input("mcp_tool_suggester", "not json {{{");
    assert_eq!(exit_code, 0, "Hook should exit 0 on malformed JSON");
}

#[test]
fn mcp_suggester_handles_missing_prompt() {
    let (exit_code, _, _) = run_hook_with_input(
        "mcp_tool_suggester",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "UserPromptSubmit"}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 when prompt is missing");
}

#[test]
fn mcp_suggester_handles_empty_prompt() {
    let (exit_code, _, _) = run_hook_with_input(
        "mcp_tool_suggester",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "UserPromptSubmit", "prompt": ""}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 on empty prompt");
}

#[test]
fn mcp_suggester_handles_no_keyword_match() {
    let (exit_code, stdout, _) = run_hook_with_input(
        "mcp_tool_suggester",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "UserPromptSubmit", "prompt": "hello world"}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 when no keywords match");
    // Should produce skip output, not suggestion output
    assert!(
        !stdout.contains("MCP TOOL SUGGESTIONS"),
        "Should not suggest when no keywords match"
    );
}

#[test]
fn mcp_suggester_detects_sha256_keyword() {
    let (exit_code, stdout, _) = run_hook_with_input(
        "mcp_tool_suggester",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "UserPromptSubmit", "prompt": "calculate sha256 hash of file"}"#,
    );
    assert_eq!(exit_code, 0);
    assert!(
        stdout.contains("foundation_sha256"),
        "Should suggest sha256 tool"
    );
}

// ============================================================================
// mcp_bash_interceptor tests
// ============================================================================

#[test]
fn bash_interceptor_handles_empty_input() {
    let (exit_code, _, _) = run_hook_with_input("mcp_bash_interceptor", "");
    assert_eq!(exit_code, 0, "Hook should exit 0 on empty input");
}

#[test]
fn bash_interceptor_handles_malformed_json() {
    let (exit_code, _, _) = run_hook_with_input("mcp_bash_interceptor", "}{malformed");
    assert_eq!(exit_code, 0, "Hook should exit 0 on malformed JSON");
}

#[test]
fn bash_interceptor_handles_non_bash_tool() {
    let (exit_code, _, _) = run_hook_with_input(
        "mcp_bash_interceptor",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "PreToolUse", "tool_name": "Read", "tool_input": {}}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 for non-Bash tools");
}

#[test]
fn bash_interceptor_handles_missing_command() {
    let (exit_code, _, _) = run_hook_with_input(
        "mcp_bash_interceptor",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {}}"#,
    );
    assert_eq!(exit_code, 0, "Hook should exit 0 when command is missing");
}

#[test]
fn bash_interceptor_allows_non_mcp_commands() {
    let (exit_code, _, stderr) = run_hook_with_input(
        "mcp_bash_interceptor",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "ls -la"}}"#,
    );
    assert_eq!(exit_code, 0, "Hook should allow ls command");
    assert!(
        !stderr.contains("MCP ALTERNATIVE"),
        "Should not warn for ls"
    );
}

#[test]
fn bash_interceptor_warns_on_sha256sum() {
    let (exit_code, _, stderr) = run_hook_with_input(
        "mcp_bash_interceptor",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "sha256sum /etc/passwd"}}"#,
    );
    // Exit code 1 = warn but allow
    assert_eq!(exit_code, 1, "Hook should warn (exit 1) on sha256sum");
    assert!(
        stderr.contains("MCP ALTERNATIVE"),
        "Should suggest MCP alternative"
    );
}

#[test]
fn bash_interceptor_warns_on_gcloud_secrets() {
    let (exit_code, _, stderr) = run_hook_with_input(
        "mcp_bash_interceptor",
        r#"{"session_id": "test", "cwd": "/tmp", "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": {"command": "gcloud secrets list"}}"#,
    );
    assert_eq!(exit_code, 1, "Hook should warn on gcloud secrets");
    assert!(
        stderr.contains("gcloud_secrets_list"),
        "Should suggest gcloud MCP tool"
    );
}

// ============================================================================
// Edge case tests
// ============================================================================

#[test]
fn hooks_handle_unicode_input() {
    let unicode_prompt = r#"{"session_id": "test", "prompt": "计算 sha256 哈希值 🔐"}"#;

    let (exit_code, _, _) = run_hook_with_input("mcp_tool_suggester", unicode_prompt);
    assert_eq!(exit_code, 0, "Hook should handle unicode gracefully");
}

#[test]
fn hooks_handle_very_long_input() {
    let long_prompt = format!(
        r#"{{"session_id": "test", "prompt": "{}"}}"#,
        "a".repeat(100_000)
    );

    let (exit_code, _, _) = run_hook_with_input("mcp_tool_suggester", &long_prompt);
    assert_eq!(exit_code, 0, "Hook should handle very long input");
}

#[test]
fn hooks_handle_special_characters() {
    let special_prompt = r#"{"session_id": "test", "prompt": "test\n\t\r\"\\special"}"#;

    let (exit_code, _, _) = run_hook_with_input("mcp_tool_suggester", special_prompt);
    assert_eq!(exit_code, 0, "Hook should handle special characters");
}
