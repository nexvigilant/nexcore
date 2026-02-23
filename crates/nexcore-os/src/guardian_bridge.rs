// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Guardian bridge — reads/writes the Guardian homeostasis state files.
//!
//! This module bridges the OS security subsystem with the Guardian
//! immune-system control loop by reading `guardian-state.json` and
//! appending signals to `guardian-signals.jsonl`.
//!
//! ## Primitive Grounding
//!
//! - π Persistence: State files persist across sessions
//! - → Causality: OS threat events cause Guardian signal emission
//! - κ Comparison: Threat level comparison for escalation
//! - ∂ Boundary: File I/O boundary with graceful fallback

use nexcore_fs::dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Guardian state as read from `guardian-state.json`.
///
/// Only the fields we need — extra fields are silently ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianState {
    /// Homeostasis tick iteration count.
    pub iteration: u64,
    /// Current maximum threat level.
    pub max_threat_level: String,
    /// Signals detected in the current/last iteration.
    pub signals_detected: u64,
    /// Actions taken in the current/last iteration.
    #[serde(default)]
    pub actions_taken: u64,
    /// Details of the last tick (None before first tick).
    pub last_tick: Option<LastTick>,
}

/// Details of the most recent homeostasis tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastTick {
    /// RFC-3339 timestamp.
    pub timestamp: String,
    /// Iteration identifier (e.g., "iter-3").
    #[serde(default)]
    pub iteration_id: String,
    /// Signals detected this tick.
    #[serde(default)]
    pub signals_detected: u64,
    /// Actions taken this tick.
    #[serde(default)]
    pub actions_taken: u64,
    /// Tick duration in milliseconds.
    #[serde(default)]
    pub duration_ms: u64,
    /// Maximum threat level this tick.
    #[serde(default)]
    pub max_threat_level: String,
}

/// A signal entry appended to `guardian-signals.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEntry {
    /// RFC-3339 timestamp.
    pub ts: String,
    /// Signal source (e.g., "nexcore-os").
    pub src: String,
    /// Severity: "Info", "Low", "Medium", "High", "Critical".
    pub sev: String,
    /// Pattern identifier (e.g., "pamp_login_failure").
    pub pat: String,
    /// Arbitrary metadata.
    pub meta: serde_json::Value,
}

/// Bridge between NexCore OS and the Guardian homeostasis engine.
///
/// Reads guardian state from the JSON file written by `guardian_homeostasis_tick`
/// MCP tool. Writes signal entries to the JSONL file read by the `guardian-gate`
/// hook for autonomous escalation.
pub struct GuardianBridge {
    /// Path to `guardian-state.json`.
    state_path: PathBuf,
    /// Path to `guardian-signals.jsonl`.
    signals_path: PathBuf,
}

impl GuardianBridge {
    /// Create a bridge with default paths (`~/.claude/hooks/state/`).
    pub fn new() -> Option<Self> {
        let home = dirs::home_dir()?;
        let state_dir = home.join(".claude").join("hooks").join("state");
        Some(Self {
            state_path: state_dir.join("guardian-state.json"),
            signals_path: state_dir.join("guardian-signals.jsonl"),
        })
    }

    /// Create a bridge with custom paths (for testing).
    pub fn with_paths(state_path: PathBuf, signals_path: PathBuf) -> Self {
        Self {
            state_path,
            signals_path,
        }
    }

    /// Read the current guardian state. Returns `None` if the file
    /// doesn't exist or can't be parsed.
    pub fn read_state(&self) -> Option<GuardianState> {
        let data = fs::read_to_string(&self.state_path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Get the current threat level as a string.
    /// Returns "Unknown" if state can't be read.
    pub fn threat_level(&self) -> String {
        self.read_state()
            .map_or_else(|| "Unknown".to_string(), |s| s.max_threat_level)
    }

    /// Emit a signal to `guardian-signals.jsonl`.
    ///
    /// Appends one JSONL line. Silently fails if the file can't be opened
    /// (Guardian is optional — the OS must not crash if it's unavailable).
    pub fn emit_signal(
        &self,
        severity: &str,
        pattern: &str,
        source: &str,
        meta: serde_json::Value,
    ) {
        let entry = SignalEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            src: source.to_string(),
            sev: severity.to_string(),
            pat: pattern.to_string(),
            meta,
        };

        if let Ok(line) = serde_json::to_string(&entry) {
            if let Ok(mut file) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.signals_path)
            {
                let _ = writeln!(file, "{line}");
            }
        }
    }

    /// Emit an OS security event as a guardian signal.
    ///
    /// Maps OS security levels to guardian severity strings.
    pub fn emit_threat(&self, os_severity: &str, pattern_name: &str, details: serde_json::Value) {
        let guardian_severity = match os_severity.to_lowercase().as_str() {
            "medium" => "Medium",
            "high" => "High",
            "critical" => "Critical",
            // "low" and everything else map to Info
            _ => "Info",
        };

        self.emit_signal(guardian_severity, pattern_name, "nexcore-os", details);
    }

    /// Format guardian state for display.
    pub fn format_status(&self) -> String {
        use std::fmt::Write;

        match self.read_state() {
            Some(state) => {
                let mut out = String::new();
                out.push_str("Guardian Homeostasis\n");
                out.push_str("────────────────────\n");
                let _ = writeln!(out, "Iteration:    {}", state.iteration);
                let _ = writeln!(out, "Threat level: {}", state.max_threat_level);
                let _ = writeln!(out, "Signals:      {}", state.signals_detected);
                let _ = writeln!(out, "Actions:      {}", state.actions_taken);

                if let Some(ref tick) = state.last_tick {
                    let _ = writeln!(out, "Last tick:    {}", tick.timestamp);
                    let _ = writeln!(out, "Tick ID:      {}", tick.iteration_id);
                    let _ = writeln!(out, "Duration:     {}ms", tick.duration_ms);
                } else {
                    out.push_str("Last tick:    (none)\n");
                }

                out
            }
            None => "Guardian: not connected (no state file found)\n".to_string(),
        }
    }

    /// Whether the guardian state file exists and is readable.
    pub fn is_connected(&self) -> bool {
        self.state_path.exists()
    }
}

impl Default for GuardianBridge {
    fn default() -> Self {
        Self::new().unwrap_or_else(|| Self {
            state_path: PathBuf::from("/tmp/guardian-state.json"),
            signals_path: PathBuf::from("/tmp/guardian-signals.jsonl"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_with_missing_state() {
        let bridge = GuardianBridge::with_paths(
            PathBuf::from("/tmp/nonexistent-guardian-state.json"),
            PathBuf::from("/tmp/test-guardian-signals.jsonl"),
        );
        assert!(bridge.read_state().is_none());
        assert_eq!(bridge.threat_level(), "Unknown");
        assert!(!bridge.is_connected());
    }

    #[test]
    fn format_status_disconnected() {
        let bridge = GuardianBridge::with_paths(
            PathBuf::from("/tmp/nonexistent-state.json"),
            PathBuf::from("/tmp/nonexistent-signals.jsonl"),
        );
        let status = bridge.format_status();
        assert!(status.contains("not connected"));
    }

    #[test]
    fn emit_signal_creates_file() {
        let dir = tempfile::TempDir::new();
        let dir = match dir {
            Ok(d) => d,
            Err(_) => return,
        };
        let signals_path = dir.path().join("signals.jsonl");
        let bridge = GuardianBridge::with_paths(
            PathBuf::from("/tmp/nonexistent.json"),
            signals_path.clone(),
        );

        bridge.emit_signal(
            "High",
            "test_threat",
            "nexcore-os",
            serde_json::json!({"test": true}),
        );

        let content = fs::read_to_string(&signals_path).unwrap_or_default();
        assert!(content.contains("test_threat"));
        assert!(content.contains("High"));
    }

    #[test]
    fn parse_real_state_format() {
        let json = r#"{
            "actions_taken": 2,
            "iteration": 3,
            "last_tick": {
                "actions_taken": 2,
                "duration_ms": 60,
                "iteration_id": "iter-3",
                "max_threat_level": "Low",
                "results": [],
                "signals_detected": 2,
                "timestamp": "2026-02-17T12:13:07.847723001+00:00"
            },
            "max_threat_level": "Low",
            "signals_detected": 2
        }"#;

        let state: GuardianState = serde_json::from_str(json).unwrap_or_else(|_| GuardianState {
            iteration: 0,
            max_threat_level: "Unknown".to_string(),
            signals_detected: 0,
            actions_taken: 0,
            last_tick: None,
        });

        assert_eq!(state.iteration, 3);
        assert_eq!(state.max_threat_level, "Low");
        assert_eq!(state.signals_detected, 2);
        assert!(state.last_tick.is_some());
    }
}
