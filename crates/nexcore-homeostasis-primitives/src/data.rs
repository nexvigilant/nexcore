//! Data transfer structs for sensor readings, actions, and metric snapshots.
//!
//! These are plain data types with no behavior — the cross-module contracts
//! that prevent import coupling between sensing, response, and core modules.

use crate::enums::{ActionType, SensorType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Instant;

/// A point-in-time reading from a sensor.
///
/// Maps to Python's `SensorReadingData` dataclass in `interfaces/base.py`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorReading {
    /// The observed value.
    pub value: f64,
    /// Wall-clock timestamp of the reading.
    pub timestamp: DateTime<Utc>,
    /// Whether the value is anomalous.
    pub is_anomalous: bool,
    /// Severity of the anomaly, 0–1 scale.
    pub anomaly_severity: f64,
    /// Confidence in the anomaly assessment, 0–1.
    pub anomaly_confidence: f64,
    /// Human-readable name of the sensor that produced this reading.
    pub sensor_name: String,
    /// Sensor category.
    pub sensor_type: SensorType,
    /// Arbitrary additional context.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SensorReading {
    /// Create a normal (non-anomalous) reading.
    pub fn normal(value: f64, sensor_name: impl Into<String>, sensor_type: SensorType) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
            is_anomalous: false,
            anomaly_severity: 0.0,
            anomaly_confidence: 0.0,
            sensor_name: sensor_name.into(),
            sensor_type,
            metadata: HashMap::new(),
        }
    }

    /// Create an anomalous reading.
    pub fn anomalous(
        value: f64,
        sensor_name: impl Into<String>,
        sensor_type: SensorType,
        severity: f64,
        confidence: f64,
    ) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
            is_anomalous: true,
            anomaly_severity: severity.clamp(0.0, 1.0),
            anomaly_confidence: confidence.clamp(0.0, 1.0),
            sensor_name: sensor_name.into(),
            sensor_type,
            metadata: HashMap::new(),
        }
    }
}

/// Data contract for an action to be executed by actuators.
///
/// Maps to Python's `ActionData` dataclass in `interfaces/base.py`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionData {
    /// The type of action to perform.
    pub action_type: ActionType,
    /// Desired response level after this action completes.
    pub target_response_level: f64,
    /// Human-readable reason for the action.
    pub reason: String,
    /// Arbitrary additional context.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Wall-clock timestamp when the action was decided.
    pub timestamp: DateTime<Utc>,
}

impl ActionData {
    /// Create an action with current timestamp.
    pub fn new(
        action_type: ActionType,
        target_response_level: f64,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            action_type,
            target_response_level,
            reason: reason.into(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

/// Result of executing an action through an actuator.
///
/// Maps to Python's `ActionResultData` dataclass in `interfaces/base.py`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the action succeeded.
    pub success: bool,
    /// The action that was attempted.
    pub action_type: ActionType,
    /// Actual response level achieved (if measurable).
    pub actual_value: Option<f64>,
    /// Error message on failure.
    pub error_message: Option<String>,
    /// How long the action took in milliseconds.
    pub execution_time_ms: f64,
    /// Arbitrary additional context.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ActionResult {
    /// Create a successful result.
    pub fn success(action_type: ActionType, actual_value: Option<f64>, ms: f64) -> Self {
        Self {
            success: true,
            action_type,
            actual_value,
            error_message: None,
            execution_time_ms: ms,
            metadata: HashMap::new(),
        }
    }

    /// Create a failure result.
    pub fn failure(action_type: ActionType, error: impl Into<String>, ms: f64) -> Self {
        Self {
            success: false,
            action_type,
            actual_value: None,
            error_message: Some(error.into()),
            execution_time_ms: ms,
            metadata: HashMap::new(),
        }
    }
}

/// A point-in-time capture of a named metric value.
///
/// Used inside [`MetricHistory`](crate::state::MetricHistory) to track
/// rolling time-series data for trend analysis.
///
/// Note: `Instant` is tokio's `Instant` for deterministic time control in tests
/// (use `tokio::time::pause()` + `tokio::time::advance()`).
#[derive(Clone, Debug)]
pub struct MetricSnapshot {
    /// Metric name.
    pub name: String,
    /// Observed value.
    pub value: f64,
    /// When this snapshot was taken.
    pub timestamp: Instant,
}

impl MetricSnapshot {
    /// Create a snapshot with the current tokio time.
    pub fn now(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            timestamp: Instant::now(),
        }
    }

    /// Age of this snapshot in seconds.
    pub fn age_secs(&self) -> f64 {
        self.timestamp.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ActionType;

    #[test]
    fn sensor_reading_normal_not_anomalous() {
        let r = SensorReading::normal(0.5, "cpu", SensorType::SelfMeasurement);
        assert!(!r.is_anomalous);
        assert_eq!(r.anomaly_severity, 0.0);
    }

    #[test]
    fn sensor_reading_anomalous_clamps_severity() {
        let r = SensorReading::anomalous(1.5, "cpu", SensorType::SelfMeasurement, 2.0, -0.5);
        assert!(r.is_anomalous);
        assert_eq!(r.anomaly_severity, 1.0); // clamped from 2.0
        assert_eq!(r.anomaly_confidence, 0.0); // clamped from -0.5
    }

    #[test]
    fn action_data_roundtrip_json() {
        let a = ActionData::new(ActionType::Dampen, 50.0, "proportionality exceeded");
        let json = serde_json::to_string(&a).unwrap();
        let back: ActionData = serde_json::from_str(&json).unwrap();
        assert_eq!(back.action_type, ActionType::Dampen);
        assert_eq!(back.target_response_level, 50.0);
    }

    #[test]
    fn action_result_success_failure() {
        let ok = ActionResult::success(ActionType::Amplify, Some(75.0), 1.2);
        assert!(ok.success);
        assert_eq!(ok.actual_value, Some(75.0));

        let err = ActionResult::failure(ActionType::EmergencyDampen, "actuator offline", 0.5);
        assert!(!err.success);
        assert!(err.error_message.is_some());
    }

    #[tokio::test]
    async fn metric_snapshot_age() {
        let snap = MetricSnapshot::now("latency", 123.4);
        assert!(snap.age_secs() >= 0.0);
        assert_eq!(snap.name, "latency");
        assert_eq!(snap.value, 123.4);
    }
}
