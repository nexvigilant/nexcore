//! Sensor taxonomy for the Homeostasis Machine.
//!
//! Ports Python `homeostasis_machine/sensing/` into three modules:
//!
//! | Module | Biological analog | Purpose |
//! |--------|-------------------|---------|
//! | [`external`] | PAMPs (pathogen-associated molecular patterns) | Detect anomalies from outside the system |
//! | [`internal`] | DAMPs (damage-associated molecular patterns) | Detect internal stress and damage |
//! | [`self_measurement`] | Proprioception | Measure the system's own response level |
//!
//! The anomaly-assessment plumbing is common to all three; it lives here as
//! [`anomaly::AnomalyAssessor`].
//!
//! ## Integration with `nexcore-homeostasis`
//!
//! The existing [`nexcore_homeostasis::traits::Sensor`] trait is the contract
//! the control loop depends on. Every concrete sensor in this crate implements
//! the [`SyncSensor`] trait, which exposes the synchronous read interface.
//! The async bridge lives in `nexcore-homeostasis::traits::SyncSensorAdapter`.
//!
//! ## Example
//!
//! ```no_run
//! use nexcore_homeostasis_sensing::external::{ErrorRateSensor, ThresholdPattern};
//!
//! let sensor = ErrorRateSensor::new(0.01, 0.05); // warning=1%, critical=5%
//! let assessment = sensor.assess(0.03);
//! assert!(assessment.is_anomalous);
//! ```

#![warn(missing_docs)]

pub mod anomaly;
pub mod external;
pub mod internal;
pub mod self_measurement;

pub use anomaly::{AnomalyAssessment, AnomalyAssessor, TrendDirection};

use nexcore_homeostasis_primitives::SensorType;

// =============================================================================
// SyncSensor trait
// =============================================================================

/// A synchronous sensor interface for sensors that maintain internal state.
///
/// ## Design
///
/// The sensors in this crate compute readings synchronously — they hold an
/// in-process value history and run pattern matching locally, with no I/O.
/// The `nexcore-homeostasis` control loop consumes the async [`Sensor`] trait
/// instead. [`SyncSensor`] bridges the two: concrete types implement this
/// synchronous interface; the async bridge lives in
/// `nexcore_homeostasis::traits::SyncSensorAdapter`.
///
/// ## Implementing
///
/// Implementors must call [`record`](SyncSensor) (or an equivalent that updates
/// `last_reading`) before `read_current` can return a value.
///
/// [`Sensor`]: nexcore_homeostasis_primitives::SensorType
pub trait SyncSensor: Send + Sync {
    /// Human-readable sensor name.
    fn name(&self) -> &str;

    /// Sensor category used by the control loop for classification.
    fn sensor_type(&self) -> SensorType;

    /// Return the most-recently recorded value and its anomaly assessment.
    ///
    /// Returns `None` when [`record`](Self) has not been called since
    /// construction (i.e., the sensor has no data yet).
    fn read_current(&self) -> Option<(f64, AnomalyAssessment)>;
}

// =============================================================================
// SyncSensor impls for external sensors
// =============================================================================

impl SyncSensor for external::ErrorRateSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::ExternalThreat
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for external::LatencySensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::ExternalThreat
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for external::TrafficSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::ExternalThreat
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for external::QueueDepthSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::ExternalThreat
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

// =============================================================================
// SyncSensor impls for internal sensors
// =============================================================================

impl SyncSensor for internal::MemoryPressureSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::InternalDamage
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for internal::CpuPressureSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::InternalDamage
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for internal::ConnectionPoolSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::InternalDamage
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for internal::ThreadPoolSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::InternalDamage
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}

impl SyncSensor for internal::SelfInflictedDamageSensor {
    fn name(&self) -> &str {
        self.name()
    }

    fn sensor_type(&self) -> SensorType {
        SensorType::InternalDamage
    }

    fn read_current(&self) -> Option<(f64, AnomalyAssessment)> {
        self.last_reading()
    }
}
