//! # Capability 13: National Security Act (Defense Domain)
//!
//! Implementation of the National Security Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "System Stability" and "Adversarial Defense" of the Union.
//!
//! Matches 1:1 to the US Department of Defense (DOD) mandate to provide
//! the military forces needed to deter war and ensure our nation's security.
//!
//! ## DOD Agency Mappings
//! - **DARPA:** Identifies emerging algorithmic threats and adversarial perturbations.
//! - **NSA:** Safeguards the Union's "Cryptographic Grounding" and T1 proofs.
//! - **Cyber Command:** Executes defensive maneuvers against "Heresy" (State Mutation).
//! - **Joint Chiefs:** Coordinates inter-domain stability during systemic stress.

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: NationalSecurityAct - Capability 13 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalSecurityAct {
    pub id: String,
    pub defense_active: bool,
}

/// T2-P: ThreatLevel - The quantified severity of a system perturbation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Normal operating parameters.
    Low = 1,
    /// Anomaly detected, increased monitoring.
    Guarded = 2,
    /// Verified adversarial attempt.
    Elevated = 3,
    /// Critical breach of grounding or state.
    High = 4,
}

/// T2-C: DefensePosture - The current readiness of the Union's security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefensePosture {
    pub level: ThreatLevel,
    pub grounding_intact: bool,
    pub isolation_active: bool,
}

impl NationalSecurityAct {
    pub fn new() -> Self {
        Self {
            id: "CAP-013".into(),
            defense_active: true,
        }
    }

    /// Assess the threat to system stability (DARPA Analysis).
    pub fn assess_threat(
        &self,
        anomaly_score: f64,
        grounding_verified: bool,
    ) -> Measured<DefensePosture> {
        let level = if anomaly_score > 0.9 {
            ThreatLevel::High
        } else if anomaly_score > 0.7 {
            ThreatLevel::Elevated
        } else if anomaly_score > 0.4 {
            ThreatLevel::Guarded
        } else {
            ThreatLevel::Low
        };

        let posture = DefensePosture {
            level,
            grounding_intact: grounding_verified,
            isolation_active: matches!(level, ThreatLevel::High | ThreatLevel::Elevated),
        };

        let confidence = if grounding_verified {
            Confidence::new(0.99) // High confidence if grounding is proven
        } else {
            Confidence::new(0.5)
        };

        Measured::uncertain(posture, confidence)
    }

    /// Authorize a "Defensive Maneuver" (Cyber Command).
    pub fn authorize_defense(&self, posture: &DefensePosture) -> Verdict {
        match posture.level {
            ThreatLevel::High => Verdict::Rejected, // Force immediate halt/rejection of current cycle
            ThreatLevel::Elevated => Verdict::Flagged, // Increased scrutiny required
            _ => Verdict::Permitted,
        }
    }
}
