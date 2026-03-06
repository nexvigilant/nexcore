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
pub struct CascadeResult {
    pub rim: RimIntegrityResult,
    pub momentum: MomentumResult,
    pub friction: FrictionResult,
    pub gyroscopic: GyroscopicResult,
    pub elastic: ElasticResult,
    pub system_state: SystemState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemState {
    Thriving,
    Stressed,
    Critical,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeInput {
    pub rim: RimInput,
    pub momentum: MomentumInput,
    pub friction: FrictionInput,
    pub gyroscopic: GyroscopicInput,
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
