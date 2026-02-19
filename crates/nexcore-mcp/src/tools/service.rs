//! Systemctl tools: structured wrappers for service management
//!
//! Highway Class III (Orchestration) — local operations, <500ms SLA.
//! Only allows --user scope by default for safety.

use crate::params::service::{
    SystemctlListParams, SystemctlRestartParams, SystemctlStartParams, SystemctlStatusParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::process::Command;
use std::time::Instant;

fn format_result(result: serde_json::Value, success: bool) -> Result<CallToolResult, McpError> {
    let content = vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )];
    if success {
        Ok(CallToolResult::success(content))
    } else {
        Ok(CallToolResult::error(content))
    }
}

/// Run a systemctl command and capture output.
fn run_systemctl(args: &[&str]) -> Result<(bool, String, String, u128), McpError> {
    let start = Instant::now();
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to execute systemctl: {e}").into(),
            data: None,
        })?;
    let elapsed = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stdout, stderr, elapsed))
}

/// systemctl status for a unit.
pub fn systemctl_status(params: SystemctlStatusParams) -> Result<CallToolResult, McpError> {
    let mut args = vec![];
    if params.user {
        args.push("--user");
    }
    args.extend(["status", &params.unit]);

    let (success, stdout, stderr, elapsed) = run_systemctl(&args)?;

    // Parse common fields from status output
    let mut active_state = String::new();
    let mut sub_state = String::new();
    let mut pid = String::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Active:") {
            let rest = rest.trim();
            if let Some(paren_start) = rest.find('(') {
                active_state = rest[..paren_start].trim().to_string();
                if let Some(paren_end) = rest.find(')') {
                    sub_state = rest[paren_start + 1..paren_end].to_string();
                }
            } else {
                active_state = rest.to_string();
            }
        } else if let Some(rest) = trimmed.strip_prefix("Main PID:") {
            pid = rest
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
        }
    }

    let result = json!({
        "command": "systemctl status",
        "success": success,
        "elapsed_ms": elapsed,
        "unit": params.unit,
        "user_scope": params.user,
        "active": active_state,
        "sub_state": sub_state,
        "pid": pid,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    // Note: systemctl status returns exit code 3 for inactive units, which is not an error
    format_result(result, true)
}

/// systemctl restart (user scope only by default).
pub fn systemctl_restart(params: SystemctlRestartParams) -> Result<CallToolResult, McpError> {
    if !params.user {
        let result = json!({
            "command": "systemctl restart",
            "success": false,
            "error": "BLOCKED: System-wide restart is not allowed. Use user=true for --user scope.",
            "unit": params.unit,
        });
        return format_result(result, false);
    }

    let (success, stdout, stderr, elapsed) = run_systemctl(&["--user", "restart", &params.unit])?;

    let result = json!({
        "command": "systemctl restart",
        "success": success,
        "elapsed_ms": elapsed,
        "unit": params.unit,
        "user_scope": true,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// systemctl start (user scope only by default).
pub fn systemctl_start(params: SystemctlStartParams) -> Result<CallToolResult, McpError> {
    if !params.user {
        let result = json!({
            "command": "systemctl start",
            "success": false,
            "error": "BLOCKED: System-wide start is not allowed. Use user=true for --user scope.",
            "unit": params.unit,
        });
        return format_result(result, false);
    }

    let (success, stdout, stderr, elapsed) = run_systemctl(&["--user", "start", &params.unit])?;

    let result = json!({
        "command": "systemctl start",
        "success": success,
        "elapsed_ms": elapsed,
        "unit": params.unit,
        "user_scope": true,
        "output": stdout.trim(),
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}

/// systemctl list-units.
pub fn systemctl_list(params: SystemctlListParams) -> Result<CallToolResult, McpError> {
    let mut args = vec![];
    if params.user {
        args.push("--user");
    }
    args.push("list-units");
    args.push("--no-pager");

    let state_flag: String;
    if let Some(ref state) = params.state {
        state_flag = format!("--state={state}");
        args.push(&state_flag);
    }

    let (success, stdout, stderr, elapsed) = run_systemctl(&args)?;

    // Parse units from output
    let mut units: Vec<serde_json::Value> = Vec::new();
    for line in stdout.lines().skip(1) {
        // Skip header line
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("LOAD")
            || trimmed.contains("loaded units listed")
        {
            continue;
        }
        let parts: Vec<&str> = trimmed.splitn(5, char::is_whitespace).collect();
        if parts.len() >= 4 {
            units.push(json!({
                "unit": parts[0].trim_start_matches('●').trim(),
                "load": parts.get(1).unwrap_or(&""),
                "active": parts.get(2).unwrap_or(&""),
                "sub": parts.get(3).unwrap_or(&""),
                "description": parts.get(4).unwrap_or(&""),
            }));
        }
    }

    let result = json!({
        "command": "systemctl list-units",
        "success": success,
        "elapsed_ms": elapsed,
        "user_scope": params.user,
        "count": units.len(),
        "units": units,
        "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
    });

    format_result(result, success)
}
