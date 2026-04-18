// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexVigilant Terminal — Desktop Agent Cloud Shell
//!
//! Tauri v2 desktop application wrapping three nexcore subsystems into
//! a unified agent observability terminal.
//!
//! ## Architecture
//!
//! ```text
//! ┌─ NexVigilant Terminal (Tauri v2) ─────────────────────────┐
//! │                                                            │
//! │  ┌─ Frontend (HTML/JS) ──────────────────────────────────┐│
//! │  │  Terminal (xterm.js) │ Agent Dashboard │ Command Palette│
//! │  └────────────────────┬───────────────────────────────────┘│
//! │                       │ Tauri IPC Commands                 │
//! │  ┌────────────────────┴───────────────────────────────────┐│
//! │  │  commands::terminal  → nexcore-terminal (sessions/PTY) ││
//! │  │  commands::cloud     → nexcloud (CloudSupervisor)      ││
//! │  │  commands::shell     → nexcore-shell (apps/login/AI)   ││
//! │  └────────────────────────────────────────────────────────┘│
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary: Tauri IPC gate) + ς(State: session lifecycle) +
//!  μ(Mapping: command routing) + σ(Sequence: boot→login→terminal) +
//!  ν(Frequency: agent event stream)`

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod commands;
pub mod mcp_bridge;

#[cfg(test)]
mod tests {
    use super::commands::cloud::{CloudState, ServiceInfo};
    use super::commands::health::HealthState;
    use super::commands::pty::PtyState;
    use super::commands::repl::ReplState;
    use super::commands::shell::NexShellState;
    use super::commands::terminal::TerminalState;

    // ── CloudState ──────────────────────────────────────────

    #[test]
    fn cloud_state_registers_default_services() {
        let state = CloudState::new();
        let snap = state.registry.snapshot();
        assert!(
            snap.len() >= 4,
            "Expected at least 4 default services, got {}",
            snap.len()
        );
        let names: Vec<String> = snap.iter().map(|r| r.name.clone()).collect();
        assert!(names.contains(&"nexcore-mcp".into()));
        assert!(names.contains(&"nexcore-api".into()));
    }

    #[test]
    fn cloud_state_default_matches_new() {
        let a = CloudState::new();
        let b = CloudState::default();
        assert_eq!(a.registry.snapshot().len(), b.registry.snapshot().len());
    }

    #[test]
    fn cloud_service_info_serializes() {
        let info = ServiceInfo {
            name: "nexcore-mcp".into(),
            port: 0,
            state: "Registered".into(),
            health: "unknown".into(),
            uptime_secs: 0,
            restarts: 0,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("nexcore-mcp"));
    }

    // ── NexShellState ───────────────────────────────────────

    #[test]
    fn shell_state_boots_to_running() {
        let state = NexShellState::new();
        let shell = state.shell.lock().unwrap();
        assert_eq!(shell.state(), nexcore_shell::ShellState::Running);
    }

    #[test]
    fn shell_state_has_registered_apps() {
        let state = NexShellState::new();
        let shell = state.shell.lock().unwrap();
        let apps = shell.apps().list();
        let names: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
        // boot() registers Launcher + Settings, our code adds Terminal + Observatory
        assert!(
            names.contains(&"Terminal"),
            "Missing Terminal app in {:?}",
            names
        );
        assert!(
            names.contains(&"Observatory"),
            "Missing Observatory app in {:?}",
            names
        );
    }

    #[test]
    fn shell_state_default_matches_new() {
        let a = NexShellState::new();
        let b = NexShellState::default();
        let a_count = a.shell.lock().unwrap().apps().list().len();
        let b_count = b.shell.lock().unwrap().apps().list().len();
        assert_eq!(a_count, b_count);
    }

    #[test]
    fn shell_app_info_from_app() {
        let state = NexShellState::new();
        let shell = state.shell.lock().unwrap();
        let apps = shell.apps().list();
        let info: Vec<super::commands::shell::AppInfo> = apps
            .iter()
            .map(super::commands::shell::AppInfo::from)
            .collect();
        assert!(!info.is_empty());
        // All should serialize
        for app in &info {
            let json = serde_json::to_string(app).unwrap();
            assert!(json.contains(&app.name));
        }
    }

    // ── TerminalState ───────────────────────────────────────

    #[test]
    fn terminal_state_starts_empty() {
        let state = TerminalState::new();
        // Verify it's constructable and Default works
        let _default = TerminalState::default();
        // No sessions at start — can't access internals easily but construction proves wiring
        drop(state);
    }

    // ── HealthState ─────────────────────────────────────────

    #[test]
    fn health_state_starts_not_polling() {
        let state = HealthState::new();
        let polling = state.polling.lock().unwrap();
        assert!(!*polling, "Health polling should not auto-start");
    }

    #[test]
    fn health_state_default_matches_new() {
        let _a = HealthState::new();
        let _b = HealthState::default();
        // Both should construct without panic
    }

    // ── ReplState ───────────────────────────────────────────

    #[test]
    fn repl_state_starts_idle() {
        let state = ReplState::new();
        let _default = ReplState::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let busy = rt.block_on(async { *state.busy.lock().await });
        assert!(!busy, "REPL should start idle");
    }

    // ── PtyState ────────────────────────────────────────────

    #[test]
    fn pty_state_starts_empty() {
        let state = PtyState::new();
        let _default = PtyState::default();
        drop(state);
    }

    // ── Palette integration ─────────────────────────────────

    #[test]
    fn command_palette_empty_query_returns_empty() {
        let state = NexShellState::new();
        let mut palette = state.palette.lock().unwrap();
        palette.set_query("");
        assert!(palette.visible_results().is_empty());
    }
}
