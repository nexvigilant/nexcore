//! # Homeostasis Machine — Primitives
//!
//! Shared types, enums, and mathematical primitives for building
//! self-regulating systems inspired by biological homeostasis.
//!
//! This is the leaf crate — no internal nexcore dependencies.
//!
//! ## Five Laws
//!
//! This crate enforces 5 biological design laws structurally:
//!
//! 1. **Paired Controls** — `amplification::PairedAmplificationSystem` rejects
//!    attenuators weaker than their amplifier at registration time.
//! 2. **Signal Decay** — `signals::DecayingSignal` requires a `half_life`
//!    duration; signals cannot persist indefinitely.
//! 3. **Response Ceilings** — `hill::HillCurve` mathematically guarantees
//!    `response < max_response` for any finite signal.
//! 4. **Self-Measurement** — `enums::SensorType::SelfMeasurement` enables
//!    proprioceptive monitoring.
//! 5. **Proportionality** — `state::SystemState::needs_dampening()` flags
//!    over-response via configurable thresholds.

#![warn(missing_docs)]
pub mod amplification;
pub mod baseline;
pub mod cascade_order;
pub mod data;
pub mod enums;
pub mod hill;
pub mod signals;
pub mod state;

// Convenience re-exports for the most commonly used types.
pub use amplification::{
    AmplificationViolation, Amplifier, Attenuator, PairedAmplificationSystem, create_standard_pair,
};
pub use baseline::{Baseline, BaselineConfig, BaselineMetric};
pub use data::{ActionData, ActionResult, MetricSnapshot, SensorReading};
pub use enums::{
    ActionType, BaselineMetricType, CircuitState, DecayFunction, HealthStatus, ResponsePhase,
    SensorType, SignalType, StormPhase, TrendDirection, sensor_to_signal_type,
};
pub use hill::{HillCurve, ResponseCeiling, SaturatingResponse, create_biological_response_curve};
pub use signals::{DecayingSignal, SignalManager};
pub use state::{MetricHistory, StateTracker, SystemState};
