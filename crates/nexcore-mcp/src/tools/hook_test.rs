//! Hook testing tools.
//!
//! Test hooks with mock input without triggering real tool events.

use std::io::Write;
use std::time::Instant;

use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::hook_test::{HookTestAllParams, HookTestParams};

/// Test a single hook with mock input.
pub fn hook_test(params: HookTestParams) -> Result<CallToolResult, McpError> {
    let hooks_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/matthew"))
        .join(".claude/hooks/bash");

    let hook_path = hooks_dir.join(&params.hook_name);

    if !hook_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Hook not found: {}. Available hooks in {}",
            params.hook_name,
            hooks_dir.display()
        ))]));
    }

    let mock_json = serde_json::to_string(&params.mock_input).unwrap_or_else(|_| "{}".to_string());

    let start = Instant::now();

    let mut child = match std::process::Command::new("bash")
        .arg(hook_path.to_str().unwrap_or(""))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to spawn hook: {e}"
            ))]));
        }
    };

    // Write mock input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(mock_json.as_bytes());
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Hook execution failed: {e}"
            ))]));
        }
    };

    let elapsed = start.elapsed();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Try to parse stdout as JSON
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|_| serde_json::json!({"raw": stdout}));

    let result = serde_json::json!({
        "success": exit_code == 0,
        "hook_name": params.hook_name,
        "event_type": params.event_type,
        "exit_code": exit_code,
        "elapsed_ms": elapsed.as_millis(),
        "stdout": stdout_json,
        "stderr": if stderr.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(stderr) },
    });

    if exit_code == 0 {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    } else {
        Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }
}

/// Test all hooks in the hooks directory.
pub fn hook_test_all(params: HookTestAllParams) -> Result<CallToolResult, McpError> {
    let hooks_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/home/matthew"))
        .join(".claude/hooks/bash");

    if !hooks_dir.exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Hooks directory not found: ~/.claude/hooks/bash/",
        )]));
    }

    let entries = match std::fs::read_dir(&hooks_dir) {
        Ok(e) => e,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to read hooks dir: {e}"
            ))]));
        }
    };

    let mut results = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().is_some_and(|e| e == "sh") {
            continue;
        }

        let hook_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Create minimal mock input based on event type
        let event_type = params
            .event_type
            .clone()
            .unwrap_or_else(|| "PostToolUse".to_string());
        let mock_input = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {"command": "echo test"},
            "tool_response": "test",
            "session_id": "test-session"
        });

        let test_result = hook_test(HookTestParams {
            hook_name: hook_name.clone(),
            event_type: event_type.clone(),
            mock_input,
        });

        match test_result {
            Ok(r) => {
                // Extract success/exit from the result content
                let content_text = r
                    .content
                    .first()
                    .and_then(|c| match &c.raw {
                        rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let parsed: serde_json::Value =
                    serde_json::from_str(&content_text).unwrap_or_default();
                results.push(serde_json::json!({
                    "hook": hook_name,
                    "exit_code": parsed.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(-1),
                    "elapsed_ms": parsed.get("elapsed_ms").and_then(|v| v.as_u64()).unwrap_or(0),
                    "success": parsed.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                }));
            }
            Err(_) => {
                results.push(serde_json::json!({
                    "hook": hook_name,
                    "exit_code": -1,
                    "elapsed_ms": 0,
                    "success": false,
                    "error": "dispatch error",
                }));
            }
        }
    }

    let passed = results
        .iter()
        .filter(|r| r.get("success").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();
    let total = results.len();

    let result = serde_json::json!({
        "success": true,
        "total_hooks": total,
        "passed": passed,
        "failed": total - passed,
        "event_type": params.event_type,
        "results": results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
