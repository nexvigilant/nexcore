//! # The Administrative Procedure Act (APA)
//!
//! Implementation of SOP execution and agency regulation.
//! This module defines how agencies (e.g., RiskMinimizer) must
//! follow codified rules and provide public logs for their actions.

use crate::primitives::governance::{Rule, Verdict};
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (ADMINISTRATION)
// ============================================================================

/// T1: Procedure - An irreducible sequence of operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Procedure(pub Vec<String>);

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: Compliance - A measure of how well an agency followed a Procedure.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Compliance(pub f64);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: AdministrativeRule - A rule that governs an Agency's internal logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdministrativeRule {
    pub target_agency_id: String,
    pub procedure: Procedure,
    pub constraint: Rule,
}

/// T2-C: AgencyLog - The public record of administrative action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgencyLog {
    pub agency_id: String,
    pub action_taken: String,
    pub compliance_score: Compliance,
}

impl AdministrativeRule {
    /// Verify an agency action against the SOP.
    pub fn verify_compliance(&self, action_trace: &[String]) -> Compliance {
        // Simplified compliance check: sequence alignment
        let matches = action_trace
            .iter()
            .filter(|step| self.procedure.0.contains(step))
            .count();
        Compliance(matches as f64 / self.procedure.0.len() as f64)
    }

    /// Judicial review of an administrative action.
    pub fn judicial_review(&self, log: &AgencyLog) -> Verdict {
        if log.compliance_score.0 > 0.9 {
            Verdict::Permitted
        } else {
            Verdict::Flagged
        }
    }
}
