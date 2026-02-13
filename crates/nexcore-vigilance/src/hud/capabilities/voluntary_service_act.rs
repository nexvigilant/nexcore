//! # Capability 33: Voluntary Service Act (Community Signals)
//!
//! Implementation of the Voluntary Service Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Community Reporting" and "Voluntary Participation" of the Union.
//!
//! Matches 1:1 to the US Peace Corps mandate to promote world peace
//! and friendship by fulfilling three missions.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: VoluntaryServiceAct - Capability 33 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoluntaryServiceAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the community reporting engine is active.
    pub community_active: bool,
}

/// T2-P: EngagementLevel - The quantified volume of voluntary reports.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EngagementLevel(pub f64);

/// T2-C: VolunteerReport - A suspected signal reported by the community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolunteerReport {
    /// The identifier of the reporter.
    pub reporter_id: String,
    /// The suspected observation (e.g., "Drug-Event Link").
    pub observation: String,
    /// The perceived severity.
    pub severity: u8,
}

impl VoluntaryServiceAct {
    /// Creates a new instance of the VoluntaryServiceAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-033".into(),
            community_active: true,
        }
    }

    /// Process a voluntary report from the community.
    pub fn process_report(&self, reporter: &str, obs: &str) -> Measured<VolunteerReport> {
        let report = VolunteerReport {
            reporter_id: reporter.to_string(),
            observation: obs.to_string(),
            severity: 2,
        };

        Measured::uncertain(report, Confidence::new(0.65))
    }
}
