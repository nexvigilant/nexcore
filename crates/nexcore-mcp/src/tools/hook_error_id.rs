//! Hook error identification tool.
//!
//! Reads settings.json, extracts all hook registrations, tests each
//! command-type hook, and reports which ones are failing.

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Instant;

use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::hook_error_id::HookErrorIdentifyParams;

/// Identify failing hooks by scanning settings.json registrations.
pub fn hook_error_identify(params: HookErrorIdentifyParams) -> Result<CallToolResult, McpError> {
    let settings_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/matthew"))
        .join(".claude/settings.json");

    if !settings_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "settings.json not found at ~/.claude/settings.json",
        )]));
    }

    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to read settings.json: {e}"
            ))]));
        }
    };

    let settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to parse settings.json: {e}"
            ))]));
        }
    };

    let hooks_obj = match settings.get("hooks").and_then(|h| h.as_object()) {
        Some(h) => h,
        None => {
            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::json!({"total": 0, "ok": 0, "failing": 0, "hooks": []}).to_string(),
            )]));
        }
    };

    let mut all_results = Vec::new();
    let mut failing = Vec::new();

    for (event, groups) in hooks_obj {
        // Filter by event if specified
        if let Some(ref filter_event) = params.event {
            if event != filter_event {
                continue;
            }
        }

        let groups_arr = match groups.as_array() {
            Some(a) => a,
            None => continue,
        };

        for group in groups_arr {
            let matcher = group.get("matcher").and_then(|m| m.as_str()).unwrap_or("*");

            // Filter by matcher if specified
            if let Some(ref filter_matcher) = params.matcher {
                if !matcher.contains(filter_matcher.as_str()) {
                    continue;
                }
            }

            let hooks_arr = match group.get("hooks").and_then(|h| h.as_array()) {
                Some(a) => a,
                None => continue,
            };

            for hook in hooks_arr {
                let hook_type = hook.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if hook_type != "command" {
                    continue;
                }

                let command = match hook.get("command").and_then(|c| c.as_str()) {
                    Some(c) => c,
                    None => continue,
                };

                let status_msg = hook
                    .get("statusMessage")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                // Extract script path (first token of command)
                let script_path = command.split_whitespace().next().unwrap_or(command);
                let path = PathBuf::from(script_path);
                let hook_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let start = Instant::now();
                let (status, error) = check_hook(&path, command);
                let elapsed = start.elapsed().as_millis();

                let entry = serde_json::json!({
                    "event": event,
                    "matcher": matcher,
                    "hook": hook_name,
                    "command": command,
                    "status_message": status_msg,
                    "status": status,
                    "error": error,
                    "elapsed_ms": elapsed,
                });

                if status != "OK" {
                    failing.push(entry.clone());
                }
                all_results.push(entry);
            }
        }
    }

    let ok_count = all_results.len() - failing.len();

    let result = serde_json::json!({
        "total": all_results.len(),
        "ok": ok_count,
        "failing": failing.len(),
        "hooks": failing,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Check a single hook: exists, executable, runs without error.
fn check_hook(path: &PathBuf, command: &str) -> (&'static str, serde_json::Value) {
    if !path.exists() {
        return (
            "MISSING",
            serde_json::Value::String(format!("Script not found: {}", path.display())),
        );
    }

    let is_executable = path
        .metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false);

    if !is_executable {
        return (
            "NOT_EXECUTABLE",
            serde_json::Value::String(format!("Missing +x: {}", path.display())),
        );
    }

    // Run with mock input, 5s timeout
    let mock_input = r#"{"tool_name":"Bash","tool_input":{"command":"echo test"}}"#;

    let mut child = match std::process::Command::new("bash")
        .arg(path.to_str().unwrap_or(""))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                "SPAWN_ERROR",
                serde_json::Value::String(format!("Failed to spawn: {e}")),
            );
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(mock_input.as_bytes());
    }

    // Wait with timeout via channel
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    let timeout = std::time::Duration::from_secs(5);
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            let exit_code = output.status.code().unwrap_or(-1);
            if exit_code != 0 {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let truncated = if stderr.len() > 200 {
                    format!("{}...", &stderr[..200])
                } else {
                    stderr.to_string()
                };
                (
                    "ERROR",
                    serde_json::json!({
                        "exit_code": exit_code,
                        "stderr": truncated,
                    }),
                )
            } else {
                // Check if stdout is valid JSON with decision: block
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&stdout.trim()) {
                    if parsed.get("decision").and_then(|d| d.as_str()) == Some("block") {
                        let reason = parsed
                            .get("reason")
                            .and_then(|r| r.as_str())
                            .unwrap_or("unknown");
                        return (
                            "BLOCKS",
                            serde_json::Value::String(format!("Blocks on test input: {reason}")),
                        );
                    }
                }
                ("OK", serde_json::Value::Null)
            }
        }
        Ok(Err(e)) => (
            "ERROR",
            serde_json::Value::String(format!("Wait failed: {e}")),
        ),
        Err(_) => (
            "TIMEOUT",
            serde_json::Value::String("Exceeded 5s timeout".to_string()),
        ),
    }
}
