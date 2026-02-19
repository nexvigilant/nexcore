//! # Dependency Graph
//!
//! The 4-layer dependency graph from primitives to mathematical constants.
// complexity_override: approved by user for comprehensive graph mapping

use crate::bedrock::BedrockAtom;
use crate::constants::MathConstant;
use crate::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Mathematical foundation category grounding computational primitives.
///
/// Each bedrock atom traces through one of these 10 foundational branches
/// to reach its terminal mathematical constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MathFoundation {
    /// Natural number construction (0, S(n), induction).
    PeanoAxioms,
    /// σ-algebras, integration, probability spaces.
    MeasureTheory,
    /// Partial/total orders, lattices, well-orderings.
    OrderTheory,
    /// Objects, morphisms, functors, natural transformations.
    CategoryTheory,
    /// Membership, union, intersection, complement.
    SetTheory,
    /// Limits, continuity, differentiation, convergence.
    Analysis,
    /// Fourier transforms, frequency domain, sampling.
    SignalTheory,
    /// Entropy, mutual information, channel capacity.
    InformationTheory,
    /// Heat, entropy increase, Boltzmann statistics.
    Thermodynamics,
    /// Banach, Brouwer, Knaster-Tarski theorems.
    FixedPointTheory,
}

impl MathFoundation {
    /// Returns the human-readable name of this foundation.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PeanoAxioms => "Peano Axioms",
            Self::MeasureTheory => "Measure Theory",
            Self::OrderTheory => "Order Theory",
            Self::CategoryTheory => "Category Theory",
            Self::SetTheory => "Set Theory",
            Self::Analysis => "Analysis",
            Self::SignalTheory => "Signal Theory",
            Self::InformationTheory => "Information Theory",
            Self::Thermodynamics => "Thermodynamics",
            Self::FixedPointTheory => "Fixed-Point Theory",
        }
    }

    /// Returns the terminal mathematical constants reachable from this foundation.
    #[must_use]
    pub fn terminal_constants(&self) -> Vec<MathConstant> {
        match self {
            Self::PeanoAxioms => vec![MathConstant::ZERO, MathConstant::ONE, MathConstant::OMEGA],
            Self::MeasureTheory => vec![MathConstant::ZERO, MathConstant::INFINITY],
            Self::OrderTheory => vec![MathConstant::ZERO, MathConstant::ONE],
            Self::CategoryTheory => vec![MathConstant::ZERO, MathConstant::ONE],
            Self::SetTheory => vec![MathConstant::ZERO, MathConstant::ONE],
            Self::Analysis => vec![
                MathConstant::ZERO,
                MathConstant::EPSILON,
                MathConstant::INFINITY,
            ],
            Self::SignalTheory => vec![MathConstant::ONE, MathConstant::PI, MathConstant::E],
            Self::InformationTheory => vec![MathConstant::ZERO, MathConstant::LN_2],
            Self::Thermodynamics => vec![
                MathConstant::ZERO,
                MathConstant::K_BOLTZMANN,
                MathConstant::LN_2,
            ],
            Self::FixedPointTheory => vec![MathConstant::ONE, MathConstant::PHI, MathConstant::E],
        }
    }

    /// Returns all 10 mathematical foundations.
    #[must_use]
    pub const fn all() -> [Self; 10] {
        [
            Self::PeanoAxioms,
            Self::MeasureTheory,
            Self::OrderTheory,
            Self::CategoryTheory,
            Self::SetTheory,
            Self::Analysis,
            Self::SignalTheory,
            Self::InformationTheory,
            Self::Thermodynamics,
            Self::FixedPointTheory,
        ]
    }
}

/// A trace from primitive to constant through the 4-layer grounding hierarchy.
///
/// Captures the full path: `LexPrimitiva → BedrockAtom → MathFoundation → MathConstant`.
#[derive(Debug, Clone, Serialize)]
pub struct GroundingTrace {
    /// The top-level Lex Primitiva (e.g., Quantity, Causality).
    pub primitive: LexPrimitiva,
    /// The intermediate bedrock atom (e.g., Magnitude, Mechanism).
    pub bedrock: BedrockAtom,
    /// The mathematical foundation branch (e.g., MeasureTheory).
    pub foundation: MathFoundation,
    /// The terminal constant (e.g., 0, 1, π, φ).
    pub constant: MathConstant,
    /// Confidence score [0.0, 1.0] for this grounding path.
    pub confidence: f64,
}

impl GroundingTrace {
    /// Creates a new trace with default confidence of 1.0.
    #[must_use]
    pub fn new(
        primitive: LexPrimitiva,
        bedrock: BedrockAtom,
        foundation: MathFoundation,
        constant: MathConstant,
    ) -> Self {
        Self {
            primitive,
            bedrock,
            foundation,
            constant,
            confidence: 1.0,
        }
    }
}

impl std::fmt::Display for GroundingTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} → {} → {} → {}",
            self.primitive.symbol(),
            self.bedrock.name(),
            self.foundation.name(),
            self.constant.symbol
        )
    }
}

// Bedrock → Foundation mappings as const arrays (split by primitive)
const QUANTITY_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::MeasureTheory,
    MathFoundation::MeasureTheory,
    MathFoundation::PeanoAxioms,
    MathFoundation::MeasureTheory,
    MathFoundation::MeasureTheory,
];
const CAUSALITY_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::CategoryTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::OrderTheory,
    MathFoundation::SetTheory,
];
const COMPARISON_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::OrderTheory,
    MathFoundation::SetTheory,
    MathFoundation::OrderTheory,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
];
const SEQUENCE_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::OrderTheory,
    MathFoundation::PeanoAxioms,
    MathFoundation::PeanoAxioms,
    MathFoundation::CategoryTheory,
    MathFoundation::PeanoAxioms,
];
const MAPPING_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::CategoryTheory,
];
const STATE_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::PeanoAxioms,
    MathFoundation::CategoryTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
];
const RECURSION_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::PeanoAxioms,
    MathFoundation::PeanoAxioms,
    MathFoundation::FixedPointTheory,
    MathFoundation::OrderTheory,
    MathFoundation::PeanoAxioms,
];
const VOID_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::SetTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::SetTheory,
];
const BOUNDARY_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::Analysis,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
];
const FREQUENCY_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SignalTheory,
    MathFoundation::SignalTheory,
    MathFoundation::Analysis,
    MathFoundation::SignalTheory,
    MathFoundation::SignalTheory,
];
const EXISTENCE_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
];
const PERSISTENCE_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::InformationTheory,
    MathFoundation::InformationTheory,
    MathFoundation::InformationTheory,
    MathFoundation::InformationTheory,
    MathFoundation::InformationTheory,
];
const LOCATION_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
    MathFoundation::Analysis,
];
const IRREVERSIBILITY_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::CategoryTheory,
    MathFoundation::InformationTheory,
    MathFoundation::Thermodynamics,
    MathFoundation::CategoryTheory,
    MathFoundation::CategoryTheory,
];
const SUM_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory,
    MathFoundation::CategoryTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
    MathFoundation::SetTheory,
];
const PRODUCT_FOUNDATIONS: [MathFoundation; 5] = [
    MathFoundation::SetTheory, // ComponentField — Cartesian product components
    MathFoundation::CategoryTheory, // Projection — categorical product projections π₁, π₂
    MathFoundation::CategoryTheory, // Pairing — universal property of products
    MathFoundation::SetTheory, // RecordAccess — indexed field selection
    MathFoundation::PeanoAxioms, // Arity — natural number counting components
];

/// Get foundation mappings for a primitive.
#[must_use]
pub const fn foundations_for_primitive(primitive: LexPrimitiva) -> &'static [MathFoundation; 5] {
    match primitive {
        LexPrimitiva::Quantity => &QUANTITY_FOUNDATIONS,
        LexPrimitiva::Causality => &CAUSALITY_FOUNDATIONS,
        LexPrimitiva::Comparison => &COMPARISON_FOUNDATIONS,
        LexPrimitiva::Sequence => &SEQUENCE_FOUNDATIONS,
        LexPrimitiva::Mapping => &MAPPING_FOUNDATIONS,
        LexPrimitiva::State => &STATE_FOUNDATIONS,
        LexPrimitiva::Recursion => &RECURSION_FOUNDATIONS,
        LexPrimitiva::Void => &VOID_FOUNDATIONS,
        LexPrimitiva::Boundary => &BOUNDARY_FOUNDATIONS,
        LexPrimitiva::Frequency => &FREQUENCY_FOUNDATIONS,
        LexPrimitiva::Existence => &EXISTENCE_FOUNDATIONS,
        LexPrimitiva::Persistence => &PERSISTENCE_FOUNDATIONS,
        LexPrimitiva::Location => &LOCATION_FOUNDATIONS,
        LexPrimitiva::Irreversibility => &IRREVERSIBILITY_FOUNDATIONS,
        LexPrimitiva::Sum => &SUM_FOUNDATIONS,
        LexPrimitiva::Product => &PRODUCT_FOUNDATIONS,
    }
}

/// The dependency graph.
#[derive(Debug, Clone, Default)]
pub struct DependencyGraph;

impl DependencyGraph {
    /// Get all terminal constants reachable from a primitive.
    #[must_use]
    pub fn constants_for_primitive(primitive: LexPrimitiva) -> HashSet<&'static str> {
        let foundations = foundations_for_primitive(primitive);
        let mut constants = HashSet::new();
        for foundation in foundations {
            for c in foundation.terminal_constants() {
                constants.insert(c.symbol);
            }
        }
        constants
    }

    /// Generate traces from primitive to constants.
    #[must_use]
    pub fn trace(primitive: LexPrimitiva) -> Vec<GroundingTrace> {
        let atoms = BedrockAtom::for_primitive(primitive);
        let foundations = foundations_for_primitive(primitive);
        atoms
            .iter()
            .zip(foundations.iter())
            .map(|(atom, foundation)| {
                GroundingTrace::new(primitive, *atom, *foundation, atom.primary_constant())
            })
            .collect()
    }

    /// Get primary constant for a primitive.
    #[must_use]
    pub fn primary_constant(primitive: LexPrimitiva) -> MathConstant {
        primitive.primary_constant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_for_void() {
        let constants = DependencyGraph::constants_for_primitive(LexPrimitiva::Void);
        assert!(constants.contains("0"));
    }

    #[test]
    fn test_constants_for_frequency() {
        let constants = DependencyGraph::constants_for_primitive(LexPrimitiva::Frequency);
        assert!(constants.contains("π"));
    }

    #[test]
    fn test_trace_generation() {
        let traces = DependencyGraph::trace(LexPrimitiva::Quantity);
        assert_eq!(traces.len(), 5);
    }

    #[test]
    fn test_all_primitives_reach_constants() {
        for primitive in LexPrimitiva::all() {
            let constants = DependencyGraph::constants_for_primitive(primitive);
            assert!(!constants.is_empty());
        }
    }

    #[test]
    fn test_each_foundation_reaches_root() {
        use std::collections::HashSet;
        for f in MathFoundation::all() {
            let constants: HashSet<_> = f.terminal_constants().iter().map(|c| c.symbol).collect();
            let has_root = constants.contains("0") || constants.contains("1");
            assert!(has_root, "{:?} has no root constant", f);
        }
    }
}
