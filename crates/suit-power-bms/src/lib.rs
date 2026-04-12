#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Battery Management System for federated wearable power architecture.
//!
//! Four modules covering the BMS firmware decision loop:
//!
//! - [`soc`] — State-of-Charge estimation (EKF + OCV + coulomb counting)
//! - [`soh`] — State-of-Health tracking (capacity fade + resistance growth)
//! - [`fault`] — Fault tree (overvolt/undervolt/overcurrent/overtemp/isolation)
//! - [`power_router`] — Federated power tier routing (main/ultracap/limb/aux)
//!
//! Maps 1:1 to the microgram decision trees in `rsk-core/rsk/micrograms/`:
//! `soc-estimator`, `power-load-router`, `limb-charge-scheduler`,
//! `voltage-topology-tradeoff`.

pub mod fault;
pub mod power_router;
pub mod soc;
pub mod soh;
