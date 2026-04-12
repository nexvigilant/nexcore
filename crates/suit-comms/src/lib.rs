#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Communication subsystem for wearable suit architecture.
//!
//! Two modules covering the comms decision pipeline:
//!
//! - [`link`] — Link types, routing, and properties (7 links, 4 domains)
//! - [`security`] — Security gate validation (mTLS, firmware, replay, cert pin)
//!
//! Maps 1:1 to rsk-core micrograms: `comms-link-router`, `comms-security-gate`.

pub mod link;
pub mod security;
