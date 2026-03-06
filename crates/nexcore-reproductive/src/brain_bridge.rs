//! Brain subsystem integration for reproductive health persistence.
//!
//! Provides typed artifact persistence for `ReproductiveHealth` so the
//! reproductive system can persist CI/CD health metrics across sessions.
//!
//! ## T1 Grounding
//!
//! - `persist_reproductive_health` → π (persistence) + → (causality: reproductive → brain)
//! - `restore_reproductive_health` → ∃ (existence check) + ς (state restoration)

use crate::ReproductiveHealth;
use nexcore_brain::typed_artifact::TypedArtifact;
use nexcore_brain::{BrainSession, Result};

/// Artifact name for reproductive health snapshots.
const ARTIFACT_NAME: &str = "reproductive-snapshot.json";

/// The typed artifact handle for reproductive health.
fn artifact() -> TypedArtifact<ReproductiveHealth> {
    TypedArtifact::new(ARTIFACT_NAME)
}

/// Persist the current reproductive health to a brain artifact.
///
/// Serializes the `ReproductiveHealth` to JSON and saves it as a `Custom`
/// artifact in the given brain session.
///
/// # Errors
///
/// Returns an error if serialization or artifact persistence fails.
pub fn persist_reproductive_health(
    health: &ReproductiveHealth,
    session: &BrainSession,
) -> Result<()> {
    artifact().save(session, health)
}

/// Restore reproductive health from a brain artifact.
///
/// Returns `Ok(None)` if no prior snapshot exists (first session).
///
/// # Errors
///
/// Returns an error if deserialization or session access fails.
pub fn restore_reproductive_health(session: &BrainSession) -> Result<Option<ReproductiveHealth>> {
    artifact().load(session)
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn sample_health() -> ReproductiveHealth {
        ReproductiveHealth {
            pipeline_passing: true,
            deployment_success_rate: 0.95,
            trimester_gate_count: 4,
            last_birth_success: true,
            mutation_rate: 0.15,
        }
    }

    #[test]
    fn test_round_trip() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let health = sample_health();
        persist_reproductive_health(&health, &session).unwrap();

        let restored = restore_reproductive_health(&session).unwrap().unwrap();
        assert!(restored.pipeline_passing);
        assert!((restored.deployment_success_rate - 0.95).abs() < f64::EPSILON);
        assert_eq!(restored.trimester_gate_count, 4);
        assert!(restored.last_birth_success);
        assert!((restored.mutation_rate - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn test_restore_no_prior_state() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let result = restore_reproductive_health(&session).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_overwrite_preserves_latest() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let health1 = sample_health();
        persist_reproductive_health(&health1, &session).unwrap();

        let health2 = ReproductiveHealth {
            pipeline_passing: false,
            deployment_success_rate: 0.60,
            last_birth_success: false,
            ..sample_health()
        };
        persist_reproductive_health(&health2, &session).unwrap();

        let restored = restore_reproductive_health(&session).unwrap().unwrap();
        assert!(!restored.pipeline_passing);
        assert!((restored.deployment_success_rate - 0.60).abs() < f64::EPSILON);
        assert!(!restored.last_birth_success);
    }
}
