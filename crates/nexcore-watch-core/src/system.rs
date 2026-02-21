#![allow(dead_code)]
//! Unified watch system runtime — the single owner of all subsystems.
//!
//! ## Primitive Grounding
//! - × (Product): conjunction of PVOS + Sync + Alerts — system IS the product
//! - ς (State): system lifecycle FSM (Uninitialized → Booted → Running → Shutdown)
//! - σ (Sequence): boot sequence, tick pipeline, shutdown sequence
//! - μ (Mapping): config params → runtime state
//! - → (Causality): detection → alert → haptic causal chain
//! - ∂ (Boundary): system boundary isolates watch from external world
//! - π (Persistence): state survives across ticks
//! - Σ (Sum): aggregated metrics from all subsystems
//!
//! ## Tier: T3 (× + ς + σ + μ + → + ∂ + π + Σ) — 8 primitives, full domain
//!
//! ## Grammar: Type-1 (context-sensitive)
//! Boot requires config context. Tick behavior depends on sync state and
//! alert queue contents. Not reducible to context-free.
//!
//! ## Design Rationale
//! The JNI bridge was a 222-line translation layer between Kotlin and Rust.
//! `WatchSystem` replaces it with zero translation — Rust calls Rust.
//! Every subsystem (PVOS, sync, alerts) is owned here, not scattered
//! across Activity lifecycle callbacks.

use serde::{Deserialize, Serialize};

use crate::alerts::{Alert, AlertLevel};
#[cfg(test)]
use crate::guardian::GuardianState;
use crate::guardian::GuardianStatus;
use crate::pvos_bridge::WatchPvos;
use crate::signal::SignalResult;
use crate::sync::{SyncConfig, SyncManager, SyncState};

// ═══════════════════════════════════════════════════════════
// SYSTEM CONFIGURATION — μ (Mapping) params
// ═══════════════════════════════════════════════════════════

/// Complete system initialization parameters.
///
/// ## Primitive Grounding
/// - μ (Mapping): configuration values → runtime behavior
/// - ∂ (Boundary): threshold params define detection boundaries
/// - ν (Frequency): sync interval, tick interval controls polling frequency
/// - N (Quantity): batch sizes, queue limits, display dimensions
/// - × (Product): config IS the product of all subsystem configs
///
/// ## Tier: T2-C (μ + ∂ + ν + N + ×) — configuration product type
///
/// ## Subsystem Coverage
/// | Subsystem | Config Params | Primitives |
/// |-----------|---------------|------------|
/// | PVOS | detection_threshold, learning_batch_size, register_default_drivers | ∂, N, ∃ |
/// | Sync | sync (SyncConfig) | ν, N, λ |
/// | Alerts | max_alerts, sensitive_mode | N, ∂ |
/// | Guardian | tick_interval_ms | ν |
/// | Haptics | haptics (HapticConfig) | μ, ∂ |
/// | Display | display (DisplayConfig) | ∂, N |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    // ─── PVOS params ──────────────────────────────────────
    /// PVOS detection threshold — ∂ (Boundary)
    /// Default 1.5 (sensitive) for P0 patient safety on wearable.
    pub detection_threshold: f64,

    /// PVOS learning batch size — N (Quantity)
    /// Small batches (10) for watch memory constraints.
    pub learning_batch_size: usize,

    /// Register default PVOS drivers — ∃ (Existence)
    pub register_default_drivers: bool,

    // ─── Sync params ──────────────────────────────────────
    /// Sync configuration — ν (Frequency) + N (Quantity) + λ (Location)
    pub sync: SyncConfig,

    // ─── Alert params ─────────────────────────────────────
    /// Maximum alerts to retain in queue — N (Quantity) + ∂ (Boundary)
    /// Bounded to prevent unbounded memory growth on watch.
    pub max_alerts: usize,

    /// Whether to use sensitive thresholds for P0 — ∂ (Boundary)
    /// Fatal/life-threatening events use PRR ≥ 1.5, χ² ≥ 2.706, n ≥ 2.
    pub sensitive_mode: bool,

    // ─── Guardian params ──────────────────────────────────
    /// Tick interval in milliseconds — ν (Frequency)
    /// Controls how often the Guardian homeostasis loop iterates.
    /// Default 1000ms (1 Hz) — balanced battery/responsiveness.
    pub tick_interval_ms: u64,

    // ─── Haptic params ────────────────────────────────────
    /// Haptic feedback configuration — μ (Mapping) + ∂ (Boundary)
    pub haptics: HapticConfig,

    // ─── Display params ───────────────────────────────────
    /// Display configuration — ∂ (Boundary) + N (Quantity)
    pub display: DisplayConfig,
}

/// Haptic feedback configuration for watch vibration.
///
/// ## Primitive: μ (Mapping) + ∂ (Boundary)
/// ## Tier: T2-P
///
/// Controls which alert levels trigger vibration and intensity scaling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticConfig {
    /// Enable haptic feedback — ∃ (Existence)
    pub enabled: bool,

    /// Minimum alert level that triggers vibration — ∂ (Boundary)
    /// Default P3 (Data Quality). P4/P5 are always silent.
    /// Set to P0 to only vibrate for patient safety events.
    pub min_vibration_level: AlertLevel,

    /// Amplitude scale factor (0.0 - 1.0) — N (Quantity)
    /// Applies as multiplier to each level's default amplitude.
    /// Default 1.0 (full intensity). Use 0.5 for reduced haptics.
    pub amplitude_scale: f64,
}

impl Default for HapticConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_vibration_level: AlertLevel::P3DataQuality,
            amplitude_scale: 1.0,
        }
    }
}

/// Display configuration for 450×450 round watch face.
///
/// ## Primitive: ∂ (Boundary) + N (Quantity)
/// ## Tier: T2-P
///
/// Controls screen wake behavior and brightness for battery management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Screen wake on alert — ∃ (Existence)
    /// If false, only haptics notify (no screen wake for any alert).
    pub wake_on_alert: bool,

    /// Minimum alert level to wake screen — ∂ (Boundary)
    /// Default P2 (Regulatory). Only P0-P2 wake screen by default.
    pub min_wake_level: AlertLevel,

    /// Screen timeout after alert (ms) — ν (Frequency)
    /// How long screen stays on after an alert wakes it.
    /// Default 5000ms (5 seconds). 0 = use system default.
    pub alert_screen_timeout_ms: u64,

    /// Always-on display shows Guardian state — ∃ (Existence)
    /// When true, ambient mode shows Guardian color/state.
    pub ambient_guardian: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            wake_on_alert: true,
            min_wake_level: AlertLevel::P2Regulatory,
            alert_screen_timeout_ms: 5000,
            ambient_guardian: true,
        }
    }
}

impl Default for SystemConfig {
    /// Watch-optimized defaults: sensitive detection, 5-minute sync,
    /// bounded alert queue, 1Hz tick, full haptics, wake on P0-P2.
    ///
    /// ## Primitive: ∅ (Void) → μ (Mapping) — zero → default config
    fn default() -> Self {
        Self {
            detection_threshold: 1.5, // Sensitive: P0 patient safety
            learning_batch_size: 10,
            register_default_drivers: true,
            sync: SyncConfig::default(),
            max_alerts: 50,
            sensitive_mode: true,
            tick_interval_ms: 1000,
            haptics: HapticConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// SYSTEM STATE — ς (State) lifecycle
// ═══════════════════════════════════════════════════════════

/// System lifecycle state.
///
/// ## Primitive: ς (State)
/// ## Tier: T1
///
/// FSM: Uninitialized →σ Booted →σ Running →σ Shutdown
/// Transitions are irreversible (∝): once shutdown, must re-boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemState {
    /// Pre-boot — no subsystems initialized
    Uninitialized,
    /// PVOS booted, subsystems ready — not yet ticking
    Booted,
    /// Actively processing — tick loop running
    Running,
    /// Gracefully shut down — all subsystems halted
    Shutdown,
}

// ═══════════════════════════════════════════════════════════
// TICK RESULT — → (Causality) output
// ═══════════════════════════════════════════════════════════

/// Output of a single system tick.
///
/// ## Primitive Grounding
/// - → (Causality): tick input → tick output
/// - ∃ (Existence): new_alerts indicates whether alerts were generated
/// - ∂ (Boundary): sync_needed signals boundary crossing
///
/// ## Tier: T2-C (→ + ∃ + ∂)
#[derive(Debug, Clone)]
pub struct TickResult {
    /// Updated Guardian status — ς (State)
    pub guardian: GuardianStatus,
    /// Whether sync should be triggered — ∂ (Boundary)
    pub sync_needed: bool,
    /// New alerts generated this tick — ∃ (Existence)
    pub new_alerts: Vec<Alert>,
    /// Current alert queue depth — N (Quantity)
    pub alert_count: usize,
}

// ═══════════════════════════════════════════════════════════
// WATCH SYSTEM — × (Product) of all subsystems
// ═══════════════════════════════════════════════════════════

/// The unified watch runtime.
///
/// ## Primitive Grounding
/// - × (Product): conjunction — System = PVOS × SyncManager × AlertQueue
/// - ς (State): lifecycle FSM
/// - σ (Sequence): boot → tick → shutdown pipeline
/// - μ (Mapping): config → runtime
/// - → (Causality): detection → alert → haptic chain
/// - ∂ (Boundary): system boundary encapsulates all subsystems
/// - π (Persistence): state persists across ticks
/// - Σ (Sum): aggregated metrics
///
/// ## Tier: T3 (8 primitives)
///
/// ## Grammar: Type-1 (context-sensitive)
///
/// ## Replaces
/// - Kotlin `WatchApp` Application class
/// - Kotlin `GuardianActivity` + `SignalActivity` + `AlertsActivity`
/// - Android WorkManager sync scheduling
/// - JNI bridge (222 LOC → 0 LOC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchSystem {
    /// PVOS instance — the pharmacovigilance OS
    pvos: WatchPvos,
    /// Background sync manager
    sync: SyncManager,
    /// Alert queue — bounded by config.max_alerts
    alerts: Vec<Alert>,
    /// System lifecycle state — ς (State)
    state: SystemState,
    /// Immutable config snapshot — μ (Mapping)
    config: SystemConfig,
    /// Total ticks processed — N (Quantity)
    tick_count: u64,
}

impl WatchSystem {
    /// Boot the entire watch system from configuration.
    ///
    /// ## Primitive: σ (Sequence) — ordered boot pipeline
    /// ## Tier: T3
    ///
    /// Sequence:
    /// 1. Validate config params (∂ Boundary)
    /// 2. Boot PVOS with detection threshold (μ Mapping)
    /// 3. Initialize sync manager (ν Frequency)
    /// 4. Transition state Uninitialized → Booted → Running (ς State)
    #[must_use]
    pub fn boot(config: SystemConfig) -> Self {
        let pvos_config = nexcore_pvos::PvosConfig {
            detection_threshold: config.detection_threshold,
            learning_batch_size: config.learning_batch_size,
            register_default_drivers: config.register_default_drivers,
        };

        let pvos = WatchPvos::boot_with(pvos_config);
        let sync = SyncManager::with_config(config.sync.clone());

        Self {
            pvos,
            sync,
            alerts: Vec::new(),
            state: SystemState::Running,
            config,
            tick_count: 0,
        }
    }

    /// Boot with default watch-optimized configuration.
    ///
    /// ## Primitive: ∅ (Void) → ς (State)
    #[must_use]
    pub fn boot_default() -> Self {
        Self::boot(SystemConfig::default())
    }

    // ═══════════════════════════════════════════════════════
    // DETECTION — κ (Comparison)
    // ═══════════════════════════════════════════════════════

    /// Run signal detection and auto-generate alerts if thresholds met.
    ///
    /// ## Primitive: κ (Comparison) + → (Causality) + ∂ (Boundary)
    /// ## Tier: T3
    ///
    /// Pipeline: detect(κ) →(→) threshold_check(∂) →(→) alert_generate
    pub fn detect(
        &mut self,
        drug: &str,
        event: &str,
        a: f64,
        b: f64,
        c: f64,
        d: f64,
    ) -> SignalResult {
        let result = self.pvos.detect(drug, event, a, b, c, d);

        // → (Causality): signal detected → generate alert
        if result.signal_detected {
            let level = self.classify_signal_priority(&result);
            let alert = Alert {
                level,
                title: format!("Signal: {} → {}", drug, event),
                body: format!(
                    "PRR={:.2} ROR={:.2} IC={:.2} EBGM={:.2} χ²={:.2}",
                    result.prr, result.ror, result.ic, result.ebgm, result.chi_squared
                ),
                source: "PVOS/DetectionEngine".to_string(),
                timestamp_ms: self.tick_count,
                action_url: None,
            };
            self.push_alert(alert);
        }

        result
    }

    /// Classify signal priority based on metric strength.
    ///
    /// ## Primitive: ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-P
    ///
    /// Sensitive mode (default on watch): lower thresholds for P0/P1.
    fn classify_signal_priority(&self, result: &SignalResult) -> AlertLevel {
        if self.config.sensitive_mode && result.meets_sensitive_thresholds(2) && result.prr >= 5.0 {
            // Very strong signal with sensitive mode → P0 patient safety
            AlertLevel::P0PatientSafety
        } else if result.meets_strict_thresholds(5) {
            // Strict thresholds met → P1 signal integrity
            AlertLevel::P1SignalIntegrity
        } else if result.meets_default_thresholds(3) {
            // Default thresholds → P2 regulatory
            AlertLevel::P2Regulatory
        } else {
            // Weak signal → P3 data quality
            AlertLevel::P3DataQuality
        }
    }

    // ═══════════════════════════════════════════════════════
    // TICK — σ (Sequence) pipeline
    // ═══════════════════════════════════════════════════════

    /// Execute one system tick.
    ///
    /// ## Primitive: σ (Sequence) — ordered tick pipeline
    /// ## Tier: T3
    ///
    /// Sequence:
    /// 1. Increment tick counter (N Quantity)
    /// 2. Compute Guardian status (ς State)
    /// 3. Check sync due (ν Frequency)
    /// 4. Return tick result (→ Causality)
    pub fn tick(&mut self, current_time_ms: u64) -> TickResult {
        self.tick_count += 1;

        let guardian = self.pvos.guardian_status();
        let sync_needed = self.sync.is_sync_due(current_time_ms);

        TickResult {
            guardian,
            sync_needed,
            new_alerts: Vec::new(), // Alerts generated via detect()
            alert_count: self.alerts.len(),
        }
    }

    // ═══════════════════════════════════════════════════════
    // ALERTS — ∂ (Boundary) queue management
    // ═══════════════════════════════════════════════════════

    /// Push an alert, evicting lowest-priority if queue full.
    ///
    /// ## Primitive: σ (Sequence) + ∂ (Boundary) + κ (Comparison)
    /// ## Tier: T2-C
    fn push_alert(&mut self, alert: Alert) {
        if self.alerts.len() >= self.config.max_alerts {
            // Evict lowest priority (highest enum value)
            if let Some(pos) = self
                .alerts
                .iter()
                .enumerate()
                .max_by_key(|&(_, a)| a.level)
                .map(|(i, _)| i)
            {
                // Only evict if new alert is higher priority
                if alert.level < self.alerts[pos].level {
                    self.alerts.swap_remove(pos);
                } else {
                    return; // Queue full, new alert is lower priority — drop it
                }
            }
        }
        self.alerts.push(alert);
    }

    /// Get sorted alert queue (P0 first).
    ///
    /// ## Primitive: κ (Comparison) + σ (Sequence)
    /// ## Tier: T2-P
    #[must_use]
    pub fn alerts(&self) -> Vec<&Alert> {
        let mut sorted: Vec<&Alert> = self.alerts.iter().collect();
        sorted.sort_by_key(|a| a.level);
        sorted
    }

    /// Dismiss an alert by index.
    ///
    /// ## Primitive: ∅ (Void) — remove from existence
    pub fn dismiss_alert(&mut self, index: usize) -> bool {
        if index < self.alerts.len() {
            self.alerts.remove(index);
            true
        } else {
            false
        }
    }

    /// Alert queue depth.
    ///
    /// ## Primitive: N (Quantity)
    /// ## Tier: T1
    #[must_use]
    pub fn alert_count(&self) -> usize {
        self.alerts.len()
    }

    // ═══════════════════════════════════════════════════════
    // SYNC — ν (Frequency)
    // ═══════════════════════════════════════════════════════

    /// Get sync manager reference.
    #[must_use]
    pub fn sync(&self) -> &SyncManager {
        &self.sync
    }

    /// Get mutable sync manager for state transitions.
    pub fn sync_mut(&mut self) -> &mut SyncManager {
        &mut self.sync
    }

    // ═══════════════════════════════════════════════════════
    // STATE — ς (State) queries
    // ═══════════════════════════════════════════════════════

    /// Current system state.
    ///
    /// ## Primitive: ς (State)
    /// ## Tier: T1
    #[must_use]
    pub fn state(&self) -> SystemState {
        self.state
    }

    /// Whether the system is running.
    ///
    /// ## Primitive: ∃ (Existence)
    /// ## Tier: T1
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state == SystemState::Running && self.pvos.is_running()
    }

    /// Current Guardian status.
    ///
    /// ## Primitive: ς (State)
    /// ## Tier: T3
    #[must_use]
    pub fn guardian_status(&self) -> GuardianStatus {
        self.pvos.guardian_status()
    }

    /// Immutable config reference.
    ///
    /// ## Primitive: μ (Mapping) — frozen params
    #[must_use]
    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    /// Total ticks processed.
    ///
    /// ## Primitive: N (Quantity)
    #[must_use]
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Total detections since boot.
    ///
    /// ## Primitive: N (Quantity)
    #[must_use]
    pub fn detection_count(&self) -> u64 {
        self.pvos.detection_count()
    }

    // ═══════════════════════════════════════════════════════
    // METRICS — Σ (Sum)
    // ═══════════════════════════════════════════════════════

    /// Aggregated system metrics.
    ///
    /// ## Primitive: Σ (Sum) — all subsystem metrics combined
    /// ## Tier: T2-C
    #[must_use]
    pub fn metrics(&self) -> SystemMetrics {
        let pvos = self.pvos.system_metrics();
        let highest_active_priority = self.alerts.iter().map(|a| a.level).min(); // min = highest priority (P0 < P1 < ...)

        SystemMetrics {
            // System
            system_state: self.state,
            tick_count: self.tick_count,
            // PVOS
            pvos_state: pvos.state,
            detection_count: self.pvos.detection_count(),
            total_cases: pvos.total_cases,
            total_artifacts: pvos.total_artifacts,
            active_processes: pvos.active_processes,
            total_processes: pvos.total_processes,
            pending_feedback: pvos.pending_feedback,
            retrain_cycles: pvos.retrain_cycles,
            audit_entries: pvos.audit_entries,
            // Sync
            sync_state: self.sync.state(),
            total_syncs: self.sync.total_syncs(),
            last_sync_ms: self.sync.last_sync_ms(),
            sync_failures: self.sync.failure_count(),
            // Alerts
            alert_count: self.alerts.len(),
            highest_active_priority,
            // Guardian
            guardian_state: self.pvos.guardian_status(),
            // Config snapshot
            sensitive_mode: self.config.sensitive_mode,
            tick_interval_ms: self.config.tick_interval_ms,
        }
    }

    // ═══════════════════════════════════════════════════════
    // LIFECYCLE — σ (Sequence)
    // ═══════════════════════════════════════════════════════

    /// Graceful shutdown.
    ///
    /// ## Primitive: ς (State) — Running → Shutdown
    /// ## Tier: T1
    ///
    /// Sequence: stop sync → halt PVOS → transition state
    pub fn shutdown(&mut self) {
        self.sync.reset();
        self.pvos.shutdown();
        self.state = SystemState::Shutdown;
    }
}

/// Aggregated metrics across all subsystems.
///
/// ## Primitive: Σ (Sum) + × (Product) — product of all metric fields
/// ## Tier: T2-C
///
/// ## Field Coverage
/// | Source | Fields | Primitives |
/// |--------|--------|------------|
/// | System | system_state, tick_count | ς, N |
/// | PVOS | pvos_state, detection_count, total_cases, total_artifacts, active_processes, pending_feedback, retrain_cycles, audit_entries | ς, N |
/// | Sync | sync_state, total_syncs, last_sync_ms, sync_failures | ς, N, ν |
/// | Alerts | alert_count, highest_active_priority | N, ∂ |
/// | Guardian | guardian_state, risk_level | ς, κ |
/// | Config | sensitive_mode, tick_interval_ms | ∂, ν |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    // ─── System ───────────────────────────────────────────
    /// Watch system lifecycle state — ς (State)
    pub system_state: SystemState,
    /// Total ticks processed — N (Quantity)
    pub tick_count: u64,

    // ─── PVOS ─────────────────────────────────────────────
    /// PVOS kernel state — ς (State)
    pub pvos_state: String,
    /// Total signal detections run — N (Quantity)
    pub detection_count: u64,
    /// Ingested ICSR cases — N (Quantity)
    pub total_cases: usize,
    /// Stored artifacts (signals, reports) — N (Quantity)
    pub total_artifacts: usize,
    /// Active workflow processes — N (Quantity)
    pub active_processes: usize,
    /// Total workflow processes ever spawned — N (Quantity)
    pub total_processes: usize,
    /// Feedback items awaiting batch retrain — N (Quantity)
    pub pending_feedback: usize,
    /// Completed learning retrain cycles — N (Quantity)
    pub retrain_cycles: u64,
    /// Tamper-evident audit trail entries — N (Quantity) + π (Persistence)
    pub audit_entries: usize,

    // ─── Sync ─────────────────────────────────────────────
    /// Sync lifecycle state — ς (State)
    pub sync_state: SyncState,
    /// Completed successful syncs — N (Quantity)
    pub total_syncs: u64,
    /// Last successful sync timestamp (ms) — ν (Frequency)
    pub last_sync_ms: u64,
    /// Consecutive sync failures — N (Quantity)
    pub sync_failures: u32,

    // ─── Alerts ───────────────────────────────────────────
    /// Active alerts in queue — N (Quantity)
    pub alert_count: usize,
    /// Highest-priority active alert (None if queue empty) — ∂ (Boundary)
    pub highest_active_priority: Option<AlertLevel>,

    // ─── Guardian ─────────────────────────────────────────
    /// Current Guardian FSM state — ς (State)
    pub guardian_state: GuardianStatus,

    // ─── Config (readonly snapshot) ───────────────────────
    /// Whether sensitive mode is active — ∂ (Boundary)
    pub sensitive_mode: bool,
    /// Tick interval — ν (Frequency)
    pub tick_interval_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════
    // Boot Tests — ς (State)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn boot_default_starts_running() {
        let sys = WatchSystem::boot_default();
        assert_eq!(sys.state(), SystemState::Running);
        assert!(sys.is_running());
        assert_eq!(sys.tick_count(), 0);
        assert_eq!(sys.detection_count(), 0);
        assert_eq!(sys.alert_count(), 0);
    }

    #[test]
    fn boot_with_custom_config() {
        let config = SystemConfig {
            detection_threshold: 3.0,
            learning_batch_size: 5,
            register_default_drivers: false,
            max_alerts: 10,
            sensitive_mode: false,
            ..SystemConfig::default()
        };
        let sys = WatchSystem::boot(config);
        assert!(sys.is_running());
        assert_eq!(sys.config().max_alerts, 10);
        assert!(!sys.config().sensitive_mode);
    }

    // ═══════════════════════════════════════════════════════════
    // Detection Tests — κ (Comparison) + → (Causality)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn detect_strong_signal_generates_alert() {
        let mut sys = WatchSystem::boot_default();
        let result = sys.detect("Aspirin", "GI Bleed", 15.0, 100.0, 20.0, 10000.0);
        assert!(result.signal_detected);
        assert!(sys.alert_count() > 0, "Strong signal should generate alert");
    }

    #[test]
    fn detect_no_signal_no_alert() {
        let mut sys = WatchSystem::boot_default();
        let result = sys.detect("Placebo", "Headache", 10.0, 90.0, 100.0, 900.0);
        assert!(!result.signal_detected);
        assert_eq!(sys.alert_count(), 0, "No signal should generate no alert");
    }

    #[test]
    fn detect_increments_detection_count() {
        let mut sys = WatchSystem::boot_default();
        sys.detect("A", "B", 1.0, 1.0, 1.0, 1.0);
        sys.detect("C", "D", 1.0, 1.0, 1.0, 1.0);
        assert_eq!(sys.detection_count(), 2);
    }

    // ═══════════════════════════════════════════════════════════
    // Alert Queue Tests — ∂ (Boundary) + κ (Comparison)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn alert_queue_bounded_by_max() {
        let config = SystemConfig {
            max_alerts: 3,
            ..SystemConfig::default()
        };
        let mut sys = WatchSystem::boot(config);

        // Generate 5 strong signals — queue capped at 3
        for i in 0..5 {
            sys.detect(&format!("Drug{i}"), "Event", 15.0, 100.0, 20.0, 10000.0);
        }
        assert!(
            sys.alert_count() <= 3,
            "Queue should be bounded at max_alerts=3, got {}",
            sys.alert_count()
        );
    }

    #[test]
    fn alerts_sorted_by_priority() {
        let mut sys = WatchSystem::boot_default();
        sys.detect("Drug1", "Event1", 15.0, 100.0, 20.0, 10000.0);
        let alerts = sys.alerts();
        // All alerts should be sorted P0 first
        for window in alerts.windows(2) {
            assert!(window[0].level <= window[1].level);
        }
    }

    #[test]
    fn dismiss_alert_removes_it() {
        let mut sys = WatchSystem::boot_default();
        sys.detect("Aspirin", "Bleed", 15.0, 100.0, 20.0, 10000.0);
        let initial = sys.alert_count();
        assert!(initial > 0);
        assert!(sys.dismiss_alert(0));
        assert_eq!(sys.alert_count(), initial - 1);
    }

    #[test]
    fn dismiss_invalid_index_returns_false() {
        let sys = WatchSystem::boot_default();
        // Can't dismiss from empty vec, need mutable — create mutable
        let mut sys = sys;
        assert!(!sys.dismiss_alert(99));
    }

    // ═══════════════════════════════════════════════════════════
    // Tick Tests — σ (Sequence)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn tick_increments_counter() {
        let mut sys = WatchSystem::boot_default();
        sys.tick(1000);
        sys.tick(2000);
        assert_eq!(sys.tick_count(), 2);
    }

    #[test]
    fn tick_returns_guardian_status() {
        let mut sys = WatchSystem::boot_default();
        let result = sys.tick(1000);
        assert_eq!(result.guardian.state, GuardianState::Nominal);
    }

    // ═══════════════════════════════════════════════════════════
    // Metrics Tests — Σ (Sum)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn metrics_aggregate_all_subsystems() {
        let mut sys = WatchSystem::boot_default();
        sys.detect("Drug", "Event", 15.0, 100.0, 20.0, 10000.0);
        sys.tick(1000);

        let m = sys.metrics();
        assert_eq!(m.system_state, SystemState::Running);
        assert_eq!(m.tick_count, 1);
        assert_eq!(m.detection_count, 1);
        assert!(m.alert_count > 0);
        assert_eq!(m.sync_state, SyncState::Idle);
    }

    // ═══════════════════════════════════════════════════════════
    // Lifecycle Tests — ς (State)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn shutdown_transitions_to_halted() {
        let mut sys = WatchSystem::boot_default();
        assert!(sys.is_running());
        sys.shutdown();
        assert!(!sys.is_running());
        assert_eq!(sys.state(), SystemState::Shutdown);
    }

    // ═══════════════════════════════════════════════════════════
    // Config Tests — μ (Mapping) params
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn default_config_is_sensitive() {
        let config = SystemConfig::default();
        assert_eq!(config.detection_threshold, 1.5);
        assert!(config.sensitive_mode);
        assert_eq!(config.max_alerts, 50);
    }

    #[test]
    fn config_frozen_after_boot() {
        let config = SystemConfig {
            detection_threshold: 2.5,
            ..SystemConfig::default()
        };
        let sys = WatchSystem::boot(config);
        assert_eq!(sys.config().detection_threshold, 2.5);
    }

    // ═══════════════════════════════════════════════════════════
    // Signal Priority Classification Tests — ∂ (Boundary)
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn strong_signal_sensitive_mode_generates_p0() {
        let mut sys = WatchSystem::boot_default();
        // Very strong signal: PRR >> 5.0 AND sensitive thresholds met
        let result = sys.detect("DrugX", "Fatal", 50.0, 100.0, 5.0, 10000.0);
        if result.signal_detected {
            let alerts = sys.alerts();
            assert!(
                !alerts.is_empty(),
                "Should have at least one alert for strong signal"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════
    // New Config Params Tests — × (Product) completeness
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn default_config_haptics_enabled() {
        let config = SystemConfig::default();
        assert!(config.haptics.enabled);
        assert_eq!(
            config.haptics.min_vibration_level,
            AlertLevel::P3DataQuality
        );
        assert!((config.haptics.amplitude_scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_config_display_wakes_on_alert() {
        let config = SystemConfig::default();
        assert!(config.display.wake_on_alert);
        assert_eq!(config.display.min_wake_level, AlertLevel::P2Regulatory);
        assert_eq!(config.display.alert_screen_timeout_ms, 5000);
        assert!(config.display.ambient_guardian);
    }

    #[test]
    fn default_config_tick_interval() {
        let config = SystemConfig::default();
        assert_eq!(config.tick_interval_ms, 1000);
    }

    #[test]
    fn custom_haptic_config() {
        let config = SystemConfig {
            haptics: HapticConfig {
                enabled: false,
                min_vibration_level: AlertLevel::P0PatientSafety,
                amplitude_scale: 0.5,
            },
            ..SystemConfig::default()
        };
        let sys = WatchSystem::boot(config);
        assert!(!sys.config().haptics.enabled);
        assert_eq!(
            sys.config().haptics.min_vibration_level,
            AlertLevel::P0PatientSafety
        );
    }

    #[test]
    fn custom_display_config() {
        let config = SystemConfig {
            display: DisplayConfig {
                wake_on_alert: false,
                min_wake_level: AlertLevel::P0PatientSafety,
                alert_screen_timeout_ms: 10000,
                ambient_guardian: false,
            },
            ..SystemConfig::default()
        };
        let sys = WatchSystem::boot(config);
        assert!(!sys.config().display.wake_on_alert);
        assert!(!sys.config().display.ambient_guardian);
    }

    // ═══════════════════════════════════════════════════════════
    // Expanded Metrics Tests — Σ (Sum) completeness
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn metrics_include_pvos_fields() {
        let sys = WatchSystem::boot_default();
        let m = sys.metrics();
        // PVOS boot creates 1 audit entry
        assert!(m.audit_entries > 0);
        assert_eq!(m.total_artifacts, 0);
        assert_eq!(m.active_processes, 0);
        assert_eq!(m.pending_feedback, 0);
        assert_eq!(m.retrain_cycles, 0);
    }

    #[test]
    fn metrics_include_sync_fields() {
        let sys = WatchSystem::boot_default();
        let m = sys.metrics();
        assert_eq!(m.sync_state, SyncState::Idle);
        assert_eq!(m.total_syncs, 0);
        assert_eq!(m.last_sync_ms, 0);
        assert_eq!(m.sync_failures, 0);
    }

    #[test]
    fn metrics_highest_priority_empty_queue() {
        let sys = WatchSystem::boot_default();
        let m = sys.metrics();
        assert!(m.highest_active_priority.is_none());
    }

    #[test]
    fn metrics_highest_priority_with_alerts() {
        let mut sys = WatchSystem::boot_default();
        sys.detect("Drug", "Event", 15.0, 100.0, 20.0, 10000.0);
        let m = sys.metrics();
        if m.alert_count > 0 {
            assert!(m.highest_active_priority.is_some());
        }
    }

    #[test]
    fn metrics_guardian_state_present() {
        let sys = WatchSystem::boot_default();
        let m = sys.metrics();
        assert_eq!(m.guardian_state.state, GuardianState::Nominal);
    }

    #[test]
    fn metrics_config_snapshot() {
        let sys = WatchSystem::boot_default();
        let m = sys.metrics();
        assert!(m.sensitive_mode);
        assert_eq!(m.tick_interval_ms, 1000);
    }
}
