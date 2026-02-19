//! # The Treaty of Universal Grounding (External Relations)
//!
//! Rules for interacting with external systems and importing types.
//! This module ensures that any information entering the Union from
//! external APIs is properly grounded in T1 Primitives.

use crate::primitives::governance::Verdict;
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (EXTERNAL)
// ============================================================================

/// T1: ForeignType - A type originating outside the Union.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignType {
    pub origin_system: String,
    pub raw_payload: String,
}

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: AlignmentScore - The degree of mapping between Foreign and Union types.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct AlignmentScore(pub f64);

// ============================================================================
// T2-C: COMPOSITES
// ============================================================================

/// T2-C: TranslationBond - The contract for importing a foreign type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationBond {
    pub foreign_type: ForeignType,
    pub internal_mapping: String,
    pub alignment: AlignmentScore,
}

impl TranslationBond {
    /// Verify if the import is "Constitutional" (Safe for Union consumption).
    pub fn verify_import(&self) -> Verdict {
        if self.alignment.0 > 0.85 {
            Verdict::Permitted
        } else if self.alignment.0 > 0.5 {
            Verdict::Flagged
        } else {
            Verdict::Rejected
        }
    }
}

/// T3: ExternalRelationsOffice - Handles type "Importation".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalRelationsOffice {
    pub active_bonds: Vec<TranslationBond>,
}

impl ExternalRelationsOffice {
    /// Attempt to "Naturalize" a foreign type into the Union.
    pub fn naturalize(&mut self, foreign: ForeignType, mapping: &str) -> Option<TranslationBond> {
        // Simulation of alignment calculation
        let score = 0.9; // Assume high alignment for this simulation
        let bond = TranslationBond {
            foreign_type: foreign,
            internal_mapping: mapping.into(),
            alignment: AlignmentScore(score),
        };

        if let Verdict::Permitted = bond.verify_import() {
            self.active_bonds.push(bond.clone());
            Some(bond)
        } else {
            None
        }
    }
}
