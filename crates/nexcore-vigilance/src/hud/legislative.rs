//! # Legislative Branch (Department of State Mapping)
//!
//! Implementation of the Legislative Branch within the HUD domain.
//! This module manages the "State of the Union" via Protocol Codification
//! and the bicameral Legislative body (Congress).
//!
//! Matches 1:1 to the US Department of State (DOS) mandate for
//! international (inter-domain) relations and protocol management.

use crate::primitives::governance::{Congress, Resolution, Verdict};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: LegislativeBranch - Capability 15 of 37 (The State Act).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegislativeBranch {
    /// The unique capability identifier.
    pub id: String,
    /// The bicameral legislative body (Congress).
    pub congress: Congress,
}

/// T2-B: Bill - A proposed legislative action (Resolution wrapper).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bill {
    /// The unique bill identifier.
    pub id: String,
    /// The proposed resolution containing the rule change.
    pub resolution: Resolution,
    /// The identifier of the sponsoring agent.
    pub sponsor_id: String,
}

impl LegislativeBranch {
    /// Creates a new instance of the LegislativeBranch with the provided Congress.
    pub fn new(congress: Congress) -> Self {
        Self {
            id: "CAP-015".into(),
            congress,
        }
    }

    /// Process a Bill through the bicameral legislature.
    pub fn process_bill(&self, bill: &Bill) -> Measured<Verdict> {
        let passed = self.congress.pass_bill(&bill.resolution);

        let verdict = if passed {
            Verdict::Permitted
        } else {
            Verdict::Rejected
        };

        // Legislative confidence is tied to the bill's resolution confidence
        Measured::uncertain(verdict, bill.resolution.confidence)
    }

    /// Codify a new Protocol (The "Treaty" mapping).
    pub fn codify_protocol(&self, _protocol_id: &str) -> Confidence {
        // Implementation of protocol stabilization logic
        Confidence::new(1.0)
    }
}
