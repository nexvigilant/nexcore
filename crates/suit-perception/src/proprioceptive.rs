use serde::{Deserialize, Serialize};

/// Tracks the suit's awareness of its own mechanical configuration and user health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyState {
    /// Absolute joint positions in radians (ordered by kinematic chain)
    pub joint_angles: Vec<f32>,
    /// Joint velocities in rad/s
    pub joint_velocities: Vec<f32>,
    /// User's heart rate in beats per minute
    pub heart_rate: u8,
    /// User's blood oxygen saturation level (percent)
    pub spo2: u8,
    /// Foot pressure distribution (normalized weights per sensor point)
    pub foot_pressure: Vec<f32>,
}

/// Interface for joint position encoders.
pub trait JointEncoder {
    /// Reads the absolute angle of a specific joint.
    fn get_angle(&mut self, joint_id: usize) -> Result<f32, nexcore_error::NexError>;
}

/// Interface for Electromyography (EMG) electrodes (user intent).
pub trait EmgSensor {
    /// Reads raw EMG signal intensity for intent classification.
    fn get_signal(&mut self) -> Result<Vec<f32>, nexcore_error::NexError>;
}

/// Interface for foot pressure insoles.
pub trait PressureInsoles {
    /// Reads current pressure distribution across foot contact points.
    fn get_pressure_distribution(&mut self) -> Result<Vec<f32>, nexcore_error::NexError>;
}

/// Interface for physiological sensors (Heart rate, SpO2).
pub trait VitalSigns {
    /// Reads current heart rate (BPM) and blood oxygen (SpO2).
    fn get_vitals(&mut self) -> Result<(u8, u8), nexcore_error::NexError>;
}
