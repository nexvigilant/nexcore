//! # Flight Control Bridge
//! Translates high-level perception WorldState into FlightCommands and ExoStatus.

use suit_perception::fusion::WorldState;
use suit_perception::intent::Intent;
use wksp_types::control::{ExoStatus, FlightCommand};

/// Bridges high-level perception inputs to physical controller commands.
pub struct ControlBridge;

impl ControlBridge {
    /// Translates perception WorldState into a flight command.
    pub fn translate_perception(state: &WorldState, intent: Intent) -> FlightCommand {
        // Translation logic:
        // Maps the world position/attitude and intent into a thrust/attitude vector.
        let target = [
            -state.position[0], // Compensate for world position
            -state.position[1],
            -state.position[2],
        ];

        FlightCommand {
            target_vector: target,
            power_constraint: if intent == Intent::Bracing { 100 } else { 50 },
        }
    }
}
