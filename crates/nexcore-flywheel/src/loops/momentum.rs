//! Loop 2: Momentum Conservation — Inertial Persistence.
//!
//! ## T1 Primitive Grounding: ν (Frequency) + ∂ (Boundary)

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumInput {
    pub inertia: f64,
    pub omega: f64,
    pub friction_drain: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumResult {
    pub l: f64,
    pub classification: MomentumClassification,
    pub above_critical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MomentumClassification {
    High,
    Normal,
    Low,
    Stalled,
}

impl std::fmt::Display for MomentumClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Normal => write!(f, "normal"),
            Self::Low => write!(f, "low"),
            Self::Stalled => write!(f, "stalled"),
        }
    }
}

pub fn evaluate(input: &MomentumInput, thresholds: &FlywheelThresholds) -> MomentumResult {
    let l = input.inertia * input.omega - input.friction_drain;
    let critical = thresholds.min_momentum_for_stability;
    let classification = if l > 2.0 * critical {
        MomentumClassification::High
    } else if l > critical {
        MomentumClassification::Normal
    } else if l > 0.5 * critical {
        MomentumClassification::Low
    } else {
        MomentumClassification::Stalled
    };
    MomentumResult {
        l,
        classification,
        above_critical: l > critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    #[test]
    fn high() {
        let r = evaluate(
            &MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            &t(),
        );
        assert_eq!(r.classification, MomentumClassification::High);
        assert!(r.above_critical);
    }
    #[test]
    fn normal() {
        assert_eq!(
            evaluate(
                &MomentumInput {
                    inertia: 12.0,
                    omega: 5.0,
                    friction_drain: 0.0
                },
                &t()
            )
            .classification,
            MomentumClassification::Normal
        );
    }
    #[test]
    fn low() {
        assert_eq!(
            evaluate(
                &MomentumInput {
                    inertia: 10.0,
                    omega: 3.0,
                    friction_drain: 0.0
                },
                &t()
            )
            .classification,
            MomentumClassification::Low
        );
    }
    #[test]
    fn stalled() {
        assert_eq!(
            evaluate(
                &MomentumInput {
                    inertia: 1.0,
                    omega: 1.0,
                    friction_drain: 5.0
                },
                &t()
            )
            .classification,
            MomentumClassification::Stalled
        );
    }
    #[test]
    fn friction_drains() {
        assert_eq!(
            evaluate(
                &MomentumInput {
                    inertia: 10.0,
                    omega: 5.0,
                    friction_drain: 50.0
                },
                &t()
            )
            .classification,
            MomentumClassification::Stalled
        );
    }
    #[test]
    fn at_boundary() {
        let r = evaluate(
            &MomentumInput {
                inertia: 10.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            &t(),
        );
        assert_eq!(r.classification, MomentumClassification::Low);
        assert!(!r.above_critical);
    }
    #[test]
    fn just_above() {
        assert_eq!(
            evaluate(
                &MomentumInput {
                    inertia: 10.0,
                    omega: 5.1,
                    friction_drain: 0.0
                },
                &t()
            )
            .classification,
            MomentumClassification::Normal
        );
    }
}
