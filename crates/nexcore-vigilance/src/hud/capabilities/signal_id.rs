//! # Capability 1: Signal Identification Protocol (BDI Engine)
//!
//! Implementation of the Betting Disproportionality Index (BDI) as a
//! core structural capability within the HUD domain.
//!
//! Grounding: Adapted from PRR in pharmacovigilance.
//! Law: Evans Criteria (BDI >= 2.0, χ² >= 3.841, N >= 3).

use nexcore_labs::betting::bdi::{BdiResult, ContingencyTable, calculate_bdi};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: SignalIdentificationProtocol - Capability 1 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalIdentificationProtocol {
    pub id: String,
    pub bdi_engine_active: bool,
}

impl SignalIdentificationProtocol {
    pub fn new() -> Self {
        Self {
            id: "CAP-001".into(),
            bdi_engine_active: true,
        }
    }

    /// Identify a signal using the BDI engine.
    /// Returns a Measured<BdiResult> ensuring uncertainty is quantified.
    pub fn identify_signal(&self, table: ContingencyTable) -> Measured<BdiResult> {
        let result = calculate_bdi(table);
        let confidence = if result.meets_criteria {
            Confidence::new(0.95) // High confidence in validated signal
        } else {
            Confidence::new(0.4) // Low confidence in unvalidated noise
        };

        Measured::uncertain(result, confidence)
    }
}
