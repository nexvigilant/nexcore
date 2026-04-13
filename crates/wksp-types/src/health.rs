//! # Health & Physiological Types
//! Shared state for user health monitoring and thermal status.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Heart rate in BPM
    pub heart_rate: u8,
    /// SpO2 percentage
    pub spo2: u8,
    /// Thermal stress index (0.0 - 1.0)
    pub thermal_stress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthAlert {
    /// High heart rate threshold exceeded
    HighHR,
    /// Low oxygen levels detected
    LowO2,
    /// Thermal limit reached
    ThermalWarning,
}
