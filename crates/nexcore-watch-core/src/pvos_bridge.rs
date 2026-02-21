#![allow(dead_code)]
//! PVOS bridge for wearable — boots the full Pharmacovigilance OS on-device.
//!
//! ## Primitive Grounding
//! - μ (Mapping): PVOS is a μ-machine — maps raw data → structured vigilance
//! - ς (State): system state lifecycle (Booting → Running → Halted)
//! - κ (Comparison): detection engine for disproportionality
//! - → (Causality): event routing from detection → alerts
//! - π (Persistence): durable state across watch restarts
//!
//! ## Tier: T3 (μ + ς + κ + → + π) — full domain OS facade
//!
//! ## Grammar: Type-1 (context-sensitive)
//! Boot sequence requires configuration context; detection requires
//! algorithm selection context beyond the data itself.

use nexcore_pvos::{Pvos, PvosConfig, PvosState};
use serde::{Deserialize, Serialize};

use crate::guardian::{GuardianState, GuardianStatus, RiskLevel};
use crate::signal::SignalResult;

/// Watch-optimized PVOS facade.
///
/// Wraps `Pvos::boot()` with watch-specific defaults and provides
/// simplified APIs for the constrained wearable environment.
///
/// ## Primitive: μ (Mapping) — domain OS → watch-friendly API
/// ## Tier: T3
#[derive(Clone, Serialize, Deserialize)]
pub struct WatchPvos {
    /// The underlying PVOS instance — full 15-layer OS
    pvos: Pvos,
    /// Watch-specific: last known Guardian state
    guardian_state: GuardianState,
    /// Watch-specific: detection count since boot
    detection_count: u64,
}

impl std::fmt::Debug for WatchPvos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatchPvos")
            .field("pvos_state", &self.pvos.state())
            .field("guardian_state", &self.guardian_state)
            .field("detection_count", &self.detection_count)
            .finish()
    }
}

impl WatchPvos {
    /// Boot PVOS with watch-optimized configuration.
    ///
    /// ## Primitive: ς (State) — transition ∅ → Booting → Running
    /// ## Tier: T3
    ///
    /// Uses sensitive detection threshold (1.5) for P0 patient safety —
    /// lower thresholds catch weak signals early on wearable.
    #[must_use]
    pub fn boot() -> Self {
        let config = PvosConfig {
            detection_threshold: 1.5, // Sensitive: P0 patient safety
            learning_batch_size: 10,  // Small batches for watch memory
            register_default_drivers: true,
        };

        Self {
            pvos: Pvos::boot(config),
            guardian_state: GuardianState::Nominal,
            detection_count: 0,
        }
    }

    /// Boot with custom configuration.
    ///
    /// ## Primitive: ς (State) — configurable boot
    /// ## Tier: T3
    #[must_use]
    pub fn boot_with(config: PvosConfig) -> Self {
        Self {
            pvos: Pvos::boot(config),
            guardian_state: GuardianState::Nominal,
            detection_count: 0,
        }
    }

    /// Run signal detection using the local computation engine.
    ///
    /// ## Primitive: κ (Comparison) — all 5 disproportionality metrics
    /// ## Tier: T2-C
    ///
    /// Uses `SignalResult::compute_all` for on-device detection,
    /// independent of network connectivity.
    pub fn detect(
        &mut self,
        drug: &str,
        event: &str,
        a: f64,
        b: f64,
        c: f64,
        d: f64,
    ) -> SignalResult {
        self.detection_count += 1;
        SignalResult::compute_all(drug, event, a, b, c, d)
    }

    /// Get current Guardian status for watch display.
    ///
    /// ## Primitive: ς (State) + μ (Mapping)
    /// ## Tier: T3
    #[must_use]
    pub fn guardian_status(&self) -> GuardianStatus {
        let metrics = self.pvos.metrics();

        // Map PVOS metrics → Guardian risk level
        let risk_level = if metrics.total_detections > 10 {
            RiskLevel::Critical
        } else if metrics.total_detections > 5 {
            RiskLevel::High
        } else if metrics.total_detections > 0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        // Map risk → state
        let state = match risk_level {
            RiskLevel::Critical => GuardianState::Critical,
            RiskLevel::High => GuardianState::Alert,
            RiskLevel::Medium => GuardianState::Elevated,
            RiskLevel::Low => GuardianState::Nominal,
        };

        GuardianStatus {
            state,
            iteration: self.detection_count,
            active_sensors: 1,   // Watch is one sensor
            active_actuators: 1, // Haptic is one actuator
            risk_level,
            last_tick_ms: 0, // Updated by sync layer
        }
    }

    /// Check if PVOS is running.
    ///
    /// ## Primitive: ∃ (Existence) — is the OS alive?
    /// ## Tier: T1
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.pvos.state() == PvosState::Running
    }

    /// Get total detection count since boot.
    ///
    /// ## Primitive: N (Quantity)
    /// ## Tier: T1
    #[must_use]
    pub fn detection_count(&self) -> u64 {
        self.detection_count
    }

    /// Graceful shutdown.
    ///
    /// ## Primitive: ς (State) — Running → Halted
    /// ## Tier: T1
    pub fn shutdown(&mut self) {
        self.pvos.shutdown();
    }

    /// Access underlying PVOS kernel metrics.
    ///
    /// ## Primitive: Σ (Sum) — aggregated system metrics
    /// ## Tier: T2-P
    #[must_use]
    pub fn system_metrics(&self) -> PvosSystemMetrics {
        let m = self.pvos.metrics();
        PvosSystemMetrics {
            state: format!("{:?}", m.state),
            total_cases: m.total_cases,
            total_artifacts: m.total_artifacts,
            total_detections: m.total_detections,
            active_processes: m.active_processes,
            total_processes: m.total_processes,
            pending_feedback: m.pending_feedback,
            retrain_cycles: m.retrain_cycles,
            audit_entries: m.audit_entries,
        }
    }
}

/// PVOS metrics surfaced for watch system aggregation.
///
/// ## Primitive: Σ (Sum) + μ (Mapping)
/// ## Tier: T2-C
///
/// Maps 1:1 to `PvosMetrics` from nexcore-pvos kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PvosSystemMetrics {
    /// System state as string — ς (State)
    pub state: String,
    /// Total ingested cases — N (Quantity)
    pub total_cases: usize,
    /// Stored artifacts (signals, reports) — N (Quantity)
    pub total_artifacts: usize,
    /// Total detections run — N (Quantity)
    pub total_detections: u64,
    /// Active workflow processes — N (Quantity)
    pub active_processes: usize,
    /// Total workflow processes ever spawned — N (Quantity)
    pub total_processes: usize,
    /// Feedback items awaiting batch retrain — N (Quantity)
    pub pending_feedback: usize,
    /// Completed learning retrain cycles — N (Quantity)
    pub retrain_cycles: u64,
    /// Audit log entries — N (Quantity) + π (Persistence)
    pub audit_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_starts_running() {
        let pvos = WatchPvos::boot();
        assert!(pvos.is_running());
        assert_eq!(pvos.detection_count(), 0);
    }

    #[test]
    fn detect_increments_counter() {
        let mut pvos = WatchPvos::boot();
        let _ = pvos.detect("Aspirin", "GI Bleed", 15.0, 100.0, 20.0, 10000.0);
        assert_eq!(pvos.detection_count(), 1);
        let _ = pvos.detect("Ibuprofen", "Headache", 10.0, 90.0, 100.0, 900.0);
        assert_eq!(pvos.detection_count(), 2);
    }

    #[test]
    fn detect_returns_valid_signal() {
        let mut pvos = WatchPvos::boot();
        let result = pvos.detect("Aspirin", "GI Bleed", 15.0, 100.0, 20.0, 10000.0);
        assert!(result.signal_detected, "Strong signal should be detected");
        assert!(result.prr > 2.0);
    }

    #[test]
    fn guardian_status_nominal_at_boot() {
        let pvos = WatchPvos::boot();
        let status = pvos.guardian_status();
        assert_eq!(status.state, GuardianState::Nominal);
        assert_eq!(status.risk_level, RiskLevel::Low);
    }

    #[test]
    fn shutdown_halts_system() {
        let mut pvos = WatchPvos::boot();
        assert!(pvos.is_running());
        pvos.shutdown();
        assert!(!pvos.is_running());
    }

    #[test]
    fn system_metrics_at_boot() {
        let pvos = WatchPvos::boot();
        let metrics = pvos.system_metrics();
        assert_eq!(metrics.total_cases, 0);
        assert_eq!(metrics.total_detections, 0);
        assert!(metrics.audit_entries > 0, "Boot should create audit entry");
    }

    #[test]
    fn boot_with_custom_config() {
        let config = PvosConfig {
            detection_threshold: 3.0,
            learning_batch_size: 5,
            register_default_drivers: false,
        };
        let pvos = WatchPvos::boot_with(config);
        assert!(pvos.is_running());
    }
}
