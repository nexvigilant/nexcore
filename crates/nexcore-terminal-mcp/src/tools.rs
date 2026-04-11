// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! MCP tools for terminal control — full PTY CLI/API.

use nexcore_terminal::pty::{PtyConfig, PtyProcess, PtySize};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

// ── Param structs ──────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SpawnParams {
    /// Working directory (default: home)
    pub working_dir: Option<String>,
    /// Shell to use (default: /bin/bash)
    pub shell: Option<String>,
    /// Terminal columns (default: 120)
    pub cols: Option<u16>,
    /// Terminal rows (default: 40)
    pub rows: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct WriteParams {
    /// Session ID from terminal_spawn
    pub session_id: String,
    /// Input to send (use \n for Enter)
    pub input: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReadParams {
    /// Session ID from terminal_spawn
    pub session_id: String,
    /// Clear buffer after reading (default: true)
    pub clear: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExecParams {
    /// Session ID from terminal_spawn
    pub session_id: String,
    /// Command to execute
    pub command: String,
    /// Milliseconds to wait for output (default: 2000)
    pub wait_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SessionIdParam {
    /// Session ID
    pub session_id: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ResizeParams {
    /// Session ID
    pub session_id: String,
    /// New column count
    pub cols: u16,
    /// New row count
    pub rows: u16,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SpeakParams {
    /// Text to speak
    pub text: String,
    /// Voice profile: clean|dark|tron1|tron2|tron3
    pub profile: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct AutopilotQueueParams {
    /// Prompt text to inject
    pub prompt: String,
}

// ── Server struct ──────────────────────────────────────────────

#[derive(Clone)]
pub struct TerminalMcp {
    sessions: Arc<Mutex<HashMap<String, PtyProcess>>>,
    counter: Arc<std::sync::atomic::AtomicU64>,
    output_buffer: Arc<Mutex<HashMap<String, String>>>,
    /// Background reader tasks — aborted on session kill to prevent zombies.
    reader_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    pub tool_router: ToolRouter<Self>,
}

impl TerminalMcp {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(std::sync::atomic::AtomicU64::new(1)),
            output_buffer: Arc::new(Mutex::new(HashMap::new())),
            reader_tasks: Arc::new(Mutex::new(HashMap::new())),
            tool_router: Self::tool_router(),
        }
    }

    fn next_id(&self) -> String {
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("mcp-pty-{n:04}")
    }

    fn ok_json(value: &serde_json::Value) -> Result<CallToolResult, McpError> {
        let text = serde_json::to_string_pretty(value)
            .unwrap_or_else(|e| format!("{{\"error\": \"JSON serialization failed: {e}\"}}"));
        Ok(CallToolResult::success(vec![Content::text(&text)]))
    }

    fn ok_result(text: &str) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    fn err_result(text: &str) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::error(vec![Content::text(text)]))
    }
}

// ── Tool implementations ───────────────────────────────────────

#[tool_router]
impl TerminalMcp {
    #[tool(
        description = "Spawn a new PTY terminal session with bash shell. Returns session_id for subsequent commands."
    )]
    async fn terminal_spawn(
        &self,
        Parameters(params): Parameters<SpawnParams>,
    ) -> Result<CallToolResult, McpError> {
        let shell = params.shell.unwrap_or_else(|| "/bin/bash".to_string());
        let working_dir = params
            .working_dir
            .unwrap_or_else(|| std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));
        let cols = params.cols.unwrap_or(120);
        let rows = params.rows.unwrap_or(40);

        let config = PtyConfig::new(&shell, &working_dir)
            .with_size(PtySize::new(cols, rows))
            .with_env("TERM", "xterm-256color")
            .with_env("COLORTERM", "truecolor")
            .with_env("LANG", "en_US.UTF-8");

        let process = match PtyProcess::spawn(config) {
            Ok(p) => p,
            Err(e) => return Self::err_result(&format!("PTY spawn failed: {e}")),
        };

        let session_id = self.next_id();
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id.clone(), process);

        // Start background output reader — tracked for cleanup on kill.
        let sessions_ref = Arc::clone(&self.sessions);
        let buffer_ref = Arc::clone(&self.output_buffer);
        let sid = session_id.clone();
        let handle = tokio::spawn(async move {
            loop {
                let read_result = {
                    let mut procs = sessions_ref.lock().await;
                    match procs.get_mut(&sid) {
                        Some(proc) if !proc.has_exited() => Some(
                            tokio::time::timeout(
                                std::time::Duration::from_millis(100),
                                proc.read(8192),
                            )
                            .await,
                        ),
                        _ => None,
                    }
                };
                match read_result {
                    None => break,
                    Some(Ok(Ok(data))) => {
                        if let Ok(text) = String::from_utf8(data) {
                            let mut buf = buffer_ref.lock().await;
                            let entry = buf.entry(sid.clone()).or_default();
                            entry.push_str(&text);
                            if entry.len() > 65536 {
                                let drain = entry.len() - 32768;
                                entry.drain(..drain);
                            }
                        }
                    }
                    Some(Ok(Err(_))) => break,
                    Some(Err(_)) => {
                        tokio::task::yield_now().await;
                    }
                }
            }
        });
        self.reader_tasks
            .lock()
            .await
            .insert(session_id.clone(), handle);

        let json = serde_json::json!({
            "session_id": session_id,
            "cols": cols,
            "rows": rows,
            "shell": shell,
        });
        Self::ok_json(&json)
    }

    #[tool(
        description = "Send input to a terminal session. Use \\n for Enter, \\t for Tab, \\x03 for Ctrl+C."
    )]
    async fn terminal_write(
        &self,
        Parameters(params): Parameters<WriteParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut sessions = self.sessions.lock().await;
        let proc = match sessions.get_mut(&params.session_id) {
            Some(p) => p,
            None => return Self::err_result(&format!("No session: {}", params.session_id)),
        };

        let data = params
            .input
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t")
            .replace("\\x03", "\x03")
            .replace("\\x04", "\x04");

        match proc.write(data.as_bytes()).await {
            Ok(()) => Self::ok_result(&format!(
                "Wrote {} bytes to {}",
                data.len(),
                params.session_id
            )),
            Err(e) => Self::err_result(&format!("Write failed: {e}")),
        }
    }

    #[tool(
        description = "Read buffered output from a terminal session. Returns text accumulated since last read."
    )]
    async fn terminal_read(
        &self,
        Parameters(params): Parameters<ReadParams>,
    ) -> Result<CallToolResult, McpError> {
        let clear = params.clear.unwrap_or(true);
        let mut buf = self.output_buffer.lock().await;

        let output = if clear {
            buf.remove(&params.session_id).unwrap_or_default()
        } else {
            buf.get(&params.session_id).cloned().unwrap_or_default()
        };

        let json = serde_json::json!({
            "session_id": params.session_id,
            "bytes_read": output.len(),
            "output": output,
        });
        Self::ok_json(&json)
    }

    #[tool(
        description = "Execute a command and return output. Sends command + Enter, waits, returns result."
    )]
    async fn terminal_exec(
        &self,
        Parameters(params): Parameters<ExecParams>,
    ) -> Result<CallToolResult, McpError> {
        let wait_ms = params.wait_ms.unwrap_or(2000);

        // Clear buffer
        {
            self.output_buffer.lock().await.remove(&params.session_id);
        }

        // Send command
        {
            let mut sessions = self.sessions.lock().await;
            let proc = match sessions.get_mut(&params.session_id) {
                Some(p) => p,
                None => return Self::err_result(&format!("No session: {}", params.session_id)),
            };
            if let Err(e) = proc.write(format!("{}\n", params.command).as_bytes()).await {
                return Self::err_result(&format!("Write failed: {e}"));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

        let mut buf = self.output_buffer.lock().await;
        let output = buf.remove(&params.session_id).unwrap_or_default();

        let json = serde_json::json!({
            "command": params.command,
            "output": output,
            "bytes": output.len(),
        });
        Self::ok_json(&json)
    }

    #[tool(description = "Resize a terminal session to new dimensions.")]
    async fn terminal_resize(
        &self,
        Parameters(params): Parameters<ResizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut sessions = self.sessions.lock().await;
        let proc = match sessions.get_mut(&params.session_id) {
            Some(p) => p,
            None => return Self::err_result(&format!("No session: {}", params.session_id)),
        };
        proc.resize(PtySize::new(params.cols, params.rows));
        Self::ok_result(&format!(
            "Resized {} to {}x{}",
            params.session_id, params.cols, params.rows
        ))
    }

    #[tool(description = "Kill a terminal session and its child processes.")]
    async fn terminal_kill(
        &self,
        Parameters(params): Parameters<SessionIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(mut proc) = sessions.remove(&params.session_id) {
            if let Err(e) = proc.kill().await {
                return Self::err_result(&format!("Kill failed: {e}"));
            }
            self.output_buffer.lock().await.remove(&params.session_id);
            // Abort background reader to prevent zombie task.
            if let Some(handle) = self.reader_tasks.lock().await.remove(&params.session_id) {
                handle.abort();
            }
            Self::ok_result(&format!("Killed {}", params.session_id))
        } else {
            Self::err_result(&format!("No session: {}", params.session_id))
        }
    }

    #[tool(description = "List all active PTY terminal sessions with status.")]
    async fn terminal_list(&self) -> Result<CallToolResult, McpError> {
        let sessions = self.sessions.lock().await;
        let mut infos = Vec::new();
        for (id, proc) in sessions.iter() {
            let size = proc.size();
            infos.push(serde_json::json!({
                "session_id": id,
                "cols": size.cols,
                "rows": size.rows,
                "exited": proc.has_exited(),
            }));
        }
        let json = serde_json::Value::Array(infos);
        Self::ok_json(&json)
    }

    #[tool(description = "Send Ctrl+C (SIGINT) to interrupt the running process.")]
    async fn terminal_interrupt(
        &self,
        Parameters(params): Parameters<SessionIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let mut sessions = self.sessions.lock().await;
        let proc = match sessions.get_mut(&params.session_id) {
            Some(p) => p,
            None => return Self::err_result(&format!("No session: {}", params.session_id)),
        };
        match proc.write(b"\x03").await {
            Ok(()) => Self::ok_result(&format!("Sent Ctrl+C to {}", params.session_id)),
            Err(e) => Self::err_result(&format!("Interrupt failed: {e}")),
        }
    }

    #[tool(description = "Speak text aloud via vigil-say neural TTS.")]
    async fn terminal_speak(
        &self,
        Parameters(params): Parameters<SpeakParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = tokio::process::Command::new("vigil-say");
        if let Some(ref p) = params.profile {
            cmd.arg("-p").arg(p);
        }
        cmd.arg(&params.text);
        match cmd.output().await {
            Ok(o) if o.status.success() => Self::ok_result(&format!(
                "Spoke: {}",
                &params.text[..params.text.len().min(80)]
            )),
            Ok(o) => Self::err_result(&format!(
                "TTS error: {}",
                String::from_utf8_lossy(&o.stderr)
            )),
            Err(e) => Self::err_result(&format!("TTS failed: {e}")),
        }
    }

    #[tool(
        description = "Queue a prompt for vigil-autopilot. Injected into Claude Code after current response."
    )]
    async fn terminal_autopilot_queue(
        &self,
        Parameters(params): Parameters<AutopilotQueueParams>,
    ) -> Result<CallToolResult, McpError> {
        match tokio::fs::write("/tmp/vigil-next-prompt", &params.prompt).await {
            Ok(()) => Self::ok_result(&format!(
                "Queued: {}",
                &params.prompt[..params.prompt.len().min(80)]
            )),
            Err(e) => Self::err_result(&format!("Queue failed: {e}")),
        }
    }

    #[tool(
        description = "Check vigil-autopilot status — active/stopped, prompt count, queue depth."
    )]
    async fn terminal_autopilot_status(&self) -> Result<CallToolResult, McpError> {
        let output = tokio::process::Command::new("vigil-autopilot")
            .arg("--status")
            .output()
            .await;
        match output {
            Ok(o) => Self::ok_result(&String::from_utf8_lossy(&o.stdout)),
            Err(e) => Self::err_result(&format!("Status failed: {e}")),
        }
    }

    #[tool(description = "Terminal MCP server health check.")]
    async fn terminal_health(&self) -> Result<CallToolResult, McpError> {
        let sessions = self.sessions.lock().await;
        let active = sessions.values().filter(|p| !p.has_exited()).count();
        let json = serde_json::json!({
            "status": "ok",
            "server": "nexcore-terminal-mcp",
            "sessions_active": active,
            "sessions_total": sessions.len(),
            "version": env!("CARGO_PKG_VERSION"),
        });
        Self::ok_json(&json)
    }
}

// ── ServerHandler (required for Service<RoleServer>) ───────────

impl ServerHandler for TerminalMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "NexVigilant Terminal MCP — full PTY control. Spawn sessions, \
                 execute commands, read output, resize, kill, speak via TTS, \
                 queue autopilot prompts. 12 tools."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "nexcore-terminal-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("NexVigilant Terminal MCP".into()),
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
            self.tool_router.call(tcc).await
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            Ok(ListToolsResult {
                tools: self.tool_router.list_all(),
                next_cursor: None,
                meta: None,
            })
        }
    }
}
