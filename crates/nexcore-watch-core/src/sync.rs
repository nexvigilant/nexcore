#![allow(dead_code)]
//! Background state synchronization for watch ↔ phone/cloud.
//!
//! ## Primitive Grounding
//! - σ (Sequence): sync cycle → fetch → merge → persist
//! - π (Persistence): durable state across sync intervals
//! - ν (Frequency): periodic sync timer
//! - ∂ (Boundary): network availability check
//! - ς (State): sync lifecycle (Idle → Syncing → Done | Failed)
//!
//! ## Tier: T2-C (σ + π + ν + ∂ + ς)
//!
//! ## Grammar: Type-3 (regular)
//! Sync state machine is a finite automaton:
//! Idle →ν Syncing →∂ (Done | Failed) →ν Idle

use serde::{Deserialize, Serialize};

/// Sync lifecycle state.
///
/// ## Primitive: ς (State)
/// ## Tier: T1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    /// No sync in progress — ∅ (Void)
    Idle,
    /// Sync in progress — σ (Sequence) executing
    Syncing,
    /// Last sync succeeded — ∃ (Existence) confirmed
    Done,
    /// Last sync failed — ∅ (Void) result
    Failed,
}

/// Sync configuration.
///
/// ## Primitive: ν (Frequency) + N (Quantity)
/// ## Tier: T2-P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync interval in seconds — ν (Frequency)
    pub interval_secs: u64,
    /// Maximum retry attempts — N (Quantity)
    pub max_retries: u32,
    /// Sync endpoint URL — λ (Location)
    pub endpoint: Option<String>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: 300, // 5 minutes — battery-conscious
            max_retries: 3,
            endpoint: None,
        }
    }
}

/// Background sync manager.
///
/// Replaces Android WorkManager with a pure-Rust sync loop.
/// Uses `std::thread` + looper timer instead of JVM scheduling.
///
/// ## Primitive: σ (Sequence) + ν (Frequency) + π (Persistence)
/// ## Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManager {
    /// Current sync state — ς (State)
    state: SyncState,
    /// Configuration — ν + N
    config: SyncConfig,
    /// Last successful sync timestamp (epoch ms) — ν (Frequency)
    last_sync_ms: u64,
    /// Consecutive failure count — N (Quantity)
    failure_count: u32,
    /// Total syncs completed — N (Quantity)
    total_syncs: u64,
}

impl SyncManager {
    /// Create a new sync manager with default config.
    ///
    /// ## Primitive: ∅ (Void) → ς (State)
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: SyncState::Idle,
            config: SyncConfig::default(),
            last_sync_ms: 0,
            failure_count: 0,
            total_syncs: 0,
        }
    }

    /// Create with custom config.
    ///
    /// ## Primitive: μ (Mapping)
    #[must_use]
    pub fn with_config(config: SyncConfig) -> Self {
        Self {
            state: SyncState::Idle,
            config,
            last_sync_ms: 0,
            failure_count: 0,
            total_syncs: 0,
        }
    }

    /// Check if sync is due based on elapsed time.
    ///
    /// ## Primitive: κ (Comparison) + ν (Frequency)
    /// ## Tier: T2-P
    #[must_use]
    pub fn is_sync_due(&self, current_time_ms: u64) -> bool {
        if self.state == SyncState::Syncing {
            return false; // Already syncing
        }
        if self.config.endpoint.is_none() {
            return false; // No endpoint configured
        }
        let elapsed_ms = current_time_ms.saturating_sub(self.last_sync_ms);
        elapsed_ms >= self.config.interval_secs * 1000
    }

    /// Begin a sync cycle.
    ///
    /// ## Primitive: ς (State) — Idle → Syncing
    pub fn begin_sync(&mut self) {
        self.state = SyncState::Syncing;
    }

    /// Record sync success.
    ///
    /// ## Primitive: ς (State) — Syncing → Done, reset failures
    pub fn sync_succeeded(&mut self, current_time_ms: u64) {
        self.state = SyncState::Done;
        self.last_sync_ms = current_time_ms;
        self.failure_count = 0;
        self.total_syncs += 1;
    }

    /// Record sync failure.
    ///
    /// ## Primitive: ς (State) — Syncing → Failed, increment counter
    pub fn sync_failed(&mut self) {
        self.state = SyncState::Failed;
        self.failure_count += 1;
    }

    /// Reset to idle state.
    ///
    /// ## Primitive: ς (State) — * → Idle
    pub fn reset(&mut self) {
        self.state = SyncState::Idle;
    }

    /// Whether retries are exhausted.
    ///
    /// ## Primitive: κ (Comparison) + ∂ (Boundary)
    #[must_use]
    pub fn retries_exhausted(&self) -> bool {
        self.failure_count >= self.config.max_retries
    }

    /// Current state.
    #[must_use]
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Last sync timestamp.
    #[must_use]
    pub fn last_sync_ms(&self) -> u64 {
        self.last_sync_ms
    }

    /// Total successful syncs.
    #[must_use]
    pub fn total_syncs(&self) -> u64 {
        self.total_syncs
    }

    /// Consecutive failure count — N (Quantity).
    #[must_use]
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_idle() {
        let sync = SyncManager::new();
        assert_eq!(sync.state(), SyncState::Idle);
        assert_eq!(sync.last_sync_ms(), 0);
        assert_eq!(sync.total_syncs(), 0);
    }

    #[test]
    fn sync_not_due_without_endpoint() {
        let sync = SyncManager::new();
        assert!(!sync.is_sync_due(999_999_999));
    }

    #[test]
    fn sync_due_after_interval() {
        let config = SyncConfig {
            interval_secs: 60,
            max_retries: 3,
            endpoint: Some("https://nexvigilant.com/api/sync".to_string()),
        };
        let sync = SyncManager::with_config(config);
        // 61 seconds elapsed → sync due
        assert!(sync.is_sync_due(61_000));
    }

    #[test]
    fn sync_not_due_during_active_sync() {
        let config = SyncConfig {
            interval_secs: 60,
            max_retries: 3,
            endpoint: Some("https://nexvigilant.com/api/sync".to_string()),
        };
        let mut sync = SyncManager::with_config(config);
        sync.begin_sync();
        assert!(!sync.is_sync_due(999_999));
    }

    #[test]
    fn sync_success_resets_failures() {
        let mut sync = SyncManager::new();
        sync.begin_sync();
        sync.sync_failed();
        sync.sync_failed();
        assert_eq!(sync.failure_count, 2);
        sync.begin_sync();
        sync.sync_succeeded(1000);
        assert_eq!(sync.failure_count, 0);
        assert_eq!(sync.total_syncs(), 1);
        assert_eq!(sync.last_sync_ms(), 1000);
    }

    #[test]
    fn retries_exhausted_at_max() {
        let config = SyncConfig {
            interval_secs: 60,
            max_retries: 2,
            endpoint: None,
        };
        let mut sync = SyncManager::with_config(config);
        assert!(!sync.retries_exhausted());
        sync.sync_failed();
        assert!(!sync.retries_exhausted());
        sync.sync_failed();
        assert!(sync.retries_exhausted());
    }

    #[test]
    fn default_config_values() {
        let config = SyncConfig::default();
        assert_eq!(config.interval_secs, 300);
        assert_eq!(config.max_retries, 3);
        assert!(config.endpoint.is_none());
    }
}
