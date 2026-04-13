//! # Perception Types
//! Shared perception data structures for middleware communication.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InertialMessage {
    pub acceleration: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub heading: f32,
    pub altitude_agl: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyMessage {
    pub joint_angles: Vec<f32>,
    pub heart_rate: u8,
}
