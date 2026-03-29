//! Brain subsystem integration for cytokine bus state persistence.
//!
//! Provides typed artifact persistence for `BusStats` so the cytokine
//! bus can persist signal distribution metrics across sessions.
//!
//! ## T1 Grounding
//!
//! - `persist_bus_state` → π (persistence) + → (causality: bus → brain)
//! - `restore_bus_state` → ∃ (existence check) + ς (state restoration)

use crate::BusStats;
use nexcore_brain::typed_artifact::TypedArtifact;
use nexcore_brain::{BrainSession, Result};

/// Artifact name for cytokine bus state snapshots.
const ARTIFACT_NAME: &str = "cytokine-snapshot.json";

/// The typed artifact handle for cytokine bus state.
fn artifact() -> TypedArtifact<BusStats> {
    TypedArtifact::new(ARTIFACT_NAME)
}

/// Persist the current bus statistics to a brain artifact.
///
/// Serializes the `BusStats` to JSON and saves it as a `Custom` artifact
/// in the given brain session.
///
/// # Errors
///
/// Returns an error if serialization or artifact persistence fails.
pub fn persist_bus_state(stats: &BusStats, session: &BrainSession) -> Result<()> {
    artifact().save(session, stats)
}

/// Restore bus statistics from a brain artifact.
///
/// Returns `Ok(None)` if no prior snapshot exists (first session).
///
/// # Errors
///
/// Returns an error if deserialization or session access fails.
pub fn restore_bus_state(session: &BrainSession) -> Result<Option<BusStats>> {
    artifact().load(session)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn make_test_session(dir: &std::path::Path) -> BrainSession {
        std::fs::create_dir_all(dir).unwrap();
        BrainSession {
            id: "test-session".to_string(),
            created_at: nexcore_chrono::DateTime::now(),
            project: None,
            git_commit: None,
            session_dir: dir.to_path_buf(),
        }
    }

    #[test]
    fn test_round_trip_empty_stats() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let stats = BusStats::default();
        persist_bus_state(&stats, &session).unwrap();

        let restored = restore_bus_state(&session).unwrap();
        assert!(restored.is_some());
        let restored = restored.unwrap();
        assert_eq!(restored.signals_emitted, 0);
        assert_eq!(restored.signals_delivered, 0);
    }

    #[test]
    fn test_round_trip_with_data() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let mut by_family = HashMap::new();
        by_family.insert("IL-1".to_string(), 15);
        by_family.insert("TNF-α".to_string(), 3);
        by_family.insert("IFN-γ".to_string(), 7);

        let mut by_severity = HashMap::new();
        by_severity.insert("high".to_string(), 10);
        by_severity.insert("critical".to_string(), 5);

        let stats = BusStats {
            signals_emitted: 25,
            signals_delivered: 22,
            signals_dropped: 3,
            cascades_triggered: 4,
            by_family,
            by_severity,
        };

        persist_bus_state(&stats, &session).unwrap();
        let restored = restore_bus_state(&session).unwrap().unwrap();

        assert_eq!(restored.signals_emitted, 25);
        assert_eq!(restored.signals_delivered, 22);
        assert_eq!(restored.signals_dropped, 3);
        assert_eq!(restored.cascades_triggered, 4);
        assert_eq!(restored.by_family.len(), 3);
        assert_eq!(restored.by_family.get("IL-1"), Some(&15));
        assert_eq!(restored.by_family.get("TNF-α"), Some(&3));
        assert_eq!(restored.by_severity.get("critical"), Some(&5));
    }

    #[test]
    fn test_restore_no_prior_state() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let result = restore_bus_state(&session).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_overwrite_preserves_latest() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let stats1 = BusStats {
            signals_emitted: 10,
            ..Default::default()
        };
        persist_bus_state(&stats1, &session).unwrap();

        let stats2 = BusStats {
            signals_emitted: 50,
            ..Default::default()
        };
        persist_bus_state(&stats2, &session).unwrap();

        let restored = restore_bus_state(&session).unwrap().unwrap();
        assert_eq!(restored.signals_emitted, 50);
    }
}
