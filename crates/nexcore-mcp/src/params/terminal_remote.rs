//! Typed parameter structs for Terminal Remote Controller MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `terminal_remote_execute` — execute a remote action.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TerminalRemoteExecuteParams {
    /// Action to execute. One of: CreateSession, SendInput, SwitchMode,
    /// GetHealth, StartHealthPolling, StopHealthPolling, ListSessions,
    /// CloudOverview, CloudServices, ShellStatus, SystemSnapshot.
    pub action: String,
    /// Optional session ID (for SendInput, SwitchMode).
    #[serde(default)]
    pub session_id: Option<String>,
    /// Optional input data (for SendInput).
    #[serde(default)]
    pub data: Option<String>,
    /// Optional mode (for CreateSession, SwitchMode).
    #[serde(default)]
    pub mode: Option<String>,
}

/// Parameters for `terminal_remote_snapshot` — get system state.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TerminalRemoteSnapshotParams {
    /// Optional: include action history count in response.
    #[serde(default)]
    pub include_action_count: bool,
}

/// Parameters for `terminal_remote_batch` — execute multiple actions.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TerminalRemoteBatchParams {
    /// List of action names to execute in sequence.
    pub actions: Vec<String>,
}

/// Parameters for `terminal_remote_health_stream` — start/stop health polling.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct TerminalRemoteHealthStreamParams {
    /// Whether to start (true) or stop (false) health polling.
    pub start: bool,
}
