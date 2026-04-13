//! # Auxiliary and Transient Storage
//!
//! Interfaces for Auxiliary LFP packs, Ultracapacitor banks, and Limb buffers.

use nexcore_error::NexError as Error;
use serde::{Deserialize, Serialize};

/// Status report for auxiliary or limb-level storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxiliaryStatus {
    /// State of charge (0.0 to 1.0).
    pub soc: f32,
    /// Available capacity (Wh).
    pub capacity_wh: f32,
}

/// Interface for Auxiliary LFP Life-Critical reserve (1.1.2).
pub trait AuxiliaryPack {
    /// Checks status of the aux pack.
    fn get_status(&self) -> AuxiliaryStatus;
    /// Triggers autonomous cutover to aux power.
    fn trigger_cutover(&mut self) -> Result<(), Error>;
}

/// Interface for Ultracapacitor Bank (1.1.3).
pub trait UltracapBank {
    /// Gets burst power capability (W).
    fn get_peak_power(&self) -> f32;
    /// Enables/disables the bidirectional DC-DC converter.
    fn set_converter_active(&mut self, active: bool) -> Result<(), Error>;
}

/// Interface for Limb-level Buffer Cells (1.1.4).
pub trait LimbBuffer {
    /// Reads status of a specific limb buffer.
    fn get_status(&self) -> AuxiliaryStatus;
    /// Commands a charge cycle from the main bus.
    fn request_charge(&mut self) -> Result<(), Error>;
}
