//! NPM tools: structured wrappers for Node.js package management
//!
//! Highway Class III (Orchestration) — <500ms for local, <5000ms for network.
//! Uses tokio::process::Command for async execution (install/outdated hit network).

use crate::params::npm::{NpmInstallParams, NpmListParams, NpmOutdatedParams, NpmRunParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;

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

/// Run npm command with timeout.
async fn run_npm(
    args: &[&str],
    path: &Option<String>,
    timeout_secs: u64,
) -> Result<(bool, String, String, u128), nexcore_error::NexError> {
    let start = Instant::now();
    let mut cmd = Command::new("npm");
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(p) = path {
        cmd.current_dir(p);
    }

    let output = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| nexcore_error::nexerror!("npm timed out after {timeout_secs}s"))?
        .map_err(|e| nexcore_error::nexerror!("Failed to execute npm: {e}"))?;

    let elapsed = start.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status.success(), stdout, stderr, elapsed))
}

/// npm run <script>
pub async fn npm_run(params: NpmRunParams) -> Result<CallToolResult, McpError> {
    match run_npm(&["run", &params.script], &params.path, 120).await {
        Ok((success, stdout, stderr, elapsed)) => {
            // Truncate if too large
            let output = if stdout.len() > 50_000 {
                format!(
                    "{}...\n[truncated — {} bytes total]",
                    &stdout[..50_000],
                    stdout.len()
                )
            } else {
                stdout
            };

            let result = json!({
                "command": format!("npm run {}", params.script),
                "success": success,
                "elapsed_ms": elapsed,
                "output": output.trim(),
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// npm install [packages] [--save-dev]
pub async fn npm_install(params: NpmInstallParams) -> Result<CallToolResult, McpError> {
    let mut args = vec!["install"];
    if params.dev {
        args.push("--save-dev");
    }
    let pkg_refs: Vec<&str> = params.packages.iter().map(|s| s.as_str()).collect();
    args.extend(pkg_refs.iter());

    match run_npm(&args, &params.path, 120).await {
        Ok((success, stdout, stderr, elapsed)) => {
            let result = json!({
                "command": "npm install",
                "success": success,
                "elapsed_ms": elapsed,
                "packages": params.packages,
                "dev": params.dev,
                "output": stdout.trim(),
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// npm list --depth=N --json
pub async fn npm_list(params: NpmListParams) -> Result<CallToolResult, McpError> {
    let depth_str = format!("--depth={}", params.depth);
    match run_npm(&["list", &depth_str, "--json"], &params.path, 30).await {
        Ok((success, stdout, stderr, elapsed)) => {
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let result = json!({
                "command": "npm list",
                "success": success,
                "elapsed_ms": elapsed,
                "depth": params.depth,
                "data": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, success)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}

/// npm outdated --json
pub async fn npm_outdated(params: NpmOutdatedParams) -> Result<CallToolResult, McpError> {
    match run_npm(&["outdated", "--json"], &params.path, 30).await {
        Ok((_, stdout, stderr, elapsed)) => {
            // npm outdated returns exit code 1 when packages are outdated (not an error)
            let data: serde_json::Value =
                serde_json::from_str(&stdout).unwrap_or_else(|_| json!(stdout.trim()));
            let count = data.as_object().map(|o| o.len()).unwrap_or(0);
            let result = json!({
                "command": "npm outdated",
                "success": true,
                "elapsed_ms": elapsed,
                "outdated_count": count,
                "packages": data,
                "stderr": if stderr.trim().is_empty() { serde_json::Value::Null } else { json!(stderr.trim()) },
            });
            format_result(result, true)
        }
        Err(e) => format_result(json!({"success": false, "error": e.to_string()}), false),
    }
}
