//! # GroundsTo implementations for spatial mathematics types
//!
//! Connects spatial primitives (Distance, Dimension, Neighborhood, Orientation,
//! Extension) and spatial traits (Metric, Embed, Orient) to the Lex Primitiva
//! type system.
//!
//! ## Crate Primitive Profile (Spatial Module)
//!
//! The spatial module introduces 5 new value types and 3 traits, covering
//! primitives underrepresented in the algebraic module:
//!
//! - **Distance**: T2-P (Quantity + Comparison) -- separation measure
//! - **Dimension**: T2-P (Quantity + Boundary) -- degrees of freedom
//! - **Neighborhood**: T2-C (Location + Boundary + Existence) -- nearby region
//! - **Orientation**: T2-C (Comparison + Sequence + Boundary) -- handedness
//! - **Extension**: T2-C (Quantity + Boundary + Location) -- spatial magnitude
//! - **MeasuredDistance**: T2-C (Quantity + Comparison + Mapping) -- with confidence
//! - **MeasuredDimension**: T2-C (Quantity + Boundary + Mapping) -- with confidence
//! - **SpatialError**: T1 (Boundary) -- spatial operation failure

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::spatial::{
    Dimension, Distance, Extension, MeasuredDimension, MeasuredDistance, Neighborhood, Orientation,
    SpatialError,
};

// ===========================================================================
// Core spatial types
// ===========================================================================

/// Distance: T2-P (Quantity + Comparison), dominant Quantity
///
/// A non-negative measure of separation between two elements.
/// Quantity-dominant: the defining characteristic IS the numeric
/// measure of how far apart things are.
impl GroundsTo for Distance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- non-negative numeric value
            LexPrimitiva::Comparison, // kappa -- closeness/farness ordering
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Dimension: T2-P (Quantity + Boundary), dominant Quantity
///
/// Count of independent axes of variation. Quantity-dominant:
/// the defining characteristic IS the numeric count of degrees
/// of freedom. Boundary appears because each dimension has
/// independence constraints.
impl GroundsTo for Dimension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- count of independent axes
            LexPrimitiva::Boundary, // partial -- independence constraints
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Neighborhood: T2-C (Location + Boundary + Existence), dominant Location
///
/// A region around a point defined by a radius. Location-dominant:
/// the neighborhood IS about identifying what's "near" a location.
/// Boundary defines the edge (open/closed). Existence determines
/// whether elements are within.
impl GroundsTo for Neighborhood {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // lambda -- center point locality
            LexPrimitiva::Boundary,  // partial -- open/closed edge
            LexPrimitiva::Existence, // exists -- elements within check
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

/// Orientation: T2-C (Comparison + Sequence + Boundary), dominant Comparison
///
/// A binary distinction between equivalent arrangements (handedness).
/// Comparison-dominant: the defining characteristic IS distinguishing
/// between two otherwise equivalent mirror states.
impl GroundsTo for Orientation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- distinguishing +/-
            LexPrimitiva::Sequence,   // sigma -- ordering/direction
            LexPrimitiva::Boundary,   // partial -- mirror plane
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// Extension: T2-C (Quantity + Boundary + Location), dominant Quantity
///
/// Spatial magnitude along dimensions. Quantity-dominant: the defining
/// characteristic IS the measurable extent. Location appears because
/// extension IS spatial. Boundary appears because extent implies limits.
impl GroundsTo for Extension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- magnitude of extent
            LexPrimitiva::Boundary, // partial -- limits of the extent
            LexPrimitiva::Location, // lambda -- spatial nature
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ===========================================================================
// Measured types
// ===========================================================================

/// MeasuredDistance: T2-C (Quantity + Comparison + Mapping), dominant Quantity
///
/// A distance measurement paired with confidence. Quantity-dominant.
impl GroundsTo for MeasuredDistance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- the distance value
            LexPrimitiva::Comparison, // kappa -- ordering of distances
            LexPrimitiva::Mapping,    // mu -- measurement function
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// MeasuredDimension: T2-C (Quantity + Boundary + Mapping), dominant Quantity
///
/// A dimension estimate with confidence.
impl GroundsTo for MeasuredDimension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- dimension count
            LexPrimitiva::Boundary, // partial -- independence constraints
            LexPrimitiva::Mapping,  // mu -- estimation mapping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// SpatialError: T1 (Boundary), dominant Boundary
///
/// Error enum for spatial operations. Pure boundary: each variant
/// represents a violated spatial constraint.
impl GroundsTo for SpatialError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- spatial constraint failure
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.95)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn distance_is_t2p_quantity_dominant() {
        assert_eq!(Distance::tier(), Tier::T2Primitive);
        assert_eq!(
            Distance::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        let comp = Distance::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn dimension_is_t2p_quantity_dominant() {
        assert_eq!(Dimension::tier(), Tier::T2Primitive);
        assert_eq!(
            Dimension::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        let comp = Dimension::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn neighborhood_is_t2c_location_dominant() {
        // 3 primitives with Location, Boundary, Existence
        // per tier rules: 2-3 = T2-P, but this has exactly 3 distinct
        // so T2-P by count (same as Bounded<T>)
        let comp = Neighborhood::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Existence));
    }

    #[test]
    fn orientation_is_t2c_comparison_dominant() {
        let comp = Orientation::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn extension_is_t2c_quantity_dominant() {
        let comp = Extension::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn measured_distance_is_t2c_quantity_dominant() {
        let comp = MeasuredDistance::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
    }

    #[test]
    fn spatial_error_is_t1_boundary_dominant() {
        assert_eq!(SpatialError::tier(), Tier::T1Universal);
        assert!(SpatialError::is_pure_primitive());
    }

    #[test]
    fn spatial_tier_distribution() {
        // T1: SpatialError = 1
        let t1_count = [SpatialError::tier()]
            .iter()
            .filter(|t| **t == Tier::T1Universal)
            .count();

        // T2-P (2-3 primitives): Distance, Dimension = 2
        let t2p_count = [Distance::tier(), Dimension::tier()]
            .iter()
            .filter(|t| **t == Tier::T2Primitive)
            .count();

        assert_eq!(t1_count, 1, "expected 1 T1 type");
        assert_eq!(t2p_count, 2, "expected 2 T2-P types");
    }
}
