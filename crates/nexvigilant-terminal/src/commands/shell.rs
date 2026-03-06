// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for the NexCore Shell (app launcher, login, AI partner).
//!
//! Bridges nexcore-shell's device-level concepts to the Tauri desktop app.

use serde::{Deserialize, Serialize};

/// App info for the launcher grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current state.
    pub state: String,
    /// Icon identifier or path.
    pub icon: Option<String>,
}

/// Shell status returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStatus {
    /// Current shell state (idle, booting, running, locked, sleeping, stopped).
    pub state: String,
    /// Current user (if authenticated).
    pub user: Option<String>,
    /// Active app count.
    pub active_apps: usize,
    /// Notification count.
    pub notifications: usize,
}

/// Get shell status.
#[tauri::command]
pub fn shell_status() -> ShellStatus {
    // Stub — will wire to nexcore_shell::Shell state
    ShellStatus {
        state: "running".into(),
        user: Some("matthew".into()),
        active_apps: 0,
        notifications: 0,
    }
}

/// List registered apps in the shell.
#[tauri::command]
pub fn shell_list_apps() -> Vec<AppInfo> {
    // Stub — will wire to nexcore_shell::AppRegistry
    vec![
        AppInfo {
            id: "terminal".into(),
            name: "Terminal".into(),
            state: "running".into(),
            icon: None,
        },
        AppInfo {
            id: "observatory".into(),
            name: "Observatory".into(),
            state: "stopped".into(),
            icon: None,
        },
    ]
}

/// Launch an app by ID.
#[tauri::command]
pub fn shell_launch_app(app_id: String) -> Result<AppInfo, String> {
    Ok(AppInfo {
        id: app_id.clone(),
        name: app_id,
        state: "running".into(),
        icon: None,
    })
}

/// Get command palette suggestions for a query.
#[tauri::command]
pub fn shell_command_palette(query: String) -> Vec<PaletteEntry> {
    let _query = query;
    // Stub — will wire to nexcore_shell::CommandPalette
    vec![]
}

/// Command palette entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    /// Display label.
    pub label: String,
    /// Action to perform.
    pub action: String,
    /// Source of this entry (app, system, mcp).
    pub source: String,
}
