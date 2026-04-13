//! # Flight and Control Types
//! Shared state between perception, power, and flight/exo controllers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightCommand {
    /// Desired attitude/thrust vector.
    pub target_vector: [f32; 3],
    /// Power-shedding override from the PowerEngine.
    pub power_constraint: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExoStatus {
    /// Actuator engagement state.
    pub engagement: f32,
    /// Muscle intent signal from perception module.
    pub intent_index: u8,
}
