//! # GroundsTo implementations for nexcore-aggregate types
//!
//! Connects fold, tree, and ranked combinators to the Lex Primitiva type system.
//!
//! ## Σ (Sum) Grounding
//!
//! This crate is the canonical home of the Σ primitive in the workspace.
//! Every fold operation reduces to Σ: accumulation over a sequence.
//! The three modules ground the three weakest T1 primitives:
//! - `fold` → Σ (Sum)
//! - `tree` → ρ (Recursion)
//! - `ranked` → κ (Comparison)

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::error::AggregateError;
use crate::fold::{
    CountFold, FoldResults, MaxFold, MeanAccumulator, MeanFold, MinFold, ProductFold, SumFold,
    VarianceAccumulator, VarianceFold,
};
use crate::ranked::{OutlierDirection, Ranked};
use crate::tree::{SimpleNode, TraversalConfig};

// ---------------------------------------------------------------------------
// Fold types — Σ dominant
// ---------------------------------------------------------------------------

/// SumFold: T1 (Σ), dominant Σ
///
/// Pure summation — the irreducible Σ primitive in Rust.
/// Zero-dep accumulation: init(0) → step(acc + x) → result.
impl GroundsTo for SumFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ — the fold IS summation
        ])
        .with_dominant(LexPrimitiva::Sum, 0.95)
    }
}

/// ProductFold: T2-P (Σ · N), dominant Σ
///
/// Multiplicative fold — Σ in the product semiring.
/// Sum-dominant: still a fold, just with multiplication as the monoid.
impl GroundsTo for ProductFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — fold pattern (init · step · result)
            LexPrimitiva::Quantity, // N — numeric multiplication
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// CountFold: T2-P (Σ · N), dominant Σ
///
/// Counting fold — Σ(1) for each element.
/// Sum-dominant: structurally a fold that accumulates quantity.
impl GroundsTo for CountFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — accumulative counting
            LexPrimitiva::Quantity, // N — count value
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// MinFold: T2-P (Σ · κ), dominant κ
///
/// Minimum-finding fold — comparison selects the survivor at each step.
/// Comparison-dominant: the essence is the `<` test, fold is the vehicle.
impl GroundsTo for MinFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — fold structure
            LexPrimitiva::Comparison, // κ — less-than selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// MaxFold: T2-P (Σ · κ), dominant κ
///
/// Maximum-finding fold — comparison selects the survivor at each step.
/// Comparison-dominant: the essence is the `>` test, fold is the vehicle.
impl GroundsTo for MaxFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — fold structure
            LexPrimitiva::Comparison, // κ — greater-than selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// MeanFold: T2-P (Σ · N · ∝), dominant Σ
///
/// Arithmetic mean: Σ(x) / N. Ratio of sum to count.
/// Sum-dominant: mean is fundamentally sum divided by count.
impl GroundsTo for MeanFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,             // Σ — accumulation
            LexPrimitiva::Quantity,        // N — count for division
            LexPrimitiva::Irreversibility, // ∝ — normalization (proportional)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// VarianceFold: T2-C (Σ · N · ∝ · κ), dominant Σ
///
/// Sample variance via Welford's algorithm: Σ(x-μ)²/(N-1).
/// Sum-dominant: still a fold, but composing multiple running statistics.
impl GroundsTo for VarianceFold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,             // Σ — running accumulation
            LexPrimitiva::Quantity,        // N — count, delta values
            LexPrimitiva::Irreversibility, // ∝ — normalization
            LexPrimitiva::Comparison,      // κ — deviation from mean
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// FoldResults: T2-C (Σ · σ · ς · N), dominant Σ
///
/// Aggregate results from fold_all: sum, count, min, max, mean, variance.
/// Sum-dominant: the container exists to hold fold outputs.
impl GroundsTo for FoldResults {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — contains fold results
            LexPrimitiva::Sequence, // σ — ordered field set
            LexPrimitiva::State,    // ς — snapshot of computed state
            LexPrimitiva::Quantity, // N — numeric values throughout
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ---------------------------------------------------------------------------
// Accumulator types — ς dominant
// ---------------------------------------------------------------------------

/// MeanAccumulator: T2-P (ς · Σ · N), dominant ς
///
/// Running state of a mean computation: sum and count.
/// State-dominant: the type IS mutable running state (ς-acc).
impl GroundsTo for MeanAccumulator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς — running accumulator state
            LexPrimitiva::Sum,      // Σ — sum field
            LexPrimitiva::Quantity, // N — count field
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// VarianceAccumulator: T2-C (ς · Σ · N · κ), dominant ς
///
/// Running state of Welford's variance computation: count, mean, M2.
/// State-dominant: the type IS mutable running state (ς-acc).
impl GroundsTo for VarianceAccumulator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — running Welford state
            LexPrimitiva::Sum,        // Σ — M2 accumulation
            LexPrimitiva::Quantity,   // N — count, mean values
            LexPrimitiva::Comparison, // κ — deviation from mean
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ---------------------------------------------------------------------------
// Ranked types — κ dominant
// ---------------------------------------------------------------------------

/// Ranked: T2-P (κ · N · λ), dominant κ
///
/// A named value with an assigned rank position.
/// Comparison-dominant: the entire purpose is ordinal comparison.
impl GroundsTo for Ranked {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — Ord impl, rank assignment
            LexPrimitiva::Quantity,   // N — numeric value and rank position
            LexPrimitiva::Location,   // λ — named identity
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// OutlierDirection: T2-P (κ · ∂), dominant ∂
///
/// Direction of an outlier: above or below the fence.
/// Boundary-dominant: outlier detection IS boundary violation detection.
impl GroundsTo for OutlierDirection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — above/below comparison
            LexPrimitiva::Boundary,   // ∂ — fence boundary violation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tree types — ρ dominant
// ---------------------------------------------------------------------------

/// SimpleNode: T2-P (ρ · N · σ), dominant ρ
///
/// Concrete recursive tree node with id, value, and children.
/// Recursion-dominant: the type IS recursive (children: Vec<Self>).
impl GroundsTo for SimpleNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ — self-referential children
            LexPrimitiva::Quantity,  // N — numeric value
            LexPrimitiva::Sequence,  // σ — ordered children
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.90)
    }
}

/// TraversalConfig: T2-P (∂ · ρ · N), dominant ∂
///
/// Configuration bounding recursive traversal: max depth, cycle detection.
/// Boundary-dominant: the config exists to set traversal limits.
impl GroundsTo for TraversalConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — max_depth limit, cycle boundary
            LexPrimitiva::Recursion, // ρ — constrains recursion
            LexPrimitiva::Quantity,  // N — max_depth numeric value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types — ∂ dominant
// ---------------------------------------------------------------------------

/// AggregateError: T2-P (∂ · ∅), dominant ∂
///
/// Error variants representing boundary violations during aggregation.
/// Boundary-dominant: errors ARE boundary conditions (empty, cycle, depth).
impl GroundsTo for AggregateError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — violated constraints
            LexPrimitiva::Void,     // ∅ — absence condition (empty input)
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn sum_fold_is_t1() {
        // 1 primitive = T1
        assert_eq!(SumFold::tier(), Tier::T1Universal);
        assert_eq!(
            SumFold::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn product_fold_is_t2p() {
        assert_eq!(ProductFold::tier(), Tier::T2Primitive);
    }

    #[test]
    fn count_fold_is_t2p() {
        assert_eq!(CountFold::tier(), Tier::T2Primitive);
    }

    #[test]
    fn min_fold_is_comparison_dominant() {
        let comp = MinFold::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn max_fold_is_comparison_dominant() {
        let comp = MaxFold::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn mean_fold_is_t2p() {
        assert_eq!(MeanFold::tier(), Tier::T2Primitive);
        assert_eq!(
            MeanFold::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn variance_fold_is_t2c() {
        // 4 primitives = T2-C
        assert_eq!(VarianceFold::tier(), Tier::T2Composite);
    }

    #[test]
    fn fold_results_is_t2c() {
        assert_eq!(FoldResults::tier(), Tier::T2Composite);
        assert_eq!(
            FoldResults::primitive_composition().dominant,
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn ranked_is_comparison_dominant() {
        let comp = Ranked::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn outlier_direction_is_boundary_dominant() {
        let comp = OutlierDirection::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn simple_node_is_recursion_dominant() {
        let comp = SimpleNode::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert_eq!(SimpleNode::tier(), Tier::T2Primitive);
    }

    #[test]
    fn traversal_config_is_boundary_dominant() {
        let comp = TraversalConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn aggregate_error_is_boundary_dominant() {
        let comp = AggregateError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Void));
    }

    #[test]
    fn all_primitives_from_three_modules() {
        // The three weakest primitives are represented
        let sum_comp = SumFold::primitive_composition();
        let node_comp = SimpleNode::primitive_composition();
        let ranked_comp = Ranked::primitive_composition();

        assert!(sum_comp.primitives.contains(&LexPrimitiva::Sum));
        assert!(node_comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(ranked_comp.primitives.contains(&LexPrimitiva::Comparison));
    }
}
