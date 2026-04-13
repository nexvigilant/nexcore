//! # BMS Hardware Abstraction
//!
//! Driver interfaces for distributed slave BMS boards and central master.

use nexcore_error::NexError as Error;
use serde::{Deserialize, Serialize};

/// Represents raw cell voltages from a 16-channel monitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellTelemetry {
    /// Cell voltages in mV.
    pub voltages: [u16; 16],
    /// Cell temperatures in 0.1°C.
    pub temperatures: [i16; 4],
}

/// Interface for distributed slave BMS boards (e.g., BQ79616-Q1).
pub trait BmsSlave {
    /// Polls the slave board for cell voltages and temperatures.
    fn get_telemetry(&mut self) -> Result<CellTelemetry, Error>;
    /// Activates passive balancing for a specific cell.
    fn set_balancing(&mut self, cell_mask: u16) -> Result<(), Error>;
}

/// Interface for the master BMS controller (STM32H7).
pub trait BmsMaster {
    /// Returns aggregated current sense (±500 A).
    fn get_current(&self) -> Result<f32, Error>;
    /// Executes the contactor safety sequence (precharge -> connect).
    fn connect_main_bus(&mut self) -> Result<(), Error>;
    /// Performs insulation resistance check.
    fn check_isolation(&mut self) -> Result<f32, Error>;
}
