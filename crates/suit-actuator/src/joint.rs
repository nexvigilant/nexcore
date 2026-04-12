//! Joint classification and actuator sizing.
//!
//! Maps to `actuator-load-classifier` microgram.

use serde::{Deserialize, Serialize};

/// Suit joint locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Joint {
    Shoulder,
    Elbow,
    Wrist,
    Hip,
    Knee,
    Ankle,
    Spine,
    Neck,
}

/// Motion type at a joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Motion {
    Flexion,
    Extension,
    Rotation,
    Stabilization,
}

/// Actuation intensity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Intensity {
    /// Augment human motion, reduce fatigue.
    PassiveAssist,
    /// Normal powered operation.
    ActiveNormal,
    /// Maximum exo-driven effort.
    PeakEffort,
}

/// Torque class determining motor sizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorqueClass {
    /// Hip/knee/ankle/spine at peak: 3kW.
    HighPeak,
    /// Hip/knee/ankle/spine active: 1.5kW.
    HighActive,
    /// Hip/knee/ankle/spine passive: 500W.
    HighPassive,
    /// Shoulder/elbow peak: 1.5kW.
    MediumPeak,
    /// Shoulder/elbow active: 750W.
    MediumActive,
    /// Shoulder/elbow passive: 250W.
    MediumPassive,
    /// Wrist/neck: 100W.
    Low,
}

/// Motor control mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMode {
    /// Standard FOC — smooth torque delivery.
    FieldOrientedControl,
    /// FOC with torque limiting to prevent pilot injury.
    FocWithTorqueLimit,
    /// Spring-damper behavior for natural feel.
    ImpedanceControl,
    /// Precision positioning for fine manipulation.
    PositionControl,
}

/// Actuator classification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuatorSpec {
    pub joint: Joint,
    pub torque_class: TorqueClass,
    pub peak_power_w: u32,
    pub control_mode: ControlMode,
}

impl Joint {
    /// Whether this is a high-torque (lower body + spine) joint.
    pub fn is_high_torque(&self) -> bool {
        matches!(self, Self::Hip | Self::Knee | Self::Ankle | Self::Spine)
    }

    /// Whether this is a medium-torque (upper body) joint.
    pub fn is_medium_torque(&self) -> bool {
        matches!(self, Self::Shoulder | Self::Elbow)
    }

    /// Classify actuator requirements.
    pub fn classify(&self, intensity: Intensity) -> ActuatorSpec {
        if self.is_high_torque() {
            match intensity {
                Intensity::PeakEffort => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::HighPeak,
                    peak_power_w: 3000,
                    control_mode: ControlMode::FocWithTorqueLimit,
                },
                Intensity::ActiveNormal => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::HighActive,
                    peak_power_w: 1500,
                    control_mode: ControlMode::FieldOrientedControl,
                },
                Intensity::PassiveAssist => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::HighPassive,
                    peak_power_w: 500,
                    control_mode: ControlMode::ImpedanceControl,
                },
            }
        } else if self.is_medium_torque() {
            match intensity {
                Intensity::PeakEffort => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::MediumPeak,
                    peak_power_w: 1500,
                    control_mode: ControlMode::FocWithTorqueLimit,
                },
                Intensity::ActiveNormal => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::MediumActive,
                    peak_power_w: 750,
                    control_mode: ControlMode::FieldOrientedControl,
                },
                Intensity::PassiveAssist => ActuatorSpec {
                    joint: *self,
                    torque_class: TorqueClass::MediumPassive,
                    peak_power_w: 250,
                    control_mode: ControlMode::ImpedanceControl,
                },
            }
        } else {
            ActuatorSpec {
                joint: *self,
                torque_class: TorqueClass::Low,
                peak_power_w: 100,
                control_mode: ControlMode::PositionControl,
            }
        }
    }

    /// Total peak power for a full suit (all joints at given intensity).
    pub fn full_suit_peak_power_w(intensity: Intensity) -> u32 {
        [
            Self::Hip,
            Self::Hip,
            Self::Knee,
            Self::Knee,
            Self::Ankle,
            Self::Ankle,
            Self::Spine,
            Self::Shoulder,
            Self::Shoulder,
            Self::Elbow,
            Self::Elbow,
            Self::Wrist,
            Self::Wrist,
            Self::Neck,
        ]
        .iter()
        .map(|j| j.classify(intensity).peak_power_w)
        .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knee_peak() {
        let s = Joint::Knee.classify(Intensity::PeakEffort);
        assert_eq!(s.torque_class, TorqueClass::HighPeak);
        assert_eq!(s.peak_power_w, 3000);
    }

    #[test]
    fn test_shoulder_active() {
        let s = Joint::Shoulder.classify(Intensity::ActiveNormal);
        assert_eq!(s.torque_class, TorqueClass::MediumActive);
        assert_eq!(s.peak_power_w, 750);
    }

    #[test]
    fn test_wrist_always_low() {
        let s = Joint::Wrist.classify(Intensity::PeakEffort);
        assert_eq!(s.torque_class, TorqueClass::Low);
        assert_eq!(s.control_mode, ControlMode::PositionControl);
    }

    #[test]
    fn test_passive_uses_impedance() {
        let s = Joint::Hip.classify(Intensity::PassiveAssist);
        assert_eq!(s.control_mode, ControlMode::ImpedanceControl);
    }

    #[test]
    fn test_high_torque_classification() {
        assert!(Joint::Hip.is_high_torque());
        assert!(Joint::Knee.is_high_torque());
        assert!(!Joint::Shoulder.is_high_torque());
        assert!(!Joint::Wrist.is_high_torque());
    }

    #[test]
    fn test_full_suit_power() {
        let peak = Joint::full_suit_peak_power_w(Intensity::PeakEffort);
        // 2×hip(3k) + 2×knee(3k) + 2×ankle(3k) + spine(3k) + 2×shoulder(1.5k) + 2×elbow(1.5k) + 2×wrist(100) + neck(100)
        // = 21000 + 6000 + 400 = 27400
        assert_eq!(peak, 27400);
        let passive = Joint::full_suit_peak_power_w(Intensity::PassiveAssist);
        assert!(passive < peak);
    }
}
