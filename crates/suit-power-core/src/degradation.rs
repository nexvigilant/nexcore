//! # Graceful Degradation Sequencer (1.4.4)
//! FMEA-driven safety state machine for system failures.

use crate::error::PowerError;
use serde::{Deserialize, Serialize};

/// Operational power states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SystemState {
    Nominal = 0,
    Caution = 1,
    Warning = 2,
    Emergency = 3,
    LandNow = 4,
}

/// A failure mode that triggers a specific system response.
pub enum FailureMode {
    CellOverVoltage,
    ThermalRunaway,
    CommunicationLoss,
    InsulationFault,
}

/// Manages system degradation state.
pub struct DegradationSequencer {
    pub state: SystemState,
}

impl DegradationSequencer {
    pub fn new() -> Self {
        Self {
            state: SystemState::Nominal,
        }
    }

    /// Transitions the system state based on the identified failure mode.
    pub fn handle_fault(&mut self, fault: FailureMode) -> SystemState {
        self.state = match fault {
            FailureMode::CellOverVoltage => SystemState::Caution,
            FailureMode::ThermalRunaway => SystemState::LandNow,
            FailureMode::CommunicationLoss => SystemState::Emergency,
            FailureMode::InsulationFault => SystemState::Emergency,
        };
        self.state
    }
}
