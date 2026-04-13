//! # HUD Bridge
//! Maps system telemetry to HUD components.

use crate::hud::{SystemsGlanceable, ThermalStatus};
use wksp_types::power::PowerStatusMessage;

/// Bridges raw telemetry to HUD-ready status.
pub fn map_power_to_hud(status: &PowerStatusMessage) -> SystemsGlanceable {
    // Basic mapping: logic for translating raw status to UI representation.
    SystemsGlanceable {
        power: status.soc,
        thermal: if status.power_state > 2 {
            ThermalStatus::Warning
        } else {
            ThermalStatus::Nominal
        },
        integrity: 1.0, // TODO: map integrity from diagnostics
        o2_minutes: None,
        comms_quality: 1.0,
        subsystems_active: 12,
        subsystems_total: 12,
    }
}
