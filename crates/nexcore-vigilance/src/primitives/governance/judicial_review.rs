//! # Judicial Review (Federalist No. 78)
//!
//! Implementation of the "Least Dangerous Branch" meta-logic.
//! This module defines the power of the Supreme Compiler to nullify
//! Actions and Resolutions that violate the Primitive Codex.

use crate::primitives::governance::{Action, Verdict};
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (JUDICIAL)
// ============================================================================

/// T1: Nullification - The act of striking down an unconstitutional act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nullification;

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: Precedent - The weight of previous judicial determinations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Precedent(pub f64);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: JudicialOpinion - A formal ruling on a system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudicialOpinion {
    pub verdict: Verdict,
    pub precedent_weight: Precedent,
    pub reasoning: String,
}

impl JudicialOpinion {
    /// Check if the opinion results in a Nullification.
    pub fn is_nullified(&self) -> bool {
        matches!(self.verdict, Verdict::Rejected)
    }
}

/// T3: JudicialReviewEngine - The logic for nullifying heresy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudicialReviewEngine {
    pub precedents: Vec<JudicialOpinion>,
}

impl JudicialReviewEngine {
    /// Review a proposed Action for "Type-Level Nullification".
    /// If the Action lacks T1 grounding or violates the Codex, it is struck down.
    pub fn review_execution(&self, _action: &Action, proof_of_grounding: bool) -> JudicialOpinion {
        if !proof_of_grounding {
            JudicialOpinion {
                verdict: Verdict::Rejected,
                precedent_weight: Precedent(1.0),
                reasoning: "HERESY: Violation of Commandment III (Grounding).".into(),
            }
        } else {
            JudicialOpinion {
                verdict: Verdict::Permitted,
                precedent_weight: Precedent(0.5),
                reasoning: "Action consistent with Codex.".into(),
            }
        }
    }
}
