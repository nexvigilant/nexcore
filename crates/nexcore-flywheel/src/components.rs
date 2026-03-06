// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! The five structural components — mechanical-to-strategic mapping.
//!
//! Each component is a bounded domain with its own health metric.
//! Components compose into the flywheel via loop contributions.
//!
//! ## T1 Primitive Grounding: ∂ (Boundary) + λ (Location) + × (Product)
//!
//! | Component | Mechanical    | Strategic                        |
//! |-----------|---------------|----------------------------------|
//! | Rim       | Outer mass    | Value network (users, community) |
//! | Hub       | Central bore  | Nucleus — coordination layer     |
//! | Spokes    | Hub↔Rim bridge| Module interconnections (APIs)   |
//! | Shaft     | Input/output  | External interface layer         |
//! | Housing   | Bearings      | Infrastructure (GCP/Firebase)    |

use serde::{Deserialize, Serialize};

/// Component identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    Rim,
    Hub,
    Spokes,
    Shaft,
    Housing,
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rim => write!(f, "rim"),
            Self::Hub => write!(f, "hub"),
            Self::Spokes => write!(f, "spokes"),
            Self::Shaft => write!(f, "shaft"),
            Self::Housing => write!(f, "housing"),
        }
    }
}

/// Health status of a single component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub kind: ComponentKind,
    /// 0.0 (failed) to 1.0 (perfect).
    pub health: f64,
    /// Which loops this component primarily serves.
    pub primary_loops: Vec<u8>,
    /// Strategic description.
    pub role: String,
}

/// The five-component flywheel anatomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelAnatomy {
    pub rim: ComponentHealth,
    pub hub: ComponentHealth,
    pub spokes: ComponentHealth,
    pub shaft: ComponentHealth,
    pub housing: ComponentHealth,
}

impl FlywheelAnatomy {
    /// Overall structural health — weighted by energy contribution.
    ///
    /// Rim contributes most (r²), housing is critical path (single point of failure).
    #[must_use]
    pub fn overall_health(&self) -> f64 {
        let weights = [0.30, 0.20, 0.15, 0.15, 0.20]; // rim, hub, spokes, shaft, housing
        let healths = [
            self.rim.health,
            self.hub.health,
            self.spokes.health,
            self.shaft.health,
            self.housing.health,
        ];
        let weighted: f64 = weights.iter().zip(healths.iter()).map(|(w, h)| w * h).sum();
        weighted.clamp(0.0, 1.0)
    }

    /// Identify the weakest component.
    #[must_use]
    pub fn weakest(&self) -> &ComponentHealth {
        let all = [
            &self.rim,
            &self.hub,
            &self.spokes,
            &self.shaft,
            &self.housing,
        ];
        all.into_iter()
            .min_by(|a, b| {
                a.health
                    .partial_cmp(&b.health)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&self.rim)
    }
}

/// Derive component health from a cascade result.
///
/// Maps loop outcomes to structural component health scores.
#[must_use]
pub fn derive_anatomy(cascade: &crate::loops::CascadeResult) -> FlywheelAnatomy {
    use crate::loops::{
        elastic::ElasticState, friction::FrictionClassification, gyroscopic::GyroscopicState,
        momentum::MomentumClassification, rim_integrity::RimState,
    };

    // Rim health: directly from Loop 1
    let rim_health = match cascade.rim.state {
        RimState::Thriving => 1.0,
        RimState::Critical => 0.4,
        RimState::Disintegrated => 0.0,
    };

    // Hub health: from Loop 2 (momentum) + Loop 4 (gyroscopic)
    let momentum_score = match cascade.momentum.classification {
        MomentumClassification::High => 1.0,
        MomentumClassification::Normal => 0.75,
        MomentumClassification::Low => 0.4,
        MomentumClassification::Stalled => 0.1,
    };
    let gyro_score = match cascade.gyroscopic.state {
        GyroscopicState::Stable => 1.0,
        GyroscopicState::Precessing => 0.7,
        GyroscopicState::NoStability => 0.3,
        GyroscopicState::GimbalLock => 0.0,
    };
    let hub_health = (momentum_score + gyro_score) / 2.0;

    // Spokes health: from Loop 3 (friction — inter-module overhead)
    let spokes_health = match cascade.friction.classification {
        FrictionClassification::Acceptable => 1.0,
        FrictionClassification::Warning => 0.5,
        FrictionClassification::Critical => 0.1,
    };

    // Shaft health: from Loop 1 (rim) + Loop 2 (momentum) — external interface
    let shaft_health = (rim_health + momentum_score) / 2.0;

    // Housing health: from Loop 5 (elastic) + Loop 3 (friction) — infrastructure
    let elastic_score = match cascade.elastic.state {
        ElasticState::Nominal => 1.0,
        ElasticState::YieldExceeded => 0.3,
        ElasticState::FatigueFailureImminent => 0.0,
    };
    let housing_health = (elastic_score + spokes_health) / 2.0;

    FlywheelAnatomy {
        rim: ComponentHealth {
            kind: ComponentKind::Rim,
            health: rim_health,
            primary_loops: vec![1],
            role: "Value network — users, community, content".into(),
        },
        hub: ComponentHealth {
            kind: ComponentKind::Hub,
            health: hub_health,
            primary_loops: vec![2, 4],
            role: "Nucleus — central coordination and mission axis".into(),
        },
        spokes: ComponentHealth {
            kind: ComponentKind::Spokes,
            health: spokes_health,
            primary_loops: vec![3],
            role: "Module interconnections — APIs, data flows".into(),
        },
        shaft: ComponentHealth {
            kind: ComponentKind::Shaft,
            health: shaft_health,
            primary_loops: vec![1, 2],
            role: "External interface — public API, website".into(),
        },
        housing: ComponentHealth {
            kind: ComponentKind::Housing,
            health: housing_health,
            primary_loops: vec![3, 5],
            role: "Infrastructure — GCP/Firebase, CI/CD, monitoring".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loops::{
        self, CascadeInput, ElasticInput, FrictionInput, GyroscopicInput, MomentumInput, RimInput,
    };
    use crate::thresholds::FlywheelThresholds;

    fn healthy_cascade() -> crate::loops::CascadeResult {
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 200.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.9,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 500.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        };
        loops::cascade(&input, &FlywheelThresholds::default())
    }

    fn failing_cascade() -> crate::loops::CascadeResult {
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 10.0,
                centrifugal_force: 100.0,
            },
            momentum: MomentumInput {
                inertia: 1.0,
                omega: 1.0,
                friction_drain: 50.0,
            },
            friction: FrictionInput {
                manual_processes: 100.0,
                human_touchpoints: 100.0,
                velocity: 50.0,
                automation_coverage: 0.0,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 100.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 200.0,
                yield_point: 100.0,
                fatigue_cycles: 2000,
                fatigue_limit: 1000,
            },
        };
        loops::cascade(&input, &FlywheelThresholds::default())
    }

    #[test]
    fn healthy_anatomy_high_health() {
        let anatomy = derive_anatomy(&healthy_cascade());
        assert!(
            anatomy.overall_health() > 0.8,
            "Got {}",
            anatomy.overall_health()
        );
        assert!((anatomy.rim.health - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn failing_anatomy_low_health() {
        let anatomy = derive_anatomy(&failing_cascade());
        assert!(
            anatomy.overall_health() < 0.3,
            "Got {}",
            anatomy.overall_health()
        );
    }

    #[test]
    fn weakest_component_identified() {
        let anatomy = derive_anatomy(&failing_cascade());
        let weakest = anatomy.weakest();
        assert!(weakest.health < 0.2);
    }

    #[test]
    fn all_components_have_loops() {
        let anatomy = derive_anatomy(&healthy_cascade());
        assert!(!anatomy.rim.primary_loops.is_empty());
        assert!(!anatomy.hub.primary_loops.is_empty());
        assert!(!anatomy.spokes.primary_loops.is_empty());
        assert!(!anatomy.shaft.primary_loops.is_empty());
        assert!(!anatomy.housing.primary_loops.is_empty());
    }

    #[test]
    fn overall_health_bounded() {
        let anatomy = derive_anatomy(&healthy_cascade());
        assert!(anatomy.overall_health() <= 1.0);
        assert!(anatomy.overall_health() >= 0.0);
    }

    #[test]
    fn component_kinds_display() {
        assert_eq!(ComponentKind::Rim.to_string(), "rim");
        assert_eq!(ComponentKind::Hub.to_string(), "hub");
        assert_eq!(ComponentKind::Spokes.to_string(), "spokes");
        assert_eq!(ComponentKind::Shaft.to_string(), "shaft");
        assert_eq!(ComponentKind::Housing.to_string(), "housing");
    }
}
