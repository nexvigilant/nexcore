//! # Load Prioritizer
//! Tier-based power shedding protocol for mission-critical systems.

use serde::{Deserialize, Serialize};

/// Consumer tiers (Safety first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LoadTier {
    /// 0: Safety critical (watchdogs, BRS, fire suppress)
    Safety = 0,
    /// 1: Life support (oxygen, suit cooling)
    LifeSupport = 1,
    /// 2: Flight control
    Flight = 2,
    /// 3: Exo-skeleton actuators
    Exo = 3,
    /// 4: HUD/Sensors
    Hud = 4,
    /// 5: Comms/Non-critical
    Comms = 5,
}

/// A registered power consumer.
#[derive(Debug, Clone)]
pub struct Load {
    pub id: u8,
    pub tier: LoadTier,
    pub nominal_w: f32,
    pub min_w: f32,
}

/// Evaluates load shedding requirements.
pub fn prioritize(available_w: f32, loads: &mut [Load]) -> [bool; 128] {
    let mut active = [false; 128];
    let mut remaining = available_w;

    // Sort by tier (ascending: 0=Safety)
    loads.sort_by_key(|l| l.tier);

    for load in loads {
        if remaining >= load.min_w {
            active[load.id as usize] = true;
            remaining -= load.nominal_w.min(remaining);
        } else {
            active[load.id as usize] = false;
        }
    }
    active
}
