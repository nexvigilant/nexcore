#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Actuator subsystem for wearable suit.
//!
//! - [`joint`] — Joint classification, motor sizing, control mode selection
//! - [`fault`] — Motor fault detection (7 fault types)
//!
//! Maps 1:1 to rsk-core micrograms: `actuator-load-classifier`,
//! `motor-fault-detector`.

pub mod fault;
pub mod joint;
