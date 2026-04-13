//! Strain gauge sensor abstraction.
//!
//! Measures mechanical strain (deformation) on suit structural elements.
//! Used for load monitoring, fatigue detection, and structural health.
//!
//! ## T1 Grounding
//! - `ς` (state) — strain is a state variable (deformation ratio)
//! - `∝` (irreversibility) — plastic deformation is irreversible
//! - `ν` (frequency) — cyclic loading frequency drives fatigue

/// Single strain gauge reading.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct StrainReading {
    /// Microstrain (µε = µm/m). Positive = tension, negative = compression.
    pub microstrain: f32,
    /// Bridge excitation voltage (V) for validation.
    pub excitation_v: f32,
    /// Sensor temperature (°C) for thermal compensation.
    pub temperature_c: f32,
    /// Monotonic timestamp (microseconds since boot).
    pub timestamp_us: u64,
}

/// Strain gauge configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StrainGaugeConfig {
    /// Gauge factor (dimensionless, typically 2.0-2.2 for metal foil).
    pub gauge_factor: f32,
    /// Bridge configuration.
    pub bridge_type: BridgeType,
    /// Full-scale range (µε).
    pub max_microstrain: f32,
    /// Sample rate (Hz).
    pub sample_rate_hz: u16,
    /// Location description on the suit structure.
    pub location: String,
}

/// Wheatstone bridge configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BridgeType {
    /// Single active gauge, 3 dummy resistors.
    Quarter,
    /// Two active gauges (bending or Poisson).
    Half,
    /// Four active gauges (maximum sensitivity).
    Full,
}

/// Sensitivity multiplier for bridge type.
pub fn bridge_sensitivity(bridge: BridgeType) -> f32 {
    match bridge {
        BridgeType::Quarter => 1.0,
        BridgeType::Half => 2.0,
        BridgeType::Full => 4.0,
    }
}

/// Convert raw voltage ratio (ΔV/V) to microstrain.
pub fn voltage_to_microstrain(delta_v_ratio: f32, gauge_factor: f32, bridge: BridgeType) -> f32 {
    let sensitivity = bridge_sensitivity(bridge);
    (delta_v_ratio * 1_000_000.0) / (gauge_factor * sensitivity)
}

/// Check if strain exceeds yield threshold (structural alarm).
pub fn is_yield_alarm(reading: &StrainReading, yield_microstrain: f32) -> bool {
    reading.microstrain.abs() > yield_microstrain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_sensitivity() {
        assert!((bridge_sensitivity(BridgeType::Quarter) - 1.0).abs() < 1e-6);
        assert!((bridge_sensitivity(BridgeType::Full) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_voltage_to_microstrain() {
        // 1 mV/V with GF=2.0, quarter bridge → 500 µε
        let result = voltage_to_microstrain(0.001, 2.0, BridgeType::Quarter);
        assert!((result - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_yield_alarm() {
        let reading = StrainReading {
            microstrain: 2500.0,
            excitation_v: 5.0,
            temperature_c: 25.0,
            timestamp_us: 0,
        };
        assert!(is_yield_alarm(&reading, 2000.0));
        assert!(!is_yield_alarm(&reading, 3000.0));
    }
}
