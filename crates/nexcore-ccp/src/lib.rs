//! # NexVigilant Core — ccp — Claude Care Process Pharmacokinetic Engine
//!
//! Models AI support interventions as a pharmacokinetic system with:
//! - **5-phase FSM**: Collect → Assess → Plan → Implement → FollowUp
//! - **PK math**: Absorption curves, half-life decay, steady-state dosing
//! - **Interaction dynamics**: Synergistic, antagonistic, additive, potentiating
//! - **Quality scoring**: Composite [0, 10] with bioavailability, stability, safety, persistence
//!
//! ## Primitive Foundation
//!
//! | CCP Concept | T1 Primitive | Symbol |
//! |---|---|:---:|
//! | Phase transitions | Sequence | σ |
//! | Episode state | State | ς |
//! | Therapeutic window | Boundary | ∂ |
//! | Assessment scoring | Comparison | κ |
//! | PK decay curves | Proportionality | ∝ |
//! | Follow-up loop | Recursion | ρ |
//! | Intervention check | Existence | ∃ |
//! | Unmet needs | Absence | ∅ |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_ccp::prelude::*;
//!
//! // Create a new care episode
//! let mut episode = Episode::new("ep-001", 0.0);
//!
//! // Administer an intervention
//! let intervention = Intervention {
//!     dose: Dose::new(0.7).unwrap(),
//!     bioavailability: BioAvailability::new(0.9).unwrap(),
//!     half_life: HalfLife::new(24.0).unwrap(),
//!     strategy: DosingStrategy::Loading,
//!     administered_at: 0.0,
//! };
//! episode.administer(intervention);
//!
//! // Advance through phases
//! episode.advance_phase(Phase::Assess, "context gathered", 1.0).unwrap();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
pub mod episode;
pub mod error;
pub mod grounding;
pub mod interactions;
pub mod kinetics;
pub mod prelude;
pub mod quality;
pub mod state_machine;
pub mod types;

// Integration tests
#[cfg(test)]
mod integration_tests {
    use crate::prelude::*;

    fn bio(v: f64) -> BioAvailability {
        BioAvailability::new(v)
            .unwrap_or_else(|_| BioAvailability::new(1.0).unwrap_or_else(|_| panic!("unreachable")))
    }
    fn hl(v: f64) -> HalfLife {
        HalfLife::new(v)
            .unwrap_or_else(|_| HalfLife::new(1.0).unwrap_or_else(|_| panic!("unreachable")))
    }

    #[test]
    fn full_episode_lifecycle_with_pk() {
        let mut ep = Episode::new("int-001", 0.0);

        // Phase 1: Collect — administer initial assessment
        let loading = Intervention {
            dose: Dose::new(0.7).unwrap_or(Dose::ZERO),
            bioavailability: bio(0.9),
            half_life: hl(24.0),
            strategy: DosingStrategy::Loading,
            administered_at: 0.0,
        };
        ep.administer(loading);
        assert!(ep.plasma_level.value() > 0.0);

        // Transition: Collect → Assess
        assert!(
            ep.advance_phase(Phase::Assess, "initial context gathered", 1.0)
                .is_ok()
        );

        // Phase 2: Assess → Plan
        assert!(
            ep.advance_phase(Phase::Plan, "needs identified", 2.0)
                .is_ok()
        );

        // Phase 3: Plan — compute maintenance dose
        let maint_dose = compute_maintenance_dose(hl(24.0), PlasmaLevel(0.5));
        assert!(maint_dose.is_ok());

        // Administer maintenance
        let maintenance = Intervention {
            dose: maint_dose.unwrap_or(Dose::ZERO),
            bioavailability: bio(0.85),
            half_life: hl(24.0),
            strategy: DosingStrategy::Maintenance,
            administered_at: 3.0,
        };
        ep.administer(maintenance);

        // Transition: Plan → Implement
        assert!(
            ep.advance_phase(Phase::Implement, "plan approved", 3.5)
                .is_ok()
        );

        // Phase 4: Implement — time passes, decay occurs
        ep.decay(12.0);
        assert!(ep.plasma_level.value() > 0.0);

        // Transition: Implement → FollowUp
        assert!(
            ep.advance_phase(Phase::FollowUp, "intervention complete", 15.5)
                .is_ok()
        );

        // Phase 5: Score quality
        let score = score_episode(&ep);
        assert!(score.is_ok());
        let quality = score.unwrap_or_else(|_| QualityScore {
            total: 0.0,
            components: QualityComponents {
                bioavailability: 0.0,
                stability: 0.0,
                safety_margin: 0.0,
                persistence: 0.0,
            },
            rating: QualityRating::Subtherapeutic,
        });
        assert!(quality.total >= 0.0 && quality.total <= 10.0);
    }

    #[test]
    fn recollection_loop() {
        let mut ep = Episode::new("int-002", 0.0);

        // Progress to Plan
        assert!(ep.advance_phase(Phase::Assess, "gathered", 1.0).is_ok());
        assert!(ep.advance_phase(Phase::Plan, "assessed", 2.0).is_ok());
        assert_eq!(ep.phase, Phase::Plan);

        // New information — loop back to Collect
        assert!(
            ep.advance_phase(Phase::Collect, "new info emerged", 3.0)
                .is_ok()
        );
        assert_eq!(ep.phase, Phase::Collect);

        // Re-progress
        assert!(ep.advance_phase(Phase::Assess, "re-gathered", 4.0).is_ok());
        assert!(ep.advance_phase(Phase::Plan, "re-assessed", 5.0).is_ok());
        assert_eq!(ep.transitions.len(), 5);
    }

    #[test]
    fn interaction_effects_on_episode() {
        let mut ep = Episode::new("int-003", 0.0);

        // Two concurrent interventions
        ep.administer(Intervention {
            dose: Dose::new(0.4).unwrap_or(Dose::ZERO),
            bioavailability: bio(0.8),
            half_life: hl(12.0),
            strategy: DosingStrategy::Therapeutic,
            administered_at: 0.0,
        });
        ep.administer(Intervention {
            dose: Dose::new(0.3).unwrap_or(Dose::ZERO),
            bioavailability: bio(0.9),
            half_life: hl(24.0),
            strategy: DosingStrategy::Therapeutic,
            administered_at: 0.0,
        });

        // Check synergistic interaction
        let levels = ep.level_history();
        assert_eq!(levels.len(), 2);

        let effect = compute_interaction(levels[0], levels[1], InteractionType::Synergistic);
        assert!(effect.combined_level.value() > levels[0].value() + levels[1].value());
    }

    #[test]
    fn pk_curves_are_monotonically_decreasing_after_peak() {
        let dose = Dose::new(0.8).unwrap_or(Dose::ZERO);
        let bio_val = bio(1.0);
        let hl_val = hl(24.0);
        let t_max = 1.0;

        let mut prev_level = f64::MAX;
        // Check decay phase: from t_max to t_max + 5 half-lives
        for i in 0..50 {
            let t = t_max + (i as f64 * 2.4); // 0.1 half-life increments
            let level = plasma_level_at(dose, bio_val, hl_val, t, t_max)
                .unwrap_or(PlasmaLevel::ZERO)
                .value();
            assert!(
                level <= prev_level + f64::EPSILON,
                "level should not increase after peak"
            );
            prev_level = level;
        }
    }

    #[test]
    fn plasma_never_negative() {
        let dose = Dose::new(0.5).unwrap_or(Dose::ZERO);
        let bio_val = bio(0.5);
        let hl_val = hl(1.0); // very short half-life

        for i in 0..100 {
            let t = i as f64;
            let level = plasma_level_at(dose, bio_val, hl_val, t, 1.0)
                .unwrap_or(PlasmaLevel::ZERO)
                .value();
            assert!(level >= 0.0, "plasma level should never be negative");
        }
    }

    #[test]
    fn titration_converges_toward_target() {
        let target = PlasmaLevel(0.5);
        let mut current = PlasmaLevel(0.1);

        for _ in 0..10 {
            let dose = titrate(current, target, DosingStrategy::Therapeutic).unwrap_or(Dose::ZERO);
            // Apply dose (simplified: current += dose × bioavail)
            current = PlasmaLevel(current.value() + dose.value() * 0.8);
            if (current.value() - target.value()).abs() < 0.01 {
                break;
            }
        }
        // Should converge near target
        assert!((current.value() - target.value()).abs() < 0.2);
    }

    #[test]
    fn dependency_detection_on_episode_history() {
        let mut ep = Episode::new("int-004", 0.0);

        // Administer escalating doses (dependency pattern)
        for i in 0..5 {
            ep.administer(Intervention {
                dose: Dose::new(0.3 + i as f64 * 0.1).unwrap_or(Dose::ZERO),
                bioavailability: bio(0.8),
                half_life: hl(24.0),
                strategy: DosingStrategy::Therapeutic,
                administered_at: i as f64 * 4.0,
            });
        }

        let history = ep.level_history();
        assert!(detect_dependency_risk(&history));
    }

    #[test]
    fn quality_score_serialization_roundtrip() {
        let mut ep = Episode::new("ser-001", 0.0);
        ep.administer(Intervention {
            dose: Dose::new(0.5).unwrap_or(Dose::ZERO),
            bioavailability: bio(0.8),
            half_life: hl(24.0),
            strategy: DosingStrategy::Therapeutic,
            administered_at: 0.0,
        });
        ep.plasma_level = PlasmaLevel(0.55);

        let score = score_episode(&ep).unwrap_or_else(|_| QualityScore {
            total: 0.0,
            components: QualityComponents {
                bioavailability: 0.0,
                stability: 0.0,
                safety_margin: 0.0,
                persistence: 0.0,
            },
            rating: QualityRating::Subtherapeutic,
        });

        let json = serde_json::to_string(&score).unwrap_or_default();
        assert!(!json.is_empty());
        let deserialized: Result<QualityScore, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());
    }
}
