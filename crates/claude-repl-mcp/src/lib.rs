#![forbid(unsafe_code)]
#![warn(missing_docs)]
use std::path::{Path, PathBuf};
use std::process::Stdio;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[derive(Debug, nexcore_error::Error)]
pub enum BridgeError {
    #[error("claude cli not found at {0}")]
    MissingClaudeCli(String),
    #[error("failed to spawn claude cli: {0}")]
    SpawnFailed(String),
    #[error("claude cli failed: {0}")]
    CliFailed(String),
    #[error("claude cli timed out after {0}ms")]
    Timeout(u64),
}

#[derive(Debug, Clone)]
pub struct ClaudeCliPath(PathBuf);

impl ClaudeCliPath {
    pub fn discover() -> Result<Self, BridgeError> {
        if let Ok(path) = std::env::var("CLAUDE_CLI_PATH") {
            let pb = PathBuf::from(path);
            return Self::validate(pb);
        }
        Self::validate(PathBuf::from("/home/matthew/.local/bin/claude"))
    }

    fn validate(path: PathBuf) -> Result<Self, BridgeError> {
        if path.exists() {
            Ok(Self(path))
        } else {
            Err(BridgeError::MissingClaudeCli(path.display().to_string()))
        }
    }

    fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClaudeReplParams {
    pub prompt: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub settings_path: Option<String>,
    pub mcp_config_path: Option<String>,
    pub strict_mcp_config: Option<bool>,
    pub permission_mode: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub output_format: Option<String>,
    pub system_prompt: Option<String>,
    pub append_system_prompt: Option<String>,
    pub persist_session: Option<bool>,
    pub timeout_ms: Option<u64>,
    pub max_output_bytes: Option<usize>,
}

#[derive(Clone)]
pub struct ClaudeReplMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ClaudeReplMcpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Bridge to Claude Code CLI. Provide {prompt} and optional session/model/settings."
    )]
    async fn claude_repl(
        &self,
        Parameters(params): Parameters<ClaudeReplParams>,
    ) -> Result<CallToolResult, McpError> {
        match run_claude(params).await {
            Ok(output) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                output,
            )])),
            Err(err) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                err.to_string(),
            )])),
        }
    }
}

impl Default for ClaudeReplMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for ClaudeReplMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                r#"Claude REPL MCP Bridge

Provides a single tool `claude_repl` that forwards prompts to the Claude Code CLI.
"#
                .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "claude-repl-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Claude REPL MCP Bridge".into()),
                icons: None,
                website_url: None,
            },
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tcc = ToolCallContext::new(self, request, context);
            let result = self.tool_router.call(tcc).await?;
            Ok(result)
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            meta: None,
            next_cursor: None,
        }))
    }
}

async fn run_claude(params: ClaudeReplParams) -> Result<String, BridgeError> {
    let cli = ClaudeCliPath::discover()?;

    let mut cmd = Command::new(cli.as_path());
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

    cmd.arg(params.prompt);

    cmd.stdin(Stdio::null());
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| BridgeError::SpawnFailed(err.to_string()))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| BridgeError::SpawnFailed("missing stdout".to_string()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| BridgeError::SpawnFailed("missing stderr".to_string()))?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout
            .read_to_end(&mut buf)
            .await
            .map_err(|err| BridgeError::CliFailed(err.to_string()))?;
        Ok::<Vec<u8>, BridgeError>(buf)
    });

    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr
            .read_to_end(&mut buf)
            .await
            .map_err(|err| BridgeError::CliFailed(err.to_string()))?;
        Ok::<Vec<u8>, BridgeError>(buf)
    });

    let status = if let Some(timeout_ms) = params.timeout_ms {
        match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), child.wait()).await
        {
            Ok(result) => result.map_err(|err| BridgeError::CliFailed(err.to_string()))?,
            Err(_) => {
                let _ = child.kill().await;
                return Err(BridgeError::Timeout(timeout_ms));
            }
        }
    } else {
        child
            .wait()
            .await
            .map_err(|err| BridgeError::CliFailed(err.to_string()))?
    };

    let stdout_buf = stdout_task
        .await
        .map_err(|err| BridgeError::CliFailed(err.to_string()))??;
    let stderr_buf = stderr_task
        .await
        .map_err(|err| BridgeError::CliFailed(err.to_string()))??;

    let output = std::process::Output {
        status,
        stdout: stdout_buf,
        stderr: stderr_buf,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let combined = if stderr.is_empty() { stdout } else { stderr };
        return Err(BridgeError::CliFailed(combined));
    }

    let max_bytes = params.max_output_bytes.unwrap_or(1_000_000);
    let mut out = output.stdout;
    if out.len() > max_bytes {
        out.truncate(max_bytes);
    }
    Ok(String::from_utf8_lossy(&out).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_params() {
        let params = ClaudeReplParams {
            prompt: "hello".to_string(),
            model: Some("sonnet".to_string()),
            session_id: None,
            settings_path: None,
            mcp_config_path: None,
            strict_mcp_config: None,
            permission_mode: None,
            allowed_tools: None,
            output_format: None,
            system_prompt: None,
            append_system_prompt: None,
            persist_session: Some(false),
            timeout_ms: Some(2000),
            max_output_bytes: Some(1024),
        };
        let json = serde_json::to_string(&params).expect("serialize");
        assert!(json.contains("\"prompt\""));
    }
}
