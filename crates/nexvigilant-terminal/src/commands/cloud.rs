// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Tauri IPC commands for NexCloud agent observability.
//!
//! Wired to `nexcloud::supervisor::registry::ServiceRegistry` for live
//! process state from the CloudSupervisor.

use nexcloud::supervisor::registry::{ProcessState, ServiceRegistry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Tauri-managed cloud state.
pub struct CloudState {
    /// Live service registry from nexcloud.
    pub registry: Arc<ServiceRegistry>,
}

impl Default for CloudState {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudState {
    /// Create a new cloud state with default NexVigilant services registered.
    #[must_use]
    pub fn new() -> Self {
        let registry = Arc::new(ServiceRegistry::new());
        // Register the core NexVigilant services
        registry.register("nexcore-mcp".into(), 0);
        registry.register("nexcore-api".into(), 3030);
        registry.register("nexvigilant-station".into(), 0);
        registry.register("nexcore-brain".into(), 0);
        Self { registry }
    }
}

/// Summary of a cloud-managed service process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name.
    pub name: String,
    /// Assigned port.
    pub port: u16,
    /// Current process state.
    pub state: String,
    /// Health status from last check.
    pub health: String,
    /// Uptime in seconds (0 if not running).
    pub uptime_secs: u64,
    /// Restart count.
    pub restarts: u32,
}

/// Aggregate cloud status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudOverview {
    /// Total registered services.
    pub total_services: usize,
    /// Services currently healthy.
    pub healthy: usize,
    /// Services degraded or unhealthy.
    pub unhealthy: usize,
    /// Platform name from manifest.
    pub platform: String,
}

/// List all services managed by the CloudSupervisor.
#[tauri::command]
pub fn cloud_list_services(state: tauri::State<'_, CloudState>) -> Vec<ServiceInfo> {
    state
        .registry
        .snapshot()
        .into_iter()
        .map(|r| ServiceInfo {
            name: r.name,
            port: r.port,
            state: r.state.to_string(),
            health: match r.state {
                ProcessState::Healthy => "healthy".into(),
                ProcessState::Unhealthy | ProcessState::Failed => "unhealthy".into(),
                _ => "unknown".into(),
            },
            uptime_secs: r.started_at.map_or(0, |s| {
                let elapsed = nexcore_chrono::DateTime::now().timestamp() - s.timestamp();
                elapsed.unsigned_abs()
            }),
            restarts: r.restarts,
        })
        .collect()
}

/// Get aggregate cloud health overview.
#[tauri::command]
pub fn cloud_overview(state: tauri::State<'_, CloudState>) -> CloudOverview {
    let snap = state.registry.snapshot();
    let healthy = snap
        .iter()
        .filter(|r| r.state == ProcessState::Healthy)
        .count();
    let unhealthy = snap
        .iter()
        .filter(|r| matches!(r.state, ProcessState::Unhealthy | ProcessState::Failed))
        .count();
    CloudOverview {
        total_services: snap.len(),
        healthy,
        unhealthy,
        platform: "nexcore".into(),
    }
}

/// Get recent cloud events.
#[tauri::command]
pub fn cloud_events(limit: Option<usize>) -> Vec<CloudEvent> {
    let _limit = limit.unwrap_or(50);
    // EventBus integration deferred — requires async Tauri command + tokio receiver
    vec![]
}

/// A cloud platform event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Event type.
    pub event_type: String,
    /// Service name.
    pub service: String,
    /// Human-readable message.
    pub message: String,
}
