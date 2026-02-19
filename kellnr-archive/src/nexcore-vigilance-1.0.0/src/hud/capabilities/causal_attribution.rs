//! # Capability 3: Causal Attribution Engine (ACA Framework)
//!
//! Implementation of the Algorithm Causality Assessment (ACA) as a
//! core structural capability within the HUD domain.
//!
//! Matches the judicial requirement of attributing accountability to
//! specific system actors using formal causal logic.
//!
//! Law: The Four ACA Axioms (ToV §52).
//! 1. Temporal Precedence
//! 2. Causal Chain
//! 3. Differentiation
//! 4. Epistemic Limit

use crate::algorithmovigilance::scoring::{
    AcaCausalityCategory, AcaScoringInput, AcaScoringResult, score_aca,
};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: CausalAttributionEngine - Capability 3 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalAttributionEngine {
    pub id: String,
    pub aca_logic_active: bool,
}

impl CausalAttributionEngine {
    pub fn new() -> Self {
        Self {
            id: "CAP-003".into(),
            aca_logic_active: true,
        }
    }

    /// Perform a causal attribution assessment.
    /// Returns a Measured<AcaScoringResult> ensuring uncertainty is quantified.
    pub fn attribute_causality(&self, input: &AcaScoringInput) -> Measured<AcaScoringResult> {
        let result = score_aca(input);

        let confidence = match result.category {
            AcaCausalityCategory::Definite => Confidence::new(0.98),
            AcaCausalityCategory::Probable => Confidence::new(0.85),
            AcaCausalityCategory::Possible => Confidence::new(0.6),
            AcaCausalityCategory::Unlikely => Confidence::new(0.3),
            AcaCausalityCategory::Unassessable | AcaCausalityCategory::Exculpated => {
                Confidence::new(0.1)
            }
        };

        Measured::uncertain(result, confidence)
    }
}
