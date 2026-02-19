//! # GroundsTo implementations for nexcore-edit-distance types
//!
//! Connects edit distance operations, cost models, solvers, adapters, and transfer
//! types to the Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! This crate is κ (Comparison) dominant at its core: edit distance IS a comparison
//! metric between two sequences. The secondary primitives vary by layer:
//!
//! - **ops** (operation sets): σ Sequence + × Product (discrete operation enumeration)
//! - **cost** (cost models): μ Mapping (element -> cost transformation)
//! - **solver** (algorithms): σ Sequence + ρ Recursion (DP recurrence)
//! - **adapter** (domain adapters): μ Mapping (domain -> element transformation)
//! - **transfer** (cross-domain): μ Mapping + N Quantity (confidence scoring)

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::adapter::{ByteAdapter, DnaAdapter, StringAdapter, TokenAdapter};
use crate::cost::{UniformCost, WeightedCost};
use crate::metric::Missing;
use crate::ops::{DamerauOps, IndelOps, StdOps};
use crate::solver::{BandedDp, FullMatrixDp, TwoRowDp};
use crate::transfer::{TransferMap, TransferMapBuilder, TransferRegistry};

// ===========================================================================
// Operation Sets -- sigma + Product dominant
// ===========================================================================

/// StdOps: T2-P (sigma + Product), dominant Product
///
/// Standard Levenshtein operation set: insert, delete, substitute.
/// Zero-sized type enumerating which operations are permitted.
/// Product-dominant: this is a product type of boolean flags (allows_insert x
/// allows_delete x allows_substitute) -- a fixed Cartesian configuration.
impl GroundsTo for StdOps {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,  // x -- Cartesian product of boolean operation flags
            LexPrimitiva::Sequence, // sigma -- ordered operation enumeration
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

/// DamerauOps: T2-P (Product + sigma), dominant Product
///
/// Damerau-Levenshtein operation set: insert, delete, substitute, transpose.
/// Extends StdOps with transposition -- same structural pattern, one more flag.
/// Product-dominant: same Cartesian boolean flag configuration as StdOps.
impl GroundsTo for DamerauOps {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,  // x -- four boolean flags
            LexPrimitiva::Sequence, // sigma -- ordered operation enumeration
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

/// IndelOps: T2-P (Product + sigma), dominant Product
///
/// LCS operation set: insert and delete only, no substitution.
/// Subset of StdOps -- same structural pattern with fewer enabled flags.
/// Product-dominant: same Cartesian boolean flag configuration.
impl GroundsTo for IndelOps {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,  // x -- two boolean flags (insert, delete)
            LexPrimitiva::Sequence, // sigma -- ordered operation enumeration
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

// ===========================================================================
// Cost Models -- mu Mapping dominant
// ===========================================================================

/// UniformCost: T1 (mu), dominant mu
///
/// All edit operations cost 1.0 -- the simplest possible mapping from
/// operation to cost. This is a degenerate (constant) Mapping.
/// Mapping-dominant: it IS a function from operation -> f64.
impl GroundsTo for UniformCost {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- operation -> cost function
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// WeightedCost: T2-P (mu + N), dominant mu
///
/// Per-operation-type weight model: each operation type has a configurable cost.
/// Mapping-dominant: it maps operation types to numeric weights.
/// The Quantity primitive appears because the weights are numeric f64 values.
impl GroundsTo for WeightedCost {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- operation-type -> cost function
            LexPrimitiva::Quantity, // N -- numeric weight values
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ===========================================================================
// Solvers -- sigma Sequence + rho Recursion + kappa Comparison dominant
// ===========================================================================

/// FullMatrixDp: T2-C (sigma + rho + kappa + N), dominant sigma
///
/// Classic Wagner-Fischer algorithm with full (m+1)x(n+1) matrix.
/// O(mn) time and space. Supports traceback for operation reconstruction.
/// Sequence-dominant: the algorithm iterates in strict row-column order,
/// filling each cell from its predecessors -- a double-nested sequence.
impl GroundsTo for FullMatrixDp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- row-major matrix fill order
            LexPrimitiva::Recursion, // rho -- DP recurrence relation (cell depends on predecessors)
            LexPrimitiva::Comparison, // kappa -- min() selection at each cell
            LexPrimitiva::Quantity,  // N -- distance values in matrix cells
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// TwoRowDp: T2-C (sigma + rho + kappa + N), dominant sigma
///
/// Two-row Wagner-Fischer: O(mn) time, O(min(m,n)) space.
/// Does not support traceback. Same recurrence as FullMatrixDp but
/// only retains prev_row and curr_row.
/// Sequence-dominant: sequential row iteration with row swapping.
impl GroundsTo for TwoRowDp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- sequential row iteration
            LexPrimitiva::Recursion,  // rho -- DP recurrence (curr from prev)
            LexPrimitiva::Comparison, // kappa -- min() selection
            LexPrimitiva::Quantity,   // N -- distance values
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// BandedDp: T2-C (sigma + rho + kappa + N + Boundary), dominant Boundary
///
/// Banded Wagner-Fischer for bounded edit distance.
/// Only computes cells within band_width of the diagonal.
/// Returns infinity if true distance exceeds the band.
/// Boundary-dominant: the defining characteristic is the band constraint
/// that limits computation and enables early termination.
impl GroundsTo for BandedDp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- band width limit, early termination
            LexPrimitiva::Sequence,   // sigma -- sequential row iteration within band
            LexPrimitiva::Recursion,  // rho -- DP recurrence
            LexPrimitiva::Comparison, // kappa -- min() selection
            LexPrimitiva::Quantity,   // N -- distance values, max_distance parameter
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ===========================================================================
// Metric types -- kappa Comparison dominant
// ===========================================================================

/// Missing: T1 (Void), dominant Void
///
/// Typestate marker indicating a builder field has not been set.
/// This is the absence of a value -- pure Void.
impl GroundsTo for Missing {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void, // empty -- absence of a value
        ])
        .with_dominant(LexPrimitiva::Void, 0.95)
    }
}

// ===========================================================================
// Domain Adapters -- mu Mapping dominant
// ===========================================================================

/// StringAdapter: T1 (mu), dominant mu
///
/// Adapts Unicode strings to Vec<char>. The simplest possible adapter:
/// identity mapping on character sequences.
/// Mapping-dominant: it IS a transformation from &str -> Vec<char>.
impl GroundsTo for StringAdapter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- str -> Vec<char> transformation
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// ByteAdapter: T1 (mu), dominant mu
///
/// Adapts byte slices for binary or ASCII-only comparison.
/// Identity mapping on byte sequences.
/// Mapping-dominant: it IS a transformation from &str -> Vec<u8>.
impl GroundsTo for ByteAdapter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- str -> Vec<u8> transformation
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

/// DnaAdapter: T2-P (mu + Boundary), dominant mu
///
/// Adapts DNA sequences with validation and normalization to {A,C,G,T,N}.
/// Mapping-dominant: transforms input bytes to a normalized nucleotide alphabet.
/// The Boundary primitive appears because unknown characters are mapped to N
/// (boundary of valid alphabet).
impl GroundsTo for DnaAdapter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- byte -> nucleotide normalization
            LexPrimitiva::Boundary, // partial -- alphabet boundary validation (unknown -> N)
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// TokenAdapter: T2-P (mu + sigma), dominant mu
///
/// Adapts text to whitespace-split word tokens for word error rate.
/// Mapping-dominant: transforms a string into a sequence of word tokens.
/// The Sequence primitive appears because token ordering matters.
impl GroundsTo for TokenAdapter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- str -> Vec<String> transformation
            LexPrimitiva::Sequence, // sigma -- ordered token sequence
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ===========================================================================
// Transfer types -- mu Mapping + N Quantity dominant
// ===========================================================================

/// TransferMap: T2-C (mu + N + kappa + Boundary), dominant mu
///
/// Three-dimensional confidence assessment for cross-domain transfer.
/// Maps source/target domain pairs to structural, functional, and contextual
/// similarity scores.
/// Mapping-dominant: it IS a mapping from domain pair -> confidence vector.
impl GroundsTo for TransferMap {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- domain pair -> confidence scores
            LexPrimitiva::Quantity,   // N -- structural/functional/contextual f64 scores
            LexPrimitiva::Comparison, // kappa -- limiting_factor comparison
            LexPrimitiva::Boundary,   // partial -- confidence bounds [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// TransferMapBuilder: T2-C (State + mu + N + Boundary), dominant State
///
/// Builder for TransferMap with progressive field accumulation.
/// State-dominant: the builder IS encapsulated mutable state that
/// accumulates configuration until build() freezes it.
impl GroundsTo for TransferMapBuilder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- mutable builder state
            LexPrimitiva::Mapping,  // mu -- will produce a TransferMap (domain -> confidence)
            LexPrimitiva::Quantity, // N -- accumulates numeric score values
            LexPrimitiva::Boundary, // partial -- clamp(0.0, 1.0) on score inputs
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// TransferRegistry: T2-C (sigma + mu + Existence + kappa), dominant sigma
///
/// Registry of known cross-domain transfer mappings with lookup.
/// Sequence-dominant: it IS an ordered collection (Vec<TransferMap>).
/// Existence appears because lookup returns matches or empty -- an existence check.
impl GroundsTo for TransferRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered Vec<TransferMap>
            LexPrimitiva::Mapping,    // mu -- lookup: (source, target) -> Vec<&TransferMap>
            LexPrimitiva::Existence,  // exists -- lookup returns present/absent
            LexPrimitiva::Comparison, // kappa -- bidirectional matching in filter
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // -----------------------------------------------------------------------
    // Operation Sets
    // -----------------------------------------------------------------------

    #[test]
    fn std_ops_is_t2p_product_dominant() {
        assert_eq!(StdOps::tier(), Tier::T2Primitive);
        assert_eq!(
            StdOps::primitive_composition().dominant,
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn damerau_ops_is_t2p_product_dominant() {
        assert_eq!(DamerauOps::tier(), Tier::T2Primitive);
        assert_eq!(
            DamerauOps::primitive_composition().dominant,
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn indel_ops_is_t2p_product_dominant() {
        assert_eq!(IndelOps::tier(), Tier::T2Primitive);
        assert_eq!(
            IndelOps::primitive_composition().dominant,
            Some(LexPrimitiva::Product)
        );
    }

    // -----------------------------------------------------------------------
    // Cost Models
    // -----------------------------------------------------------------------

    #[test]
    fn uniform_cost_is_t1_mapping_dominant() {
        assert_eq!(UniformCost::tier(), Tier::T1Universal);
        assert_eq!(
            UniformCost::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        assert!(UniformCost::is_pure_primitive());
    }

    #[test]
    fn weighted_cost_is_t2p_mapping_dominant() {
        assert_eq!(WeightedCost::tier(), Tier::T2Primitive);
        assert_eq!(
            WeightedCost::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    // -----------------------------------------------------------------------
    // Solvers
    // -----------------------------------------------------------------------

    #[test]
    fn full_matrix_dp_is_t2c_sequence_dominant() {
        assert_eq!(FullMatrixDp::tier(), Tier::T2Composite);
        assert_eq!(
            FullMatrixDp::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
        let comp = FullMatrixDp::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn two_row_dp_is_t2c_sequence_dominant() {
        assert_eq!(TwoRowDp::tier(), Tier::T2Composite);
        assert_eq!(
            TwoRowDp::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn banded_dp_is_t2c_boundary_dominant() {
        assert_eq!(BandedDp::tier(), Tier::T2Composite);
        assert_eq!(
            BandedDp::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        let comp = BandedDp::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
    }

    // -----------------------------------------------------------------------
    // Metric Markers
    // -----------------------------------------------------------------------

    #[test]
    fn missing_is_t1_void_dominant() {
        assert_eq!(Missing::tier(), Tier::T1Universal);
        assert_eq!(
            Missing::primitive_composition().dominant,
            Some(LexPrimitiva::Void)
        );
        assert!(Missing::is_pure_primitive());
    }

    // -----------------------------------------------------------------------
    // Domain Adapters
    // -----------------------------------------------------------------------

    #[test]
    fn string_adapter_is_t1_mapping_dominant() {
        assert_eq!(StringAdapter::tier(), Tier::T1Universal);
        assert_eq!(
            StringAdapter::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        assert!(StringAdapter::is_pure_primitive());
    }

    #[test]
    fn byte_adapter_is_t1_mapping_dominant() {
        assert_eq!(ByteAdapter::tier(), Tier::T1Universal);
        assert_eq!(
            ByteAdapter::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn dna_adapter_is_t2p_mapping_dominant() {
        assert_eq!(DnaAdapter::tier(), Tier::T2Primitive);
        assert_eq!(
            DnaAdapter::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = DnaAdapter::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn token_adapter_is_t2p_mapping_dominant() {
        assert_eq!(TokenAdapter::tier(), Tier::T2Primitive);
        assert_eq!(
            TokenAdapter::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = TokenAdapter::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
    }

    // -----------------------------------------------------------------------
    // Transfer Types
    // -----------------------------------------------------------------------

    #[test]
    fn transfer_map_is_t2c_mapping_dominant() {
        assert_eq!(TransferMap::tier(), Tier::T2Composite);
        assert_eq!(
            TransferMap::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = TransferMap::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn transfer_map_builder_is_t2c_state_dominant() {
        assert_eq!(TransferMapBuilder::tier(), Tier::T2Composite);
        assert_eq!(
            TransferMapBuilder::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn transfer_registry_is_t2c_sequence_dominant() {
        assert_eq!(TransferRegistry::tier(), Tier::T2Composite);
        assert_eq!(
            TransferRegistry::primitive_composition().dominant,
            Some(LexPrimitiva::Sequence)
        );
        let comp = TransferRegistry::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    // -----------------------------------------------------------------------
    // Cross-cutting assertions
    // -----------------------------------------------------------------------

    #[test]
    fn all_operation_sets_share_product_dominant() {
        let ops_types = [
            StdOps::dominant_primitive(),
            DamerauOps::dominant_primitive(),
            IndelOps::dominant_primitive(),
        ];
        for dom in ops_types {
            assert_eq!(dom, Some(LexPrimitiva::Product));
        }
    }

    #[test]
    fn all_adapters_share_mapping_dominant() {
        let adapter_types = [
            StringAdapter::dominant_primitive(),
            ByteAdapter::dominant_primitive(),
            DnaAdapter::dominant_primitive(),
            TokenAdapter::dominant_primitive(),
        ];
        for dom in adapter_types {
            assert_eq!(dom, Some(LexPrimitiva::Mapping));
        }
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: UniformCost, Missing, StringAdapter, ByteAdapter = 4
        // T2-P: StdOps, DamerauOps, IndelOps, WeightedCost, DnaAdapter, TokenAdapter = 6
        // T2-C: FullMatrixDp, TwoRowDp, BandedDp, TransferMap, TransferMapBuilder, TransferRegistry = 6
        let t1_count = [
            UniformCost::tier(),
            Missing::tier(),
            StringAdapter::tier(),
            ByteAdapter::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T1Universal)
        .count();

        let t2p_count = [
            StdOps::tier(),
            DamerauOps::tier(),
            IndelOps::tier(),
            WeightedCost::tier(),
            DnaAdapter::tier(),
            TokenAdapter::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        let t2c_count = [
            FullMatrixDp::tier(),
            TwoRowDp::tier(),
            BandedDp::tier(),
            TransferMap::tier(),
            TransferMapBuilder::tier(),
            TransferRegistry::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Composite)
        .count();

        assert_eq!(t1_count, 4, "expected 4 T1 types");
        assert_eq!(t2p_count, 6, "expected 6 T2-P types");
        assert_eq!(t2c_count, 6, "expected 6 T2-C types");
    }
}
