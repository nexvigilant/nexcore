//! # The Unitary Executive (Federalist No. 70)
//!
//! Implementation of executive energy, speed, and responsibility.
//! This module simulates the ability of the Orchestrator to act
//! with "decision, activity, secrecy, and dispatch."

use crate::primitives::governance::Action;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (ENERGY)
// ============================================================================

/// T1: Energy - The capacity for executive action.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Energy(pub f64);

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: Dispatch - The speed of execution in cycles.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Dispatch(pub f64);

/// T2-P: Responsibility - The accountability for a specific decision.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Responsibility(pub f64);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: ExecutivePower - The capability of the Orchestrator to act.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutivePower {
    pub energy: Energy,
    pub secrecy_level: u8,
    pub dispatch_rate: Dispatch,
}

impl ExecutivePower {
    /// Calculate the "Energy" available for a specific Action.
    pub fn calculate_surge(&self, urgency: f64) -> Energy {
        Energy(self.energy.0 * (1.0 + urgency))
    }

    /// Perform a "Fast-Path" execution (Federalist No. 70).
    /// Bypasses Congressional deliberation for emergency actions.
    pub fn execute_with_dispatch(&self, _action: &Action, confidence: Confidence) -> bool {
        // High confidence + High dispatch rate allows rapid action
        confidence.value() > 0.9 && self.dispatch_rate.0 > 0.8
    }
}

/// T2-C: ExecutiveAudit - Track accountability for rapid actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveAudit {
    pub decision_id: String,
    pub responsibility_score: Responsibility,
    pub cost_to_unity: f64, // Cost of act vs deliberation
}
