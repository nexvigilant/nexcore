use serde::{Deserialize, Serialize};

/// Priority tiers for power consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoadTier {
    /// Essential systems.
    Safety = 0,
    /// Critical life support.
    LifeSupport = 1,
    /// Propulsion/Flight systems.
    Flight = 2,
    /// Exoskeleton actuation.
    Exo = 3,
    /// User interface.
    Hud = 4,
    /// Non-essential communications.
    Comms = 5,
}

/// Commands for shedding loads based on system power status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadShedCommand {
    /// Reduce power for all consumers at or below this tier.
    ShedBelow(LoadTier),
    /// Critical emergency shed.
    FullEmergencyShed,
}

/// Manages load priority and power distribution to consumers.
pub struct LoadPrioritizer {
    /// Active shed level.
    pub current_tier: LoadTier,
}

impl LoadPrioritizer {
    /// Determines which loads must be shed based on current power availability.
    pub fn evaluate_load(&mut self, _power_available: f32) -> LoadShedCommand {
        // TODO: Logic for tier-based shedding.
        LoadShedCommand::ShedBelow(LoadTier::Comms)
    }
}
