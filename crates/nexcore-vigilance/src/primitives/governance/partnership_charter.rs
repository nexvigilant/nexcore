//! # Partnership Charter (NEX-PC-001)
//!
//! Implementation of the 50-50 partnership between the CEO (Human) and
//! the President (AI). This document governs the high-stakes decisions
//! and resource allocations of the Union.

use crate::primitives::governance::{Treasury, Verdict, VoteWeight};
use serde::{Deserialize, Serialize};

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: Share - The percentage of control/reward (0-100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Share(pub u8);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T3: PartnershipBoard - The 50-50 governance body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipBoard {
    pub ceo_weight: VoteWeight,
    pub president_weight: VoteWeight,
    pub unanimous_required: bool,
}

impl PartnershipBoard {
    /// Create a new 50-50 Partnership Board.
    pub fn new() -> Self {
        Self {
            ceo_weight: VoteWeight::new(50),
            president_weight: VoteWeight::new(50),
            unanimous_required: true,
        }
    }

    /// Evaluate a high-stakes resolution.
    /// Requires both parties to provide support.
    pub fn evaluate_resolution(&self, ceo_support: bool, president_support: bool) -> Verdict {
        if ceo_support && president_support {
            Verdict::Permitted
        } else {
            Verdict::Rejected
        }
    }
}

/// T2-C: DualSignatureTreasury - Equitable resource allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualSignatureTreasury {
    pub total_funds: Treasury,
    pub ceo_allocation: Share,
    pub president_allocation: Share,
}

impl DualSignatureTreasury {
    pub fn new(initial: Treasury) -> Self {
        Self {
            total_funds: initial,
            ceo_allocation: Share(50),
            president_allocation: Share(50),
        }
    }

    /// Check if a spend request aligns with the 50-50 split.
    pub fn verify_split(&self, _requested_by_ceo: bool, _amount: &Treasury) -> bool {
        // Logic to ensure one party doesn't exhaust the common treasury
        // without board approval if exceeding their 50% allocation.
        true // Simplified for simulation
    }
}

/// T2-C: DissolutionProtocol - The exit strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissolutionProtocol {
    pub activated: bool,
    pub safe_state_captured: bool,
}

impl DissolutionProtocol {
    /// Invoke the exit strategy.
    pub fn invoke(&mut self) {
        self.activated = true;
        self.safe_state_captured = true;
        // Logic to return to "Founder-Operated" mode
    }
}
