//! # Capability 36: National Intelligence Act (Analytics & Intelligence)
//!
//! Implementation of the National Intelligence Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Intelligence Cycle" and "Advanced Analytics" of the Union.
//!
//! Matches 1:1 to the US Office of the Director of National Intelligence (ODNI)
//! mandate to lead intelligence integration and forge an Intelligence
//! Community that delivers the most insightful intelligence possible.
//!
//! ## Intelligence Agency Mappings
//! - **CIA (Human/Agent):** Manages the "Covert" and autonomous agent operations.
//! - **NSA (Signals):** Analyzes the raw "Signal" traffic for pattern recognition.
//! - **DIA (Defense):** Focuses on "Threat" assessment and defensive intelligence.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: NationalIntelligenceAct - Capability 36 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationalIntelligenceAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the intelligence cycle is active.
    pub intelligence_active: bool,
}

/// T2-P: IntelligenceScore - The quantified insight derived from a signal.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct IntelligenceScore(pub f64);

/// T2-C: IntelligenceReport - A summarized insight from the community.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceReport {
    /// The identifier of the subject signal/agent.
    pub subject_id: String,
    /// The calculated insight score.
    pub insight: IntelligenceScore,
    /// The confidence in the intelligence assessment.
    pub confidence: Confidence,
}

impl NationalIntelligenceAct {
    /// Creates a new instance of the NationalIntelligenceAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-036".into(),
            intelligence_active: true,
        }
    }

    /// Run the intelligence cycle on a signal.
    pub fn derive_insight(
        &self,
        signal_id: &str,
        raw_data: &[f64],
    ) -> Measured<IntelligenceReport> {
        let score = if raw_data.is_empty() {
            0.0
        } else {
            raw_data.iter().sum::<f64>() / raw_data.len() as f64
        };

        let report = IntelligenceReport {
            subject_id: signal_id.to_string(),
            insight: IntelligenceScore(score),
            confidence: Confidence::new(0.88),
        };

        Measured::uncertain(report, Confidence::new(0.95))
    }
}
