//! Pilot biometric sensor abstraction.
//!
//! Physiological monitoring: heart rate, SpO2, skin temperature,
//! galvanic skin response, and respiration rate.
//!
//! ## T1 Grounding
//! - `ν` (frequency) — heart rate, respiration rate
//! - `N` (quantity) — SpO2 percentage, temperature
//! - `∂` (boundary) — alarm thresholds define safe operating envelope

/// Consolidated biometric snapshot from all body sensors.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BiometricReading {
    /// Heart rate (beats per minute). `None` if sensor not available.
    pub heart_rate_bpm: Option<f32>,
    /// Blood oxygen saturation (%). `None` if sensor not available.
    pub spo2_pct: Option<f32>,
    /// Skin temperature (°C).
    pub skin_temp_c: Option<f32>,
    /// Core body temperature estimate (°C).
    pub core_temp_c: Option<f32>,
    /// Galvanic skin response (µS — microsiemens). Stress/arousal indicator.
    pub gsr_us: Option<f32>,
    /// Respiration rate (breaths per minute).
    pub respiration_bpm: Option<f32>,
    /// Monotonic timestamp (microseconds since boot).
    pub timestamp_us: u64,
}

/// Biometric alarm thresholds.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiometricThresholds {
    /// Heart rate upper limit (bpm).
    pub hr_max_bpm: f32,
    /// Heart rate lower limit (bpm).
    pub hr_min_bpm: f32,
    /// SpO2 lower limit (%).
    pub spo2_min_pct: f32,
    /// Core temperature upper limit (°C).
    pub core_temp_max_c: f32,
    /// Core temperature lower limit (°C).
    pub core_temp_min_c: f32,
}

impl Default for BiometricThresholds {
    fn default() -> Self {
        Self {
            hr_max_bpm: 180.0,
            hr_min_bpm: 40.0,
            spo2_min_pct: 90.0,
            core_temp_max_c: 39.5,
            core_temp_min_c: 35.0,
        }
    }
}

/// Biometric alarm severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlarmSeverity {
    /// Within normal range.
    Normal,
    /// Approaching limit — advisory.
    Warning,
    /// Threshold exceeded — immediate attention.
    Critical,
}

/// Evaluate biometric reading against thresholds.
pub fn evaluate(reading: &BiometricReading, thresholds: &BiometricThresholds) -> AlarmSeverity {
    // SpO2 critical check
    if let Some(spo2) = reading.spo2_pct {
        if spo2 < thresholds.spo2_min_pct {
            return AlarmSeverity::Critical;
        }
    }
    // Core temperature check
    if let Some(core_t) = reading.core_temp_c {
        if core_t > thresholds.core_temp_max_c || core_t < thresholds.core_temp_min_c {
            return AlarmSeverity::Critical;
        }
    }
    // Heart rate check
    if let Some(hr) = reading.heart_rate_bpm {
        if hr > thresholds.hr_max_bpm || hr < thresholds.hr_min_bpm {
            return AlarmSeverity::Critical;
        }
        // Warning at 90% of max
        if hr > thresholds.hr_max_bpm * 0.9 {
            return AlarmSeverity::Warning;
        }
    }
    AlarmSeverity::Normal
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reading(hr: f32, spo2: f32, core_t: f32) -> BiometricReading {
        BiometricReading {
            heart_rate_bpm: Some(hr),
            spo2_pct: Some(spo2),
            skin_temp_c: Some(33.0),
            core_temp_c: Some(core_t),
            gsr_us: None,
            respiration_bpm: Some(16.0),
            timestamp_us: 0,
        }
    }

    #[test]
    fn test_normal_vitals() {
        let reading = make_reading(72.0, 98.0, 37.0);
        assert_eq!(
            evaluate(&reading, &BiometricThresholds::default()),
            AlarmSeverity::Normal
        );
    }

    #[test]
    fn test_low_spo2_critical() {
        let reading = make_reading(72.0, 85.0, 37.0);
        assert_eq!(
            evaluate(&reading, &BiometricThresholds::default()),
            AlarmSeverity::Critical
        );
    }

    #[test]
    fn test_high_hr_warning() {
        let reading = make_reading(165.0, 98.0, 37.0);
        assert_eq!(
            evaluate(&reading, &BiometricThresholds::default()),
            AlarmSeverity::Warning
        );
    }

    #[test]
    fn test_hyperthermia_critical() {
        let reading = make_reading(72.0, 98.0, 40.0);
        assert_eq!(
            evaluate(&reading, &BiometricThresholds::default()),
            AlarmSeverity::Critical
        );
    }
}
