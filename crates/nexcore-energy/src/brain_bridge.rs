//! Brain subsystem integration for energy state persistence.
//!
//! Provides typed artifact persistence for `EnergyState` so the energy
//! system can persist token budget snapshots across sessions.
//!
//! ## T1 Grounding
//!
//! - `persist_energy_state` → π (persistence) + → (causality: energy → brain)
//! - `restore_energy_state` → ∃ (existence check) + ς (state restoration)

use crate::EnergyState;
use nexcore_brain::typed_artifact::TypedArtifact;
use nexcore_brain::{BrainSession, Result};

/// Artifact name for energy budget snapshots.
const ARTIFACT_NAME: &str = "energy-snapshot.json";

/// The typed artifact handle for energy state.
fn artifact() -> TypedArtifact<EnergyState> {
    TypedArtifact::new(ARTIFACT_NAME)
}

/// Persist the current energy state to a brain artifact.
///
/// Serializes the `EnergyState` to JSON and saves it as a `Custom` artifact
/// in the given brain session.
///
/// # Errors
///
/// Returns an error if serialization or artifact persistence fails.
pub fn persist_energy_state(state: &EnergyState, session: &BrainSession) -> Result<()> {
    artifact().save(session, state)
}

/// Restore energy state from a brain artifact.
///
/// Returns `Ok(None)` if no prior snapshot exists (first session).
///
/// # Errors
///
/// Returns an error if deserialization or session access fails.
pub fn restore_energy_state(session: &BrainSession) -> Result<Option<EnergyState>> {
    artifact().load(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EnergySystem, Regime, Strategy, TokenPool};
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

    fn sample_state() -> EnergyState {
        EnergyState {
            pool: TokenPool::new(100_000),
            regime: Regime::Homeostatic,
            energy_charge: 0.78,
            coupling_efficiency: 1.8,
            waste_ratio: 0.12,
            burn_rate: 0.05,
            recommended_strategy: Strategy::Sonnet,
            energy_system: EnergySystem::Oxidative,
        }
    }

    #[test]
    fn test_round_trip() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let state = sample_state();
        persist_energy_state(&state, &session).unwrap();

        let restored = restore_energy_state(&session).unwrap().unwrap();
        assert_eq!(restored.pool.t_atp, 100_000);
        assert_eq!(restored.pool.t_adp, 0);
        assert!((restored.energy_charge - 0.78).abs() < f64::EPSILON);
        assert!((restored.coupling_efficiency - 1.8).abs() < f64::EPSILON);
        assert!((restored.waste_ratio - 0.12).abs() < f64::EPSILON);
    }

    #[test]
    fn test_restore_no_prior_state() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let result = restore_energy_state(&session).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_overwrite_preserves_latest() {
        let temp = TempDir::new().unwrap();
        let session = make_test_session(&temp.path().join("sess"));

        let state1 = sample_state();
        persist_energy_state(&state1, &session).unwrap();

        let state2 = EnergyState {
            energy_charge: 0.42,
            regime: Regime::Crisis,
            recommended_strategy: Strategy::Checkpoint,
            ..sample_state()
        };
        persist_energy_state(&state2, &session).unwrap();

        let restored = restore_energy_state(&session).unwrap().unwrap();
        assert!((restored.energy_charge - 0.42).abs() < f64::EPSILON);
    }
}
