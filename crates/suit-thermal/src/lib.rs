#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Thermal management subsystem for wearable suit.
//!
//! - [`zone`] — Zone monitoring, cooling routing, heat sink/loop modeling
//! - [`runaway`] — 3-stage NMC thermal runaway detection
//!
//! Maps 1:1 to rsk-core micrograms: `thermal-management-router`,
//! `thermal-runaway-detector`.

pub mod runaway;
pub mod zone;
