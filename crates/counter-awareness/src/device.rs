//! # Device Configuration, State Management & Measurement System
//!
//! The Device is the top-level T3 composition: a counter-awareness platform
//! with configurable countermeasures, mission profiles, environmental state,
//! and continuous measurement feedback.
//!
//! ## State Machine
//! ```text
//! Idle → Configuring → Armed → Active → Assessing
//!  ↑                                        │
//!  └────────────────────────────────────────┘
//! ```
//!
//! ## Lex Primitiva Grounding
//! - `Device` → ς (State) × π (Persistence) — stateful entity with persistent config
//! - `DeviceState` → Σ (Sum) — enumerated state variants
//! - `Measurement` → N (Quantity) × ν (Frequency) — quantified observations over time
//! - `MeasurementLog` → σ (Sequence) × π (Persistence) — ordered persistent record

use serde::{Deserialize, Serialize};

use crate::detection::DetectionAssessment;
use crate::fusion::{FusionResult, OptimalLoadout, compute_fusion, optimize_loadout};
use crate::matrix::EffectivenessMatrix;
use crate::primitives::{
    CounterPrimitive, Countermeasure, EnergyMode, LatencyClass, SensingPrimitive, SensorSystem,
    SpectralBand,
};

// ── State Machine ──────────────────────────────────────────────────────

/// Device operational state.
///
/// Tier: T1 (maps directly to Σ — sum type with discrete variants)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceState {
    /// Not configured. Accepting configuration commands.
    Idle,
    /// Countermeasures and mission profile being set.
    Configuring,
    /// Configuration locked. Ready to activate.
    Armed,
    /// Countermeasures active. Consuming power.
    Active,
    /// Running detection assessment against threat model.
    Assessing,
}

impl DeviceState {
    /// Valid state transitions.
    pub fn can_transition_to(&self, next: DeviceState) -> bool {
        matches!(
            (self, next),
            (DeviceState::Idle, DeviceState::Configuring)
                | (DeviceState::Configuring, DeviceState::Armed)
                | (DeviceState::Configuring, DeviceState::Idle) // abort config
                | (DeviceState::Armed, DeviceState::Active)
                | (DeviceState::Armed, DeviceState::Idle) // disarm
                | (DeviceState::Active, DeviceState::Assessing)
                | (DeviceState::Active, DeviceState::Idle) // emergency stop
                | (DeviceState::Assessing, DeviceState::Active) // continue ops
                | (DeviceState::Assessing, DeviceState::Idle) // full reset
        )
    }
}

// ── Measurements ───────────────────────────────────────────────────────

/// A single measurement reading from a sensor or state probe.
///
/// Tier: T2-P (N × ν — quantity measured at a frequency)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// What is being measured
    pub metric: MetricKind,
    /// Measured value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Timestamp (seconds since device activation)
    pub timestamp_s: f64,
    /// Measurement confidence [0.0, 1.0]
    pub confidence: f64,
}

/// Categories of measurable quantities.
///
/// Tier: T2-C (composes multiple primitives per variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricKind {
    /// Detection probability from a specific sensor
    DetectionProbability,
    /// Fused detection probability across all threats
    FusedDetectionProbability,
    /// Residual signature in a specific band
    ResidualSignature,
    /// Surface temperature delta (target - ambient)
    ThermalDelta,
    /// Radar cross-section equivalent (m²)
    RadarCrossSection,
    /// Visual contrast ratio against background
    VisualContrast,
    /// Power consumption of active countermeasures (watts)
    PowerDraw,
    /// Weight budget remaining (kg)
    WeightRemaining,
    /// Countermeasure effectiveness degradation over time
    EffectivenessDegradation,
    /// Range to nearest threat sensor
    ThreatRange,
}

/// Aggregated measurement statistics for a metric over a time window.
///
/// Tier: T2-C (σ + N + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementStats {
    pub metric: MetricKind,
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    /// Time window in seconds
    pub window_s: f64,
}

/// Ordered log of measurements with statistical aggregation.
///
/// Tier: T2-C (σ + π — sequence with persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementLog {
    entries: Vec<Measurement>,
    /// Maximum entries before oldest are evicted
    capacity: usize,
}

impl MeasurementLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Record a measurement. Evicts oldest if at capacity.
    pub fn record(&mut self, measurement: Measurement) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(measurement);
    }

    /// Get all measurements of a specific metric kind.
    pub fn by_metric(&self, kind: MetricKind) -> Vec<&Measurement> {
        self.entries.iter().filter(|m| m.metric == kind).collect()
    }

    /// Compute statistics for a metric within a time window.
    pub fn stats(&self, kind: MetricKind, window_s: f64) -> Option<MeasurementStats> {
        let now = self.entries.last().map(|m| m.timestamp_s)?;
        let cutoff = now - window_s;

        let values: Vec<f64> = self
            .entries
            .iter()
            .filter(|m| m.metric == kind && m.timestamp_s >= cutoff)
            .map(|m| m.value)
            .collect();

        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Some(MeasurementStats {
            metric: kind,
            count,
            min,
            max,
            mean,
            std_dev,
            window_s,
        })
    }

    /// Total number of recorded measurements.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get latest measurement of a specific kind.
    pub fn latest(&self, kind: MetricKind) -> Option<&Measurement> {
        self.entries.iter().rev().find(|m| m.metric == kind)
    }

    /// Trend detection: is the metric increasing, decreasing, or stable?
    pub fn trend(&self, kind: MetricKind, window_s: f64) -> Trend {
        let now = match self.entries.last() {
            Some(m) => m.timestamp_s,
            None => return Trend::Insufficient,
        };
        let cutoff = now - window_s;

        let values: Vec<(f64, f64)> = self
            .entries
            .iter()
            .filter(|m| m.metric == kind && m.timestamp_s >= cutoff)
            .map(|m| (m.timestamp_s, m.value))
            .collect();

        if values.len() < 3 {
            return Trend::Insufficient;
        }

        // Simple linear regression slope
        let n = values.len() as f64;
        let sum_x: f64 = values.iter().map(|(t, _)| t).sum();
        let sum_y: f64 = values.iter().map(|(_, v)| v).sum();
        let sum_xy: f64 = values.iter().map(|(t, v)| t * v).sum();
        let sum_x2: f64 = values.iter().map(|(t, _)| t * t).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            return Trend::Stable;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;

        if slope > 0.01 {
            Trend::Increasing { slope }
        } else if slope < -0.01 {
            Trend::Decreasing { slope }
        } else {
            Trend::Stable
        }
    }
}

/// Trend direction for a measurement series.
///
/// Tier: T2-P (→ Causality — direction of change over time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Increasing { slope: f64 },
    Decreasing { slope: f64 },
    Stable,
    Insufficient,
}

// ── Environment ────────────────────────────────────────────────────────

/// Environmental conditions affecting detection and counter-detection.
///
/// Tier: T2-C (composes multiple physical primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// Ambient temperature in Celsius
    pub ambient_temp_c: f64,
    /// Visibility in meters (affected by fog, rain, smoke)
    pub visibility_m: f64,
    /// Wind speed in m/s (affects thermal plume dispersion)
    pub wind_speed_ms: f64,
    /// Time of day (affects EO effectiveness)
    pub is_daytime: bool,
    /// Precipitation level [0.0 = none, 1.0 = heavy]
    pub precipitation: f64,
    /// Background clutter density [0.0 = open, 1.0 = dense urban]
    pub clutter: f64,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            ambient_temp_c: 20.0,
            visibility_m: 10000.0,
            wind_speed_ms: 5.0,
            is_daytime: true,
            precipitation: 0.0,
            clutter: 0.3,
        }
    }
}

impl Environment {
    /// Environmental modifier for EO detection [0.0, 1.0].
    /// Low visibility or nighttime reduces EO effectiveness.
    pub fn eo_modifier(&self) -> f64 {
        let vis_factor = (self.visibility_m / 10000.0).min(1.0);
        let day_factor = if self.is_daytime { 1.0 } else { 0.3 };
        let precip_factor = 1.0 - 0.5 * self.precipitation;
        vis_factor * day_factor * precip_factor
    }

    /// Environmental modifier for IR detection [0.0, 1.0].
    /// Higher ambient temperature reduces thermal contrast.
    pub fn ir_modifier(&self) -> f64 {
        // Thermal contrast decreases as ambient approaches body/engine temp
        let thermal_contrast = ((80.0 - self.ambient_temp_c) / 80.0).clamp(0.2, 1.0);
        let wind_dispersion = (1.0 - self.wind_speed_ms / 30.0).clamp(0.3, 1.0);
        thermal_contrast * wind_dispersion
    }

    /// Environmental modifier for radar detection [0.0, 1.0].
    /// Radar is relatively weather-independent.
    pub fn radar_modifier(&self) -> f64 {
        let precip_factor = 1.0 - 0.15 * self.precipitation; // Minor rain clutter
        let clutter_factor = 1.0 - 0.3 * self.clutter; // Ground clutter
        precip_factor * clutter_factor
    }
}

// ── Mission Profile ────────────────────────────────────────────────────

/// Mission parameters that constrain countermeasure selection.
///
/// Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionProfile {
    /// Mission name/identifier
    pub name: String,
    /// Maximum weight budget for countermeasures (kg)
    pub weight_budget_kg: f64,
    /// Maximum power budget for active countermeasures (watts)
    pub power_budget_w: f64,
    /// Expected engagement range (meters)
    pub engagement_range_m: f64,
    /// Latency requirement for sensor avoidance maneuvers
    pub latency_class: LatencyClass,
    /// Target raw signature strength [0.0, 1.0]
    pub raw_signature: f64,
    /// Acceptable detection probability threshold
    pub max_acceptable_detection: f64,
}

// ── Device ─────────────────────────────────────────────────────────────

/// The top-level counter-awareness device.
///
/// Composes countermeasures, threat model, environment, and measurement
/// into a stateful system with assessment capabilities.
///
/// Tier: T3 (full domain composition)
///
/// ## Lex Primitiva Grounding
/// GroundsTo: {ς, π, Σ, μ, N, κ}
/// - ς (State): DeviceState machine
/// - π (Persistence): MeasurementLog
/// - Σ (Sum): Enumerated states + sensor fusion
/// - μ (Mapping): Effectiveness matrix
/// - N (Quantity): All measurements
/// - κ (Comparison): Detection threshold comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device identifier
    pub name: String,
    /// Current operational state
    state: DeviceState,
    /// Effectiveness matrix (configurable)
    matrix: EffectivenessMatrix,
    /// Installed countermeasures
    countermeasures: Vec<Countermeasure>,
    /// Known threat sensors to defend against
    threat_sensors: Vec<SensorSystem>,
    /// Current environment
    environment: Environment,
    /// Mission parameters
    mission: Option<MissionProfile>,
    /// Measurement log with capacity
    measurements: MeasurementLog,
    /// State transition history
    state_history: Vec<(DeviceState, f64)>, // (state, timestamp)
    /// Current simulation time
    clock_s: f64,
}

/// Error type for device operations.
///
/// Tier: T2-C
#[derive(Debug, Clone, nexcore_error::Error, Serialize, Deserialize)]
pub enum DeviceError {
    #[error("invalid state transition: {from:?} → {to:?}")]
    InvalidTransition { from: DeviceState, to: DeviceState },
    #[error("device not in required state {required:?}, currently {current:?}")]
    WrongState {
        required: DeviceState,
        current: DeviceState,
    },
    #[error("weight budget exceeded: {used:.1} kg > {budget:.1} kg")]
    WeightExceeded { used: f64, budget: f64 },
    #[error("power budget exceeded: {used:.1} W > {budget:.1} W")]
    PowerExceeded { used: f64, budget: f64 },
    #[error("no mission profile configured")]
    NoMission,
    #[error("no threat sensors configured")]
    NoThreats,
}

/// Full device assessment report.
///
/// Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentReport {
    /// Device name
    pub device_name: String,
    /// Device state at time of assessment
    pub device_state: DeviceState,
    /// Environment conditions
    pub environment: Environment,
    /// Mission profile used
    pub mission_name: String,
    /// Active countermeasures
    pub active_countermeasures: Vec<String>,
    /// Active counter-primitives
    pub active_counter_primitives: Vec<CounterPrimitive>,
    /// Per-sensor detection assessments
    pub sensor_assessments: Vec<DetectionAssessment>,
    /// Fused detection result
    pub fusion: FusionResult,
    /// Optimal loadout recommendation
    pub recommended_loadout: OptimalLoadout,
    /// Whether current config meets mission threshold
    pub mission_compliant: bool,
    /// Gap: how much detection probability exceeds acceptable threshold
    pub detection_gap: f64,
    /// Latest measurement stats
    pub measurement_summary: Vec<MeasurementStats>,
}

impl Device {
    /// Create a new device in Idle state.
    pub fn new(name: impl Into<String>, measurement_capacity: usize) -> Self {
        Self {
            name: name.into(),
            state: DeviceState::Idle,
            matrix: EffectivenessMatrix::default_physics(),
            countermeasures: Vec::new(),
            threat_sensors: Vec::new(),
            environment: Environment::default(),
            mission: None,
            measurements: MeasurementLog::new(measurement_capacity),
            state_history: vec![(DeviceState::Idle, 0.0)],
            clock_s: 0.0,
        }
    }

    /// Current device state.
    pub fn state(&self) -> DeviceState {
        self.state
    }

    /// Advance the simulation clock.
    pub fn tick(&mut self, delta_s: f64) {
        self.clock_s += delta_s;
    }

    /// Transition to a new state.
    pub fn transition(&mut self, next: DeviceState) -> Result<(), DeviceError> {
        if !self.state.can_transition_to(next) {
            return Err(DeviceError::InvalidTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        self.state_history.push((next, self.clock_s));
        Ok(())
    }

    /// Set custom effectiveness matrix.
    pub fn set_matrix(&mut self, matrix: EffectivenessMatrix) {
        self.matrix = matrix;
    }

    /// Add a countermeasure to the device.
    pub fn add_countermeasure(&mut self, cm: Countermeasure) -> Result<(), DeviceError> {
        if self.state != DeviceState::Configuring && self.state != DeviceState::Idle {
            return Err(DeviceError::WrongState {
                required: DeviceState::Configuring,
                current: self.state,
            });
        }
        self.countermeasures.push(cm);
        Ok(())
    }

    /// Add a threat sensor to defend against.
    pub fn add_threat_sensor(&mut self, sensor: SensorSystem) -> Result<(), DeviceError> {
        if self.state != DeviceState::Configuring && self.state != DeviceState::Idle {
            return Err(DeviceError::WrongState {
                required: DeviceState::Configuring,
                current: self.state,
            });
        }
        self.threat_sensors.push(sensor);
        Ok(())
    }

    /// Set the mission profile.
    pub fn set_mission(&mut self, mission: MissionProfile) -> Result<(), DeviceError> {
        if self.state != DeviceState::Configuring && self.state != DeviceState::Idle {
            return Err(DeviceError::WrongState {
                required: DeviceState::Configuring,
                current: self.state,
            });
        }
        self.mission = Some(mission);
        Ok(())
    }

    /// Update environmental conditions.
    pub fn set_environment(&mut self, env: Environment) {
        self.environment = env;
    }

    /// Record a measurement.
    pub fn record_measurement(
        &mut self,
        metric: MetricKind,
        value: f64,
        unit: &str,
        confidence: f64,
    ) {
        self.measurements.record(Measurement {
            metric,
            value,
            unit: unit.into(),
            timestamp_s: self.clock_s,
            confidence,
        });
    }

    /// Get measurement statistics for a metric.
    pub fn measurement_stats(&self, kind: MetricKind, window_s: f64) -> Option<MeasurementStats> {
        self.measurements.stats(kind, window_s)
    }

    /// Get trend for a metric.
    pub fn measurement_trend(&self, kind: MetricKind, window_s: f64) -> Trend {
        self.measurements.trend(kind, window_s)
    }

    /// Get the latest measurement of a metric.
    pub fn latest_measurement(&self, kind: MetricKind) -> Option<&Measurement> {
        self.measurements.latest(kind)
    }

    /// Collect all active counter-primitives from installed countermeasures.
    pub fn active_counters(&self) -> Vec<CounterPrimitive> {
        self.countermeasures
            .iter()
            .flat_map(|cm| cm.primary_counters.clone())
            .collect()
    }

    /// Total weight of installed countermeasures.
    pub fn total_weight(&self) -> f64 {
        self.countermeasures.iter().map(|cm| cm.weight_kg).sum()
    }

    /// Total power draw of active countermeasures.
    pub fn total_power(&self) -> f64 {
        self.countermeasures.iter().map(|cm| cm.power_w).sum()
    }

    /// Validate configuration against mission constraints.
    pub fn validate_config(&self) -> Result<(), DeviceError> {
        let mission = self.mission.as_ref().ok_or(DeviceError::NoMission)?;

        if self.threat_sensors.is_empty() {
            return Err(DeviceError::NoThreats);
        }

        let weight = self.total_weight();
        if weight > mission.weight_budget_kg {
            return Err(DeviceError::WeightExceeded {
                used: weight,
                budget: mission.weight_budget_kg,
            });
        }

        let power = self.total_power();
        if power > mission.power_budget_w {
            return Err(DeviceError::PowerExceeded {
                used: power,
                budget: mission.power_budget_w,
            });
        }

        Ok(())
    }

    /// Run a full detection assessment.
    ///
    /// Requires: Armed or Active state, mission profile, threat sensors.
    pub fn assess(&mut self) -> Result<AssessmentReport, DeviceError> {
        if self.state != DeviceState::Active
            && self.state != DeviceState::Armed
            && self.state != DeviceState::Assessing
        {
            return Err(DeviceError::WrongState {
                required: DeviceState::Active,
                current: self.state,
            });
        }

        // Extract mission data into owned values to avoid borrow conflict
        // (mission borrows self immutably; record_measurement borrows mutably)
        let mission = self.mission.clone().ok_or(DeviceError::NoMission)?;

        if self.threat_sensors.is_empty() {
            return Err(DeviceError::NoThreats);
        }

        let counters = self.active_counters();

        // Compute fusion
        let fusion = compute_fusion(
            &self.threat_sensors,
            &counters,
            &self.matrix,
            mission.engagement_range_m,
            mission.raw_signature,
            mission.max_acceptable_detection,
        );

        // Compute optimal loadout
        let optimal = optimize_loadout(
            &self.threat_sensors,
            &self.countermeasures,
            &self.matrix,
            mission.weight_budget_kg,
            mission.engagement_range_m,
            mission.raw_signature,
        );

        let mission_compliant = fusion.fused_probability <= mission.max_acceptable_detection;
        let detection_gap = if mission_compliant {
            0.0
        } else {
            fusion.fused_probability - mission.max_acceptable_detection
        };

        // Collect per-sensor detection probabilities before mutable borrow
        let sensor_probs: Vec<f64> = fusion
            .sensor_assessments
            .iter()
            .map(|a| a.detection_probability)
            .collect();

        // Pre-compute values that need immutable self
        let power = self.total_power();
        let weight_remaining = mission.weight_budget_kg - self.total_weight();

        // Record measurements (mutable borrow region)
        self.record_measurement(
            MetricKind::FusedDetectionProbability,
            fusion.fused_probability,
            "probability",
            0.9,
        );
        self.record_measurement(MetricKind::PowerDraw, power, "watts", 1.0);
        self.record_measurement(MetricKind::WeightRemaining, weight_remaining, "kg", 1.0);

        for prob in &sensor_probs {
            self.record_measurement(MetricKind::DetectionProbability, *prob, "probability", 0.85);
        }

        // Compute measurement summary
        let summary_metrics = [
            MetricKind::FusedDetectionProbability,
            MetricKind::DetectionProbability,
            MetricKind::PowerDraw,
            MetricKind::WeightRemaining,
        ];
        let measurement_summary: Vec<MeasurementStats> = summary_metrics
            .iter()
            .filter_map(|kind| self.measurements.stats(*kind, 60.0))
            .collect();

        Ok(AssessmentReport {
            device_name: self.name.clone(),
            device_state: self.state,
            environment: self.environment.clone(),
            mission_name: mission.name.clone(),
            active_countermeasures: self
                .countermeasures
                .iter()
                .map(|cm| cm.name.clone())
                .collect(),
            active_counter_primitives: counters,
            sensor_assessments: fusion.sensor_assessments.clone(),
            fusion,
            recommended_loadout: optimal,
            mission_compliant,
            detection_gap,
            measurement_summary,
        })
    }

    /// Get the full state history.
    pub fn state_history(&self) -> &[(DeviceState, f64)] {
        &self.state_history
    }

    /// Current clock time.
    pub fn clock(&self) -> f64 {
        self.clock_s
    }

    /// Number of installed countermeasures.
    pub fn countermeasure_count(&self) -> usize {
        self.countermeasures.len()
    }

    /// Number of threat sensors modeled.
    pub fn threat_count(&self) -> usize {
        self.threat_sensors.len()
    }
}

// ── Prebuilt Sensor & Countermeasure Catalogs ──────────────────────────

/// Standard threat sensor catalog.
pub mod catalog {
    use super::*;

    pub fn eo_camera() -> SensorSystem {
        SensorSystem {
            name: "EO Camera (Visible)".into(),
            energy_mode: EnergyMode::Passive,
            spectral_band: SpectralBand::Visible,
            latency_class: LatencyClass::RealTime,
            primary_primitives: vec![
                SensingPrimitive::Reflection,
                SensingPrimitive::Contrast,
                SensingPrimitive::Boundary,
                SensingPrimitive::Resolution,
            ],
            max_range_m: 5000.0,
            noise_floor: 0.05,
        }
    }

    pub fn flir_thermal() -> SensorSystem {
        SensorSystem {
            name: "FLIR Thermal Imager".into(),
            energy_mode: EnergyMode::Passive,
            spectral_band: SpectralBand::Infrared,
            latency_class: LatencyClass::RealTime,
            primary_primitives: vec![
                SensingPrimitive::Emission,
                SensingPrimitive::Contrast,
                SensingPrimitive::Boundary,
            ],
            max_range_m: 3000.0,
            noise_floor: 0.08,
        }
    }

    pub fn surveillance_radar() -> SensorSystem {
        SensorSystem {
            name: "Surveillance Radar (X-band)".into(),
            energy_mode: EnergyMode::Active,
            spectral_band: SpectralBand::Microwave,
            latency_class: LatencyClass::NearRealTime,
            primary_primitives: vec![
                SensingPrimitive::Reflection,
                SensingPrimitive::Intensity,
                SensingPrimitive::Distance,
                SensingPrimitive::Frequency, // Doppler
            ],
            max_range_m: 50000.0,
            noise_floor: 0.02,
        }
    }

    pub fn lidar_scanner() -> SensorSystem {
        SensorSystem {
            name: "LiDAR Scanner (1064nm)".into(),
            energy_mode: EnergyMode::Active,
            spectral_band: SpectralBand::NearInfrared,
            latency_class: LatencyClass::NearRealTime,
            primary_primitives: vec![
                SensingPrimitive::Reflection,
                SensingPrimitive::Distance,
                SensingPrimitive::Resolution,
            ],
            max_range_m: 2000.0,
            noise_floor: 0.10,
        }
    }

    pub fn multispectral_imager() -> SensorSystem {
        SensorSystem {
            name: "Multispectral Imager (6-band)".into(),
            energy_mode: EnergyMode::Passive,
            spectral_band: SpectralBand::Multispectral,
            latency_class: LatencyClass::PostProcessed,
            primary_primitives: vec![
                SensingPrimitive::Reflection,
                SensingPrimitive::Contrast,
                SensingPrimitive::Frequency,
            ],
            max_range_m: 8000.0,
            noise_floor: 0.06,
        }
    }

    /// Standard countermeasure catalog entry: Radar-Absorbing Material.
    pub fn ram_coating() -> Countermeasure {
        Countermeasure {
            name: "Radar-Absorbing Material (RAM)".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![CounterPrimitive::Absorption],
            weight_kg: 2.5,
            power_w: 0.0,
            effectiveness: vec![0.85],
        }
    }

    pub fn thermal_insulation() -> Countermeasure {
        Countermeasure {
            name: "Thermal Insulation Layer".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![CounterPrimitive::ThermalEquilibrium],
            weight_kg: 1.8,
            power_w: 0.0,
            effectiveness: vec![0.80],
        }
    }

    pub fn adaptive_camouflage() -> Countermeasure {
        Countermeasure {
            name: "Adaptive Visual Camouflage".into(),
            energy_mode: EnergyMode::Active,
            primary_counters: vec![
                CounterPrimitive::Homogenization,
                CounterPrimitive::Diffusion,
            ],
            weight_kg: 3.2,
            power_w: 45.0,
            effectiveness: vec![0.85, 0.75],
        }
    }

    pub fn exhaust_diffuser() -> Countermeasure {
        Countermeasure {
            name: "Exhaust Heat Diffuser".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![
                CounterPrimitive::ThermalEquilibrium,
                CounterPrimitive::Attenuation,
            ],
            weight_kg: 1.2,
            power_w: 0.0,
            effectiveness: vec![0.70, 0.40],
        }
    }

    pub fn faceted_geometry() -> Countermeasure {
        Countermeasure {
            name: "Faceted Low-Observable Geometry".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![
                CounterPrimitive::Absorption,
                CounterPrimitive::Diffusion,
                CounterPrimitive::SubResolution,
            ],
            weight_kg: 0.0, // Structural, not additive weight
            power_w: 0.0,
            effectiveness: vec![0.60, 0.80, 0.50],
        }
    }

    pub fn ir_suppressor() -> Countermeasure {
        Countermeasure {
            name: "IR Signature Suppressor".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![
                CounterPrimitive::ThermalEquilibrium,
                CounterPrimitive::Homogenization,
            ],
            weight_kg: 2.0,
            power_w: 0.0,
            effectiveness: vec![0.75, 0.50],
        }
    }

    pub fn band_selective_coating() -> Countermeasure {
        Countermeasure {
            name: "Band-Selective Absorptive Coating".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![CounterPrimitive::BandDenial, CounterPrimitive::Absorption],
            weight_kg: 1.5,
            power_w: 0.0,
            effectiveness: vec![0.70, 0.50],
        }
    }

    pub fn compact_form_factor() -> Countermeasure {
        Countermeasure {
            name: "Compact Form Factor Design".into(),
            energy_mode: EnergyMode::Passive,
            primary_counters: vec![
                CounterPrimitive::SubResolution,
                CounterPrimitive::RangeDenial,
            ],
            weight_kg: 0.0, // Structural
            power_w: 0.0,
            effectiveness: vec![0.80, 0.30],
        }
    }

    /// Look up a sensor by snake_case name.
    pub fn lookup_sensor(name: &str) -> Option<SensorSystem> {
        match name.to_ascii_lowercase().replace('-', "_").as_str() {
            "eo_camera" => Some(eo_camera()),
            "flir_thermal" => Some(flir_thermal()),
            "surveillance_radar" => Some(surveillance_radar()),
            "lidar_scanner" => Some(lidar_scanner()),
            "multispectral_imager" => Some(multispectral_imager()),
            _ => None,
        }
    }

    /// Look up a countermeasure by snake_case name.
    pub fn lookup_countermeasure(name: &str) -> Option<Countermeasure> {
        match name.to_ascii_lowercase().replace('-', "_").as_str() {
            "ram_coating" => Some(ram_coating()),
            "thermal_insulation" => Some(thermal_insulation()),
            "adaptive_camouflage" => Some(adaptive_camouflage()),
            "exhaust_diffuser" => Some(exhaust_diffuser()),
            "faceted_geometry" => Some(faceted_geometry()),
            "ir_suppressor" => Some(ir_suppressor()),
            "band_selective_coating" => Some(band_selective_coating()),
            "compact_form_factor" => Some(compact_form_factor()),
            _ => None,
        }
    }

    /// All available sensor names.
    pub fn sensor_names() -> &'static [&'static str] {
        &[
            "eo_camera",
            "flir_thermal",
            "surveillance_radar",
            "lidar_scanner",
            "multispectral_imager",
        ]
    }

    /// All available countermeasure names.
    pub fn countermeasure_names() -> &'static [&'static str] {
        &[
            "ram_coating",
            "thermal_insulation",
            "adaptive_camouflage",
            "exhaust_diffuser",
            "faceted_geometry",
            "ir_suppressor",
            "band_selective_coating",
            "compact_form_factor",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured_device() -> Device {
        let mut dev = Device::new("TestDevice", 1000);

        let result = dev.transition(DeviceState::Configuring);
        assert!(result.is_ok());

        let result = dev.add_threat_sensor(catalog::eo_camera());
        assert!(result.is_ok());
        let result = dev.add_threat_sensor(catalog::flir_thermal());
        assert!(result.is_ok());
        let result = dev.add_threat_sensor(catalog::surveillance_radar());
        assert!(result.is_ok());

        let result = dev.add_countermeasure(catalog::ram_coating());
        assert!(result.is_ok());
        let result = dev.add_countermeasure(catalog::thermal_insulation());
        assert!(result.is_ok());
        let result = dev.add_countermeasure(catalog::adaptive_camouflage());
        assert!(result.is_ok());

        let result = dev.set_mission(MissionProfile {
            name: "Test Mission".into(),
            weight_budget_kg: 10.0,
            power_budget_w: 100.0,
            engagement_range_m: 2000.0,
            latency_class: LatencyClass::NearRealTime,
            raw_signature: 0.8,
            max_acceptable_detection: 0.3,
        });
        assert!(result.is_ok());

        dev
    }

    #[test]
    fn state_transitions() {
        let mut dev = Device::new("Test", 100);
        assert_eq!(dev.state(), DeviceState::Idle);

        assert!(dev.transition(DeviceState::Configuring).is_ok());
        assert_eq!(dev.state(), DeviceState::Configuring);

        // Invalid: can't go from Configuring to Active (must arm first)
        assert!(dev.transition(DeviceState::Active).is_err());

        assert!(dev.transition(DeviceState::Armed).is_ok());
        assert!(dev.transition(DeviceState::Active).is_ok());
        assert!(dev.transition(DeviceState::Assessing).is_ok());
        assert!(dev.transition(DeviceState::Idle).is_ok()); // Reset
    }

    #[test]
    fn full_assessment_pipeline() {
        let mut dev = configured_device();

        assert!(dev.transition(DeviceState::Armed).is_ok());
        assert!(dev.transition(DeviceState::Active).is_ok());

        let report = dev.assess();
        assert!(report.is_ok());

        let report = report.ok();
        assert!(report.is_some());

        let report = report.as_ref();
        let r = report.map(|r| &r.device_name);
        assert_eq!(r, Some(&"TestDevice".to_string()));
    }

    #[test]
    fn measurement_recording() {
        let mut dev = Device::new("Test", 100);
        dev.record_measurement(MetricKind::DetectionProbability, 0.5, "probability", 0.9);
        dev.tick(1.0);
        dev.record_measurement(MetricKind::DetectionProbability, 0.4, "probability", 0.9);
        dev.tick(1.0);
        dev.record_measurement(MetricKind::DetectionProbability, 0.3, "probability", 0.9);

        let stats = dev.measurement_stats(MetricKind::DetectionProbability, 10.0);
        assert!(stats.is_some());
        let stats = stats.as_ref();
        assert_eq!(stats.map(|s| s.count), Some(3));
    }

    #[test]
    fn weight_budget_enforcement() {
        let mut dev = Device::new("Test", 100);
        let result = dev.transition(DeviceState::Configuring);
        assert!(result.is_ok());

        let result = dev.set_mission(MissionProfile {
            name: "Tight Budget".into(),
            weight_budget_kg: 1.0, // Very tight
            power_budget_w: 100.0,
            engagement_range_m: 1000.0,
            latency_class: LatencyClass::RealTime,
            raw_signature: 0.8,
            max_acceptable_detection: 0.3,
        });
        assert!(result.is_ok());

        // RAM coating is 2.5 kg — exceeds 1.0 kg budget
        let result = dev.add_countermeasure(catalog::ram_coating());
        assert!(result.is_ok());

        let result = dev.add_threat_sensor(catalog::eo_camera());
        assert!(result.is_ok());

        let validation = dev.validate_config();
        assert!(validation.is_err());
    }

    #[test]
    fn trend_detection() {
        let mut dev = Device::new("Test", 100);

        // Record decreasing detection probability
        for i in 0..10 {
            dev.record_measurement(
                MetricKind::DetectionProbability,
                0.8 - (i as f64 * 0.05),
                "probability",
                0.9,
            );
            dev.tick(1.0);
        }

        let trend = dev.measurement_trend(MetricKind::DetectionProbability, 15.0);
        assert!(matches!(trend, Trend::Decreasing { .. }));
    }

    #[test]
    fn environment_modifiers() {
        let day_clear = Environment::default();
        assert!(day_clear.eo_modifier() > 0.8);

        let night_fog = Environment {
            is_daytime: false,
            visibility_m: 500.0,
            precipitation: 0.5,
            ..Default::default()
        };
        assert!(night_fog.eo_modifier() < 0.1);

        // Radar should be less affected by weather
        assert!(night_fog.radar_modifier() > night_fog.eo_modifier());
    }
}
