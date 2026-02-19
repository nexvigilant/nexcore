//! # State Security (Bill of Rights)
//!
//! Implementation of Amendment IV: State Security and Probable Cause logic.
//! This module handles "searches and seizures" within the system's memory/state.

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: Clearance - A warrant for state inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clearance {
    pub target_id: String,
    pub probable_cause_confidence: Confidence,
    pub oath_of_rigor: bool,
}

impl Clearance {
    /// Verify if the clearance is constitutional.
    pub fn is_constitutional(&self) -> bool {
        self.probable_cause_confidence.value() > 0.7 && self.oath_of_rigor
    }
}

/// T3: StateGuardian - The agency responsible for system security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateGuardian {
    pub alerts: Vec<SecurityAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub source_id: String,
    pub anomaly_score: f64,
}

impl StateGuardian {
    /// Request a Clearance based on an alert.
    pub fn request_clearance(&self, alert_index: usize) -> Option<Clearance> {
        let alert = self.alerts.get(alert_index)?;
        Some(Clearance {
            target_id: alert.source_id.clone(),
            probable_cause_confidence: Confidence::new(alert.anomaly_score),
            oath_of_rigor: true,
        })
    }

    /// Execute a search and seizure.
    pub fn execute_search(&self, clearance: &Clearance) -> Verdict {
        if clearance.is_constitutional() {
            Verdict::Permitted
        } else {
            Verdict::Rejected
        }
    }
}
