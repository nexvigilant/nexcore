//! # Compound: CONTROL
//!
//! Decision → actuation. The suit's "what to do next" layer.
//!
//! ## Components
//! - `suit_compute`  — hardware orchestration, RTOS bridge, TMR voting.
//! - `suit_actuator` — joint classification, motor sizing, fault detection.
//! - `suit_flight`   — flight controller for mission-critical maneuvers.

/// Compound identifier for telemetry and registry.
pub const CONTROL_COMPOUND_NAME: &str = "control";

/// Re-export the entire public surface of `suit_compute`.
pub use suit_compute as compute;

/// Re-export the entire public surface of `suit_actuator`.
pub use suit_actuator as actuator;

/// Re-export the entire public surface of `suit_flight`.
pub use suit_flight as flight;

/// Convenience: the flight control bridge.
pub use suit_flight::ControlBridge;
