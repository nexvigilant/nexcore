//! Shared enums for the Homeostasis Machine.
//!
//! All enums are `Copy + Clone + Debug + PartialEq + Eq + Hash` and
//! serde-serializable with snake_case string representations matching
//! the Python SDK values.

use serde::{Deserialize, Serialize};

/// Categories of sensors — the vocabulary for sensor classification.
///
/// Maps to Python's `SensorType` enum in `interfaces/base.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    /// External anomaly detection (PAMPs analog).
    ExternalThreat,
    /// Internal stress/damage detection (DAMPs analog).
    InternalDamage,
    /// Response monitoring — proprioception.
    SelfMeasurement,
    /// Context sensing.
    Environmental,
    /// User-defined sensor category.
    Custom,
}

/// Types of actions the system can take.
///
/// Shared vocabulary for control decisions across modules.
/// Maps to Python's `ActionType` enum in `interfaces/base.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// No action required.
    Idle,
    /// Increase system response.
    Amplify,
    /// Hold current response level.
    Maintain,
    /// Reduce system response.
    Dampen,
    /// Rapid reduction for imminent storm.
    EmergencyDampen,
    /// Full system shutdown.
    EmergencyShutdown,
    /// Gradually restore to baseline.
    ReturnToBaseline,
    /// Scale out resources.
    ScaleUp,
    /// Scale in resources.
    ScaleDown,
    /// Open a circuit breaker.
    CircuitOpen,
    /// Close a circuit breaker.
    CircuitClose,
    /// Emit an alert.
    Alert,
    /// Apply rate limiting.
    RateLimit,
    /// User-defined action.
    Custom,
}

/// Categories of signals in the system.
///
/// Maps to Python's `SignalType` enum in `core/signals.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// External threat detected.
    Threat,
    /// Internal damage detected.
    Damage,
    /// Response action taken.
    Response,
    /// System recovering.
    Recovery,
    /// Anti-inflammatory / dampening signal.
    Dampening,
    /// User-defined signal.
    Custom,
}

/// Mathematical functions for signal decay.
///
/// Maps to Python's `DecayFunction` enum in `core/signals.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayFunction {
    /// Exponential decay — most biologically realistic.
    Exponential,
    /// Linear decay.
    Linear,
    /// Step decay — full value until half-life, then zero.
    Step,
    /// Sigmoid decay — slow-fast-slow profile.
    Sigmoid,
}

/// Overall health status of the system.
///
/// Maps to Python's `HealthStatus` enum in `core/state.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// At or near baseline.
    Healthy,
    /// Above baseline but manageable.
    Elevated,
    /// Requires attention.
    Warning,
    /// Requires immediate action.
    Critical,
    /// System at risk of failure.
    Emergency,
    /// Returning to baseline.
    Recovering,
    /// In a cascading failure state.
    Storm,
}

/// Phase of the response cycle.
///
/// Mirrors biological inflammation phases plus failure states.
/// Maps to Python's `ResponsePhase` enum in `core/state.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponsePhase {
    /// No active response.
    Idle,
    /// Threat detected, evaluating.
    Detection,
    /// Active response in progress.
    Acute,
    /// Response increasing.
    Escalating,
    /// Response at steady state.
    Plateau,
    /// Response decreasing.
    Resolving,
    /// Returning to baseline.
    Restoration,
    /// DANGER: Uncontrolled amplification.
    Storm,
    /// Resources depleted.
    Exhaustion,
}

/// Direction of change over a recent time window.
///
/// Maps to Python's `TrendDirection` enum in `core/state.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Values decreasing (improving health).
    Improving,
    /// No significant change.
    Stable,
    /// Values increasing (worsening health).
    Degrading,
    /// Rate of degradation itself increasing — storm signature.
    AcceleratingDegradation,
}

/// Types of metrics the baseline can track.
///
/// Maps to Python's `BaselineMetricType` enum in `core/baseline.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineMetricType {
    /// HTTP/RPC error rate.
    ErrorRate,
    /// Request latency.
    Latency,
    /// Request throughput (lower is worse).
    Throughput,
    /// CPU/memory/disk utilization.
    ResourceUtilization,
    /// Queue depth.
    QueueDepth,
    /// Current response level (self-measurement).
    ResponseLevel,
    /// User-defined metric.
    Custom,
}

/// Phase of a cytokine-storm-style cascade.
///
/// Maps to Python's `StormPhase` enum in `storm/detection.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StormPhase {
    /// Normal operating conditions.
    Clear,
    /// Minor indicators present.
    Watching,
    /// Elevated risk — pre-storm.
    Warning,
    /// Storm very likely in next cycle.
    Imminent,
    /// Storm is actively occurring.
    Active,
    /// Storm at maximum intensity.
    Peak,
    /// Storm subsiding.
    Resolving,
}

/// State of a circuit breaker.
///
/// Maps to Python's `CircuitState` enum in `storm/prevention.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    /// Normal — requests allowed through.
    Closed,
    /// Tripped — requests rejected.
    Open,
    /// Testing recovery — single request allowed.
    HalfOpen,
}

/// Map a [`SensorType`] to the appropriate [`SignalType`].
///
/// Used by the control loop to classify sensor readings into signal categories.
///
/// ```
/// use nexcore_homeostasis_primitives::enums::{SensorType, SignalType, sensor_to_signal_type};
/// assert_eq!(sensor_to_signal_type(SensorType::ExternalThreat), SignalType::Threat);
/// assert_eq!(sensor_to_signal_type(SensorType::InternalDamage), SignalType::Damage);
/// assert_eq!(sensor_to_signal_type(SensorType::SelfMeasurement), SignalType::Response);
/// ```
pub fn sensor_to_signal_type(sensor: SensorType) -> SignalType {
    match sensor {
        SensorType::ExternalThreat => SignalType::Threat,
        SensorType::InternalDamage => SignalType::Damage,
        SensorType::SelfMeasurement => SignalType::Response,
        SensorType::Environmental => SignalType::Custom,
        SensorType::Custom => SignalType::Custom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_to_signal_mapping() {
        assert_eq!(
            sensor_to_signal_type(SensorType::ExternalThreat),
            SignalType::Threat
        );
        assert_eq!(
            sensor_to_signal_type(SensorType::InternalDamage),
            SignalType::Damage
        );
        assert_eq!(
            sensor_to_signal_type(SensorType::SelfMeasurement),
            SignalType::Response
        );
        assert_eq!(
            sensor_to_signal_type(SensorType::Environmental),
            SignalType::Custom
        );
        assert_eq!(
            sensor_to_signal_type(SensorType::Custom),
            SignalType::Custom
        );
    }

    #[test]
    fn enums_serialize_roundtrip() {
        let json = serde_json::to_string(&SensorType::ExternalThreat).unwrap();
        let back: SensorType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, SensorType::ExternalThreat);
    }

    #[test]
    fn action_type_all_serialize() {
        for action in [
            ActionType::Idle,
            ActionType::Amplify,
            ActionType::Maintain,
            ActionType::Dampen,
            ActionType::EmergencyDampen,
            ActionType::EmergencyShutdown,
            ActionType::ReturnToBaseline,
            ActionType::ScaleUp,
            ActionType::ScaleDown,
            ActionType::CircuitOpen,
            ActionType::CircuitClose,
            ActionType::Alert,
            ActionType::RateLimit,
            ActionType::Custom,
        ] {
            let json = serde_json::to_string(&action).unwrap();
            let back: ActionType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, action);
        }
    }

    #[test]
    fn health_status_serialize_roundtrip() {
        let json = serde_json::to_string(&HealthStatus::Storm).unwrap();
        let back: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, HealthStatus::Storm);
    }

    #[test]
    fn circuit_state_serialize_roundtrip() {
        for state in [
            CircuitState::Closed,
            CircuitState::Open,
            CircuitState::HalfOpen,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: CircuitState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, state);
        }
    }

    #[test]
    fn storm_phase_all_variants_serialize() {
        for phase in [
            StormPhase::Clear,
            StormPhase::Watching,
            StormPhase::Warning,
            StormPhase::Imminent,
            StormPhase::Active,
            StormPhase::Peak,
            StormPhase::Resolving,
        ] {
            let json = serde_json::to_string(&phase).unwrap();
            let back: StormPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(back, phase);
        }
    }

    #[test]
    fn decay_function_serialize_roundtrip() {
        for func in [
            DecayFunction::Exponential,
            DecayFunction::Linear,
            DecayFunction::Step,
            DecayFunction::Sigmoid,
        ] {
            let json = serde_json::to_string(&func).unwrap();
            let back: DecayFunction = serde_json::from_str(&json).unwrap();
            assert_eq!(back, func);
        }
    }
}
