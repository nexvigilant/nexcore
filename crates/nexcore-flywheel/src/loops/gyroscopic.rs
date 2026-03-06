//! Loop 4: Gyroscopic Stability — Mission Alignment.
//!
//! ## T1 Primitive Grounding: κ (Comparison) + ∂ (Boundary) + ∝ (Proportionality)

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GyroscopicInput {
    pub momentum_l: f64,
    pub perturbation_torque: f64,
    pub critical_momentum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GyroscopicResult {
    pub score: f64,
    pub state: GyroscopicState,
    pub stability_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GyroscopicState {
    Stable,
    Precessing,
    GimbalLock,
    NoStability,
}

impl std::fmt::Display for GyroscopicState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Precessing => write!(f, "precessing"),
            Self::GimbalLock => write!(f, "gimbal_lock"),
            Self::NoStability => write!(f, "no_stability"),
        }
    }
}

pub fn evaluate(input: &GyroscopicInput, thresholds: &FlywheelThresholds) -> GyroscopicResult {
    let l_abs = input.momentum_l.abs();
    let critical = input
        .critical_momentum
        .max(thresholds.min_momentum_for_stability);

    if l_abs < critical {
        return GyroscopicResult {
            score: 0.0,
            state: GyroscopicState::NoStability,
            stability_ratio: 0.0,
        };
    }
    let perturbation_abs = input.perturbation_torque.abs();
    if perturbation_abs < f64::EPSILON {
        return GyroscopicResult {
            score: 1.0,
            state: GyroscopicState::Stable,
            stability_ratio: f64::MAX,
        };
    }
    let stability_ratio = l_abs / perturbation_abs;
    if stability_ratio > thresholds.gyroscopic_stable_ratio {
        let score = (1.0 - perturbation_abs / l_abs).clamp(0.0, 1.0);
        GyroscopicResult {
            score,
            state: GyroscopicState::Stable,
            stability_ratio,
        }
    } else if stability_ratio > 1.0 {
        let score = (1.0 - perturbation_abs / l_abs).clamp(0.0, 1.0);
        GyroscopicResult {
            score,
            state: GyroscopicState::Precessing,
            stability_ratio,
        }
    } else {
        GyroscopicResult {
            score: 0.0,
            state: GyroscopicState::GimbalLock,
            stability_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    #[test]
    fn stable() {
        let r = evaluate(
            &GyroscopicInput {
                momentum_l: 100.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            &t(),
        );
        assert_eq!(r.state, GyroscopicState::Stable);
        assert!((r.score - 0.9).abs() < f64::EPSILON);
    }
    #[test]
    fn no_stability() {
        assert_eq!(
            evaluate(
                &GyroscopicInput {
                    momentum_l: 10.0,
                    perturbation_torque: 5.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .state,
            GyroscopicState::NoStability
        );
    }
    #[test]
    fn gimbal_lock() {
        assert_eq!(
            evaluate(
                &GyroscopicInput {
                    momentum_l: 60.0,
                    perturbation_torque: 100.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .state,
            GyroscopicState::GimbalLock
        );
    }
    #[test]
    fn zero_perturbation() {
        assert!(
            (evaluate(
                &GyroscopicInput {
                    momentum_l: 100.0,
                    perturbation_torque: 0.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .score
                - 1.0)
                .abs()
                < f64::EPSILON
        );
    }
    #[test]
    fn precessing() {
        assert_eq!(
            evaluate(
                &GyroscopicInput {
                    momentum_l: 100.0,
                    perturbation_torque: 60.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .state,
            GyroscopicState::Precessing
        );
    }
    #[test]
    fn at_boundary() {
        assert_eq!(
            evaluate(
                &GyroscopicInput {
                    momentum_l: 50.0,
                    perturbation_torque: 20.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .state,
            GyroscopicState::Stable
        );
    }
    #[test]
    fn just_above() {
        assert_eq!(
            evaluate(
                &GyroscopicInput {
                    momentum_l: 51.0,
                    perturbation_torque: 20.0,
                    critical_momentum: 50.0
                },
                &t()
            )
            .state,
            GyroscopicState::Stable
        );
    }
}
