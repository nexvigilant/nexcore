//! Loop 3: Friction Dissipation — Parasitic Drain (DEGENERATIVE).
//!
//! ## T1 Primitive Grounding: ν (Frequency) + ∝ (Proportionality)

use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionInput {
    pub manual_processes: f64,
    pub human_touchpoints: f64,
    pub velocity: f64,
    pub automation_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionResult {
    pub contact_friction: f64,
    pub aero_drag: f64,
    pub total_drain: f64,
    pub net_drain: f64,
    pub classification: FrictionClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrictionClassification {
    Acceptable,
    Warning,
    Critical,
}

impl std::fmt::Display for FrictionClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Acceptable => write!(f, "acceptable"),
            Self::Warning => write!(f, "warning"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

pub fn evaluate(input: &FrictionInput, thresholds: &FlywheelThresholds) -> FrictionResult {
    let contact_friction = input.manual_processes * input.human_touchpoints;
    let aero_drag = input.velocity.powi(3) * thresholds.drag_coefficient;
    let total_drain = contact_friction + aero_drag;
    let coverage = input.automation_coverage.clamp(0.0, 1.0);
    let net_drain = total_drain * (1.0 - coverage);
    let classification = if net_drain < thresholds.friction_acceptable_threshold {
        FrictionClassification::Acceptable
    } else if net_drain < thresholds.friction_warning_threshold {
        FrictionClassification::Warning
    } else {
        FrictionClassification::Critical
    };
    FrictionResult {
        contact_friction,
        aero_drag,
        total_drain,
        net_drain,
        classification,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn t() -> FlywheelThresholds {
        FlywheelThresholds::default()
    }

    #[test]
    fn acceptable() {
        assert_eq!(
            evaluate(
                &FrictionInput {
                    manual_processes: 1.0,
                    human_touchpoints: 2.0,
                    velocity: 1.0,
                    automation_coverage: 0.5
                },
                &t()
            )
            .classification,
            FrictionClassification::Acceptable
        );
    }
    #[test]
    fn full_automation() {
        assert!(
            (evaluate(
                &FrictionInput {
                    manual_processes: 100.0,
                    human_touchpoints: 100.0,
                    velocity: 100.0,
                    automation_coverage: 1.0
                },
                &t()
            )
            .net_drain)
                .abs()
                < f64::EPSILON
        );
    }
    #[test]
    fn zero_auto() {
        assert_eq!(
            evaluate(
                &FrictionInput {
                    manual_processes: 5.0,
                    human_touchpoints: 5.0,
                    velocity: 10.0,
                    automation_coverage: 0.0
                },
                &t()
            )
            .classification,
            FrictionClassification::Warning
        );
    }
    #[test]
    fn cubic_drag() {
        let r = evaluate(
            &FrictionInput {
                manual_processes: 0.0,
                human_touchpoints: 0.0,
                velocity: 50.0,
                automation_coverage: 0.0,
            },
            &t(),
        );
        assert_eq!(r.classification, FrictionClassification::Critical);
        assert!((r.aero_drag - 125.0).abs() < f64::EPSILON);
    }
    #[test]
    fn zero_all() {
        assert!(
            (evaluate(
                &FrictionInput {
                    manual_processes: 0.0,
                    human_touchpoints: 0.0,
                    velocity: 0.0,
                    automation_coverage: 0.0
                },
                &t()
            )
            .net_drain)
                .abs()
                < f64::EPSILON
        );
    }
    #[test]
    fn partial() {
        assert_eq!(
            evaluate(
                &FrictionInput {
                    manual_processes: 5.0,
                    human_touchpoints: 5.0,
                    velocity: 5.0,
                    automation_coverage: 0.8
                },
                &t()
            )
            .classification,
            FrictionClassification::Acceptable
        );
    }
    #[test]
    fn clamp() {
        assert!(
            (evaluate(
                &FrictionInput {
                    manual_processes: 10.0,
                    human_touchpoints: 10.0,
                    velocity: 1.0,
                    automation_coverage: 1.5
                },
                &t()
            )
            .net_drain)
                .abs()
                < f64::EPSILON
        );
    }
}
