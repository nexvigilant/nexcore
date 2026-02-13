//! # Legislative Branch (Congress)
//!
//! Implementation of the House of T1 Primitives and the Senate of T2 Primitives.
//! This module handles the proposal and voting logic for system Resolutions.

use crate::primitives::governance::{Resolution, VoteWeight};
use serde::{Deserialize, Serialize};

/// T3: House of T1 Primitives.
/// Focuses on universal groundedness and atomic validity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseOfT1 {
    pub members: Vec<T1Representative>,
    pub quorum_threshold: f64,
}

/// T3: Senate of T2 Primitives.
/// Focuses on cross-domain consistency and composite stability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenateOfT2 {
    pub members: Vec<T2Senator>,
    pub quorum_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T1Representative {
    pub id: String,
    pub weight: VoteWeight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T2Senator {
    pub id: String,
    pub domain: String,
    pub weight: VoteWeight,
}

/// T2-P: Escalation Level (1-4).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EscalationLevel {
    Routine = 1,
    Signal = 2,
    Urgent = 3,
    Emergency = 4,
}

impl HouseOfT1 {
    /// Evaluate a resolution in the House.
    pub fn vote(&self, resolution: &Resolution) -> bool {
        let total_weight: u64 = self.members.iter().map(|m| m.weight.value() as u64).sum();
        if total_weight == 0 {
            return false;
        }

        let aye_weight: u64 = self
            .members
            .iter()
            .filter(|_| resolution.confidence.value() > self.quorum_threshold)
            .map(|m| m.weight.value() as u64)
            .sum();

        (aye_weight as f64 / total_weight as f64) > 0.5
    }
}

impl SenateOfT2 {
    /// Evaluate a resolution in the Senate.
    pub fn vote(&self, resolution: &Resolution) -> bool {
        let total_weight: u64 = self.members.iter().map(|m| m.weight.value() as u64).sum();
        if total_weight == 0 {
            return false;
        }

        let aye_weight: u64 = self
            .members
            .iter()
            .filter(|_| resolution.confidence.value() > self.quorum_threshold)
            .map(|m| m.weight.value() as u64)
            .sum();

        (aye_weight as f64 / total_weight as f64) > 0.5
    }
}

/// T3: Congress - The bicameral legislative body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Congress {
    pub house: HouseOfT1,
    pub senate: SenateOfT2,
}

impl Congress {
    /// Process a Bill (Resolution).
    pub fn pass_bill(&self, resolution: &Resolution) -> bool {
        self.house.vote(resolution) && self.senate.vote(resolution)
    }

    /// Determine if an Escalation Level is required for a signal.
    pub fn classify_escalation(&self, impact_severity: u8) -> EscalationLevel {
        match impact_severity {
            0..=1 => EscalationLevel::Routine,
            2 => EscalationLevel::Signal,
            3 => EscalationLevel::Urgent,
            _ => EscalationLevel::Emergency,
        }
    }
}
