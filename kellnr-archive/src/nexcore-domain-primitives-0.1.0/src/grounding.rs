//! # GroundsTo implementations for nexcore-domain-primitives
//!
//! Primitive grounding for domain taxonomy types: `Tier`, `Primitive`,
//! `DomainTaxonomy`, `DecompositionNode`, `TransferScore`, `DomainTransfer`,
//! `TaxonomyComparison`, `SharedPrimitive`, `CycleError`, `Bottleneck`,
//! `TaxonomyRegistry`, `TransferMatrix`, `MatrixCell`, and `Bridge`.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::analysis::{Bottleneck, CycleError};
use crate::compare::{SharedPrimitive, TaxonomyComparison};
use crate::registry::TaxonomyRegistry;
use crate::taxonomy::{DecompositionNode, DomainTaxonomy, Primitive, Tier};
use crate::transfer::{DomainTransfer, TransferScore};
use crate::transfer_matrix::{Bridge, MatrixCell, TransferMatrix};

// ============================================================================
// Tier: Σ (Sum) + κ (Comparison)
// A 4-variant enum classifying primitive universality level.
// Dominant: Sum (exclusive choice among tier levels).
// ============================================================================

impl GroundsTo for Tier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

// ============================================================================
// Primitive: × (Product) + ς (State) + σ (Sequence)
// A struct holding name, definition, tier, dependencies, and domain_examples.
// Product of multiple fields with state-like data; dependencies form a sequence.
// ============================================================================

impl GroundsTo for Primitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
    }
}

// ============================================================================
// DomainTaxonomy: σ (Sequence) + ρ (Recursion) + ς (State) + μ (Mapping)
// A collection of primitives with dependency graph (DAG), decomposition
// (recursive), and transfer computations (mapping).
// ============================================================================

impl GroundsTo for DomainTaxonomy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Recursion,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

// ============================================================================
// DecompositionNode: ρ (Recursion) + σ (Sequence) + Σ (Sum)
// A recursive tree node: each node has children of the same type.
// Classic self-referential structure.
// ============================================================================

impl GroundsTo for DecompositionNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

// ============================================================================
// TransferScore: N (Quantity) + × (Product)
// Three f64 dimensions (structural, functional, contextual) producing a
// weighted confidence score. Pure quantitative measurement.
// ============================================================================

impl GroundsTo for TransferScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ============================================================================
// DomainTransfer: μ (Mapping) + N (Quantity) + × (Product)
// Maps a primitive from one domain to another with confidence score.
// Dominant: Mapping (cross-domain transformation).
// ============================================================================

impl GroundsTo for DomainTransfer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ============================================================================
// TaxonomyComparison: κ (Comparison) + Σ (Sum) + N (Quantity)
// Result of comparing two taxonomies: shared/unique partition and Jaccard.
// Dominant: Comparison (core operation is set comparison).
// ============================================================================

impl GroundsTo for TaxonomyComparison {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ============================================================================
// SharedPrimitive: κ (Comparison) + × (Product)
// A primitive found in both taxonomies with tier comparison.
// Dominant: Comparison (tier_match is the key insight).
// ============================================================================

impl GroundsTo for SharedPrimitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

// ============================================================================
// CycleError: ∂ (Boundary) + N (Quantity)
// Error indicating cycle detection in DAG -- a boundary violation with counts.
// Dominant: Boundary (represents a constraint violation).
// ============================================================================

impl GroundsTo for CycleError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ============================================================================
// Bottleneck: N (Quantity) + → (Causality) + κ (Comparison)
// Fan-out analysis: a primitive ranked by transitive reach percentage.
// Dominant: Quantity (fan_out count and reach_pct are the core metrics).
// ============================================================================

impl GroundsTo for Bottleneck {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ============================================================================
// TaxonomyRegistry: μ (Mapping) + ς (State) + π (Persistence)
// A HashMap-based registry that maps names to taxonomies with save/load I/O.
// Dominant: Mapping (name -> taxonomy lookup).
// ============================================================================

impl GroundsTo for TaxonomyRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ============================================================================
// TransferMatrix: N (Quantity) + μ (Mapping) + σ (Sequence) + × (Product)
// An N*N matrix of pairwise transfer cells plus bridge analysis.
// Dominant: Quantity (matrix of numeric confidence values).
// ============================================================================

impl GroundsTo for TransferMatrix {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

// ============================================================================
// MatrixCell: × (Product) + N (Quantity) + μ (Mapping)
// One directed cell (from -> to) with avg confidence, counts, strongest primitive.
// Dominant: Product (a record of multiple numeric fields).
// ============================================================================

impl GroundsTo for MatrixCell {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
    }
}

// ============================================================================
// Bridge: λ (Location) + κ (Comparison) + N (Quantity)
// A cross-domain bridge primitive appearing in 2+ taxonomies.
// Dominant: Location (positional context across domain boundaries).
// ============================================================================

impl GroundsTo for Bridge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier as LexTier;

    #[test]
    fn tier_grounding() {
        let comp = Tier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn tier_lex_tier() {
        assert_eq!(Tier::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn primitive_grounding() {
        let comp = Primitive::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn primitive_lex_tier() {
        assert_eq!(Primitive::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn domain_taxonomy_grounding() {
        let comp = DomainTaxonomy::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn domain_taxonomy_tier() {
        assert_eq!(DomainTaxonomy::tier(), LexTier::T2Composite);
    }

    #[test]
    fn decomposition_node_grounding() {
        let comp = DecompositionNode::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    #[test]
    fn decomposition_node_tier() {
        assert_eq!(DecompositionNode::tier(), LexTier::T2Primitive);
    }

    #[test]
    fn transfer_score_grounding() {
        let comp = TransferScore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn domain_transfer_grounding() {
        let comp = DomainTransfer::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn taxonomy_comparison_grounding() {
        let comp = TaxonomyComparison::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn shared_primitive_grounding() {
        let comp = SharedPrimitive::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(comp.primitives.len(), 2);
    }

    #[test]
    fn cycle_error_grounding() {
        let comp = CycleError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn bottleneck_grounding() {
        let comp = Bottleneck::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn taxonomy_registry_grounding() {
        let comp = TaxonomyRegistry::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }

    #[test]
    fn transfer_matrix_grounding() {
        let comp = TransferMatrix::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn transfer_matrix_tier() {
        assert_eq!(TransferMatrix::tier(), LexTier::T2Composite);
    }

    #[test]
    fn matrix_cell_grounding() {
        let comp = MatrixCell::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn bridge_grounding() {
        let comp = Bridge::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn all_types_not_pure() {
        // None of these domain types are pure primitives
        assert!(!Tier::is_pure_primitive());
        assert!(!Primitive::is_pure_primitive());
        assert!(!DomainTaxonomy::is_pure_primitive());
        assert!(!DecompositionNode::is_pure_primitive());
        assert!(!TransferScore::is_pure_primitive());
        assert!(!DomainTransfer::is_pure_primitive());
        assert!(!TaxonomyComparison::is_pure_primitive());
        assert!(!SharedPrimitive::is_pure_primitive());
        assert!(!CycleError::is_pure_primitive());
        assert!(!Bottleneck::is_pure_primitive());
        assert!(!TaxonomyRegistry::is_pure_primitive());
        assert!(!TransferMatrix::is_pure_primitive());
        assert!(!MatrixCell::is_pure_primitive());
        assert!(!Bridge::is_pure_primitive());
    }
}
