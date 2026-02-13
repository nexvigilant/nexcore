#![allow(dead_code)]
//! Guardian screen — Slint bindings for homeostasis display.
//!
//! ## Primitive Grounding
//! - ς (State): Guardian loop state visualization
//! - μ (Mapping): state → color, state → label, state → gauge
//! - κ (Comparison): risk level gauge position
//!
//! ## Tier: T3

use nexcore_watch_core::GuardianStatus;

/// Guardian screen view model — data ready for Slint binding.
///
/// ## Primitive: μ (Mapping) — domain → view
/// ## Tier: T2-C
#[derive(Debug, Clone)]
pub struct GuardianViewModel {
    /// State label text — μ (Mapping)
    pub state_label: &'static str,
    /// State color hex — μ (Mapping)
    pub state_color: &'static str,
    /// Risk gauge fraction [0.0, 1.0] — N (Quantity)
    pub risk_fraction: f32,
    /// Iteration count display — N (Quantity)
    pub iteration_text: String,
    /// Sensor count — N (Quantity)
    pub sensors: u32,
    /// Actuator count — N (Quantity)
    pub actuators: u32,
    /// Whether attention indicator should pulse — ∂ (Boundary)
    pub requires_attention: bool,
}

impl GuardianViewModel {
    /// Build view model from Guardian status.
    ///
    /// ## Primitive: μ (Mapping) — GuardianStatus → ViewModel
    /// ## Tier: T2-C
    #[must_use]
    pub fn from_status(status: &GuardianStatus) -> Self {
        Self {
            state_label: status.label(),
            state_color: status.color_hex(),
            risk_fraction: status.risk_level.as_fraction(),
            iteration_text: format!("#{}", status.iteration),
            sensors: status.active_sensors,
            actuators: status.active_actuators,
            requires_attention: status.state.requires_attention(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_watch_core::{GuardianState, RiskLevel};

    #[test]
    fn nominal_view_model() {
        let status = GuardianStatus::nominal();
        let vm = GuardianViewModel::from_status(&status);
        assert_eq!(vm.state_label, "NOMINAL");
        assert_eq!(vm.state_color, "#4CAF50");
        assert!((vm.risk_fraction - 0.0).abs() < f32::EPSILON);
        assert!(!vm.requires_attention);
    }

    #[test]
    fn critical_view_model() {
        let status = GuardianStatus {
            state: GuardianState::Critical,
            iteration: 42,
            active_sensors: 3,
            active_actuators: 2,
            risk_level: RiskLevel::Critical,
            last_tick_ms: 0,
        };
        let vm = GuardianViewModel::from_status(&status);
        assert_eq!(vm.state_label, "CRITICAL");
        assert_eq!(vm.state_color, "#F44336");
        assert!((vm.risk_fraction - 1.0).abs() < f32::EPSILON);
        assert!(vm.requires_attention);
        assert_eq!(vm.iteration_text, "#42");
    }
}
