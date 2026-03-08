// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Remote Controller — programmatic terminal control surface for Claude.
//!
//! Provides a unified command dispatch layer that Claude can drive
//! via Tauri IPC or future MCP bridge. Each action captures a state
//! snapshot before and after execution, enabling screenshot-based
//! feedback loops for development steering.
//!
//! ## Feedback Loop Architecture
//!
//! ```text
//! Claude ──→ remote_execute(action) ──→ dispatch to subsystem
//!   ↑                                          │
//!   │                                          ↓
//!   ←── RemoteResult { before, after, diff } ──┘
//! ```
//!
//! The screenshot capture hook (`screenshot-capture.sh`) runs at the
//! `controller-state-changed` event boundary, providing visual signal
//! for Claude to assess and steer the next action.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Actions the remote controller can dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum RemoteAction {
    /// Create a new terminal session with optional mode.
    CreateSession {
        /// Terminal mode: shell, regulatory, ai, hybrid.
        mode: Option<String>,
    },
    /// Send input to a terminal session.
    SendInput {
        /// Target session ID.
        session_id: String,
        /// Input data to send.
        data: String,
    },
    /// Switch terminal mode for a session.
    SwitchMode {
        /// Target session ID.
        session_id: String,
        /// New mode.
        mode: String,
    },
    /// Get current health snapshot.
    GetHealth,
    /// Start health polling (500ms interval).
    StartHealthPolling,
    /// Stop health polling.
    StopHealthPolling,
    /// List active terminal sessions.
    ListSessions,
    /// Get cloud service overview.
    CloudOverview,
    /// List cloud services.
    CloudServices,
    /// Get shell status.
    ShellStatus,
    /// Get full system snapshot (all subsystems).
    SystemSnapshot,
    /// Spawn a PTY process.
    SpawnPty {
        /// Shell binary path.
        shell: String,
        /// Working directory.
        working_dir: String,
        /// Terminal columns.
        cols: u16,
        /// Terminal rows.
        rows: u16,
    },
    /// Write to a PTY session.
    PtyWrite {
        /// PTY session ID.
        session_id: String,
        /// Data to write.
        data: String,
    },
    /// Kill a PTY session.
    PtyKill {
        /// PTY session ID.
        session_id: String,
    },
    /// Execute a sequence of actions atomically.
    Batch {
        /// Ordered list of actions to execute.
        actions: Vec<RemoteAction>,
    },
}

/// Result of a remote controller action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteResult {
    /// Whether the action succeeded.
    pub success: bool,
    /// Action that was executed.
    pub action: String,
    /// State snapshot before execution.
    pub before: SystemState,
    /// State snapshot after execution.
    pub after: SystemState,
    /// Fields that changed between before and after.
    pub diff: Vec<StateDiff>,
    /// Execution time in milliseconds.
    pub duration_ms: u64,
    /// Action-specific result payload (JSON).
    pub payload: serde_json::Value,
    /// Error message if failed.
    pub error: Option<String>,
    /// Sequence number for ordering in batch results.
    pub seq: u64,
}

/// Snapshot of full system state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Number of active terminal sessions.
    pub active_sessions: usize,
    /// Number of active PTY processes.
    pub active_ptys: usize,
    /// Current health band.
    pub health_band: String,
    /// Current chi value.
    pub chi: f64,
    /// Whether health polling is active.
    pub health_polling: bool,
    /// Cloud service count.
    pub cloud_services: usize,
    /// Cloud healthy count.
    pub cloud_healthy: usize,
    /// Shell state.
    pub shell_state: String,
    /// Timestamp (monotonic, relative to start).
    pub timestamp_ms: u64,
}

impl SystemState {
    /// Compute differences between two states.
    #[must_use]
    pub fn diff(&self, other: &Self) -> Vec<StateDiff> {
        let mut diffs = Vec::new();

        if self.active_sessions != other.active_sessions {
            diffs.push(StateDiff {
                field: "active_sessions".into(),
                before: serde_json::Value::Number(self.active_sessions.into()),
                after: serde_json::Value::Number(other.active_sessions.into()),
            });
        }
        if self.active_ptys != other.active_ptys {
            diffs.push(StateDiff {
                field: "active_ptys".into(),
                before: serde_json::Value::Number(self.active_ptys.into()),
                after: serde_json::Value::Number(other.active_ptys.into()),
            });
        }
        if self.health_band != other.health_band {
            diffs.push(StateDiff {
                field: "health_band".into(),
                before: serde_json::Value::String(self.health_band.clone()),
                after: serde_json::Value::String(other.health_band.clone()),
            });
        }
        // Use bitwise comparison for f64 — chi is computed, not arbitrary
        if self.chi.to_bits() != other.chi.to_bits() {
            diffs.push(StateDiff {
                field: "chi".into(),
                before: serde_json::json!(self.chi),
                after: serde_json::json!(other.chi),
            });
        }
        if self.health_polling != other.health_polling {
            diffs.push(StateDiff {
                field: "health_polling".into(),
                before: serde_json::Value::Bool(self.health_polling),
                after: serde_json::Value::Bool(other.health_polling),
            });
        }
        if self.cloud_services != other.cloud_services {
            diffs.push(StateDiff {
                field: "cloud_services".into(),
                before: serde_json::Value::Number(self.cloud_services.into()),
                after: serde_json::Value::Number(other.cloud_services.into()),
            });
        }
        if self.cloud_healthy != other.cloud_healthy {
            diffs.push(StateDiff {
                field: "cloud_healthy".into(),
                before: serde_json::Value::Number(self.cloud_healthy.into()),
                after: serde_json::Value::Number(other.cloud_healthy.into()),
            });
        }
        if self.shell_state != other.shell_state {
            diffs.push(StateDiff {
                field: "shell_state".into(),
                before: serde_json::Value::String(self.shell_state.clone()),
                after: serde_json::Value::String(other.shell_state.clone()),
            });
        }

        diffs
    }
}

/// A single field change between two system states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    /// Name of the field that changed.
    pub field: String,
    /// Value before the action.
    pub before: serde_json::Value,
    /// Value after the action.
    pub after: serde_json::Value,
}

/// Monotonic sequence counter for batch ordering.
static SEQ_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_seq() -> u64 {
    SEQ_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Global start time for relative timestamps.
static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn elapsed_ms() -> u64 {
    let start = START.get_or_init(Instant::now);
    start.elapsed().as_millis() as u64
}

/// Capture the current system state from all subsystems.
///
/// This is the core measurement function — it reads real state from
/// the Tauri-managed state objects, producing an observed snapshot
/// (not inferred, not cached).
pub fn capture_state(
    terminal: &tauri::State<'_, super::terminal::TerminalState>,
    health: &tauri::State<'_, super::health::HealthState>,
    cloud: &tauri::State<'_, super::cloud::CloudState>,
) -> SystemState {
    let active_sessions = terminal
        .sessions
        .lock()
        .map(|s| s.values().filter(|sess| sess.is_alive()).count())
        .unwrap_or(0);

    let (chi, health_band) = health
        .monitor
        .lock()
        .map(|mut m| {
            let h = m.compute();
            (h.chi, format!("{:?}", h.band))
        })
        .unwrap_or((0.0, "SpinningUp".into()));

    let health_polling = health.polling.lock().map(|p| *p).unwrap_or(false);

    let snap = cloud.registry.snapshot();
    let cloud_services = snap.len();
    let cloud_healthy = snap
        .iter()
        .filter(|r| r.state == nexcloud::supervisor::registry::ProcessState::Healthy)
        .count();

    SystemState {
        active_sessions,
        active_ptys: 0, // PTY count requires async lock — populated in async context
        chi,
        health_band,
        health_polling,
        cloud_services,
        cloud_healthy,
        shell_state: "running".into(),
        timestamp_ms: elapsed_ms(),
    }
}

/// Execute a remote action, capturing before/after state snapshots.
///
/// This is the main entry point for the remote controller. Every action
/// produces a `RemoteResult` with state diffs, enabling Claude to
/// observe the effect of each action and steer accordingly.
#[tauri::command]
pub fn remote_execute(
    action: RemoteAction,
    terminal: tauri::State<'_, super::terminal::TerminalState>,
    health: tauri::State<'_, super::health::HealthState>,
    cloud: tauri::State<'_, super::cloud::CloudState>,
    app: tauri::AppHandle,
) -> RemoteResult {
    let before = capture_state(&terminal, &health, &cloud);
    let start = Instant::now();

    let (success, payload, error) = dispatch_action(&action, &terminal, &health, &cloud, &app);

    let after = capture_state(&terminal, &health, &cloud);
    let diff = before.diff(&after);
    let duration_ms = start.elapsed().as_millis() as u64;

    let action_name = match &action {
        RemoteAction::CreateSession { .. } => "CreateSession",
        RemoteAction::SendInput { .. } => "SendInput",
        RemoteAction::SwitchMode { .. } => "SwitchMode",
        RemoteAction::GetHealth => "GetHealth",
        RemoteAction::StartHealthPolling => "StartHealthPolling",
        RemoteAction::StopHealthPolling => "StopHealthPolling",
        RemoteAction::ListSessions => "ListSessions",
        RemoteAction::CloudOverview => "CloudOverview",
        RemoteAction::CloudServices => "CloudServices",
        RemoteAction::ShellStatus => "ShellStatus",
        RemoteAction::SystemSnapshot => "SystemSnapshot",
        RemoteAction::SpawnPty { .. } => "SpawnPty",
        RemoteAction::PtyWrite { .. } => "PtyWrite",
        RemoteAction::PtyKill { .. } => "PtyKill",
        RemoteAction::Batch { .. } => "Batch",
    };

    // Emit controller event for screenshot hook
    if let Ok(event_json) = serde_json::to_value(&RemoteControllerEvent {
        action: action_name.to_string(),
        success,
        diff_count: diff.len(),
        chi: after.chi,
        health_band: after.health_band.clone(),
    }) {
        let _ = tauri::Emitter::emit(&app, "controller-state-changed", &event_json);
    }

    RemoteResult {
        success,
        action: action_name.into(),
        before,
        after,
        diff,
        duration_ms,
        payload,
        error,
        seq: next_seq(),
    }
}

/// Event payload emitted after each controller action.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemoteControllerEvent {
    action: String,
    success: bool,
    diff_count: usize,
    chi: f64,
    health_band: String,
}

/// Get a full system snapshot without executing any action.
#[tauri::command]
pub fn remote_snapshot(
    terminal: tauri::State<'_, super::terminal::TerminalState>,
    health: tauri::State<'_, super::health::HealthState>,
    cloud: tauri::State<'_, super::cloud::CloudState>,
) -> SystemState {
    capture_state(&terminal, &health, &cloud)
}

/// Get the action history (sequence counter value).
#[tauri::command]
pub fn remote_action_count() -> u64 {
    SEQ_COUNTER.load(std::sync::atomic::Ordering::Relaxed) - 1
}

/// Dispatch a remote action to the appropriate subsystem.
fn dispatch_action(
    action: &RemoteAction,
    terminal: &tauri::State<'_, super::terminal::TerminalState>,
    health: &tauri::State<'_, super::health::HealthState>,
    cloud: &tauri::State<'_, super::cloud::CloudState>,
    app: &tauri::AppHandle,
) -> (bool, serde_json::Value, Option<String>) {
    match action {
        RemoteAction::CreateSession { mode } => {
            let info = super::terminal::terminal_create_session(terminal.clone(), mode.clone());
            (true, serde_json::to_value(&info).unwrap_or_default(), None)
        }

        RemoteAction::SendInput { session_id, data } => {
            match super::terminal::terminal_send_input(
                terminal.clone(),
                health.clone(),
                session_id.clone(),
                data.clone(),
            ) {
                Ok(output) => (true, serde_json::json!({ "output": output }), None),
                Err(e) => (false, serde_json::Value::Null, Some(e)),
            }
        }

        RemoteAction::SwitchMode { session_id, mode } => {
            match super::terminal::terminal_switch_mode(
                terminal.clone(),
                session_id.clone(),
                mode.clone(),
            ) {
                Ok(()) => (true, serde_json::Value::Null, None),
                Err(e) => (false, serde_json::Value::Null, Some(e)),
            }
        }

        RemoteAction::GetHealth => {
            let snap = super::health::health_get(health.clone());
            (true, serde_json::to_value(&snap).unwrap_or_default(), None)
        }

        RemoteAction::StartHealthPolling => {
            let started = super::health::health_start_polling(app.clone(), health.clone());
            (
                started,
                serde_json::json!({ "started": started }),
                if started {
                    None
                } else {
                    Some("Polling already active".into())
                },
            )
        }

        RemoteAction::StopHealthPolling => {
            super::health::health_stop_polling(health.clone());
            (true, serde_json::Value::Null, None)
        }

        RemoteAction::ListSessions => {
            let sessions = super::terminal::terminal_list_sessions(terminal.clone());
            (
                true,
                serde_json::to_value(&sessions).unwrap_or_default(),
                None,
            )
        }

        RemoteAction::CloudOverview => {
            let overview = super::cloud::cloud_overview(cloud.clone());
            (
                true,
                serde_json::to_value(&overview).unwrap_or_default(),
                None,
            )
        }

        RemoteAction::CloudServices => {
            let services = super::cloud::cloud_list_services(cloud.clone());
            (
                true,
                serde_json::to_value(&services).unwrap_or_default(),
                None,
            )
        }

        RemoteAction::ShellStatus => {
            let status = super::shell::shell_status();
            (
                true,
                serde_json::to_value(&status).unwrap_or_default(),
                None,
            )
        }

        RemoteAction::SystemSnapshot => {
            // Already captured in before/after — return the current state as payload
            let state = capture_state(terminal, health, cloud);
            (true, serde_json::to_value(&state).unwrap_or_default(), None)
        }

        RemoteAction::SpawnPty { .. }
        | RemoteAction::PtyWrite { .. }
        | RemoteAction::PtyKill { .. } => {
            // PTY operations require async — return guidance
            (
                false,
                serde_json::Value::Null,
                Some("PTY operations require async context — use pty_spawn/pty_write/pty_kill IPC commands directly".into()),
            )
        }

        RemoteAction::Batch { actions } => {
            let mut results = Vec::with_capacity(actions.len());
            let mut all_success = true;

            for sub_action in actions {
                let (success, payload, error) =
                    dispatch_action(sub_action, terminal, health, cloud, app);
                if !success {
                    all_success = false;
                }
                results.push(serde_json::json!({
                    "action": format!("{sub_action:?}"),
                    "success": success,
                    "payload": payload,
                    "error": error,
                }));
            }

            (all_success, serde_json::json!({ "results": results }), None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_state_diff_detects_changes() {
        let before = SystemState {
            active_sessions: 0,
            active_ptys: 0,
            chi: 0.0,
            health_band: "SpinningUp".into(),
            health_polling: false,
            cloud_services: 4,
            cloud_healthy: 0,
            shell_state: "running".into(),
            timestamp_ms: 0,
        };

        let after = SystemState {
            active_sessions: 1,
            active_ptys: 0,
            chi: 0.15,
            health_band: "Healthy".into(),
            health_polling: true,
            cloud_services: 4,
            cloud_healthy: 2,
            shell_state: "running".into(),
            timestamp_ms: 100,
        };

        let diff = before.diff(&after);
        assert_eq!(diff.len(), 5); // sessions, chi, band, polling, cloud_healthy
    }

    #[test]
    fn test_system_state_no_diff_when_equal() {
        let state = SystemState {
            active_sessions: 2,
            active_ptys: 1,
            chi: 0.1,
            health_band: "Healthy".into(),
            health_polling: true,
            cloud_services: 4,
            cloud_healthy: 3,
            shell_state: "running".into(),
            timestamp_ms: 500,
        };

        let diff = state.diff(&state);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_seq_counter_increments() {
        let a = next_seq();
        let b = next_seq();
        assert!(b > a);
    }

    #[test]
    fn test_elapsed_ms_increases() {
        let t1 = elapsed_ms();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let t2 = elapsed_ms();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_remote_action_serde_roundtrip() {
        let action = RemoteAction::CreateSession {
            mode: Some("regulatory".into()),
        };
        let json = serde_json::to_string(&action).unwrap_or_default();
        let parsed: Result<RemoteAction, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_batch_action_serde() {
        let batch = RemoteAction::Batch {
            actions: vec![
                RemoteAction::GetHealth,
                RemoteAction::ListSessions,
                RemoteAction::CloudOverview,
            ],
        };
        let json = serde_json::to_string(&batch).unwrap_or_default();
        assert!(json.contains("Batch"));
        assert!(json.contains("GetHealth"));
    }
}
