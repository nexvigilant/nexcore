//! CTVP Phase 0-1 Hook Validation Tests
//!
//! This module provides comprehensive validation of nexcore hooks
//! following the Clinical Trial Validation Paradigm.
//!
//! Phase 0: Mechanism validity (correct output for known inputs)
//! Phase 1: Safety (graceful handling of failures)

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Test result for a single hook
#[derive(Debug)]
struct HookTestResult {
    hook_name: String,
    passed: bool,
    latency_ms: u64,
    decision: Option<String>,
    #[allow(dead_code)]
    error: Option<String>,
}

/// Create Claude Code format input JSON
fn claude_input(tool_name: &str, file_path: &str, content: &str) -> String {
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": tool_name,
        "tool_input": {
            "file_path": file_path,
            "content": content
        },
        "session_id": "test-session",
        "cwd": "/tmp"
    })
    .to_string()
}

/// Run a hook with given input and return result
fn run_hook(hook_path: &str, input: &str, timeout_ms: u64) -> HookTestResult {
    let start = Instant::now();
    let hook_name = std::path::Path::new(hook_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let result = Command::new(hook_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match result {
        Ok(c) => c,
        Err(e) => {
            return HookTestResult {
                hook_name,
                passed: false,
                latency_ms: start.elapsed().as_millis() as u64,
                decision: None,
                error: Some(format!("Failed to spawn: {}", e)),
            };
        }
    };

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }

    // Wait with timeout
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return HookTestResult {
                hook_name,
                passed: false,
                latency_ms: start.elapsed().as_millis() as u64,
                decision: None,
                error: Some(format!("Failed to wait: {}", e)),
            };
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    // Check if exceeded timeout
    if latency_ms > timeout_ms {
        return HookTestResult {
            hook_name,
            passed: false,
            latency_ms,
            decision: None,
            error: Some(format!(
                "Timeout exceeded: {}ms > {}ms",
                latency_ms, timeout_ms
            )),
        };
    }

    // Parse output - check both stdout JSON and exit code
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    // Decision can come from:
    // 1. JSON stdout with "decision" field
    // 2. JSON stdout with "continue" field (toolchain_validator)
    // 3. Exit code: 0=allow, 1=warn, 2=block
    let decision = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        // Check for "decision" field first
        if let Some(d) = json.get("decision").and_then(|d| d.as_str()) {
            Some(d.to_string())
        // Check for "continue" field (toolchain_validator style)
        } else if json
            .get("continue")
            .and_then(|c| c.as_bool())
            .unwrap_or(false)
        {
            Some("allow".to_string())
        } else {
            None
        }
    } else {
        // No JSON output - derive from exit code
        match exit_code {
            0 => Some("allow".to_string()),
            1 => Some("warn".to_string()),
            2 => Some("block".to_string()),
            _ => None,
        }
    };

    HookTestResult {
        hook_name,
        passed: exit_code == 0 || exit_code == 1 || exit_code == 2,
        latency_ms,
        decision,
        error: if stderr.is_empty() {
            None
        } else {
            Some(stderr.to_string())
        },
    }
}

fn get_hook_dir() -> String {
    std::env::var("NEXCORE_RELEASE")
        .unwrap_or_else(|_| format!("{}/nexcore/target/release", std::env::var("HOME").unwrap()))
}

// ============================================================================
// PHASE 0: MECHANISM VALIDITY TESTS
// ============================================================================

#[test]
fn test_python_file_blocker_blocks_py_files() {
    let hook_path = format!("{}/python_file_blocker", get_hook_dir());
    let input = claude_input("Write", "/tmp/test.py", "print('hello')");
    let result = run_hook(&hook_path, &input, 5000);

    assert!(result.passed, "Hook should run successfully");
    assert!(
        result.decision.as_deref() == Some("block"),
        "Should block .py files, got: {:?}",
        result.decision
    );
}

#[test]
fn test_python_file_blocker_allows_rs_files() {
    let hook_path = format!("{}/python_file_blocker", get_hook_dir());
    let input = claude_input("Write", "/tmp/test.rs", "fn main() {}");
    let result = run_hook(&hook_path, &input, 5000);

    assert!(result.passed, "Hook should run successfully");
    assert!(
        result.decision.as_deref() == Some("allow"),
        "Should allow .rs files, got: {:?}",
        result.decision
    );
}

#[test]
fn test_secret_scanner_allows_clean_code() {
    let hook_path = format!("{}/secret_scanner", get_hook_dir());
    let input = claude_input(
        "Write",
        "/tmp/clean.rs",
        r#"fn hello() { println!("Hello, World!"); }"#,
    );
    let result = run_hook(&hook_path, &input, 5000);

    assert!(result.passed, "Hook should run successfully");
    assert!(
        result.decision.as_deref() == Some("allow"),
        "Should allow clean code, got: {:?}",
        result.decision
    );
}

#[test]
fn test_unsafe_gatekeeper_allows_documented_unsafe() {
    let hook_path = format!("{}/unsafe_gatekeeper", get_hook_dir());
    let input = claude_input(
        "Write",
        "/tmp/safe_unsafe.rs",
        r#"fn safe_read(ptr: *const i32) -> i32 {
    // SAFETY: Caller guarantees ptr is valid and aligned
    unsafe { *ptr }
}"#,
    );
    let result = run_hook(&hook_path, &input, 5000);

    assert!(result.passed, "Hook should run successfully");
    assert!(
        result.decision.as_deref() == Some("allow"),
        "Should allow documented unsafe, got: {:?}",
        result.decision
    );
}

// ============================================================================
// PHASE 1: SAFETY TESTS (Fault Injection)
// ============================================================================

#[test]
fn test_hooks_handle_empty_input() {
    // These hooks should fail-open on empty input
    let hooks = ["python_file_blocker", "secret_scanner", "unsafe_gatekeeper"];

    for hook_name in hooks {
        let hook_path = format!("{}/{}", get_hook_dir(), hook_name);
        let result = run_hook(&hook_path, "", 5000);

        assert!(
            result.passed,
            "{} should handle empty input gracefully",
            hook_name
        );
        assert!(
            result.decision.as_deref() == Some("allow"),
            "{} should allow on empty input (fail-open), got: {:?}",
            hook_name,
            result.decision
        );
    }
}

#[test]
fn test_toolchain_validator_handles_empty_input() {
    let hook_path = format!("{}/toolchain_validator", get_hook_dir());
    let result = run_hook(&hook_path, "", 5000);

    // Toolchain validator checks the toolchain regardless of input
    // It should still produce a valid result (allow if toolchain present)
    assert!(
        result.passed,
        "toolchain_validator should handle empty input gracefully"
    );
    // It outputs {"continue": true} which maps to "allow"
    assert!(
        result.decision.is_some(),
        "toolchain_validator should produce a decision, got: {:?}",
        result.decision
    );
}

#[test]
fn test_hooks_handle_malformed_json() {
    let hooks = ["python_file_blocker", "secret_scanner", "unsafe_gatekeeper"];

    let malformed_inputs = [
        "not json at all",
        "{incomplete",
        r#"{"tool": "Write"}"#,
        "null",
        "[]",
    ];

    for hook_name in hooks {
        let hook_path = format!("{}/{}", get_hook_dir(), hook_name);

        for input in &malformed_inputs {
            let result = run_hook(&hook_path, input, 5000);

            assert!(
                result.passed,
                "{} should handle malformed input '{}' gracefully",
                hook_name, input
            );
            assert!(
                result.decision.as_deref() == Some("allow"),
                "{} should allow on malformed input (fail-open), got: {:?}",
                hook_name,
                result.decision
            );
        }
    }
}

#[test]
fn test_hooks_respect_latency_slo() {
    // Hooks with their SLO latency in ms (2x for test variance)
    let hooks_with_slos = [
        ("python_file_blocker", 100),
        ("secret_scanner", 400),
        ("unsafe_gatekeeper", 200),
    ];

    let input = claude_input("Write", "/tmp/test.rs", "fn main() {}");

    for (hook_name, slo_ms) in hooks_with_slos {
        let hook_path = format!("{}/{}", get_hook_dir(), hook_name);
        let result = run_hook(&hook_path, &input, 5000);

        assert!(
            result.latency_ms <= slo_ms,
            "{} exceeded SLO: {}ms > {}ms",
            hook_name,
            result.latency_ms,
            slo_ms
        );
    }
}

#[test]
fn test_hooks_produce_valid_decision() {
    // Test that hooks produce a valid decision (either via JSON or exit code)
    let hooks = ["python_file_blocker", "secret_scanner", "unsafe_gatekeeper"];

    let input = claude_input("Write", "/tmp/test.rs", "fn main() {}");

    for hook_name in hooks {
        let hook_path = format!("{}/{}", get_hook_dir(), hook_name);
        let result = run_hook(&hook_path, &input, 5000);

        assert!(
            result.decision.is_some(),
            "{} should produce a valid decision (via JSON or exit code), got: {:?}",
            hook_name,
            result
        );
    }
}

// ============================================================================
// INTEGRATION TEST: Run all safety hooks on sample code
// ============================================================================

#[test]
fn test_safety_plugin_integration() {
    // Safe Rust code that should pass all safety hooks
    let safe_code = r#"
//! A safe module

/// A safe function
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
"#;

    let safety_hooks = [
        "python_file_blocker",
        "secret_scanner",
        "unsafe_gatekeeper",
        "panic_free_enforcer",
    ];

    let input = claude_input("Write", "/tmp/safe_module.rs", safe_code);

    for hook_name in safety_hooks {
        let hook_path = format!("{}/{}", get_hook_dir(), hook_name);

        if std::path::Path::new(&hook_path).exists() {
            let result = run_hook(&hook_path, &input, 5000);

            assert!(
                matches!(result.decision.as_deref(), Some("approve") | Some("allow")),
                "{} should approve/allow safe Rust code, got: {:?}",
                hook_name,
                result.decision
            );
        }
    }
}
