//! # Federated Pack Coordinator (1.4.5)
//! Droop-controlled power management for redundant battery modules.

use serde::{Deserialize, Serialize};

/// State of the federated hot-swap handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeState {
    Disconnected,
    Precharge,
    Ramping,
    Merged,
    Fault,
}

/// A handle to a battery module in the federated system.
pub struct BatteryModule {
    pub id: u8,
    pub voltage: f32,
    pub state: MergeState,
}

impl BatteryModule {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            voltage: 0.0,
            state: MergeState::Disconnected,
        }
    }

    /// Executed by the PowerEngine to perform the hot-swap handshake.
    pub fn tick(&mut self, bus_voltage: f32) {
        match self.state {
            MergeState::Disconnected => {
                // Check if module is within threshold to precharge
                if (self.voltage - bus_voltage).abs() < 5.0 {
                    self.state = MergeState::Precharge;
                }
            }
            MergeState::Precharge => {
                // Simulate precharge RC charging
                self.state = MergeState::Ramping;
            }
            MergeState::Ramping => {
                self.state = MergeState::Merged;
            }
            MergeState::Merged | MergeState::Fault => {}
        }
    }
}
