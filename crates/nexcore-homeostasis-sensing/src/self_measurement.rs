//! Self-measurement (proprioception) — Design Rule #4.
//!
//! Ports Python `homeostasis_machine/sensing/self_measurement.py`.
//!
//! The core question answered here: **"Is my response proportional to the threat?"**
//!
//! | Python class | Rust type | Role |
//! |---|---|---|
//! | `ProportionalityReading` | [`ProportionalityReading`] | Snapshot of one ratio computation |
//! | `ResponseSensor` | [`ResponseSensor`] | Proprioception — own response level |
//! | `ProportionalityCalculator` | [`ProportionalityCalculator`] | Ratio tracking + classification |
//! | `SelfMeasurementSuite` | [`SelfMeasurementSuite`] | Wires sensor + calculator together |
//!
//! Storm thresholds (all configurable, matching Python defaults):
//! - **Warning** `3.0×` — response 3× threat → elevated
//! - **Critical** `5.0×` — response 5× threat → critical
//! - **Storm** `10.0×` — response 10× threat → storm signature

use crate::anomaly::{AnomalyAssessment, AnomalyAssessor, TrendDirection};
use nexcore_chrono::DateTime;
use nexcore_error::Result;
use nexcore_homeostasis_primitives::{SensorReading, SensorType};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// ProportionalityLevel — Σ (Sum / enum)
// ---------------------------------------------------------------------------

/// Classification of the response/threat proportionality ratio.
///
/// Maps to the `is_disproportionate` + `severity` pair in Python's
/// `ProportionalityCalculator.calculate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProportionalityLevel {
    /// Ratio below warning threshold — proportionate response.
    Normal,
    /// Ratio ≥ `warning_threshold` (default 3.0).
    Warning,
    /// Ratio ≥ `critical_threshold` (default 5.0).
    Critical,
    /// Ratio ≥ `storm_threshold` (default 10.0).
    Storm,
}

impl ProportionalityLevel {
    /// Severity weight matching Python's `reading.severity` assignments.
    ///
    /// | Level | Python severity |
    /// |-------|----------------|
    /// | Normal | 0.0 |
    /// | Warning | 0.4 |
    /// | Critical | 0.7 |
    /// | Storm | 1.0 |
    pub fn severity(self) -> f64 {
        match self {
            Self::Normal => 0.0,
            Self::Warning => 0.4,
            Self::Critical => 0.7,
            Self::Storm => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// ProportionalityReading — ς (State snapshot)
// ---------------------------------------------------------------------------

/// A snapshot of one response/threat proportionality computation.
///
/// Mirrors Python `ProportionalityReading` dataclass.  The ratio is always
/// computed at construction; no `__post_init__` equivalent is needed in Rust
/// because `ProportionalityCalculator::compute` controls construction.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProportionalityReading {
    /// Observed response level at the time of this reading.
    pub response_level: f64,
    /// Observed threat level at the time of this reading.
    pub threat_level: f64,
    /// `response_level / max(threat_level, 0.001)` — the key metric.
    pub proportionality_ratio: f64,
    /// Wall-clock timestamp.
    pub timestamp: DateTime,
    /// Classification of the ratio against the configured thresholds.
    pub level: ProportionalityLevel,
    /// Severity scalar `0.0..=1.0` derived from [`level`](Self::level).
    pub severity: f64,
    /// Trend over the recent history window.
    pub trend: TrendDirection,
    /// Storm risk score `0.0..=1.0` (see [`ProportionalityCalculator`]).
    pub storm_risk: f64,
}

impl ProportionalityReading {
    /// Whether the response is over-proportional (ratio > 3.0, matching Python).
    pub fn is_over_responding(&self) -> bool {
        self.proportionality_ratio > 3.0
    }

    /// Whether the response is under-proportional (ratio < 0.5 with a real threat).
    pub fn is_under_responding(&self) -> bool {
        self.proportionality_ratio < 0.5 && self.threat_level > 0.1
    }
}

// ---------------------------------------------------------------------------
// ResponseSensor — ς + AnomalyAssessor
// ---------------------------------------------------------------------------

/// Monitors the system's own current response level (proprioception).
///
/// Ports Python `ResponseSensor`. In Python this subclasses the abstract async
/// `Sensor` base; here it wraps a callable source and an [`AnomalyAssessor`]
/// and returns a synchronous [`SensorReading`].
pub struct ResponseSensor {
    /// Name used in [`SensorReading`] output.
    name: String,
    /// Closure that returns the current response level.
    source: Box<dyn Fn() -> f64 + Send + Sync>,
    /// Maximum expected response level — used to compute utilisation.
    max_response: f64,
    /// Threshold-based anomaly assessor.
    assessor: AnomalyAssessor,
}

impl ResponseSensor {
    /// Create a new sensor.
    ///
    /// - `source` — closure returning the current response level
    /// - `max_response` — ceiling for utilisation percentage
    /// - `warning_fraction` — fraction of `max_response` that triggers warning (default 0.5)
    /// - `critical_fraction` — fraction of `max_response` that triggers critical (default 0.8)
    pub fn new(
        name: impl Into<String>,
        source: impl Fn() -> f64 + Send + Sync + 'static,
        max_response: f64,
        warning_fraction: f64,
        critical_fraction: f64,
    ) -> Self {
        let warning = max_response * warning_fraction;
        let critical = max_response * critical_fraction;
        Self {
            name: name.into(),
            source: Box::new(source),
            max_response,
            assessor: AnomalyAssessor::with_thresholds(0.0, warning, critical),
        }
    }

    /// Create with Python-default fractions (0.5 warning, 0.8 critical).
    pub fn with_defaults(
        name: impl Into<String>,
        source: impl Fn() -> f64 + Send + Sync + 'static,
        max_response: f64,
    ) -> Self {
        Self::new(name, source, max_response, 0.5, 0.8)
    }

    /// Sample the source and return a [`SensorReading`].
    ///
    /// Returns `Ok(SensorReading)` — the closure is infallible by contract;
    /// the `Result` wrapper matches crate convention for future extensibility.
    pub fn read(&self) -> Result<SensorReading> {
        let value = (self.source)();
        let assessment = self.assessor.assess(value);
        if assessment.is_anomalous {
            Ok(SensorReading::anomalous(
                value,
                &self.name,
                SensorType::SelfMeasurement,
                assessment.severity,
                assessment.confidence,
            ))
        } else {
            Ok(SensorReading::normal(
                value,
                &self.name,
                SensorType::SelfMeasurement,
            ))
        }
    }

    /// Response level as a fraction of the configured maximum.
    pub fn utilization(&self) -> f64 {
        let value = (self.source)();
        if self.max_response > 0.0 {
            value / self.max_response
        } else {
            0.0
        }
    }
}

// ---------------------------------------------------------------------------
// Internal history entry
// ---------------------------------------------------------------------------

/// One entry in the proportionality history ring buffer.
#[derive(Clone, Debug)]
struct HistoryEntry {
    threat: f64,
    response: f64,
    proportionality: f64,
}

// ---------------------------------------------------------------------------
// ProportionalityCalculator — ς + Threshold + History + Ratio
// ---------------------------------------------------------------------------

/// Computes and classifies response/threat proportionality over time.
///
/// Ports Python `ProportionalityCalculator`. Thresholds match Python defaults.
pub struct ProportionalityCalculator {
    /// Closure returning the current threat level.
    threat_source: Box<dyn Fn() -> f64 + Send + Sync>,
    /// Closure returning the current response level.
    response_source: Box<dyn Fn() -> f64 + Send + Sync>,

    // Thresholds
    /// Warning threshold (default 3.0).
    pub warning_threshold: f64,
    /// Critical threshold (default 5.0).
    pub critical_threshold: f64,
    /// Storm threshold (default 10.0).
    pub storm_threshold: f64,

    /// Ring buffer of recent readings for trend analysis.
    history: VecDeque<HistoryEntry>,
    /// Maximum number of entries to keep.
    history_size: usize,
    /// How many consecutive elevated entries have been seen.
    elevated_count: usize,
    /// Maximum number of elevated entries before a duration penalty applies.
    ///
    /// Corresponds to Python's `max_safe_duration` (converted from "samples"
    /// rather than wall time; keeps this module async/timer-free).
    max_safe_elevated_count: usize,
}

impl ProportionalityCalculator {
    /// Create with explicit parameters.
    pub fn new(
        threat_source: impl Fn() -> f64 + Send + Sync + 'static,
        response_source: impl Fn() -> f64 + Send + Sync + 'static,
        warning_threshold: f64,
        critical_threshold: f64,
        storm_threshold: f64,
        history_size: usize,
        max_safe_elevated_count: usize,
    ) -> Self {
        Self {
            threat_source: Box::new(threat_source),
            response_source: Box::new(response_source),
            warning_threshold,
            critical_threshold,
            storm_threshold,
            history: VecDeque::with_capacity(history_size),
            history_size,
            elevated_count: 0,
            max_safe_elevated_count,
        }
    }

    /// Create with Python-default thresholds.
    ///
    /// - warning: 3.0, critical: 5.0, storm: 10.0
    /// - history_size: 100
    /// - max_safe_elevated_count: 60 (proxy for ~10 min at 10s loop interval)
    pub fn with_defaults(
        threat_source: impl Fn() -> f64 + Send + Sync + 'static,
        response_source: impl Fn() -> f64 + Send + Sync + 'static,
    ) -> Self {
        Self::new(threat_source, response_source, 3.0, 5.0, 10.0, 100, 60)
    }

    /// Compute current proportionality and classify it.
    ///
    /// Matches Python `ProportionalityCalculator.calculate`.
    pub fn compute(&mut self, response: f64, threat: f64) -> ProportionalityReading {
        let ratio = response / threat.max(0.001);
        let level = self.classify(ratio);
        let severity = level.severity();

        // Track elevated duration
        if ratio > self.warning_threshold {
            self.elevated_count += 1;
        } else {
            self.elevated_count = 0;
        }

        // Push to history
        let entry = HistoryEntry {
            threat,
            response,
            proportionality: ratio,
        };
        if self.history.len() >= self.history_size {
            self.history.pop_front();
        }
        self.history.push_back(entry);

        // Trend from recent proportionality window
        let trend = self.trend_from_history();

        // Storm risk
        let storm_risk = self.storm_risk(ratio, trend);

        ProportionalityReading {
            response_level: response,
            threat_level: threat,
            proportionality_ratio: ratio,
            timestamp: DateTime::now(),
            level,
            severity,
            trend,
            storm_risk,
        }
    }

    /// Classify a ratio against the configured thresholds.
    pub fn classify(&self, ratio: f64) -> ProportionalityLevel {
        if ratio >= self.storm_threshold {
            ProportionalityLevel::Storm
        } else if ratio >= self.critical_threshold {
            ProportionalityLevel::Critical
        } else if ratio >= self.warning_threshold {
            ProportionalityLevel::Warning
        } else {
            ProportionalityLevel::Normal
        }
    }

    /// Split-mean trend over the last 10 proportionality values.
    ///
    /// Uses the same 20%-threshold algorithm as Python's `_calculate_trend`,
    /// delegating to [`TrendDirection::from_values`].
    fn trend_from_history(&self) -> TrendDirection {
        if self.history.len() < 5 {
            return TrendDirection::Stable;
        }
        let recent: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(10)
            .map(|e| e.proportionality)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        TrendDirection::from_values(&recent)
    }

    /// Trend for a specific metric in the last N history entries.
    fn metric_trend(&self, n: usize, select_response: bool) -> TrendDirection {
        let values: Vec<f64> = self
            .history
            .iter()
            .rev()
            .take(n)
            .map(|e| {
                if select_response {
                    e.response
                } else {
                    e.threat
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        TrendDirection::from_values(&values)
    }

    /// Compute storm risk `0.0..=1.0`.
    ///
    /// Four factors from Python `_calculate_storm_risk`:
    ///
    /// 1. Proportionality level (max 0.4)
    /// 2. Increasing trend (max 0.3)
    /// 3. Long elevation duration (max 0.2)
    /// 4. Response rising while threat falling (max 0.1)
    fn storm_risk(&self, ratio: f64, trend: TrendDirection) -> f64 {
        let mut risk = 0.0_f64;

        // Factor 1
        if ratio >= self.storm_threshold {
            risk += 0.4;
        } else if ratio >= self.critical_threshold {
            risk += 0.3;
        } else if ratio >= self.warning_threshold {
            risk += 0.2;
        }

        // Factor 2
        if trend == TrendDirection::Increasing {
            risk += 0.3;
        }

        // Factor 3
        if self.elevated_count > self.max_safe_elevated_count {
            risk += 0.2;
        } else if self.elevated_count > self.max_safe_elevated_count / 2 {
            risk += 0.1;
        }

        // Factor 4: divergence — response rising, threat falling
        if self.history.len() >= 5 {
            let resp_trend = self.metric_trend(5, true);
            let threat_trend = self.metric_trend(5, false);
            if resp_trend == TrendDirection::Increasing
                && threat_trend == TrendDirection::Decreasing
            {
                risk += 0.1;
            }
        }

        risk.min(1.0)
    }

    /// Most recent proportionality ratio, or `1.0` if no history.
    pub fn current_proportionality(&self) -> f64 {
        self.history
            .back()
            .map(|e| e.proportionality)
            .unwrap_or(1.0)
    }
}

// ---------------------------------------------------------------------------
// SelfMeasurementSuite — ∃ + →
// ---------------------------------------------------------------------------

/// Bundles a [`ResponseSensor`] and a [`ProportionalityCalculator`] into a
/// single assessment surface.
///
/// Ports Python `SelfMeasurementSuite`. The Python class returns a wide `dict`;
/// this type returns a typed pair for each call to [`assess`](Self::assess).
pub struct SelfMeasurementSuite {
    response_sensor: ResponseSensor,
    proportionality_calculator: ProportionalityCalculator,
}

impl SelfMeasurementSuite {
    /// Wire a sensor and calculator together.
    pub fn new(
        response_sensor: ResponseSensor,
        proportionality_calculator: ProportionalityCalculator,
    ) -> Self {
        Self {
            response_sensor,
            proportionality_calculator,
        }
    }

    /// Take a self-measurement: sample the response sensor, compute proportionality.
    ///
    /// Returns `(ProportionalityReading, AnomalyAssessment)`.
    ///
    /// - The [`ProportionalityReading`] contains the full ratio + storm risk.
    /// - The [`AnomalyAssessment`] reflects the response sensor's own threshold check.
    pub fn assess(
        &mut self,
        response: f64,
        threat: f64,
    ) -> Result<(ProportionalityReading, AnomalyAssessment)> {
        let sensor_reading = self.response_sensor.read()?;
        let prop_reading = self.proportionality_calculator.compute(response, threat);
        let assessment = AnomalyAssessment {
            is_anomalous: sensor_reading.is_anomalous,
            severity: sensor_reading.anomaly_severity,
            confidence: sensor_reading.anomaly_confidence,
        };
        Ok((prop_reading, assessment))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // --- ProportionalityReading ---

    #[test]
    fn proportionality_reading_over_responding() {
        let reading = ProportionalityReading {
            response_level: 30.0,
            threat_level: 5.0,
            proportionality_ratio: 6.0,
            timestamp: DateTime::now(),
            level: ProportionalityLevel::Critical,
            severity: 0.7,
            trend: TrendDirection::Stable,
            storm_risk: 0.3,
        };
        assert!(reading.is_over_responding());
        assert!(!reading.is_under_responding());
    }

    #[test]
    fn proportionality_reading_under_responding() {
        let reading = ProportionalityReading {
            response_level: 0.2,
            threat_level: 0.8,
            proportionality_ratio: 0.25,
            timestamp: DateTime::now(),
            level: ProportionalityLevel::Normal,
            severity: 0.0,
            trend: TrendDirection::Stable,
            storm_risk: 0.0,
        };
        assert!(reading.is_under_responding());
        assert!(!reading.is_over_responding());
    }

    // --- ProportionalityLevel ---

    #[test]
    fn level_severity_mapping() {
        assert_eq!(ProportionalityLevel::Normal.severity(), 0.0);
        assert_eq!(ProportionalityLevel::Warning.severity(), 0.4);
        assert_eq!(ProportionalityLevel::Critical.severity(), 0.7);
        assert_eq!(ProportionalityLevel::Storm.severity(), 1.0);
    }

    // --- ResponseSensor ---

    #[test]
    fn response_sensor_reads_value() {
        let sensor = ResponseSensor::with_defaults("test", || 45.0, 100.0);
        let reading = sensor.read().expect("read should succeed");
        assert_eq!(reading.value, 45.0);
        assert_eq!(reading.sensor_type, SensorType::SelfMeasurement);
    }

    #[test]
    fn response_sensor_anomalous_above_critical() {
        // warning=50 critical=80 max=100
        let sensor = ResponseSensor::with_defaults("test", || 90.0, 100.0);
        let reading = sensor.read().expect("read should succeed");
        assert!(reading.is_anomalous);
    }

    #[test]
    fn response_sensor_normal_below_warning() {
        let sensor = ResponseSensor::with_defaults("test", || 10.0, 100.0);
        let reading = sensor.read().expect("read should succeed");
        assert!(!reading.is_anomalous);
    }

    #[test]
    fn response_sensor_utilization() {
        let sensor = ResponseSensor::with_defaults("test", || 25.0, 100.0);
        let util = sensor.utilization();
        assert!((util - 0.25).abs() < f64::EPSILON);
    }

    // --- ProportionalityCalculator ---

    #[test]
    fn classify_normal() {
        let calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        assert_eq!(calc.classify(1.0), ProportionalityLevel::Normal);
        assert_eq!(calc.classify(2.9), ProportionalityLevel::Normal);
    }

    #[test]
    fn classify_warning_boundary() {
        let calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        assert_eq!(calc.classify(3.0), ProportionalityLevel::Warning);
    }

    #[test]
    fn classify_critical_boundary() {
        let calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        assert_eq!(calc.classify(5.0), ProportionalityLevel::Critical);
    }

    #[test]
    fn classify_storm_boundary() {
        let calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        assert_eq!(calc.classify(10.0), ProportionalityLevel::Storm);
    }

    #[test]
    fn compute_avoids_div_by_zero() {
        let mut calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        // threat=0.0 → uses max(0.0, 0.001) = 0.001
        let reading = calc.compute(5.0, 0.0);
        assert!(reading.proportionality_ratio.is_finite());
        // 5.0 / 0.001 = 5000 → Storm
        assert_eq!(reading.level, ProportionalityLevel::Storm);
    }

    #[test]
    fn compute_normal_proportionality() {
        let mut calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        let reading = calc.compute(2.0, 2.0);
        assert_eq!(reading.level, ProportionalityLevel::Normal);
        assert!((reading.proportionality_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn storm_risk_zero_when_normal() {
        let mut calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        let reading = calc.compute(1.0, 1.0);
        assert_eq!(reading.storm_risk, 0.0);
    }

    #[test]
    fn storm_risk_nonzero_above_warning() {
        let mut calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        let reading = calc.compute(4.0, 1.0); // ratio = 4 → Warning
        assert!(reading.storm_risk > 0.0);
    }

    #[test]
    fn current_proportionality_defaults_to_one() {
        let calc = ProportionalityCalculator::with_defaults(|| 0.0, || 0.0);
        assert_eq!(calc.current_proportionality(), 1.0);
    }

    // --- SelfMeasurementSuite ---

    #[test]
    fn suite_assess_returns_pair() {
        // response=30: below warning threshold (0.5 * 100 = 50) → sensor not anomalous
        // threat=2: ratio = 30/2 = 15 → Storm
        let sensor = ResponseSensor::with_defaults("response", || 30.0, 100.0);
        let calc = ProportionalityCalculator::with_defaults(|| 2.0, || 30.0);
        let mut suite = SelfMeasurementSuite::new(sensor, calc);

        let (prop, anomaly) = suite.assess(30.0, 2.0).expect("assess should succeed");
        // 30/2 = 15 → Storm
        assert_eq!(prop.level, ProportionalityLevel::Storm);
        // 30 < 50 (warning threshold = 0.5 * 100) → not anomalous
        assert!(!anomaly.is_anomalous);
    }

    #[test]
    fn suite_detects_anomalous_response_sensor() {
        let sensor = ResponseSensor::with_defaults("response", || 95.0, 100.0);
        let calc = ProportionalityCalculator::with_defaults(|| 5.0, || 95.0);
        let mut suite = SelfMeasurementSuite::new(sensor, calc);

        let (_prop, anomaly) = suite.assess(95.0, 5.0).expect("assess should succeed");
        // 95 > 80 → anomalous
        assert!(anomaly.is_anomalous);
    }
}
