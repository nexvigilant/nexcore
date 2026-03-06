//! Loop 1: Rim Integrity — Value Network Self-Containment.
//!
//! ## T1 Primitive Grounding: κ (Comparison) + ∂ (Boundary)

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RimInput {
    pub tensile_strength: f64,
    pub centrifugal_force: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RimIntegrityResult {
    pub state: RimState,
    pub margin: f64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RimState {
    Thriving,
    Critical,
    Disintegrated,
}

impl std::fmt::Display for RimState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thriving => write!(f, "thriving"),
            Self::Critical => write!(f, "critical"),
            Self::Disintegrated => write!(f, "disintegrated"),
        }
    }
}

pub fn evaluate(input: &RimInput, thresholds: &FlywheelThresholds) -> RimIntegrityResult {
    let margin = input.tensile_strength - input.centrifugal_force;
    let ratio = if input.centrifugal_force.abs() < f64::EPSILON {
        if input.tensile_strength.abs() < f64::EPSILON {
            1.0
        } else {
            f64::MAX
        }
    } else {
        input.tensile_strength / input.centrifugal_force
    };
    let critical_margin = thresholds.rim_critical_margin;
    let state = if ratio > 1.0 + critical_margin {
        RimState::Thriving
    } else if ratio >= 1.0 - critical_margin {
        RimState::Critical
    } else {
        RimState::Disintegrated
    };
    RimIntegrityResult {
        state,
        margin,
        ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    #[test]
    fn thriving() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 100.0,
                    centrifugal_force: 50.0
                },
                &t()
            )
            .state,
            RimState::Thriving
        );
    }
    #[test]
    fn critical() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 100.0,
                    centrifugal_force: 100.0
                },
                &t()
            )
            .state,
            RimState::Critical
        );
    }
    #[test]
    fn disintegrated() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 40.0,
                    centrifugal_force: 100.0
                },
                &t()
            )
            .state,
            RimState::Disintegrated
        );
    }
    #[test]
    fn both_zero() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 0.0,
                    centrifugal_force: 0.0
                },
                &t()
            )
            .state,
            RimState::Critical
        );
    }
    #[test]
    fn boundary_thriving() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 111.0,
                    centrifugal_force: 100.0
                },
                &t()
            )
            .state,
            RimState::Thriving
        );
    }
    #[test]
    fn boundary_disintegrated() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 89.0,
                    centrifugal_force: 100.0
                },
                &t()
            )
            .state,
            RimState::Disintegrated
        );
    }
    #[test]
    fn zero_centrifugal() {
        assert_eq!(
            evaluate(
                &RimInput {
                    tensile_strength: 50.0,
                    centrifugal_force: 0.0
                },
                &t()
            )
            .state,
            RimState::Thriving
        );
    }
}
