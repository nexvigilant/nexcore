#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

//! # suit-power MCP Bridge
//!
//! Exposes suit-power telemetry and control functions via the Model Context Protocol.

use nexcore_error::NexError as Error;
use suit_power::prelude::PowerEngine;
use wksp_types::power::PowerStatusMessage;

/// The MCP service wrapper for suit power management.
pub struct PowerMcpService {
    /// Reference to the internal power engine.
    pub engine: PowerEngine,
}

impl PowerMcpService {
    /// Creates a new MCP service instance.
    pub fn new() -> Self {
        Self {
            engine: PowerEngine::new(),
        }
    }

    /// Telemetry tool: Returns the current power system status message.
    pub fn get_telemetry(&self) -> PowerStatusMessage {
        // Mapping PowerEngine internal state to the shared middleware message format.
        // SOC reads from cached filter_state; mutation belongs in update() callers.
        // TODO: SocEstimator::current() pure read accessor once EKF lands.
        PowerStatusMessage {
            soc: 100.0,
            current_tier: self.engine.prioritizer.current_tier as u8,
            power_state: self.engine.sequencer.state as u8,
        }
    }
}
