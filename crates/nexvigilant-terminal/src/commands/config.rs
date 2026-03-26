// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Persistent configuration for NexVigilant Terminal.
//!
//! Stores user preferences, recent commands, custom workflow buttons,
//! and investigation bookmarks across terminal restarts.
//!
//! Config file: `{app_data_dir}/nvt-config.json`
//!
//! ## Primitive Grounding
//!
//! `π(Persistence: config file) + ς(State: user prefs) + ∂(Boundary: app data dir)`

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// Config file name within the Tauri app data directory.
const CONFIG_FILENAME: &str = "nvt-config.json";

/// Maximum recent commands to persist.
const MAX_RECENT: usize = 20;

/// User configuration persisted across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvtConfig {
    /// Terminal version (for migration detection).
    #[serde(default = "default_version")]
    pub version: String,

    /// Whether epistemic coloring is enabled.
    #[serde(default = "default_true")]
    pub epistemic_colors: bool,

    /// Last active terminal mode.
    #[serde(default)]
    pub last_mode: String,

    /// Recent commands (most recent first).
    #[serde(default)]
    pub recent_commands: Vec<String>,

    /// Custom workflow buttons defined by the user.
    #[serde(default)]
    pub custom_buttons: Vec<CustomButton>,

    /// Bookmarked investigations (drug + event pairs).
    #[serde(default)]
    pub bookmarks: Vec<InvestigationBookmark>,

    /// Station base URL override (empty = production).
    #[serde(default)]
    pub station_url: String,
}

/// A user-defined workflow button.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomButton {
    /// Display label.
    pub label: String,
    /// Command to inject into PTY or Station tool to call.
    pub command: String,
    /// Button accent color (hex).
    #[serde(default = "default_color")]
    pub color: String,
    /// Whether this calls Station directly (true) or injects into PTY (false).
    #[serde(default)]
    pub station_direct: bool,
}

/// A saved investigation for quick recall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationBookmark {
    /// Drug name.
    pub drug: String,
    /// Adverse event (optional).
    #[serde(default)]
    pub event: String,
    /// User notes.
    #[serde(default)]
    pub notes: String,
    /// When bookmarked (ISO 8601).
    #[serde(default)]
    pub created: String,
}

fn default_version() -> String {
    "0.2.0".to_string()
}
fn default_true() -> bool {
    true
}
fn default_color() -> String {
    "#60a5fa".to_string()
}

impl Default for NvtConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            epistemic_colors: true,
            last_mode: "Normal".into(),
            recent_commands: Vec::new(),
            custom_buttons: Vec::new(),
            bookmarks: Vec::new(),
            station_url: String::new(),
        }
    }
}

impl NvtConfig {
    /// Add a command to the recent list (deduplicates, caps at MAX_RECENT).
    pub fn add_recent(&mut self, cmd: &str) {
        self.recent_commands.retain(|c| c != cmd);
        self.recent_commands.insert(0, cmd.to_string());
        self.recent_commands.truncate(MAX_RECENT);
    }
}

/// Resolve the config file path using Tauri's app data directory.
fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;

    // Ensure directory exists
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create app data dir: {e}"))?;
    }

    Ok(data_dir.join(CONFIG_FILENAME))
}

// ── Tauri Commands ──────────────────────────────────────────────

/// Load the persisted config. Returns defaults if no config file exists.
#[tauri::command]
pub async fn config_load(app: tauri::AppHandle) -> Result<NvtConfig, String> {
    let path = config_path(&app)?;

    if !path.exists() {
        return Ok(NvtConfig::default());
    }

    let contents =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {e}"))?;

    serde_json::from_str::<NvtConfig>(&contents).map_err(|e| {
        tracing::warn!(error = %e, "Config parse failed, returning defaults");
        format!("Config parse error (using defaults): {e}")
    })
}

/// Save the config to disk.
#[tauri::command]
pub async fn config_save(app: tauri::AppHandle, config: NvtConfig) -> Result<(), String> {
    let path = config_path(&app)?;

    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(&path, json).map_err(|e| format!("Failed to write config: {e}"))?;

    tracing::debug!(path = %path.display(), "Config saved");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = NvtConfig::default();
        assert_eq!(config.version, "0.2.0");
        assert!(config.epistemic_colors);
        assert!(config.recent_commands.is_empty());
    }

    #[test]
    fn add_recent_deduplicates() {
        let mut config = NvtConfig::default();
        config.add_recent("/signal metformin");
        config.add_recent("/health");
        config.add_recent("/signal metformin"); // duplicate
        assert_eq!(config.recent_commands.len(), 2);
        assert_eq!(config.recent_commands[0], "/signal metformin");
    }

    #[test]
    fn add_recent_caps_at_max() {
        let mut config = NvtConfig::default();
        for i in 0..25 {
            config.add_recent(&format!("cmd-{i}"));
        }
        assert_eq!(config.recent_commands.len(), MAX_RECENT);
    }

    #[test]
    fn config_serializes_roundtrip() {
        let mut config = NvtConfig::default();
        config.add_recent("/test");
        config.custom_buttons.push(CustomButton {
            label: "My Button".into(),
            command: "/my-workflow".into(),
            color: "#ff0000".into(),
            station_direct: false,
        });

        let json = serde_json::to_string(&config).unwrap();
        let parsed: NvtConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.recent_commands.len(), 1);
        assert_eq!(parsed.custom_buttons.len(), 1);
    }
}
