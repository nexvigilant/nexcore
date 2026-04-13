//! # Graceful Degradation Sequencer
//!
//! FMEA-driven state machine for power system failures and recovery.

use serde::{Deserialize, Serialize};

/// The power system operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerState {
    /// Nominal operation, full redundancy.
    Nominal,
    /// Caution: Degrading performance, investigating issue.
    Caution,
    /// Warning: Impending failure, shedding non-critical loads.
    Warning,
    /// Emergency: Loss of redundancy, immediate safety action required.
    Emergency,
    /// Land Now: Critical failure, initiate emergency recovery/landing.
    LandNow,
}

/// Blackbox event log for state transitions.
pub struct BlackboxLogger;

impl BlackboxLogger {
    /// Records a transition event.
    pub fn log_transition(from: PowerState, to: PowerState, reason: &str) {
        // TODO: Write to NVMe/storage.
        tracing::warn!(
            "Transitioning from {:?} to {:?} due to: {}",
            from,
            to,
            reason
        );
    }
}

/// The sequencer managing system-wide degradation.
pub struct DegradationSequencer {
    /// Current power state.
    pub state: PowerState,
}

impl DegradationSequencer {
    /// Initiates a state transition.
    pub fn transition(&mut self, next: PowerState, reason: &str) {
        BlackboxLogger::log_transition(self.state, next, reason);
        self.state = next;
    }
}
