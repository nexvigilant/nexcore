// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS state persistence inspired by Brain working memory.
//!
//! ## Architecture
//!
//! This is the kernel-level state persistence subsystem. The full Brain
//! system runs as a managed service — this module provides the thin,
//! sync persistence layer the OS core needs for crash recovery.
//!
//! ## Primitive Grounding
//!
//! - π Persistence: State survives reboots
//! - ς State: Snapshot of OS state at a point in time
//! - σ Sequence: Ordered boot log entries
//! - ∃ Existence: State file existence validation

use chrono::{DateTime, Utc};
use nexcore_pal::Storage;
use serde::{Deserialize, Serialize};

use crate::service::ServiceState;

/// Persisted snapshot of a service's state.
///
/// Tier: T2-C (π + ς — persisted service state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    /// Service name.
    pub name: String,
    /// Service state at snapshot time.
    pub state: String,
    /// STOS machine ID (if wired).
    pub machine_id: Option<u64>,
}

/// Persisted snapshot of the full OS state.
///
/// Tier: T3 (π + ς + σ + ∃ — full OS state persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsStateSnapshot {
    /// When the snapshot was taken.
    pub timestamp: DateTime<Utc>,
    /// OS version string.
    pub version: String,
    /// Platform name.
    pub platform: String,
    /// Boot phase at snapshot time.
    pub boot_phase: String,
    /// Service states.
    pub services: Vec<ServiceSnapshot>,
    /// Total tick count at snapshot time.
    pub tick_count: u64,
    /// Total IPC events emitted.
    pub ipc_events_emitted: u64,
    /// Security level at snapshot time.
    pub security_level: String,
    /// Whether this was a clean shutdown.
    pub clean_shutdown: bool,
}

/// OS state persistence engine.
///
/// Tier: T3 (π + ∃ — persistence + existence validation)
///
/// Saves and loads OS state snapshots via the PAL Storage trait.
/// Provides crash recovery by detecting unclean shutdown.
pub struct StatePersistence {
    /// Path to the state file.
    state_path: String,
    /// Last snapshot taken.
    last_snapshot: Option<OsStateSnapshot>,
}

impl StatePersistence {
    /// Default state file path.
    pub const STATE_FILE: &'static str = "nexcore-os/state.json";

    /// Create a new persistence engine.
    pub fn new() -> Self {
        Self {
            state_path: Self::STATE_FILE.to_string(),
            last_snapshot: None,
        }
    }

    /// Create with a custom state file path.
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            state_path: path.into(),
            last_snapshot: None,
        }
    }

    /// Save an OS state snapshot to storage.
    pub fn save<S: Storage>(
        &mut self,
        storage: &mut S,
        snapshot: &OsStateSnapshot,
    ) -> Result<(), PersistenceError> {
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| PersistenceError::Serialize(e.to_string()))?;

        storage
            .write(&self.state_path, json.as_bytes())
            .map_err(|_| PersistenceError::StorageWrite)?;

        self.last_snapshot = Some(snapshot.clone());
        Ok(())
    }

    /// Load the most recent OS state snapshot from storage.
    pub fn load<S: Storage>(&mut self, storage: &S) -> Result<OsStateSnapshot, PersistenceError> {
        if !storage.exists(&self.state_path) {
            return Err(PersistenceError::NoState);
        }

        let data = storage
            .read(&self.state_path)
            .map_err(|_| PersistenceError::StorageRead)?;

        let snapshot: OsStateSnapshot = serde_json::from_slice(&data)
            .map_err(|e| PersistenceError::Deserialize(e.to_string()))?;

        self.last_snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    /// Check if a previous state exists (for crash recovery detection).
    pub fn has_previous_state<S: Storage>(&self, storage: &S) -> bool {
        storage.exists(&self.state_path)
    }

    /// Detect if the previous shutdown was unclean.
    ///
    /// Returns `Some(snapshot)` if crash recovery is needed.
    pub fn check_crash_recovery<S: Storage>(&mut self, storage: &S) -> Option<OsStateSnapshot> {
        if let Ok(snapshot) = self.load(storage) {
            if !snapshot.clean_shutdown {
                return Some(snapshot);
            }
        }
        None
    }

    /// Delete the state file (after successful boot).
    pub fn clear<S: Storage>(&mut self, storage: &mut S) -> Result<(), PersistenceError> {
        if storage.exists(&self.state_path) {
            storage
                .delete(&self.state_path)
                .map_err(|_| PersistenceError::StorageWrite)?;
        }
        self.last_snapshot = None;
        Ok(())
    }

    /// Get the last snapshot taken.
    pub fn last_snapshot(&self) -> Option<&OsStateSnapshot> {
        self.last_snapshot.as_ref()
    }
}

impl Default for StatePersistence {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from state persistence operations.
///
/// Tier: T2-C (∅ + ∂ — absence + boundary errors)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// No previous state file found.
    NoState,
    /// Failed to serialize state.
    Serialize(String),
    /// Failed to deserialize state.
    Deserialize(String),
    /// Storage write failed.
    StorageWrite,
    /// Storage read failed.
    StorageRead,
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoState => write!(f, "No previous OS state found"),
            Self::Serialize(e) => write!(f, "Failed to serialize state: {e}"),
            Self::Deserialize(e) => write!(f, "Failed to deserialize state: {e}"),
            Self::StorageWrite => write!(f, "Failed to write state to storage"),
            Self::StorageRead => write!(f, "Failed to read state from storage"),
        }
    }
}

impl std::error::Error for PersistenceError {}

/// Create a snapshot of the current OS state.
pub fn snapshot_os_state(
    platform_name: &str,
    boot_phase: &str,
    services: &[(String, ServiceState, Option<u64>)],
    tick_count: u64,
    ipc_events: u64,
    security_level: &str,
    clean_shutdown: bool,
) -> OsStateSnapshot {
    OsStateSnapshot {
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: platform_name.to_string(),
        boot_phase: boot_phase.to_string(),
        services: services
            .iter()
            .map(|(name, state, mid)| ServiceSnapshot {
                name: name.clone(),
                state: format!("{state:?}"),
                machine_id: *mid,
            })
            .collect(),
        tick_count,
        ipc_events_emitted: ipc_events,
        security_level: security_level.to_string(),
        clean_shutdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_snapshot(clean: bool) -> OsStateSnapshot {
        snapshot_os_state(
            "test-platform",
            "Running",
            &[
                ("guardian".to_string(), ServiceState::Running, Some(1)),
                ("brain".to_string(), ServiceState::Running, Some(2)),
            ],
            100,
            50,
            "GREEN",
            clean,
        )
    }

    #[test]
    fn snapshot_creation() {
        let snap = make_test_snapshot(true);
        assert_eq!(snap.platform, "test-platform");
        assert_eq!(snap.services.len(), 2);
        assert_eq!(snap.tick_count, 100);
        assert!(snap.clean_shutdown);
    }

    #[test]
    fn snapshot_serialization() {
        let snap = make_test_snapshot(true);
        let json = serde_json::to_string(&snap);
        assert!(json.is_ok());

        if let Ok(json_str) = json {
            let deserialized: Result<OsStateSnapshot, _> = serde_json::from_str(&json_str);
            assert!(deserialized.is_ok());
        }
    }

    #[test]
    fn persistence_no_state() {
        let mut persistence = StatePersistence::new();
        // Create a mock storage that has nothing
        let storage = MockStorage::empty();
        let result = persistence.load(&storage);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e, PersistenceError::NoState);
        }
    }

    #[test]
    fn persistence_save_load_cycle() {
        let mut persistence = StatePersistence::with_path("test-state.json");
        let mut storage = MockStorage::empty();

        let snap = make_test_snapshot(true);
        let save_result = persistence.save(&mut storage, &snap);
        assert!(save_result.is_ok());

        let load_result = persistence.load(&storage);
        assert!(load_result.is_ok());

        if let Ok(loaded) = load_result {
            assert_eq!(loaded.platform, "test-platform");
            assert_eq!(loaded.tick_count, 100);
            assert!(loaded.clean_shutdown);
        }
    }

    #[test]
    fn crash_recovery_detection() {
        let mut persistence = StatePersistence::with_path("test-crash.json");
        let mut storage = MockStorage::empty();

        // Save unclean shutdown state
        let snap = make_test_snapshot(false);
        let _ = persistence.save(&mut storage, &snap);

        // Check for crash recovery
        let recovery = persistence.check_crash_recovery(&storage);
        assert!(recovery.is_some());

        if let Some(recovered) = recovery {
            assert!(!recovered.clean_shutdown);
        }
    }

    #[test]
    fn clean_shutdown_no_recovery() {
        let mut persistence = StatePersistence::with_path("test-clean.json");
        let mut storage = MockStorage::empty();

        // Save clean shutdown state
        let snap = make_test_snapshot(true);
        let _ = persistence.save(&mut storage, &snap);

        // Should NOT trigger crash recovery
        let recovery = persistence.check_crash_recovery(&storage);
        assert!(recovery.is_none());
    }

    #[test]
    fn persistence_clear() {
        let mut persistence = StatePersistence::with_path("test-clear.json");
        let mut storage = MockStorage::empty();

        let snap = make_test_snapshot(true);
        let _ = persistence.save(&mut storage, &snap);
        assert!(storage.exists("test-clear.json"));

        let clear_result = persistence.clear(&mut storage);
        assert!(clear_result.is_ok());
        assert!(!storage.exists("test-clear.json"));
    }

    /// In-memory mock storage for testing.
    struct MockStorage {
        files: std::collections::HashMap<String, Vec<u8>>,
    }

    impl MockStorage {
        fn empty() -> Self {
            Self {
                files: std::collections::HashMap::new(),
            }
        }
    }

    impl Storage for MockStorage {
        fn read(&self, path: &str) -> Result<Vec<u8>, nexcore_pal::error::StorageError> {
            self.files
                .get(path)
                .cloned()
                .ok_or(nexcore_pal::error::StorageError::NotFound)
        }

        fn write(
            &mut self,
            path: &str,
            data: &[u8],
        ) -> Result<(), nexcore_pal::error::StorageError> {
            self.files.insert(path.to_string(), data.to_vec());
            Ok(())
        }

        fn delete(&mut self, path: &str) -> Result<(), nexcore_pal::error::StorageError> {
            self.files.remove(path);
            Ok(())
        }

        fn exists(&self, path: &str) -> bool {
            self.files.contains_key(path)
        }

        fn available_bytes(&self) -> Result<u64, nexcore_pal::error::StorageError> {
            Ok(1_000_000_000)
        }

        fn total_bytes(&self) -> Result<u64, nexcore_pal::error::StorageError> {
            Ok(1_000_000_000)
        }
    }
}
