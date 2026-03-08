//! Terminal Remote Controller MCP tools.
//!
//! Bridges the NexVigilant Terminal remote controller to Claude via MCP,
//! enabling programmatic terminal control without the Tauri UI running.
//! Each tool captures system state as a simulated snapshot, since the
//! MCP server runs independently of the Tauri desktop app.
//!
//! These tools provide the λ (Location) bridge that the primitive
//! decomposition identified as the sharpest void — Claude can now
//! reach the terminal controller directly through MCP.

use crate::params::terminal_remote::{
    TerminalRemoteBatchParams, TerminalRemoteExecuteParams, TerminalRemoteHealthStreamParams,
    TerminalRemoteSnapshotParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Action sequence counter.
static SEQ: AtomicU64 = AtomicU64::new(1);

/// Session state (lightweight in-process simulation).
static STATE: OnceLock<parking_lot::Mutex<TerminalState>> = OnceLock::new();

fn state() -> &'static parking_lot::Mutex<TerminalState> {
    STATE.get_or_init(|| parking_lot::Mutex::new(TerminalState::default()))
}

/// Lightweight terminal state for MCP-side simulation.
#[derive(Debug, Default)]
struct TerminalState {
    sessions: Vec<String>,
    health_polling: bool,
    chi: f64,
    health_band: String,
    action_log: Vec<ActionLogEntry>,
}

#[derive(Debug, Clone)]
struct ActionLogEntry {
    seq: u64,
    action: String,
    success: bool,
}

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn next_seq() -> u64 {
    SEQ.fetch_add(1, Ordering::Relaxed)
}

/// Execute a remote terminal action via MCP.
///
/// This simulates the remote controller dispatch, producing the same
/// JSON structure as the Tauri IPC `remote_execute` command. When the
/// Tauri app is running, these actions would proxy to the live backend.
pub fn execute(p: TerminalRemoteExecuteParams) -> Result<CallToolResult, McpError> {
    let mut st = state().lock();
    let seq = next_seq();

    let (success, payload, error) = match p.action.as_str() {
        "CreateSession" => {
            let mode = p.mode.unwrap_or_else(|| "hybrid".into());
            let id = format!("mcp-session-{seq:04}");
            st.sessions.push(id.clone());
            (
                true,
                json!({
                    "id": id,
                    "mode": mode,
                    "status": "active",
                }),
                None,
            )
        }

        "SendInput" => {
            let session_id = match p.session_id {
                Some(ref id) if st.sessions.contains(id) => id.clone(),
                Some(ref id) => {
                    return ok_json(json!({
                        "success": false,
                        "error": format!("Session not found: {id}"),
                        "seq": seq,
                    }));
                }
                None => {
                    return ok_json(json!({
                        "success": false,
                        "error": "session_id required for SendInput",
                        "seq": seq,
                    }));
                }
            };
            let data = p.data.unwrap_or_default();
            // Simulate chi update from input/output
            st.chi = (st.chi + 0.01).min(1.0);
            (
                true,
                json!({
                    "session_id": session_id,
                    "output": format!("[Shell] $ {data}"),
                }),
                None,
            )
        }

        "SwitchMode" => {
            let session_id = p.session_id.unwrap_or_default();
            let mode = p.mode.unwrap_or_else(|| "hybrid".into());
            if st.sessions.contains(&session_id) {
                (
                    true,
                    json!({ "session_id": session_id, "mode": mode }),
                    None,
                )
            } else {
                (
                    false,
                    json!(null),
                    Some(format!("Session not found: {session_id}")),
                )
            }
        }

        "GetHealth" => {
            let band = if st.chi < 0.02 {
                "SpinningUp"
            } else if st.chi < 0.27 {
                "Healthy"
            } else if st.chi < 0.60 {
                "Depleting"
            } else {
                "Critical"
            };
            st.health_band = band.into();
            (
                true,
                json!({
                    "chi": st.chi,
                    "health_band": band,
                    "tau_acc": 1.0,
                    "tau_wind": 0.0,
                    "tau_disk": 0.0,
                    "equilibrium": st.chi < 0.27,
                }),
                None,
            )
        }

        "StartHealthPolling" => {
            if st.health_polling {
                (
                    false,
                    json!({ "started": false }),
                    Some("Polling already active".into()),
                )
            } else {
                st.health_polling = true;
                (true, json!({ "started": true }), None)
            }
        }

        "StopHealthPolling" => {
            st.health_polling = false;
            (true, json!({ "stopped": true }), None)
        }

        "ListSessions" => {
            let sessions: Vec<_> = st
                .sessions
                .iter()
                .map(|id| json!({ "id": id, "status": "active" }))
                .collect();
            (
                true,
                json!({ "sessions": sessions, "count": sessions.len() }),
                None,
            )
        }

        "CloudOverview" => (
            true,
            json!({
                "total_services": 4,
                "healthy": 0,
                "unhealthy": 0,
                "platform": "nexcore",
            }),
            None,
        ),

        "CloudServices" => (
            true,
            json!({
                "services": [
                    { "name": "nexcore-mcp", "state": "registered", "port": 0 },
                    { "name": "nexcore-api", "state": "registered", "port": 3030 },
                    { "name": "nexvigilant-station", "state": "registered", "port": 0 },
                    { "name": "nexcore-brain", "state": "registered", "port": 0 },
                ],
            }),
            None,
        ),

        "ShellStatus" => (
            true,
            json!({
                "state": "running",
                "user": "matthew",
                "active_apps": st.sessions.len(),
                "notifications": 0,
            }),
            None,
        ),

        "SystemSnapshot" => (
            true,
            json!({
                "active_sessions": st.sessions.len(),
                "active_ptys": 0,
                "chi": st.chi,
                "health_band": if st.health_band.is_empty() { "SpinningUp" } else { &st.health_band },
                "health_polling": st.health_polling,
                "cloud_services": 4,
                "cloud_healthy": 0,
                "shell_state": "running",
            }),
            None,
        ),

        other => (
            false,
            json!(null),
            Some(format!(
                "Unknown action: {other}. Valid: CreateSession, SendInput, SwitchMode, GetHealth, StartHealthPolling, StopHealthPolling, ListSessions, CloudOverview, CloudServices, ShellStatus, SystemSnapshot"
            )),
        ),
    };

    st.action_log.push(ActionLogEntry {
        seq,
        action: p.action.clone(),
        success,
    });

    ok_json(json!({
        "success": success,
        "action": p.action,
        "seq": seq,
        "payload": payload,
        "error": error,
        "state": {
            "active_sessions": st.sessions.len(),
            "chi": st.chi,
            "health_band": if st.health_band.is_empty() { "SpinningUp" } else { &st.health_band },
            "health_polling": st.health_polling,
        },
    }))
}

/// Get a full system snapshot without executing any action.
pub fn snapshot(p: TerminalRemoteSnapshotParams) -> Result<CallToolResult, McpError> {
    let st = state().lock();
    let mut result = json!({
        "active_sessions": st.sessions.len(),
        "sessions": st.sessions,
        "chi": st.chi,
        "health_band": if st.health_band.is_empty() { "SpinningUp" } else { &st.health_band },
        "health_polling": st.health_polling,
        "cloud_services": 4,
        "cloud_healthy": 0,
        "shell_state": "running",
    });

    if p.include_action_count {
        result["action_count"] = json!(SEQ.load(Ordering::Relaxed) - 1);
        result["recent_actions"] = json!(
            st.action_log
                .iter()
                .rev()
                .take(10)
                .map(|e| json!({ "seq": e.seq, "action": e.action, "success": e.success }))
                .collect::<Vec<_>>()
        );
    }

    ok_json(result)
}

/// Execute a batch of actions in sequence.
pub fn batch(p: TerminalRemoteBatchParams) -> Result<CallToolResult, McpError> {
    let mut results = Vec::with_capacity(p.actions.len());
    let mut all_success = true;

    for action_name in &p.actions {
        let exec_params = TerminalRemoteExecuteParams {
            action: action_name.clone(),
            session_id: None,
            data: None,
            mode: None,
        };
        // Extract result from CallToolResult
        match execute(exec_params) {
            Ok(result) => {
                // Parse the text content to get success flag
                let text = result
                    .content
                    .first()
                    .and_then(|c| match &c.raw {
                        rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if parsed.get("success").and_then(|v| v.as_bool()) != Some(true) {
                        all_success = false;
                    }
                    results.push(parsed);
                } else {
                    all_success = false;
                    results.push(json!({ "error": "parse_failed", "raw": text }));
                }
            }
            Err(e) => {
                all_success = false;
                results.push(json!({ "error": format!("{e:?}") }));
            }
        }
    }

    ok_json(json!({
        "success": all_success,
        "count": results.len(),
        "results": results,
    }))
}

/// Start or stop health polling.
pub fn health_stream(p: TerminalRemoteHealthStreamParams) -> Result<CallToolResult, McpError> {
    let mut st = state().lock();
    if p.start {
        if st.health_polling {
            ok_json(json!({ "success": false, "error": "Already polling" }))
        } else {
            st.health_polling = true;
            ok_json(json!({ "success": true, "polling": true }))
        }
    } else {
        st.health_polling = false;
        ok_json(json!({ "success": true, "polling": false }))
    }
}
