//! Loop 5: Elastic Equilibrium — Adaptive Capacity.
//!
//! ## T1 Primitive Grounding: κ (Comparison) + ∂ (Boundary) + ν (Frequency)

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticInput {
    pub stress: f64,
    pub yield_point: f64,
    pub fatigue_cycles: u32,
    pub fatigue_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticResult {
    pub state: ElasticState,
    pub strain: f64,
    pub cycles_remaining: u32,
    pub permanent_deformation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElasticState {
    Nominal,
    YieldExceeded,
    FatigueFailureImminent,
}

impl std::fmt::Display for ElasticState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nominal => write!(f, "nominal"),
            Self::YieldExceeded => write!(f, "yield_exceeded"),
            Self::FatigueFailureImminent => write!(f, "fatigue_failure_imminent"),
        }
    }
}

pub fn evaluate(input: &ElasticInput, thresholds: &FlywheelThresholds) -> ElasticResult {
    let modulus = thresholds.default_elastic_modulus;
    let strain = if modulus.abs() < f64::EPSILON {
        0.0
    } else {
        input.stress / modulus
    };
    let fatigue_limit = if input.fatigue_limit > 0 {
        input.fatigue_limit
    } else {
        thresholds.max_fatigue_cycles
    };
    let cycles_remaining = fatigue_limit.saturating_sub(input.fatigue_cycles);
    let state = if input.fatigue_cycles > fatigue_limit {
        ElasticState::FatigueFailureImminent
    } else if input.stress >= input.yield_point {
        ElasticState::YieldExceeded
    } else {
        ElasticState::Nominal
    };
    let permanent_deformation = if input.stress > input.yield_point {
        input.stress - input.yield_point
    } else {
        0.0
    };
    ElasticResult {
        state,
        strain,
        cycles_remaining,
        permanent_deformation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    #[test]
    fn nominal() {
        let r = evaluate(
            &ElasticInput {
                stress: 50.0,
                yield_point: 100.0,
                fatigue_cycles: 100,
                fatigue_limit: 1000,
            },
            &t(),
        );
        assert_eq!(r.state, ElasticState::Nominal);
        assert_eq!(r.cycles_remaining, 900);
    }
    #[test]
    fn yield_exceeded() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 150.0,
                    yield_point: 100.0,
                    fatigue_cycles: 100,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::YieldExceeded
        );
    }
    #[test]
    fn fatigue() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 50.0,
                    yield_point: 100.0,
                    fatigue_cycles: 1500,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::FatigueFailureImminent
        );
    }
    #[test]
    fn fatigue_overrides() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 200.0,
                    yield_point: 100.0,
                    fatigue_cycles: 2000,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::FatigueFailureImminent
        );
    }
    #[test]
    fn zero_stress() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 0.0,
                    yield_point: 100.0,
                    fatigue_cycles: 0,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::Nominal
        );
    }
    #[test]
    fn at_yield() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 100.0,
                    yield_point: 100.0,
                    fatigue_cycles: 0,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::YieldExceeded
        );
    }
    #[test]
    fn at_fatigue() {
        assert_eq!(
            evaluate(
                &ElasticInput {
                    stress: 50.0,
                    yield_point: 100.0,
                    fatigue_cycles: 1000,
                    fatigue_limit: 1000
                },
                &t()
            )
            .state,
            ElasticState::Nominal
        );
    }
}
