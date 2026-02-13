//! Care episode — composed intervention state with PK dynamics.
//!
//! # T1 Grounding
//! - ς (state): Episode tracks mutable care state
//! - σ (sequence): Interventions accumulate in order
//! - ∂ (boundary): Phase transitions guard valid paths
//! - ∝ (proportionality): PK math governs level dynamics

use serde::{Deserialize, Serialize};

use crate::error::CcpError;
use crate::kinetics;
use crate::state_machine::{self, PhaseTransition};
use crate::types::{BioAvailability, Dose, DosingStrategy, HalfLife, Phase, PlasmaLevel};

/// A single intervention administered during an episode.
///
/// Tier: T2-C (composes Dose + BioAvailability + HalfLife + DosingStrategy)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intervention {
    /// Dose intensity.
    pub dose: Dose,
    /// Transfer efficiency.
    pub bioavailability: BioAvailability,
    /// Decay half-life in hours.
    pub half_life: HalfLife,
    /// Dosing strategy used.
    pub strategy: DosingStrategy,
    /// Epoch hours when administered.
    pub administered_at: f64,
}

/// A care episode tracking the full support lifecycle.
///
/// Tier: T2-C (composes Intervention + Phase + PlasmaLevel + PhaseTransition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique episode identifier.
    pub id: String,
    /// Current phase in the care process.
    pub phase: Phase,
    /// All interventions administered.
    pub interventions: Vec<Intervention>,
    /// Current plasma level (support intensity).
    pub plasma_level: PlasmaLevel,
    /// Quality score, computed on demand.
    pub quality_score: Option<f64>,
    /// Epoch hours when episode started.
    pub started_at: f64,
    /// Record of all phase transitions.
    pub transitions: Vec<PhaseTransition>,
}

impl Episode {
    /// Create a new episode at the Collect phase.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for this episode
    /// - `started_at`: Epoch hours when the episode begins
    #[must_use]
    pub fn new(id: impl Into<String>, started_at: f64) -> Self {
        Self {
            id: id.into(),
            phase: Phase::Collect,
            interventions: Vec::new(),
            plasma_level: PlasmaLevel::ZERO,
            quality_score: None,
            started_at,
            transitions: Vec::new(),
        }
    }

    /// Administer an intervention, updating the plasma level.
    ///
    /// The plasma level is computed at t_max (peak absorption),
    /// then added to the current level.
    pub fn administer(&mut self, intervention: Intervention) {
        // Compute peak contribution of this intervention
        // t_max = 1.0 hour by default (rapid absorption for AI support)
        let t_max = 1.0;
        let contribution = kinetics::plasma_level_at(
            intervention.dose,
            intervention.bioavailability,
            intervention.half_life,
            t_max,
            t_max,
        )
        .unwrap_or(PlasmaLevel::ZERO);

        self.plasma_level = PlasmaLevel(self.plasma_level.value() + contribution.value());
        self.interventions.push(intervention);
        self.quality_score = None; // invalidate cached score
    }

    /// Apply decay to the plasma level over elapsed hours.
    ///
    /// Uses the average half-life of all interventions.
    /// If no interventions exist, no decay occurs.
    pub fn decay(&mut self, elapsed_hours: f64) {
        if self.interventions.is_empty() || elapsed_hours <= 0.0 {
            return;
        }

        // Average half-life across all interventions
        let avg_hl: f64 = self
            .interventions
            .iter()
            .map(|i| i.half_life.value())
            .sum::<f64>()
            / self.interventions.len() as f64;

        if avg_hl <= 0.0 {
            return;
        }

        let k = core::f64::consts::LN_2 / avg_hl;
        let decayed = self.plasma_level.value() * (-k * elapsed_hours).exp();
        self.plasma_level = PlasmaLevel(decayed.max(0.0));
        self.quality_score = None; // invalidate cached score
    }

    /// Advance to a new phase with a reason.
    ///
    /// # Errors
    /// Returns `CcpError::InvalidPhaseTransition` if the transition is invalid.
    pub fn advance_phase(
        &mut self,
        target: Phase,
        reason: &str,
        timestamp: f64,
    ) -> Result<(), CcpError> {
        let transition = state_machine::execute_transition(self.phase, target, reason, timestamp)?;
        self.phase = target;
        self.transitions.push(transition);
        Ok(())
    }

    /// Number of interventions administered.
    #[must_use]
    pub fn intervention_count(&self) -> usize {
        self.interventions.len()
    }

    /// Duration of the episode in hours (from start to last transition or intervention).
    #[must_use]
    pub fn duration_hours(&self) -> f64 {
        let last_transition = self.transitions.last().map(|t| t.timestamp).unwrap_or(0.0);
        let last_intervention = self
            .interventions
            .last()
            .map(|i| i.administered_at)
            .unwrap_or(0.0);
        let latest = last_transition.max(last_intervention).max(self.started_at);
        latest - self.started_at
    }

    /// Get a history of plasma levels from all interventions (at their peak times).
    #[must_use]
    pub fn level_history(&self) -> Vec<PlasmaLevel> {
        let mut levels = Vec::new();
        let mut running = 0.0_f64;

        for intervention in &self.interventions {
            let contribution = intervention.dose.value() * intervention.bioavailability.value();
            running += contribution;
            levels.push(PlasmaLevel(running));
        }

        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_intervention(
        dose: f64,
        bio: f64,
        hl: f64,
        strategy: DosingStrategy,
        at: f64,
    ) -> Intervention {
        Intervention {
            dose: Dose::new(dose).unwrap_or(Dose::ZERO),
            bioavailability: BioAvailability::new(bio).unwrap_or_else(|_| {
                BioAvailability::new(1.0).unwrap_or_else(|_| panic!("unreachable"))
            }),
            half_life: HalfLife::new(hl)
                .unwrap_or_else(|_| HalfLife::new(1.0).unwrap_or_else(|_| panic!("unreachable"))),
            strategy,
            administered_at: at,
        }
    }

    #[test]
    fn new_episode_starts_at_collect() {
        let ep = Episode::new("test-1", 0.0);
        assert_eq!(ep.phase, Phase::Collect);
        assert!((ep.plasma_level.value()).abs() < f64::EPSILON);
        assert!(ep.interventions.is_empty());
    }

    #[test]
    fn administer_increases_plasma_level() {
        let mut ep = Episode::new("test-1", 0.0);
        let intervention = make_intervention(0.8, 1.0, 24.0, DosingStrategy::Loading, 0.0);
        ep.administer(intervention);
        assert!(ep.plasma_level.value() > 0.0);
        assert_eq!(ep.intervention_count(), 1);
    }

    #[test]
    fn multiple_interventions_accumulate() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.administer(make_intervention(
            0.5,
            1.0,
            24.0,
            DosingStrategy::Loading,
            0.0,
        ));
        let after_first = ep.plasma_level.value();
        ep.administer(make_intervention(
            0.3,
            1.0,
            24.0,
            DosingStrategy::Maintenance,
            1.0,
        ));
        assert!(ep.plasma_level.value() > after_first);
        assert_eq!(ep.intervention_count(), 2);
    }

    #[test]
    fn decay_reduces_plasma_level() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.administer(make_intervention(
            0.8,
            1.0,
            24.0,
            DosingStrategy::Loading,
            0.0,
        ));
        let before = ep.plasma_level.value();
        ep.decay(24.0); // one half-life
        let after = ep.plasma_level.value();
        assert!(after < before);
        // Should be approximately half
        assert!((after / before - 0.5).abs() < 0.05);
    }

    #[test]
    fn decay_with_no_interventions_is_noop() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.decay(24.0);
        assert!((ep.plasma_level.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn decay_with_zero_hours_is_noop() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.administer(make_intervention(
            0.8,
            1.0,
            24.0,
            DosingStrategy::Loading,
            0.0,
        ));
        let before = ep.plasma_level.value();
        ep.decay(0.0);
        assert!((ep.plasma_level.value() - before).abs() < f64::EPSILON);
    }

    #[test]
    fn advance_phase_forward() {
        let mut ep = Episode::new("test-1", 0.0);
        let result = ep.advance_phase(Phase::Assess, "context gathered", 1.0);
        assert!(result.is_ok());
        assert_eq!(ep.phase, Phase::Assess);
        assert_eq!(ep.transitions.len(), 1);
    }

    #[test]
    fn advance_phase_back_to_collect() {
        let mut ep = Episode::new("test-1", 0.0);
        let _ = ep.advance_phase(Phase::Assess, "gathered", 1.0);
        let _ = ep.advance_phase(Phase::Plan, "assessed", 2.0);
        let result = ep.advance_phase(Phase::Collect, "new info", 3.0);
        assert!(result.is_ok());
        assert_eq!(ep.phase, Phase::Collect);
        assert_eq!(ep.transitions.len(), 3);
    }

    #[test]
    fn advance_phase_skip_rejected() {
        let mut ep = Episode::new("test-1", 0.0);
        let result = ep.advance_phase(Phase::Plan, "skip", 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn full_lifecycle() {
        let mut ep = Episode::new("lifecycle-1", 0.0);

        // Phase 1: Collect
        ep.administer(make_intervention(
            0.3,
            0.8,
            12.0,
            DosingStrategy::Subtherapeutic,
            0.0,
        ));
        let _ = ep.advance_phase(Phase::Assess, "context ready", 1.0);

        // Phase 2: Assess
        let _ = ep.advance_phase(Phase::Plan, "needs identified", 2.0);

        // Phase 3: Plan
        ep.administer(make_intervention(
            0.7,
            0.9,
            24.0,
            DosingStrategy::Loading,
            2.5,
        ));
        let _ = ep.advance_phase(Phase::Implement, "plan ready", 3.0);

        // Phase 4: Implement
        ep.decay(6.0); // some time passes
        let _ = ep.advance_phase(Phase::FollowUp, "implemented", 9.0);

        // Phase 5: FollowUp
        assert_eq!(ep.phase, Phase::FollowUp);
        assert_eq!(ep.intervention_count(), 2);
        assert_eq!(ep.transitions.len(), 4);
        assert!(ep.plasma_level.value() > 0.0);
    }

    #[test]
    fn level_history_tracks_cumulative() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.administer(make_intervention(
            0.5,
            1.0,
            24.0,
            DosingStrategy::Loading,
            0.0,
        ));
        ep.administer(make_intervention(
            0.3,
            1.0,
            24.0,
            DosingStrategy::Maintenance,
            1.0,
        ));
        let history = ep.level_history();
        assert_eq!(history.len(), 2);
        assert!((history[0].value() - 0.5).abs() < f64::EPSILON);
        assert!((history[1].value() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn duration_hours_computed() {
        let mut ep = Episode::new("test-1", 0.0);
        ep.administer(make_intervention(
            0.5,
            1.0,
            24.0,
            DosingStrategy::Loading,
            5.0,
        ));
        assert!((ep.duration_hours() - 5.0).abs() < f64::EPSILON);
    }
}
