//! # Electrical Storage System
//!
//! Management of main batteries, aux reserve, ultracapacitors, and limb buffers.

use serde::{Deserialize, Serialize};

/// Type of storage module in the federated system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    /// High-density NMC primary pack.
    MainBattery,
    /// Safety-critical LFP reserve.
    AuxiliaryReserve,
    /// Burst-buffer ultracapacitors.
    UltracapacitorBank,
    /// Distributed limb buffers.
    LimbBuffer,
}

/// Status report for an individual storage module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageModuleStatus {
    /// Module identifier.
    pub id: String,
    /// Type of storage.
    pub kind: StorageType,
    /// State of charge (0.0 to 1.0).
    pub soc: f32,
    /// Current power output (W).
    pub power_output: f32,
    /// Module temperature (Celsius).
    pub temperature: f32,
}

/// Interface for managing federated storage modules.
pub trait StorageModule {
    /// Returns the current health and status of the module.
    fn get_status(&self) -> StorageModuleStatus;
    /// Triggers a pre-charge and merge sequence for hot-swapping.
    fn request_merge(&mut self) -> Result<(), nexcore_error::NexError>;
    /// Requests an emergency isolation (e.g., pyro disconnect).
    fn isolate(&mut self) -> Result<(), nexcore_error::NexError>;
}
