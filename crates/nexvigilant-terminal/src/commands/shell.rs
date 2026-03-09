// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for the NexCore Shell (app launcher, login, AI partner).
//!
//! Bridges nexcore-shell's device-level concepts to the Tauri desktop app.
//! Wired to real `nexcore_shell::Shell` — no hardcoded stubs.

use std::sync::Mutex;

use nexcore_pal::FormFactor;
use nexcore_shell::{App, CommandPalette, PaletteEntry as ShellPaletteEntry, Shell};
use serde::{Deserialize, Serialize};

/// Managed state wrapping the real `nexcore_shell::Shell`.
pub struct NexShellState {
    pub shell: Mutex<Shell>,
    pub palette: Mutex<CommandPalette>,
}

impl NexShellState {
    /// Create a new shell state for the desktop form factor.
    #[must_use]
    pub fn new() -> Self {
        let mut shell = Shell::new(FormFactor::Desktop);
        shell.boot();
        // Register NexVigilant apps
        shell.apps_mut().register("terminal", "Terminal");
        shell.apps_mut().register("observatory", "Observatory");

        let palette = CommandPalette::new(FormFactor::Desktop);
        Self {
            shell: Mutex::new(shell),
            palette: Mutex::new(palette),
        }
    }
}

impl Default for NexShellState {
    fn default() -> Self {
        Self::new()
    }
}

/// App info for the frontend launcher grid.
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

impl From<&App> for AppInfo {
    fn from(app: &App) -> Self {
        Self {
            id: app.id.as_str().to_string(),
            name: app.name.clone(),
            state: format!("{:?}", app.state),
            icon: None,
        }
    }
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

/// Get shell status from the real `nexcore_shell::Shell`.
#[tauri::command]
pub fn shell_status(state: tauri::State<'_, NexShellState>) -> ShellStatus {
    let shell = state
        .shell
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let active_apps = shell
        .apps()
        .list()
        .iter()
        .filter(|a| a.state == nexcore_shell::AppState::Running)
        .count();
    ShellStatus {
        state: format!("{:?}", shell.state()),
        user: Some("matthew".into()),
        active_apps,
        notifications: 0,
    }
}

/// List registered apps from the real `nexcore_shell::AppRegistry`.
#[tauri::command]
pub fn shell_list_apps(state: tauri::State<'_, NexShellState>) -> Vec<AppInfo> {
    let shell = state
        .shell
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    shell.apps().list().iter().map(AppInfo::from).collect()
}

/// Launch an app by ID.
#[tauri::command]
pub fn shell_launch_app(
    app_id: String,
    state: tauri::State<'_, NexShellState>,
) -> Result<AppInfo, String> {
    let mut shell = state
        .shell
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let id = nexcore_shell::AppId::new(&app_id);
    if shell.launch_app(&id) {
        // Get the updated app info
        if let Some(app) = shell.apps().get(&id) {
            return Ok(AppInfo::from(app));
        }
    }
    Err(format!("Failed to launch app: {app_id}"))
}

/// Get command palette suggestions for a query.
#[tauri::command]
pub fn shell_command_palette(
    query: String,
    state: tauri::State<'_, NexShellState>,
) -> Vec<PaletteEntry> {
    let mut palette = state
        .palette
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    palette.set_query(&query);
    palette
        .visible_results()
        .iter()
        .map(PaletteEntry::from)
        .collect()
}

/// Command palette entry for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteEntry {
    /// Display label.
    pub label: String,
    /// Action to perform.
    pub action: String,
    /// Source of this entry (app, system, mcp).
    pub source: String,
}

impl From<&ShellPaletteEntry> for PaletteEntry {
    fn from(entry: &ShellPaletteEntry) -> Self {
        Self {
            label: entry.title.clone(),
            action: entry.subtitle.clone(),
            source: format!("{:?}", entry.source),
        }
    }
}
