//! Claude REPL — CLI bridge to spawn Claude Code subprocesses.
//!
//! Consolidated from `claude-repl-mcp` satellite MCP server.
//! 1 tool: claude_repl.
//!
//! Tier: T3 (σ Sequence + ∂ Boundary + ς State + → Causality)

use crate::params::ClaudeReplParams;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// Bridge to Claude Code CLI.
pub async fn claude_repl(params: ClaudeReplParams) -> Result<CallToolResult, McpError> {
    match run_claude(params).await {
        Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
        Err(msg) => Ok(CallToolResult::success(vec![Content::text(msg)])),
    }
}

async fn run_claude(params: ClaudeReplParams) -> Result<String, String> {
    let cli_path = discover_cli()?;

    let mut cmd = Command::new(&cli_path);
    cmd.arg("--print");

    let output_format = params.output_format.unwrap_or_else(|| "text".to_string());
    cmd.arg("--output-format").arg(output_format);

    if let Some(model) = params.model {
        cmd.arg("--model").arg(model);
    }
    if let Some(session_id) = params.session_id {
        cmd.arg("--session-id").arg(session_id);
    } else if params.persist_session == Some(false) {
        cmd.arg("--no-session-persistence");
    }
    if let Some(settings_path) = params.settings_path {
        cmd.arg("--settings").arg(settings_path);
    }
    if let Some(mcp_config_path) = params.mcp_config_path {
        cmd.arg("--mcp-config").arg(mcp_config_path);
        if params.strict_mcp_config.unwrap_or(false) {
            cmd.arg("--strict-mcp-config");
        }
    }
    if let Some(permission_mode) = params.permission_mode {
        cmd.arg("--permission-mode").arg(permission_mode);
    }
    if let Some(allowed) = params.allowed_tools {
        if !allowed.is_empty() {
            cmd.arg("--allowedTools").arg(allowed.join(","));
        }
    }
    if let Some(system_prompt) = params.system_prompt {
        cmd.arg("--system-prompt").arg(system_prompt);
    }
    if let Some(append_system_prompt) = params.append_system_prompt {
        cmd.arg("--append-system-prompt").arg(append_system_prompt);
    }

    cmd.arg(&params.prompt);

    cmd.stdin(std::process::Stdio::null());
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude cli: {e}"))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "missing stdout".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "missing stderr".to_string())?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).await.ok();
        buf
    });

    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr.read_to_end(&mut buf).await.ok();
        buf
    });

    let status = if let Some(timeout_ms) = params.timeout_ms {
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), child.wait()).await
        {
            Ok(result) => result.map_err(|e| format!("CLI failed: {e}"))?,
            Err(_) => {
                let _ = child.kill().await;
                return Err(format!("Claude CLI timed out after {timeout_ms}ms"));
            }
        }
    } else {
        child.wait().await.map_err(|e| format!("CLI failed: {e}"))?
    };

    let stdout_buf = stdout_task.await.unwrap_or_default();
    let stderr_buf = stderr_task.await.unwrap_or_default();

    if !status.success() {
        let stderr_str = String::from_utf8_lossy(&stderr_buf).trim().to_string();
        let stdout_str = String::from_utf8_lossy(&stdout_buf).trim().to_string();
        let combined = if stderr_str.is_empty() {
            stdout_str
        } else {
            stderr_str
        };
        return Err(format!("Claude CLI failed: {combined}"));
    }

    let max_bytes = params.max_output_bytes.unwrap_or(1_000_000);
    let mut out = stdout_buf;
    if out.len() > max_bytes {
        out.truncate(max_bytes);
    }
    Ok(String::from_utf8_lossy(&out).to_string())
}

fn discover_cli() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("CLAUDE_CLI_PATH") {
        let pb = PathBuf::from(path);
        if pb.exists() {
            return Ok(pb);
        }
        return Err(format!("CLAUDE_CLI_PATH not found: {}", pb.display()));
    }
    let default = PathBuf::from("/home/matthew/.local/bin/claude");
    if default.exists() {
        Ok(default)
    } else {
        Err(format!("Claude CLI not found at {}", default.display()))
    }
}
