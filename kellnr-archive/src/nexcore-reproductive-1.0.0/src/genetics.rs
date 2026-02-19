//! # Germ-Line Genetics
//!
//! Defines the "NexCore DNA" requirements that all offspring (artifacts/code changes)
//! must satisfy to prevent architectural drift.

use serde::{Deserialize, Serialize};
use nexcore_lex_primitiva::LexPrimitiva;

/// Rules for architectural heritability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeRequirement {
    /// List of mandatory T1 primitives that must be present.
    pub mandatory_primitives: Vec<LexPrimitiva>,
    /// Whether unsafe code is forbidden (Germ-line purity).
    pub forbid_unsafe: bool,
    /// Minimum test coverage requirement.
    pub min_coverage: f64,
}

impl Default for GenomeRequirement {
    fn default() -> Self {
        Self {
            mandatory_primitives: vec![
                LexPrimitiva::Persistence, // π
                LexPrimitiva::Boundary,    // ∂
                LexPrimitiva::Sequence,    // σ
            ],
            forbid_unsafe: true,
            min_coverage: 0.85,
        }
    }
}

/// Validates if an artifact's "Genetic Material" matches the NexCore Genome.
pub struct GeneticGuard {
    pub requirement: GenomeRequirement,
}

impl GeneticGuard {
    pub fn new(requirement: GenomeRequirement) -> Self {
        Self { requirement }
    }

    /// Checks if a proposed change (Mutation) is "Lethal" (violates DNA).
    pub fn is_mutation_lethal(&self, primitives_present: &[LexPrimitiva], uses_unsafe: bool) -> bool {
        if self.requirement.forbid_unsafe && uses_unsafe {
            return true;
        }

        for mandatory in &self.requirement.mandatory_primitives {
            if !primitives_present.contains(mandatory) {
                return true; // Lethal: missing mandatory DNA
            }
        }

        false
    }
}
