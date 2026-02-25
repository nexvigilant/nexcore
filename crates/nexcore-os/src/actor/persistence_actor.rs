// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Persistence actor -- holds OS state snapshots in-memory.
//!
//! Since `StatePersistence::save/load` require a `Storage` trait object,
//! this actor holds snapshots in-memory. The kernel bridges to actual PAL storage.
//!
//! ## Primitive Grounding
//!
//! | Concept           | Primitives | Explanation                                      |
//! |-------------------|------------|--------------------------------------------------|
//! | PersistenceActor  | ς + ∂ + π  | Encapsulated state, actor boundary, persistence  |
//! | handle()          | Σ + μ      | Sum-type dispatch, each variant maps to persist op |

use crate::actor::{Actor, ActorMessage};
use crate::persistence::OsStateSnapshot;

/// Actor managing OS state snapshots in-memory.
///
/// Holds the most recent snapshot and tracks whether the previous session
/// had an unclean shutdown (for crash recovery detection).
pub struct PersistenceActor {
    last_snapshot: Option<OsStateSnapshot>,
    had_unclean_shutdown: bool,
}

impl PersistenceActor {
    /// Create a new persistence actor with no prior state.
    pub fn new() -> Self {
        Self {
            last_snapshot: None,
            had_unclean_shutdown: false,
        }
    }

    /// Create with a pre-loaded snapshot (for crash recovery).
    pub fn with_snapshot(snapshot: OsStateSnapshot) -> Self {
        let unclean = !snapshot.clean_shutdown;
        Self {
            last_snapshot: Some(snapshot),
            had_unclean_shutdown: unclean,
        }
    }
}

impl Default for PersistenceActor {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for PersistenceActor {
    fn name(&self) -> &'static str {
        "persistence"
    }

    fn handle(&mut self, msg: ActorMessage) -> bool {
        match msg {
            ActorMessage::PersistSaveSnapshot(snapshot) => {
                self.last_snapshot = Some(snapshot);
                true
            }
            ActorMessage::PersistLoadSnapshot { reply_to } => {
                let _ = reply_to.send(ActorMessage::PersistSnapshotResponse(
                    self.last_snapshot.clone(),
                ));
                true
            }
            ActorMessage::PersistCrashCheck { reply_to } => {
                let _ = reply_to.send(ActorMessage::PersistCrashResponse(
                    self.had_unclean_shutdown,
                ));
                true
            }
            // Ignore messages intended for other actors.
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::spawn_actor;
    use std::sync::mpsc;
    use std::time::Duration;

    fn test_snapshot(clean: bool) -> OsStateSnapshot {
        OsStateSnapshot {
            timestamp: nexcore_chrono::DateTime::now(),
            version: "0.1.0".to_string(),
            platform: "test".to_string(),
            boot_phase: "Running".to_string(),
            services: vec![],
            tick_count: 42,
            ipc_events_emitted: 10,
            security_level: "Green".to_string(),
            clean_shutdown: clean,
        }
    }

    #[test]
    fn save_and_load_snapshot() {
        let mut handle = spawn_actor(PersistenceActor::new(), 16);

        // Save a snapshot.
        let snapshot = test_snapshot(true);
        handle.send(ActorMessage::PersistSaveSnapshot(snapshot));
        std::thread::sleep(Duration::from_millis(50));

        // Load it back.
        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::PersistLoadSnapshot { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::PersistSnapshotResponse(Some(loaded))) => {
                assert_eq!(loaded.tick_count, 42);
                assert!(loaded.clean_shutdown);
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected PersistSnapshotResponse(Some(...))"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn crash_check_clean() {
        let mut handle = spawn_actor(PersistenceActor::new(), 16);

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::PersistCrashCheck { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::PersistCrashResponse(unclean)) => {
                assert!(!unclean, "fresh actor should report clean");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected PersistCrashResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn crash_check_unclean() {
        let snapshot = test_snapshot(false);
        let mut handle = spawn_actor(PersistenceActor::with_snapshot(snapshot), 16);

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::PersistCrashCheck { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::PersistCrashResponse(unclean)) => {
                assert!(unclean, "should report unclean shutdown");
            }
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected PersistCrashResponse"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }

    #[test]
    fn load_without_save_returns_none() {
        let mut handle = spawn_actor(PersistenceActor::new(), 16);

        let (tx, rx) = mpsc::sync_channel(1);
        handle.send(ActorMessage::PersistLoadSnapshot { reply_to: tx });

        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(ActorMessage::PersistSnapshotResponse(None)) => {}
            Err(e) => panic!("recv failed: {e}"),
            _ => panic!("expected PersistSnapshotResponse(None)"),
        }

        handle.send(ActorMessage::Shutdown);
        handle.join();
    }
}
