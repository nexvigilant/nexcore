//! # GroundsTo implementations for stem-phys types
//!
//! Connects physics primitive types and the Physics composite to the
//! Lex Primitiva type system.
//!
//! ## Crate Primitive Profile
//!
//! stem-phys defines the PHYSICS composite with 8 traits and 7 core value types.
//! The crate spans Persistence (Quantity, Inertia, Preserve), Causality (Force,
//! YieldForce, Couple), Mapping (Acceleration, Scale, ScaleFactor), Frequency
//! (Frequency, Harmonics), Quantity (Amplitude), and Sum (Superpose).
//!
//! - **Quantity**: T2-P (Persistence + Quantity) -- conserved value
//! - **Mass**: T1 (State) -- intrinsic property
//! - **Force**: T1 (Causality) -- cause of acceleration
//! - **Acceleration**: T2-P (Mapping + Causality) -- F/m ratio
//! - **Frequency**: T1 (Frequency) -- cycles per time
//! - **Amplitude**: T1 (Quantity) -- oscillation magnitude
//! - **ScaleFactor**: T1 (Mapping) -- proportional transform
//! - **MeasuredQuantity**: T2-C (Persistence + Quantity + Quantity) -- with confidence
//! - **MeasuredForce**: T2-P (Causality + Quantity) -- with confidence
//! - **PhysicsError**: T1 (Boundary) -- operation failure

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Acceleration, Amplitude, Force, Frequency, Mass, MeasuredForce, MeasuredQuantity, PhysicsError,
    Quantity, ScaleFactor,
};

// ===========================================================================
// Core value types
// ===========================================================================

/// Quantity: T2-P (Persistence + Quantity), dominant Persistence
///
/// A conserved quantity that is invariant across transformations.
/// Persistence-dominant: the defining characteristic IS that the
/// value persists unchanged through transformations.
impl GroundsTo for Quantity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- invariant across transforms
            LexPrimitiva::Quantity,    // N -- numeric value
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

/// Mass: T1 (State), dominant State
///
/// Intrinsic property of a body -- resistance to acceleration.
/// Pure state: it IS an intrinsic, unchanging property.
impl GroundsTo for Mass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // varsigma -- intrinsic property
        ])
        .with_dominant(LexPrimitiva::State, 0.95)
    }
}

/// Force: T1 (Causality), dominant Causality
///
/// The cause of acceleration. Pure causality: applying a force
/// causes proportional acceleration.
impl GroundsTo for Force {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- cause of acceleration
        ])
        .with_dominant(LexPrimitiva::Causality, 0.95)
    }
}

/// Acceleration: T2-P (Mapping + Causality), dominant Mapping
///
/// The rate of velocity change, computed as F/m.
/// Mapping-dominant: it IS a transformation from force and mass
/// to change rate.
impl GroundsTo for Acceleration {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- force/mass ratio transformation
            LexPrimitiva::Causality, // arrow -- caused by force
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Frequency: T1 (Frequency), dominant Frequency
///
/// Cycles per unit time. Pure frequency: it IS a repetition rate.
impl GroundsTo for Frequency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu -- cycles per time unit
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.95)
    }
}

/// Amplitude: T1 (Quantity), dominant Quantity
///
/// Oscillation magnitude -- displacement from center.
/// Pure quantity: it IS a numeric magnitude value.
impl GroundsTo for Amplitude {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- displacement magnitude
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// ScaleFactor: T1 (Mapping), dominant Mapping
///
/// Proportional transformation factor.
/// Pure mapping: it IS a multiplicative transformation.
impl GroundsTo for ScaleFactor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- proportional transformation
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.95)
    }
}

// ===========================================================================
// Measured types
// ===========================================================================

/// MeasuredQuantity: T2-C (Persistence + Quantity + Mapping),
/// dominant Persistence
///
/// A conserved quantity paired with confidence.
/// Persistence-dominant: the measured value's conservation is the
/// primary semantic content.
impl GroundsTo for MeasuredQuantity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- quantity conservation
            LexPrimitiva::Quantity,    // N -- quantity value + confidence
            LexPrimitiva::Mapping,     // mu -- measurement transformation
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// MeasuredForce: T2-P (Causality + Quantity), dominant Causality
///
/// A force measurement paired with confidence.
/// Causality-dominant: the force's causal nature is the primary
/// semantic content; confidence is secondary.
impl GroundsTo for MeasuredForce {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- force as cause
            LexPrimitiva::Quantity,  // N -- force value + confidence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ===========================================================================
// Error type
// ===========================================================================

/// PhysicsError: T1 (Boundary), dominant Boundary
///
/// Error enum for physics operations. Pure boundary: each variant
/// represents a failed physical constraint (conservation violated,
/// invalid mass, undefined scale).
impl GroundsTo for PhysicsError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- physical constraint failure
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
    fn quantity_is_t2p_persistence_dominant() {
        assert_eq!(Quantity::tier(), Tier::T2Primitive);
        assert_eq!(
            Quantity::primitive_composition().dominant,
            Some(LexPrimitiva::Persistence)
        );
        let comp = Quantity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn mass_is_t1_state_dominant() {
        assert_eq!(Mass::tier(), Tier::T1Universal);
        assert_eq!(
            Mass::primitive_composition().dominant,
            Some(LexPrimitiva::State)
        );
        assert!(Mass::is_pure_primitive());
    }

    #[test]
    fn force_is_t1_causality_dominant() {
        assert_eq!(Force::tier(), Tier::T1Universal);
        assert_eq!(
            Force::primitive_composition().dominant,
            Some(LexPrimitiva::Causality)
        );
        assert!(Force::is_pure_primitive());
    }

    #[test]
    fn acceleration_is_t2p_mapping_dominant() {
        assert_eq!(Acceleration::tier(), Tier::T2Primitive);
        assert_eq!(
            Acceleration::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        let comp = Acceleration::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn frequency_is_t1_frequency_dominant() {
        assert_eq!(Frequency::tier(), Tier::T1Universal);
        assert_eq!(
            Frequency::primitive_composition().dominant,
            Some(LexPrimitiva::Frequency)
        );
        assert!(Frequency::is_pure_primitive());
    }

    #[test]
    fn amplitude_is_t1_quantity_dominant() {
        assert_eq!(Amplitude::tier(), Tier::T1Universal);
        assert_eq!(
            Amplitude::primitive_composition().dominant,
            Some(LexPrimitiva::Quantity)
        );
        assert!(Amplitude::is_pure_primitive());
    }

    #[test]
    fn scale_factor_is_t1_mapping_dominant() {
        assert_eq!(ScaleFactor::tier(), Tier::T1Universal);
        assert_eq!(
            ScaleFactor::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
        assert!(ScaleFactor::is_pure_primitive());
    }

    #[test]
    fn measured_quantity_is_t2p_persistence_dominant() {
        // 3 primitives = T2-P
        assert_eq!(MeasuredQuantity::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredQuantity::primitive_composition().dominant,
            Some(LexPrimitiva::Persistence)
        );
        let comp = MeasuredQuantity::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
    }

    #[test]
    fn measured_force_is_t2p_causality_dominant() {
        assert_eq!(MeasuredForce::tier(), Tier::T2Primitive);
        assert_eq!(
            MeasuredForce::primitive_composition().dominant,
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn physics_error_is_t1_boundary_dominant() {
        assert_eq!(PhysicsError::tier(), Tier::T1Universal);
        assert_eq!(
            PhysicsError::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert!(PhysicsError::is_pure_primitive());
    }

    #[test]
    fn tier_distribution_is_reasonable() {
        // T1: Mass, Force, Frequency, Amplitude, ScaleFactor, PhysicsError = 6
        let t1_count = [
            Mass::tier(),
            Force::tier(),
            Frequency::tier(),
            Amplitude::tier(),
            ScaleFactor::tier(),
            PhysicsError::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T1Universal)
        .count();

        // T2-P (2-3 primitives): Quantity, Acceleration, MeasuredForce,
        //   MeasuredQuantity = 4
        let t2p_count = [
            Quantity::tier(),
            Acceleration::tier(),
            MeasuredForce::tier(),
            MeasuredQuantity::tier(),
        ]
        .iter()
        .filter(|t| **t == Tier::T2Primitive)
        .count();

        assert_eq!(t1_count, 6, "expected 6 T1 types");
        assert_eq!(t2p_count, 4, "expected 4 T2-P types");
    }
}
