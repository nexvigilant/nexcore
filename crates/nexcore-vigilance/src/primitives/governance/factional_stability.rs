//! # Factional Stability (Federalist No. 10)
//!
//! Foundational primitives for controlling the effects of Factions within the Union.
//! A Faction is a domain or group of agents united by an interest adverse to the
//! system's homeostasis or the rights of other domains.

use crate::primitives::governance::{Resolution, VoteWeight};
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (SEMANTICS)
// ============================================================================

/// T1: Interest - A specific resource or rule preference.
/// Grounding: Axiom: Cause (The motivation for a Resolution).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interest {
    /// Demand for higher Compute Quota.
    ComputePriority,
    /// Demand for specific Rule enforcement.
    RuleDominance(String),
    /// Demand for Data Locality.
    DataSovereignty,
}

/// T1: Adversity - A measure of conflict between two Factions.
/// Grounding: Axiom: Detect (The observation of non-alignment).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Adversity {
    /// Interlocking interests.
    Aligned = 0,
    /// Neutral/Independent.
    Neutral = 1,
    /// Competing for the same resource.
    Competing = 2,
    /// Fundamentally incompatible goals.
    Adverse = 3,
}

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: FactionDensity - The ratio of active factions in a system.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct FactionDensity(f64);

impl FactionDensity {
    pub fn new(factions: usize, total_domains: usize) -> Self {
        Self(factions as f64 / total_domains as f64)
    }

    /// High density (many small factions) is safer than low density (few large factions).
    pub fn is_stable(&self) -> bool {
        self.0 > 0.3
    }
}

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: Faction - A collective unit with shared interests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub interests: Vec<Interest>,
    pub power: VoteWeight,
}

/// T2-C: StabilityAudit - The mechanism for controlling factional effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityAudit {
    pub active_factions: Vec<Faction>,
    pub total_domains: usize,
}

impl StabilityAudit {
    /// Calculate the System Pluralism.
    pub fn pluralism(&self) -> Confidence {
        let density = FactionDensity::new(self.active_factions.len(), self.total_domains);
        if density.is_stable() {
            Confidence::new(0.9)
        } else {
            Confidence::new(0.4) // Risk of majority tyranny
        }
    }

    /// Detect if a Resolution is an "Impulse of Passion" (Adverse Interest).
    pub fn detect_adversity(&self, _proposal: &Resolution, _proposer_id: &str) -> Adversity {
        // In a real system, this would analyze the Rule within the Resolution
        // and compare it against the interests of other factions.
        Adversity::Neutral
    }

    /// Apply the "Large Republic" cure: escalate quorum if density is low.
    pub fn required_quorum(&self) -> f64 {
        let density = FactionDensity::new(self.active_factions.len(), self.total_domains);
        if density.0 < 0.2 {
            0.75 // Escalate to supermajority if too few factions exist
        } else {
            0.50 // Standard majority
        }
    }
}
