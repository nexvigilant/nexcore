// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for terminal session management.
//!
//! Wired to `nexcore_terminal` session types for real session lifecycle.

use nexcore_terminal::session::{SessionStatus, TerminalMode, TerminalSession};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use vr_core::ids::{TenantId, UserId};

/// Tauri-managed terminal state.
pub struct TerminalState {
    /// Active sessions keyed by session ID string.
    pub sessions: Mutex<HashMap<String, TerminalSession>>,
    /// Default tenant for desktop mode (single-user).
    pub tenant_id: TenantId,
    /// Default user for desktop mode.
    pub user_id: UserId,
}

impl TerminalState {
    /// Create terminal state for desktop (single-tenant, single-user).
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            tenant_id: TenantId::new(),
            user_id: UserId::new(),
        }
    }
}

/// Terminal session info returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSessionInfo {
    /// Unique session identifier.
    pub id: String,
    /// Current mode.
    pub mode: String,
    /// Session status.
    pub status: String,
    /// Terminal dimensions.
    pub cols: u16,
    /// Terminal rows.
    pub rows: u16,
    /// MCP calls made.
    pub mcp_calls: u64,
    /// AI tokens consumed.
    pub ai_tokens: u64,
}

fn session_to_info(s: &TerminalSession) -> TerminalSessionInfo {
    TerminalSessionInfo {
        id: s.id.to_string(),
        mode: serde_json::to_value(&s.mode)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "hybrid".into()),
        status: serde_json::to_value(&s.status)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".into()),
        cols: s.metadata.cols,
        rows: s.metadata.rows,
        mcp_calls: s.metadata.mcp_calls_made,
        ai_tokens: s.metadata.ai_tokens_used,
    }
}

fn parse_mode(mode: &str) -> TerminalMode {
    match mode {
        "shell" => TerminalMode::Shell,
        "regulatory" => TerminalMode::Regulatory,
        "ai" => TerminalMode::Ai,
        _ => TerminalMode::Hybrid,
    }
}

/// Create a new terminal session.
#[tauri::command]
pub fn terminal_create_session(
    state: tauri::State<'_, TerminalState>,
    mode: Option<String>,
) -> TerminalSessionInfo {
    let terminal_mode = parse_mode(mode.as_deref().unwrap_or("hybrid"));
    let mut session = TerminalSession::new(
        state.tenant_id.clone(),
        state.user_id.clone(),
        terminal_mode,
    );
    session.activate();

    let info = session_to_info(&session);
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(info.id.clone(), session);
    }
    info
}

/// List active terminal sessions.
#[tauri::command]
pub fn terminal_list_sessions(state: tauri::State<'_, TerminalState>) -> Vec<TerminalSessionInfo> {
    state
        .sessions
        .lock()
        .map(|sessions| {
            sessions
                .values()
                .filter(|s| s.is_alive())
                .map(session_to_info)
                .collect()
        })
        .unwrap_or_default()
}

/// Send input to the active terminal session.
///
/// Records input and output events on the χ health monitor for
/// real-time session health tracking.
#[tauri::command]
pub fn terminal_send_input(
    state: tauri::State<'_, TerminalState>,
    health: tauri::State<'_, super::health::HealthState>,
    session_id: String,
    data: String,
) -> Result<String, String> {
    // Record input event on the χ monitor
    if let Ok(mut monitor) = health.monitor.lock() {
        monitor.record_input();
    }

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| format!("Session not found: {session_id}"))?;

    if !session.is_alive() {
        return Err("Session is not active".into());
    }

    session.touch();

    // Route based on mode — PTY integration deferred to Phase 3
    let result = match session.mode {
        TerminalMode::Regulatory => {
            session.metadata.mcp_calls_made += 1;
            Ok(format!("[MCP] Routing regulatory query: {data}"))
        }
        TerminalMode::Ai => {
            session.metadata.ai_tokens_used += data.len() as u64;
            Ok(format!("[AI] Processing: {data}"))
        }
        TerminalMode::Shell => Ok(format!("[Shell] $ {data}")),
        TerminalMode::Hybrid => {
            // Auto-detect: if starts with nexvig> or / → MCP, @claude → AI, else → Shell
            if data.starts_with("nexvig>") || data.starts_with('/') {
                session.metadata.mcp_calls_made += 1;
                Ok(format!("[MCP] {data}"))
            } else if data.starts_with("@claude") {
                session.metadata.ai_tokens_used += data.len() as u64;
                Ok(format!("[AI] {data}"))
            } else {
                Ok(format!("[Shell] $ {data}"))
            }
        }
        _ => Ok(format!("echo: {data}")),
    };

    // Record output event on the χ monitor (response produced)
    if result.is_ok() {
        if let Ok(mut monitor) = health.monitor.lock() {
            monitor.record_output();
        }
    }

    result
}

/// Switch terminal mode for a session.
#[tauri::command]
pub fn terminal_switch_mode(
    state: tauri::State<'_, TerminalState>,
    session_id: String,
    mode: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| format!("Session not found: {session_id}"))?;
    session.mode = parse_mode(&mode);
    session.touch();
    Ok(())
}

/// Resize terminal for a session.
#[tauri::command]
pub fn terminal_resize(
    state: tauri::State<'_, TerminalState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| format!("Session not found: {session_id}"))?;
    session.metadata.cols = cols;
    session.metadata.rows = rows;
    session.touch();
    Ok(())
}

/// Get terminal session info by ID.
#[tauri::command]
pub fn terminal_get_session(
    state: tauri::State<'_, TerminalState>,
    session_id: String,
) -> Option<TerminalSessionInfo> {
    state
        .sessions
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&session_id).map(session_to_info))
}
