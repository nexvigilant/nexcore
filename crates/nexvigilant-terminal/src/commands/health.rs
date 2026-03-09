// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for χ (chi) session health monitoring.
//!
//! Bridges [`nexcore_terminal::chi_monitor::ChiMonitor`] to the frontend
//! via Tauri events. The frontend `chi-indicator.ts` listens for
//! `session-health` events emitted by the background polling task.

use std::sync::Mutex;
use std::time::Duration;

use nexcore_terminal::chi_monitor::ChiMonitor;
use nexcore_terminal::health::SessionHealth;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};

/// Default sliding window for χ computation (60 seconds).
const DEFAULT_WINDOW_SECS: u64 = 60;

/// Default polling interval for health event emission (500ms).
const DEFAULT_POLL_MS: u64 = 500;

/// Tauri-managed health monitoring state.
pub struct HealthState {
    /// The χ monitor instance (single monitor for the active session).
    pub monitor: Mutex<ChiMonitor>,
    /// Whether background polling is active.
    pub polling: Mutex<bool>,
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthState {
    /// Create a new health state with the default 60-second window.
    #[must_use]
    pub fn new() -> Self {
        Self {
            monitor: Mutex::new(ChiMonitor::new(Duration::from_secs(DEFAULT_WINDOW_SECS))),
            polling: Mutex::new(false),
        }
    }
}

/// Health snapshot returned to frontend via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Branching ratio χ = output/input.
    pub chi: f64,
    /// Accretion torque (normalised).
    pub tau_acc: f64,
    /// Wind torque (normalised).
    pub tau_wind: f64,
    /// Disk torque (backpressure).
    pub tau_disk: f64,
    /// Whether torques are in equilibrium.
    pub equilibrium: bool,
    /// Health band name.
    pub band: String,
}

impl From<&SessionHealth> for HealthSnapshot {
    fn from(h: &SessionHealth) -> Self {
        Self {
            chi: h.chi,
            tau_acc: h.tau_acc,
            tau_wind: h.tau_wind,
            tau_disk: h.tau_disk,
            equilibrium: h.equilibrium,
            band: format!("{:?}", h.band),
        }
    }
}

/// Record an input event (keystroke, command, API call arriving).
#[tauri::command]
pub fn health_record_input(state: tauri::State<'_, HealthState>) {
    if let Ok(mut monitor) = state.monitor.lock() {
        monitor.record_input();
    }
}

/// Record an output event (line rendered, response chunk emitted).
#[tauri::command]
pub fn health_record_output(state: tauri::State<'_, HealthState>) {
    if let Ok(mut monitor) = state.monitor.lock() {
        monitor.record_output();
    }
}

/// Get current health snapshot (synchronous, single poll).
#[tauri::command]
pub fn health_get(state: tauri::State<'_, HealthState>) -> HealthSnapshot {
    if let Ok(mut monitor) = state.monitor.lock() {
        HealthSnapshot::from(monitor.compute())
    } else {
        // Fallback: return SpinningUp if lock poisoned
        HealthSnapshot {
            chi: 0.0,
            tau_acc: 1.0,
            tau_wind: 0.0,
            tau_disk: 0.0,
            equilibrium: false,
            band: "SpinningUp".into(),
        }
    }
}

/// Start background health polling — emits `session-health` events at 500ms intervals.
///
/// Returns `true` if polling was started, `false` if already running.
#[tauri::command]
pub fn health_start_polling(app: tauri::AppHandle, state: tauri::State<'_, HealthState>) -> bool {
    // Guard: only one polling task at a time
    if let Ok(mut polling) = state.polling.lock() {
        if *polling {
            return false;
        }
        *polling = true;
    } else {
        return false;
    }

    // Spawn a background task that polls and emits
    let interval = Duration::from_millis(DEFAULT_POLL_MS);

    // We cannot move the Tauri State into the task, so we clone the AppHandle
    // and access HealthState via app.state() inside the task.
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;

            let snapshot = {
                let health_state: tauri::State<'_, HealthState> = app.state();
                let is_polling = health_state.polling.lock().map(|p| *p).unwrap_or(false);

                if !is_polling {
                    break;
                }

                health_state
                    .monitor
                    .lock()
                    .map(|mut monitor: std::sync::MutexGuard<'_, ChiMonitor>| {
                        HealthSnapshot::from(monitor.compute())
                    })
                    .ok()
            };

            if let Some(snap) = snapshot {
                // Emit to all windows — chi-indicator.ts listens for this
                let _ = app.emit("session-health", &snap);
            }
        }
    });

    true
}

/// Stop background health polling.
#[tauri::command]
pub fn health_stop_polling(state: tauri::State<'_, HealthState>) {
    if let Ok(mut polling) = state.polling.lock() {
        *polling = false;
    }
}
