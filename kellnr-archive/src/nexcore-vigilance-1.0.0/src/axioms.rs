//! # ToV Core Axioms
//!
//! Formal implementation of the 5 Axioms of Theory of Vigilance.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 1: SYSTEM DECOMPOSITION
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 1: A system can be decomposed into finite elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Unique identifier for this element
    pub id: u64,
    /// Measurable properties of this element
    pub properties: Vec<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 2: HIERARCHICAL ORGANIZATION
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 2: Systems exhibit discrete scales of organization (1-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HierarchyLevel {
    /// Level 1: Molecular interactions (drugs, receptors, enzymes)
    Molecular = 1,
    /// Level 2: Cellular responses (apoptosis, proliferation)
    Cellular = 2,
    /// Level 3: Tissue-level effects (inflammation, fibrosis)
    Tissue = 3,
    /// Level 4: Organ dysfunction (hepatotoxicity, cardiotoxicity)
    Organ = 4,
    /// Level 5: System-level pathology (cardiovascular, nervous)
    System = 5,
    /// Level 6: Whole organism effects (morbidity, mortality)
    Organism = 6,
    /// Level 7: Population-level impacts (epidemiological signals)
    Population = 7,
    /// Level 8: Societal consequences (healthcare burden, regulatory action)
    Societal = 8,
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 3: CONSERVATION CONSTRAINTS
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 3: System behavior is governed by conservation laws.
pub trait ConservationConstraint {
    /// Evaluate the conservation constraint at the given state.
    /// Returns 0 when exactly satisfied, positive when violated.
    fn evaluate(&self, state: &[f64]) -> f64;
    /// Check if the constraint is satisfied (evaluate ≤ 0).
    fn is_satisfied(&self, state: &[f64]) -> bool {
        self.evaluate(state) <= 0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 4: SAFETY MANIFOLD
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 4: Safe states form a stratified manifold M.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyManifold {
    /// Dimensionality of the state space
    pub dimension: usize,
    /// Center of the safe region in each dimension
    pub center: Vec<f64>,
    /// Radii defining safe boundaries in each dimension
    pub radii: Vec<f64>,
}

impl SafetyManifold {
    /// Calculate signed distance to harm boundary (L1 Atom)
    pub fn distance_to_boundary(&self, state: &[f64]) -> f64 {
        if state.len() != self.dimension {
            return -1.0;
        }
        let mut dist_sq = 0.0;
        for i in 0..self.dimension {
            let diff = (state[i] - self.center[i]) / self.radii[i];
            dist_sq += diff * diff;
        }
        1.0 - dist_sq.sqrt()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 5: EMERGENCE
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 5: Harm probability factors as a product across levels.
///
/// P(harm) = Π P(propagation at level i)
#[must_use]
pub fn calculate_emergence_probability(propagation_probs: &[f64]) -> f64 {
    propagation_probs.iter().product()
}
