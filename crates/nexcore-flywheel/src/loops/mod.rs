//! The five autonomous loops — self-sustaining feedback mechanisms.
//!
//! ## Loop Interaction Hierarchy (Critical Path: 5→1→2→4, with 3→2)

pub mod elastic;
pub mod friction;
pub mod gyroscopic;
pub mod momentum;
pub mod rim_integrity;

pub use elastic::{ElasticInput, ElasticResult, ElasticState};
pub use friction::{FrictionClassification, FrictionInput, FrictionResult};
pub use gyroscopic::{GyroscopicInput, GyroscopicResult, GyroscopicState};
pub use momentum::{MomentumClassification, MomentumInput, MomentumResult};
pub use rim_integrity::{RimInput, RimIntegrityResult, RimState};

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Result of the five-loop interaction cascade.
pub struct CascadeResult {
    /// Loop 1: structural integrity.
    pub rim: RimIntegrityResult,
    /// Loop 2: angular momentum.
    pub momentum: MomentumResult,
    /// Loop 3: parasitic friction drain.
    pub friction: FrictionResult,
    /// Loop 4: directional stability.
    pub gyroscopic: GyroscopicResult,
    /// Loop 5: fatigue resilience.
    pub elastic: ElasticResult,
    /// Composite system classification.
    pub system_state: SystemState,
}

/// Composite system health classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemState {
    /// All loops healthy.
    Thriving,
    /// One loop degraded but recoverable.
    Stressed,
    /// Two or more loops critical.
    Critical,
    /// Structural failure (rim disintegrated or fatigue imminent).
    Failed,
}

impl std::fmt::Display for SystemState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thriving => write!(f, "thriving"),
            Self::Stressed => write!(f, "stressed"),
            Self::Critical => write!(f, "critical"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Input to the five-loop cascade — one sub-input per loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeInput {
    /// Loop 1 input: structural forces.
    pub rim: RimInput,
    /// Loop 2 input: rotational dynamics.
    pub momentum: MomentumInput,
    /// Loop 3 input: friction sources.
    pub friction: FrictionInput,
    /// Loop 4 input: stability parameters.
    pub gyroscopic: GyroscopicInput,
    /// Loop 5 input: fatigue state.
    pub elastic: ElasticInput,
}

/// Run the loop interaction cascade: 5→1→2→4, with 3→2 as antagonist.
pub fn cascade(input: &CascadeInput, thresholds: &FlywheelThresholds) -> CascadeResult {
    let elastic_result = elastic::evaluate(&input.elastic, thresholds);
    let rim_result = rim_integrity::evaluate(&input.rim, thresholds);
    let friction_result = friction::evaluate(&input.friction, thresholds);

    let mut momentum_input = input.momentum.clone();
    momentum_input.friction_drain += friction_result.net_drain;
    let momentum_result = momentum::evaluate(&momentum_input, thresholds);

    let mut gyro_input = input.gyroscopic.clone();
    gyro_input.momentum_l = momentum_result.l;
    let gyroscopic_result = gyroscopic::evaluate(&gyro_input, thresholds);

    let system_state = classify_system(
        &elastic_result,
        &rim_result,
        &friction_result,
        &momentum_result,
        &gyroscopic_result,
    );
    CascadeResult {
        rim: rim_result,
        momentum: momentum_result,
        friction: friction_result,
        gyroscopic: gyroscopic_result,
        elastic: elastic_result,
        system_state,
    }
}

fn classify_system(
    elastic: &ElasticResult,
    rim: &RimIntegrityResult,
    friction: &FrictionResult,
    momentum: &MomentumResult,
    gyroscopic: &GyroscopicResult,
) -> SystemState {
    if elastic.state == ElasticState::FatigueFailureImminent {
        return SystemState::Failed;
    }
    if rim.state == RimState::Disintegrated {
        return SystemState::Failed;
    }

    let critical_count = [
        rim.state == RimState::Critical,
        momentum.classification == MomentumClassification::Stalled,
        friction.classification == FrictionClassification::Critical,
        gyroscopic.state == GyroscopicState::GimbalLock,
        elastic.state == ElasticState::YieldExceeded,
    ]
    .iter()
    .filter(|&&v| v)
    .count();

    if critical_count >= 2 {
        return SystemState::Critical;
    }

    let all_healthy = rim.state == RimState::Thriving
        && matches!(
            momentum.classification,
            MomentumClassification::High | MomentumClassification::Normal
        )
        && friction.classification == FrictionClassification::Acceptable
        && matches!(
            gyroscopic.state,
            GyroscopicState::Stable | GyroscopicState::Precessing
        )
        && elastic.state == ElasticState::Nominal;

    if all_healthy {
        SystemState::Thriving
    } else {
        SystemState::Stressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thresholds::FlywheelThresholds;

    fn healthy_input() -> CascadeInput {
        CascadeInput {
            rim: RimInput {
                tensile_strength: 100.0,
                centrifugal_force: 20.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 2.0,
                friction_drain: 1.0,
            },
            friction: FrictionInput {
                manual_processes: 2.0,
                human_touchpoints: 1.0,
                velocity: 2.0,
                automation_coverage: 0.8,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 200.0,
                perturbation_torque: 5.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 0.5,
                yield_point: 2.0,
                fatigue_cycles: 10,
                fatigue_limit: 1000,
            },
        }
    }

    #[test]
    fn healthy_cascade_is_thriving() {
        let result = cascade(&healthy_input(), &FlywheelThresholds::default());
        assert_eq!(result.system_state, SystemState::Thriving);
    }

    #[test]
    fn system_state_display() {
        assert_eq!(format!("{}", SystemState::Thriving), "thriving");
        assert_eq!(format!("{}", SystemState::Failed), "failed");
        assert_eq!(format!("{}", SystemState::Critical), "critical");
        assert_eq!(format!("{}", SystemState::Stressed), "stressed");
    }

    #[test]
    fn high_fatigue_is_failed() {
        let mut input = healthy_input();
        input.elastic.fatigue_cycles = 2000;
        input.elastic.fatigue_limit = 1000;
        let result = cascade(&input, &FlywheelThresholds::default());
        assert_eq!(result.system_state, SystemState::Failed);
    }

    #[test]
    fn high_churn_degrades_rim() {
        let mut input = healthy_input();
        input.rim.centrifugal_force = 200.0; // far exceeds tensile strength
        let result = cascade(&input, &FlywheelThresholds::default());
        assert!(matches!(
            result.system_state,
            SystemState::Stressed | SystemState::Critical | SystemState::Failed
        ));
    }

    #[test]
    fn cascade_result_serializes() {
        let result = cascade(&healthy_input(), &FlywheelThresholds::default());
        let json = serde_json::to_string(&result).expect("ser");
        assert!(json.contains("thriving"));
    }

    #[test]
    fn friction_feeds_into_momentum() {
        let mut input = healthy_input();
        input.friction.manual_processes = 50.0;
        input.friction.human_touchpoints = 20.0;
        input.friction.automation_coverage = 0.0;
        let result = cascade(&input, &FlywheelThresholds::default());
        // High friction should drain momentum
        assert!(
            result.momentum.l < 200.0
                || matches!(
                    result.system_state,
                    SystemState::Stressed | SystemState::Critical
                )
        );
    }

    #[test]
    fn cascade_input_serializes() {
        let input = healthy_input();
        let json = serde_json::to_string(&input).expect("ser");
        let back: CascadeInput = serde_json::from_str(&json).expect("de");
        assert!((back.rim.tensile_strength - 100.0).abs() < f64::EPSILON);
    }
}
