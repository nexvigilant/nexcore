//! # Suit Sensors
//!
//! Raw hardware abstraction for suit sensor subsystems.
//! Foundation layer — consumed by `suit-perception` for fusion and classification.
//!
//! ## Modules
//!
//! - **imu** — 9-DOF inertial measurement units (accelerometer, gyroscope, magnetometer)
//! - **force_plate** — Ground reaction force measurement for gait and balance
//! - **strain_gauge** — Structural strain monitoring on suit elements
//! - **biometrics** — Pilot physiological monitoring (HR, SpO2, temperature, GSR)
//!
//! ## T1 Grounding
//! - `ς` (state) — all readings are instantaneous state snapshots
//! - `ν` (frequency) — each sensor has a configurable sample rate
//! - `∂` (boundary) — calibration and thresholds define valid operating regions

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

/// Pilot biometric monitoring (HR, SpO2, temperature, GSR, respiration).
pub mod biometrics;
/// Ground reaction force measurement.
pub mod force_plate;
/// Inertial measurement unit abstraction.
pub mod imu;
/// Structural strain gauge monitoring.
pub mod strain_gauge;
