//! Pressure regulation and monitoring.
//!
//! ## Biological Model
//!
//! Blood pressure = cardiac output × peripheral resistance.
//! The baroreceptor reflex is a **negative feedback loop**:
//! - High BP → baroreceptors fire → reduce heart rate + dilate vessels
//! - Low BP → baroreceptors quiet → increase heart rate + constrict vessels
//!
//! ## Pathology Detection
//!
//! Sustained deviation from set point triggers pathology classification.
//!
//! ## T1 Grounding: κ (Comparison) dominant
//!
//! Pressure regulation IS comparison — constantly comparing current
//! pressure against the homeostatic set point and adjusting.

use serde::{Deserialize, Serialize};

use crate::types::{Pathology, Pressure};

/// Pressure set points and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureConfig {
    /// Normal/target pressure (homeostatic set point)
    pub set_point: Pressure,
    /// Above this = hypertension
    pub hypertension_threshold: f64,
    /// Below this = hypotension
    pub hypotension_threshold: f64,
    /// Cycles of sustained deviation before pathology
    pub pathology_threshold: u64,
}

impl Default for PressureConfig {
    fn default() -> Self {
        Self {
            set_point: Pressure::new(0.5),
            hypertension_threshold: 0.8,
            hypotension_threshold: 0.2,
            pathology_threshold: 10, // 10 sustained cycles
        }
    }
}

/// Baroreceptor — pressure sensor with reflex response.
///
/// Negative feedback loop:
/// - Set point: `config.set_point`
/// - Sensor: measures current pressure
/// - Effector: adjusts heart rate and vessel tone
/// - Feedback: negative (self-correcting)
///
/// # Tier: T2-C (κ · ν · → · ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baroreceptor {
    /// Configuration
    pub config: PressureConfig,
    /// History of pressure readings
    pub history: Vec<PressureReading>,
    /// Count of sustained hypertension cycles
    pub hypertension_count: u64,
    /// Count of sustained hypotension cycles
    pub hypotension_count: u64,
}

/// A single pressure reading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PressureReading {
    /// The pressure value
    pub pressure: Pressure,
    /// Classification of this reading
    pub classification: PressureClass,
}

/// Pressure classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureClass {
    /// Below normal
    Hypotensive,
    /// Normal range
    Normal,
    /// Above normal
    Hypertensive,
}

impl Default for Baroreceptor {
    fn default() -> Self {
        Self {
            config: PressureConfig::default(),
            history: Vec::new(),
            hypertension_count: 0,
            hypotension_count: 0,
        }
    }
}

impl Baroreceptor {
    /// Create a new baroreceptor with default config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom config.
    pub fn with_config(config: PressureConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Sense current pressure and classify it.
    pub fn sense(&mut self, pressure: Pressure) -> PressureReading {
        let classification = if pressure.is_hypertensive(self.config.hypertension_threshold) {
            self.hypertension_count += 1;
            self.hypotension_count = 0;
            PressureClass::Hypertensive
        } else if pressure.is_hypotensive(self.config.hypotension_threshold) {
            self.hypotension_count += 1;
            self.hypertension_count = 0;
            PressureClass::Hypotensive
        } else {
            self.hypertension_count = 0;
            self.hypotension_count = 0;
            PressureClass::Normal
        };

        let reading = PressureReading {
            pressure,
            classification,
        };
        self.history.push(reading);

        // Keep history bounded
        if self.history.len() > 100 {
            self.history.drain(0..50);
        }

        reading
    }

    /// Compute the reflex response: how much to adjust heart rate.
    ///
    /// Returns a multiplier:
    /// - < 1.0 = reduce rate (high pressure)
    /// - 1.0 = no change (normal)
    /// - > 1.0 = increase rate (low pressure)
    pub fn reflex_response(&self, current: Pressure) -> f64 {
        let error = self.config.set_point.value() - current.value();
        // Proportional response, clamped
        (1.0 + error).clamp(0.5, 2.0)
    }

    /// Check for sustained pathology.
    pub fn detect_pathology(&self) -> Option<Pathology> {
        if self.hypertension_count >= self.config.pathology_threshold {
            let latest = self.history.last().map(|r| r.pressure).unwrap_or_default();
            Some(Pathology::Hypertension {
                pressure: latest,
                duration: self.hypertension_count,
            })
        } else if self.hypotension_count >= self.config.pathology_threshold {
            let latest = self.history.last().map(|r| r.pressure).unwrap_or_default();
            Some(Pathology::Hypotension { pressure: latest })
        } else {
            None
        }
    }

    /// Number of readings taken.
    pub fn reading_count(&self) -> usize {
        self.history.len()
    }

    /// Latest reading.
    pub fn latest(&self) -> Option<&PressureReading> {
        self.history.last()
    }
}

/// Compute blood pressure from cardiac output and resistance.
///
/// BP = CO × PR (Ohm's law analog)
///
/// Maps to: system_load = throughput × downstream_latency
pub fn compute_blood_pressure(cardiac_output: f64, peripheral_resistance: f64) -> Pressure {
    Pressure::new(cardiac_output * peripheral_resistance)
}
