//! # Fuel Storage and Delivery
//!
//! Management of liquid fuel supply, pumps, dump valves, and hybrid tie-in.

use nexcore_error::NexError as Error;
use serde::{Deserialize, Serialize};

/// Status report for the fuel storage system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelStatus {
    /// Fuel quantity in Liters.
    pub quantity: f32,
    /// Fuel pressure in bar.
    pub pressure: f32,
    /// Pump status (rpm).
    pub pump_rpm: f32,
    /// Dump valve status (True = Open).
    pub dump_valve_open: bool,
}

/// Interface for Fuel Delivery and Pump management.
pub trait FuelSystem {
    /// Returns the current fuel level and pressure status.
    fn get_status(&self) -> FuelStatus;
    /// Sets pump speed for target rail pressure.
    fn set_pump_speed(&mut self, rpm: f32) -> Result<(), Error>;
    /// Activates the emergency dump sequence.
    fn dump_fuel(&mut self) -> Result<(), Error>;
}

/// Interface for the Hybrid Turbo-Electric Tie-in.
pub trait HybridGenerator {
    /// Returns the current power generation output (kW).
    fn get_power_out(&self) -> f32;
    /// Enables/disables the generator path.
    fn set_active(&mut self, active: bool) -> Result<(), Error>;
}
