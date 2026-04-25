//! # Compound: PERCEPTION
//!
//! Sensors → fusion → world state. The suit's "what is happening" layer.
//!
//! ## Components
//! - `suit_primitives` — leaf-node types (no_std, dependency-free).
//! - `suit_sensors`    — raw IMU / force / strain / biometric drivers.
//! - `suit_perception` — sensor fusion, localization, intent classification.

/// Compound identifier for telemetry and registry.
pub const PERCEPTION_COMPOUND_NAME: &str = "perception";

/// Re-export the entire public surface of `suit_primitives`.
pub use suit_primitives as primitives;

/// Re-export the entire public surface of `suit_sensors`.
pub use suit_sensors as sensors;

/// Re-export the entire public surface of `suit_perception`.
pub use suit_perception as perception;

/// Convenience: the perception engine itself.
pub use suit_perception::perception_engine::PerceptionEngine;

/// Convenience: the canonical `InertialState` type used across the compound.
pub use suit_perception::vestibular::InertialState;
