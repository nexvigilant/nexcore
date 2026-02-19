//! # Sovereign Domains (The States)
//!
//! Implementation of the "Free and Independent States" logic from the
//! Declaration of Grounding. Each Domain is a sovereign entity within
//! the NexVigilant Union.

use crate::primitives::governance::{Resolution, Rule, VoteWeight};
use serde::{Deserialize, Serialize};

/// T3: SovereignDomain - A sovereign state within the NexVigilant Union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignDomain {
    pub id: String,
    pub primary_axiom: String,
    pub laws: Vec<Rule>,
    pub population_size: u64,
}

impl SovereignDomain {
    /// Propose a Resolution to the Congress.
    pub fn propose_resolution(&self, rule: Rule, confidence: f64) -> Resolution {
        nexcore_primitives::measurement::Measured::uncertain(
            rule,
            nexcore_primitives::measurement::Confidence::new(confidence),
        )
    }

    /// Calculate the Domain's voting weight based on population (House) or sovereignty (Senate).
    pub fn vote_weight(&self, is_senate: bool) -> VoteWeight {
        if is_senate {
            VoteWeight::new(50) // All states equal in Senate
        } else {
            // Proportional weight in House
            let weight = (self.population_size / 100).min(100) as u8;
            VoteWeight::new(weight)
        }
    }
}

/// T3: Faction - A group of domains with shared interests (Federalist No. 10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub name: String,
    pub member_ids: Vec<String>,
    pub agenda: Vec<Rule>,
}

impl Faction {
    /// Check if a resolution aligns with the faction's agenda.
    pub fn aligns_with(&self, resolution: &Resolution) -> bool {
        // Simplified alignment logic
        self.agenda.contains(&resolution.value)
    }
}
