//! # Exteroceptive Sensing
//!
//! Interfaces for external sensors: cameras, LiDAR, radar, and thermal.

use serde::{Deserialize, Serialize};

/// Represents an object or terrain feature detected outside the suit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    /// Distance to the obstacle (meters)
    pub distance: f32,
    /// Bearing to the obstacle relative to suit forward (radians)
    pub bearing: f32,
    /// Classification of the obstacle (e.g., "Human", "Vehicle", "Wall")
    pub classification: String,
    /// Thermal signature level (normalized 0.0 - 1.0)
    pub thermal_signature: f32,
}

/// Point cloud data from depth sensors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    /// Flattened array of X, Y, Z coordinates
    pub points: Vec<f32>,
    /// Confidence scores for each point
    pub confidence: Vec<f32>,
}

/// Interface for 360° spherical camera systems.
pub trait CameraRing {
    /// Reads the latest spherical image frame.
    fn get_spherical_frame(&mut self) -> Result<Vec<u8>, nexcore_error::NexError>;
}

/// Interface for downward-facing LiDAR sensors.
pub trait DownwardLiDAR {
    /// Reads the current ground distance (AGL) and returns the terrain profile.
    fn get_ground_profile(&mut self) -> Result<PointCloud, nexcore_error::NexError>;
}

/// Interface for thermal imaging sensors (e.g., FLIR Lepton).
pub trait ThermalCamera {
    /// Reads the latest thermal image frame with heat intensity mapped to values.
    fn get_thermal_frame(&mut self) -> Result<Vec<f32>, nexcore_error::NexError>;
}

/// Interface for radar/ultrasonic proximity sensors.
pub trait ProximitySensor {
    /// Reads current proximity measurements.
    fn get_distance(&mut self) -> Result<f32, nexcore_error::NexError>;
}
