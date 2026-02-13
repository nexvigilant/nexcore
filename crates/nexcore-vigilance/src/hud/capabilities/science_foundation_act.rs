//! # Capability 31: Science Foundation Act (R&D)
//!
//! Implementation of the Science Foundation Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Algorithmic Research" and "Fundamental R&D" of the Union.
//!
//! Matches 1:1 to the US National Science Foundation (NSF) mandate
//! to promote the progress of science; to advance the national health,
//! prosperity, and welfare; to secure the national defense.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: ScienceFoundationAct - Capability 31 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScienceFoundationAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the research engine is active.
    pub research_active: bool,
}

/// T2-P: InnovationRate - The quantified output of new algorithms.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct InnovationRate(pub f64);

/// T2-C: ResearchGrant - Support for a specific algorithmic investigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchGrant {
    /// The title of the research project.
    pub project_title: String,
    /// The target capability being enhanced.
    pub target_cap_id: String,
    /// The expected impact score.
    pub impact: f64,
}

impl ScienceFoundationAct {
    /// Creates a new instance of the ScienceFoundationAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-031".into(),
            research_active: true,
        }
    }

    /// Fund a research project to improve signal detection.
    pub fn fund_research(&self, project: &str, target_cap: &str) -> Measured<ResearchGrant> {
        let grant = ResearchGrant {
            project_title: project.to_string(),
            target_cap_id: target_cap.to_string(),
            impact: 0.85,
        };

        Measured::uncertain(grant, Confidence::new(0.9))
    }
}
