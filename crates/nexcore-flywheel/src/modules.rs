// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Module-to-loop mapping — which NexVigilant module serves which loops.
//!
//! From spec §5:
//!
//! | Module    | Primary Loop | Role                                      |
//! |-----------|-------------|-------------------------------------------|
//! | Nucleus   | Loop 2, 4   | Momentum storage, mission axis            |
//! | Guardian  | Loop 4      | Regulatory axis stability                 |
//! | Academy   | Loop 1, 5   | Value density for rim, adaptive capacity  |
//! | Community | Loop 1      | Primary rim structure                     |
//! | Careers   | Loop 1      | Rim reinforcement (switching cost)        |
//! | Insights  | Loop 2      | Knowledge persistence                     |
//! | Neural    | Loop 2, 3   | Momentum storage, friction reduction      |
//! | Core      | Loop 3, 5   | Operational friction, founder stress      |
//!
//! ## T1 Primitive Grounding: μ (Mapping) + Σ (Sum)

use serde::{Deserialize, Serialize};

/// NexVigilant product module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Nucleus,
    Guardian,
    Academy,
    Community,
    Careers,
    Insights,
    Neural,
    Core,
}

impl Module {
    /// All modules.
    pub const ALL: [Module; 8] = [
        Module::Nucleus,
        Module::Guardian,
        Module::Academy,
        Module::Community,
        Module::Careers,
        Module::Insights,
        Module::Neural,
        Module::Core,
    ];
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nucleus => write!(f, "Nucleus"),
            Self::Guardian => write!(f, "Guardian"),
            Self::Academy => write!(f, "Academy"),
            Self::Community => write!(f, "Community"),
            Self::Careers => write!(f, "Careers"),
            Self::Insights => write!(f, "Insights"),
            Self::Neural => write!(f, "Neural"),
            Self::Core => write!(f, "Core"),
        }
    }
}

/// Module's contribution to the flywheel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleContribution {
    pub module: Module,
    /// Which loops this module primarily serves (1-indexed as in spec).
    pub primary_loops: Vec<u8>,
    /// Role description.
    pub role: String,
    /// Which structural components this module maps to.
    pub components: Vec<crate::components::ComponentKind>,
}

/// Get the canonical module-to-loop mapping from the spec.
#[must_use]
pub fn module_map() -> Vec<ModuleContribution> {
    use crate::components::ComponentKind;

    vec![
        ModuleContribution {
            module: Module::Nucleus,
            primary_loops: vec![2, 4],
            role: "Central coordination and orchestration — momentum storage, mission axis".into(),
            components: vec![ComponentKind::Hub],
        },
        ModuleContribution {
            module: Module::Guardian,
            primary_loops: vec![4],
            role: "Regulatory monitoring — axis stability".into(),
            components: vec![ComponentKind::Hub],
        },
        ModuleContribution {
            module: Module::Academy,
            primary_loops: vec![1, 5],
            role: "Education and development — value density, adaptive capacity".into(),
            components: vec![ComponentKind::Rim, ComponentKind::Housing],
        },
        ModuleContribution {
            module: Module::Community,
            primary_loops: vec![1],
            role: "User engagement — primary rim structure".into(),
            components: vec![ComponentKind::Rim],
        },
        ModuleContribution {
            module: Module::Careers,
            primary_loops: vec![1],
            role: "Professional placement — rim reinforcement via switching cost".into(),
            components: vec![ComponentKind::Rim],
        },
        ModuleContribution {
            module: Module::Insights,
            primary_loops: vec![2],
            role: "Data analysis and reporting — knowledge persistence".into(),
            components: vec![ComponentKind::Shaft],
        },
        ModuleContribution {
            module: Module::Neural,
            primary_loops: vec![2, 3],
            role: "AI/ML capabilities — momentum storage, friction reduction".into(),
            components: vec![ComponentKind::Hub, ComponentKind::Spokes],
        },
        ModuleContribution {
            module: Module::Core,
            primary_loops: vec![3, 5],
            role: "Operations and infrastructure — friction management, founder stress".into(),
            components: vec![ComponentKind::Housing],
        },
    ]
}

/// Compute per-module health scores from a cascade result.
///
/// Each module's health is the average of its primary loop outcomes,
/// weighted by loop contribution.
#[must_use]
pub fn module_health(cascade: &crate::loops::CascadeResult) -> Vec<(Module, f64)> {
    use crate::loops::{
        elastic::ElasticState, friction::FrictionClassification, gyroscopic::GyroscopicState,
        momentum::MomentumClassification, rim_integrity::RimState,
    };

    let loop_scores: [f64; 5] = [
        match cascade.rim.state {
            RimState::Thriving => 1.0,
            RimState::Critical => 0.4,
            RimState::Disintegrated => 0.0,
        },
        match cascade.momentum.classification {
            MomentumClassification::High => 1.0,
            MomentumClassification::Normal => 0.75,
            MomentumClassification::Low => 0.4,
            MomentumClassification::Stalled => 0.1,
        },
        match cascade.friction.classification {
            FrictionClassification::Acceptable => 1.0,
            FrictionClassification::Warning => 0.5,
            FrictionClassification::Critical => 0.1,
        },
        match cascade.gyroscopic.state {
            GyroscopicState::Stable => 1.0,
            GyroscopicState::Precessing => 0.7,
            GyroscopicState::NoStability => 0.3,
            GyroscopicState::GimbalLock => 0.0,
        },
        match cascade.elastic.state {
            ElasticState::Nominal => 1.0,
            ElasticState::YieldExceeded => 0.3,
            ElasticState::FatigueFailureImminent => 0.0,
        },
    ];

    module_map()
        .iter()
        .map(|contrib| {
            let score: f64 = contrib
                .primary_loops
                .iter()
                .filter_map(|&loop_num| loop_scores.get((loop_num as usize).wrapping_sub(1)))
                .sum::<f64>();
            let count = contrib.primary_loops.len().max(1) as f64;
            (contrib.module, (score / count).clamp(0.0, 1.0))
        })
        .collect()
}

/// Find modules that serve a specific loop.
#[must_use]
pub fn modules_for_loop(loop_number: u8) -> Vec<Module> {
    module_map()
        .iter()
        .filter(|c| c.primary_loops.contains(&loop_number))
        .map(|c| c.module)
        .collect()
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

    #[test]
    fn module_map_has_eight() {
        assert_eq!(module_map().len(), 8);
    }

    #[test]
    fn all_modules_covered() {
        let map = module_map();
        for module in Module::ALL {
            assert!(map.iter().any(|c| c.module == module), "Missing {module}");
        }
    }

    #[test]
    fn healthy_modules_high_score() {
        let health = module_health(&healthy_cascade());
        for (module, score) in &health {
            assert!(*score > 0.5, "{module} health too low: {score}");
        }
    }

    #[test]
    fn loop_1_modules() {
        let mods = modules_for_loop(1);
        assert!(mods.contains(&Module::Community));
        assert!(mods.contains(&Module::Careers));
        assert!(mods.contains(&Module::Academy));
    }

    #[test]
    fn loop_4_modules() {
        let mods = modules_for_loop(4);
        assert!(mods.contains(&Module::Nucleus));
        assert!(mods.contains(&Module::Guardian));
    }

    #[test]
    fn module_health_count() {
        let health = module_health(&healthy_cascade());
        assert_eq!(health.len(), 8);
    }

    #[test]
    fn all_modules_display() {
        for module in Module::ALL {
            let s = module.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn no_modules_for_loop_0() {
        assert!(modules_for_loop(0).is_empty());
    }
}
