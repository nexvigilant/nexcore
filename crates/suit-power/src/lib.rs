#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # Suit Power
//!
//! Power management system: SOC estimation, load prioritization, thermal derating.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod auxiliary;
pub mod bms;
pub mod degradation;
pub mod engine;
pub mod fuel;
pub mod load;
pub mod mission;
pub mod pack;
pub mod soc;
pub mod storage;
pub mod telemetry;
pub mod thermal;

/// Re-export of common power management types.
pub mod prelude {
    pub use crate::auxiliary::{AuxiliaryPack, LimbBuffer, UltracapBank};
    pub use crate::bms::{BmsMaster, BmsSlave};
    pub use crate::engine::PowerEngine;
    pub use crate::fuel::{FuelSystem, HybridGenerator};
    pub use crate::load::{LoadShedCommand, LoadTier};
    pub use crate::pack::PackState;
    pub use crate::soc::SocEstimate;
    pub use crate::storage::{StorageModule, StorageModuleStatus, StorageType};
    pub use crate::telemetry::PowerTelemetryBridge;
}
