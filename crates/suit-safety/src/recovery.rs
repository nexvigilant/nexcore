//! # Ballistic Recovery System (10.1)
//! Controls canopy deployment, rocket fire, and automated triggers.

use serde::{Deserialize, Serialize};

/// The state of the recovery system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    /// Armed, standby.
    Armed,
    /// Automated trigger threshold met.
    Triggered,
    /// Post-deployment.
    Deployed,
}

/// Interface for the Ballistic Recovery System.
pub trait BallisticSystem {
    /// Attempts to fire the rocket deployer (gated by safety logic).
    fn fire_canopy(&mut self) -> Result<(), nexcore_error::NexError>;
    /// Reports the current system state (armed/deployed).
    fn get_state(&self) -> RecoveryState;
}
