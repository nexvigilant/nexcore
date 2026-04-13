//! # Power Middleware Types
//! Maps PowerEngine state to shared workspace types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerStatusMessage {
    pub soc: f32,
    pub current_tier: u8,
    pub power_state: u8,
}
