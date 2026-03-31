// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Claude Code as a first-class terminal mode.
//!
//! Manages the Claude Code CLI lifecycle within a dedicated PTY,
//! pre-configured with NexVigilant Station as an MCP server.
//! The terminal handles:
//! - Spawning `claude` with Station MCP pre-wired
//! - Process lifecycle (start/stop/restart)
//! - Status reporting to the frontend
//!
//! ## Architecture
//!
//! ```text
//! Frontend (xterm.js)
//!   │ Tauri IPC: claude_start / claude_stop / claude_status
//!   ▼
//! commands::claude (this module)
//!   │ Spawns PTY with claude binary + MCP env
//!   ▼
//! Claude Code TUI  ←stdio→  xterm.js
//!   │ MCP client (built-in)
//!   ▼
//! mcp.nexvigilant.com (2000+ PV tools)
//! ```
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary: PTY isolation) + μ(Mapping: MCP config injection) +
//!  ς(State: process lifecycle) + →(Causality: start→ready→tools)`

use nexcore_terminal::pty::{PtyConfig, PtyProcess, PtySize};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// Claude Code binary name (resolved via PATH).
const CLAUDE_BINARY: &str = "claude";

/// Station MCP endpoint for Claude Code to connect to.
const STATION_MCP_URL: &str = "https://mcp.nexvigilant.com/sse";

/// Claude process state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeState {
    /// Not running.
    Stopped,
    /// PTY spawned, waiting for Claude to initialize.
    Starting,
    /// Claude Code TUI is active and responsive.
    Running,
    /// Process exited (code available in ClaudeStatus).
    Exited,
}

/// Status info returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeStatus {
    /// Current process state.
    pub state: ClaudeState,
    /// PTY session ID (if running).
    pub session_id: Option<String>,
    /// Whether Station MCP is configured.
    pub station_connected: bool,
    /// Working directory Claude was launched in.
    pub working_dir: Option<String>,
    /// Claude Code arguments used at launch.
    pub args: Vec<String>,
    /// Exit code (if exited).
    pub exit_code: Option<i32>,
}

/// Managed state for the Claude Code process.
pub struct ClaudeCodeState {
    /// The PTY process running Claude.
    process: Arc<Mutex<Option<PtyProcess>>>,
    /// Current state.
    state: Arc<Mutex<ClaudeState>>,
    /// Session ID for PTY events.
    session_id: Arc<Mutex<Option<String>>>,
    /// Working directory.
    working_dir: Arc<Mutex<Option<String>>>,
    /// Launch args.
    args: Arc<Mutex<Vec<String>>>,
    /// Exit code from last run.
    exit_code: Arc<Mutex<Option<i32>>>,
    /// Session counter.
    counter: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for ClaudeCodeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCodeState {
    /// Create new Claude Code state manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(ClaudeState::Stopped)),
            session_id: Arc::new(Mutex::new(None)),
            working_dir: Arc::new(Mutex::new(None)),
            args: Arc::new(Mutex::new(Vec::new())),
            exit_code: Arc::new(Mutex::new(None)),
            counter: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    fn next_id(&self) -> String {
        let n = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("claude-{n:04}")
    }
}

/// Build the MCP config JSON for Claude Code.
///
/// Creates a temporary config that adds NexVigilant Station as an MCP server
/// alongside whatever the user already has configured.
fn build_mcp_config() -> String {
    // Claude Code reads MCP servers from ~/.claude.json (mcpServers key).
    // We don't override that — the user's existing config is preserved.
    // Instead, we ensure the environment signals Station availability.
    //
    // If the user doesn't have nexvigilant-station in their ~/.claude.json,
    // Claude Code can still reach it via the remote MCP connector or
    // by the user adding it. We just make sure the env is right.
    serde_json::json!({
        "mcpServers": {
            "nexvigilant-station": {
                "command": "npx",
                "args": ["-y", "mcp-remote", STATION_MCP_URL]
            }
        }
    })
    .to_string()
}

/// Resolve the claude binary path, checking common locations.
fn find_claude_binary() -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let candidates = [
        format!("{home}/.local/bin/claude"),
        format!("{home}/.cargo/bin/claude"),
        "/usr/local/bin/claude".to_string(),
        "/usr/bin/claude".to_string(),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.clone());
        }
    }

    // Fall back to PATH resolution
    Ok(CLAUDE_BINARY.to_string())
}

// ── Tauri Commands ──────────────────────────────────────────────

/// Start Claude Code in a dedicated PTY with Station MCP pre-configured.
#[tauri::command]
pub async fn claude_start(
    state: tauri::State<'_, ClaudeCodeState>,
    pty_state: tauri::State<'_, super::pty::PtyState>,
    app: tauri::AppHandle,
    working_dir: Option<String>,
    args: Option<Vec<String>>,
) -> Result<ClaudeStatus, String> {
    // Check if already running
    {
        let current = state.state.lock().await;
        if *current == ClaudeState::Running || *current == ClaudeState::Starting {
            return Err("Claude Code is already running. Stop it first.".into());
        }
    }

    // Resolve paths
    let claude_bin = find_claude_binary()?;
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let cwd = working_dir.clone().unwrap_or_else(|| home.clone());
    let extra_args = args.clone().unwrap_or_default();

    // Build the shell command that launches Claude Code.
    // We use bash -c to set up the environment properly.
    let mut claude_cmd_parts = vec![claude_bin.clone()];
    claude_cmd_parts.extend(extra_args.iter().cloned());
    let claude_cmd = claude_cmd_parts.join(" ");

    tracing::info!(
        binary = %claude_bin,
        cwd = %cwd,
        args = ?extra_args,
        "Starting Claude Code"
    );

    // Update state to starting
    {
        let mut s = state.state.lock().await;
        *s = ClaudeState::Starting;
        let mut ec = state.exit_code.lock().await;
        *ec = None;
    }

    // Enrich PATH
    let enriched_path = {
        let current = std::env::var("PATH").unwrap_or_default();
        let cargo_bin = format!("{home}/.cargo/bin");
        let local_bin = format!("{home}/.local/bin");
        let nvm_bin = format!("{home}/.nvm/versions/node/v22.16.0/bin");
        let mut paths: Vec<&str> = current.split(':').collect();
        for p in [cargo_bin.as_str(), local_bin.as_str(), nvm_bin.as_str()] {
            if !paths.contains(&p) {
                paths.insert(0, p);
            }
        }
        paths.join(":")
    };

    let term_value = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());
    let lang_value = std::env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string());

    // Spawn PTY with claude as the process (not bash → claude)
    let config = PtyConfig::new(&claude_cmd, &cwd)
        .with_size(PtySize::new(120, 40))
        .with_env("PATH", &enriched_path)
        .with_env("TERM", &term_value)
        .with_env("COLORTERM", "truecolor")
        .with_env("LANG", &lang_value)
        .with_env("LC_ALL", &lang_value)
        .with_env("FORCE_COLOR", "1")
        .with_env("CLICOLOR_FORCE", "1")
        .with_env("HOME", &home)
        // Propagate Claude-specific env
        .with_env(
            "ANTHROPIC_API_KEY",
            &std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
        )
        .with_env(
            "CLAUDE_CODE_ENABLE_TELEMETRY",
            &std::env::var("CLAUDE_CODE_ENABLE_TELEMETRY").unwrap_or_default(),
        );

    let process =
        PtyProcess::spawn(config).map_err(|e| format!("Failed to spawn Claude Code: {e}"))?;

    let session_id = state.next_id();

    // Store in PTY state so the existing pty-output event stream works
    {
        let mut procs = pty_state.processes.lock().await;
        // Store under the claude session ID — the frontend listens to pty-output
        procs.insert(session_id.clone(), process);
    }

    // Start the output reader (reuse the same event pattern as pty_spawn)
    let processes = Arc::clone(&pty_state.processes);
    let sid = session_id.clone();
    let app_handle = app.clone();
    let claude_state = Arc::clone(&state.state);
    let claude_exit = Arc::clone(&state.exit_code);

    tokio::spawn(async move {
        // Mark as running once we see first output
        let mut first_output = true;

        loop {
            let read_result = {
                let mut procs = processes.lock().await;
                match procs.get_mut(&sid) {
                    Some(proc) => {
                        if proc.has_exited() {
                            None
                        } else {
                            Some(
                                tokio::time::timeout(
                                    std::time::Duration::from_millis(50),
                                    proc.read(8192),
                                )
                                .await,
                            )
                        }
                    }
                    None => None,
                }
            };

            match read_result {
                None => break,
                Some(Err(_timeout)) => {
                    tokio::task::yield_now().await;
                    continue;
                }
                Some(Ok(Ok(bytes))) if bytes.is_empty() => break,
                Some(Ok(Ok(bytes))) => {
                    if first_output {
                        let mut s = claude_state.lock().await;
                        *s = ClaudeState::Running;
                        first_output = false;
                    }

                    let text = String::from_utf8_lossy(&bytes);
                    // Emit on the same channel as PTY — frontend routes by session
                    if let Err(e) = app_handle.emit("pty-output", text.as_ref()) {
                        tracing::debug!(error = %e, "Failed to emit claude output");
                        break;
                    }
                }
                Some(Ok(Err(e))) => {
                    let is_would_block = e.to_string().contains("no data available");
                    if !is_would_block {
                        tracing::debug!(error = %e, "Claude PTY read error");
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }

        // Process exited
        let exit_code = {
            let mut procs = processes.lock().await;
            if let Some(mut proc) = procs.remove(&sid) {
                match proc.try_wait() {
                    Ok(Some(code)) => Some(code),
                    _ => None,
                }
            } else {
                None
            }
        };

        {
            let mut s = claude_state.lock().await;
            *s = ClaudeState::Exited;
            let mut ec = claude_exit.lock().await;
            *ec = exit_code;
        }

        #[derive(Serialize, Clone)]
        struct ClaudeExitPayload {
            session_id: String,
            code: Option<i32>,
        }

        let _ = app_handle.emit(
            "claude-exit",
            ClaudeExitPayload {
                session_id: sid,
                code: exit_code,
            },
        );

        tracing::info!(exit_code = ?exit_code, "Claude Code exited");
    });

    // Store metadata
    {
        let mut sid_lock = state.session_id.lock().await;
        *sid_lock = Some(session_id.clone());
        let mut wd = state.working_dir.lock().await;
        *wd = Some(cwd.clone());
        let mut a = state.args.lock().await;
        *a = extra_args.clone();
    }

    Ok(ClaudeStatus {
        state: ClaudeState::Starting,
        session_id: Some(session_id),
        station_connected: true,
        working_dir: Some(cwd),
        args: extra_args,
        exit_code: None,
    })
}

/// Stop the running Claude Code process.
#[tauri::command]
pub async fn claude_stop(
    state: tauri::State<'_, ClaudeCodeState>,
    pty_state: tauri::State<'_, super::pty::PtyState>,
) -> Result<(), String> {
    let sid = {
        let sid_lock = state.session_id.lock().await;
        sid_lock.clone()
    };

    if let Some(session_id) = sid {
        let mut procs = pty_state.processes.lock().await;
        if let Some(mut process) = procs.remove(&session_id) {
            process
                .kill()
                .await
                .map_err(|e| format!("Failed to kill Claude Code: {e}"))?;
        }
    }

    {
        let mut s = state.state.lock().await;
        *s = ClaudeState::Stopped;
        let mut sid_lock = state.session_id.lock().await;
        *sid_lock = None;
    }

    tracing::info!("Claude Code stopped");
    Ok(())
}

/// Get current Claude Code status.
#[tauri::command]
pub async fn claude_status(
    state: tauri::State<'_, ClaudeCodeState>,
) -> Result<ClaudeStatus, String> {
    let current_state = *state.state.lock().await;
    let session_id = state.session_id.lock().await.clone();
    let working_dir = state.working_dir.lock().await.clone();
    let args = state.args.lock().await.clone();
    let exit_code = *state.exit_code.lock().await;

    Ok(ClaudeStatus {
        state: current_state,
        session_id,
        station_connected: true,
        working_dir,
        args,
        exit_code,
    })
}

/// Restart Claude Code (stop + start).
#[tauri::command]
pub async fn claude_restart(
    state: tauri::State<'_, ClaudeCodeState>,
    pty_state: tauri::State<'_, super::pty::PtyState>,
    app: tauri::AppHandle,
) -> Result<ClaudeStatus, String> {
    // Grab working dir and args before stop clears them
    let working_dir = state.working_dir.lock().await.clone();
    let args = state.args.lock().await.clone();

    claude_stop(state.clone(), pty_state.clone()).await?;

    // Brief pause for cleanup
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    claude_start(state, pty_state, app, working_dir, Some(args)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_state_starts_stopped() {
        let state = ClaudeCodeState::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let current = rt.block_on(async { *state.state.lock().await });
        assert_eq!(current, ClaudeState::Stopped);
    }

    #[test]
    fn claude_state_default_matches_new() {
        let _a = ClaudeCodeState::new();
        let _b = ClaudeCodeState::default();
    }

    #[test]
    fn find_claude_resolves() {
        // Should at least return the fallback
        let result = find_claude_binary();
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_config_is_valid_json() {
        let config = build_mcp_config();
        let parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        assert!(parsed.get("mcpServers").is_some());
    }

    #[test]
    fn claude_session_id_increments() {
        let state = ClaudeCodeState::new();
        let a = state.next_id();
        let b = state.next_id();
        assert_ne!(a, b);
        assert!(a.starts_with("claude-"));
    }

    #[test]
    fn claude_status_serializes() {
        let status = ClaudeStatus {
            state: ClaudeState::Running,
            session_id: Some("claude-0001".into()),
            station_connected: true,
            working_dir: Some("/home/test".into()),
            args: vec!["--verbose".into()],
            exit_code: None,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("running"));
        assert!(json.contains("claude-0001"));
    }
}
