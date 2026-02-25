//! # Lex Primitiva — Unified Types + Local GroundsTo
//!
//! Type constructs (`LexPrimitiva`, `PrimitiveComposition`, `GroundingTier`) are
//! re-exported from the standalone `nexcore-lex-primitiva` crate, ensuring type
//! identity across the workspace.
//!
//! `GroundsTo` is defined **locally** because `grounding.rs` implements it for
//! types from external crates (nexcore-primitives, nexcore-hormones, etc.), and
//! the orphan rule requires the trait to be local. After Phase 10 distributes
//! these impls to their home crates, `GroundsTo` will also unify with the
//! standalone's definition.

// ============================================================================
// Unified Types (from standalone nexcore-lex-primitiva)
// ============================================================================

pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::state_mode::StateMode;
pub use nexcore_lex_primitiva::tier::Tier;

/// Backward-compatible type alias for `Tier`.
///
/// All existing code using `GroundingTier::classify()`, `GroundingTier::T2Composite`, etc.
/// continues to work unchanged.
pub use nexcore_lex_primitiva::GroundingTier;

// ============================================================================
// Local GroundsTo Trait
// ============================================================================

/// Trait for types that ground to T1 primitives.
///
/// Implementing this trait declares how a type composes from the 15 Lex Primitiva.
/// This enables compile-time verification of the grounding hierarchy.
///
/// **Note:** This is defined locally in vigilance (not re-exported from the
/// standalone crate) to satisfy the orphan rule for `grounding.rs` impls.
/// Types from extracted crates (pvos, etc.) use the standalone's `GroundsTo`.
///
/// # Example
///
/// ```ignore
/// impl GroundsTo for ThresholdGate {
///     fn primitive_composition() -> PrimitiveComposition {
///         PrimitiveComposition::new(vec![
///             LexPrimitiva::Comparison,  // threshold check
///             LexPrimitiva::Boundary,    // gate boundary
///             LexPrimitiva::Quantity,    // numeric threshold
///         ])
///         .with_dominant(LexPrimitiva::Comparison, 0.95)
///     }
/// }
/// ```
pub trait GroundsTo {
    /// Returns the primitive composition that grounds this type.
    fn primitive_composition() -> PrimitiveComposition;

    /// Returns the dominant primitive (convenience method).
    fn dominant_primitive() -> Option<LexPrimitiva> {
        Self::primitive_composition().dominant
    }

    /// Returns true if this type is purely one primitive.
    fn is_pure_primitive() -> bool {
        Self::primitive_composition().is_pure()
    }

    /// Returns the disambiguated State (ς) mode, if applicable.
    ///
    /// Default returns `None`. Override for types that involve state
    /// to specify whether the state is mutable, modal, or accumulated.
    fn state_mode() -> Option<StateMode> {
        None
    }
}

// ============================================================================
// Standard Library Type Groundings
// ============================================================================

impl GroundsTo for bool {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

impl GroundsTo for u8 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u16 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for u64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for usize {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for i64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for f32 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for f64 {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Quantity, 1.0)
    }
}

impl GroundsTo for () {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void]).with_dominant(LexPrimitiva::Void, 1.0)
    }
}

impl GroundsTo for String {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sequence, 0.9)
    }
}

impl<T> GroundsTo for Option<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Void, 0.95)
    }
}

impl<T, E> GroundsTo for Result<T, E> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Boundary, 0.9)
    }
}

impl<T> GroundsTo for Vec<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

impl<T> GroundsTo for Box<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Location, 0.9)
    }
}

// ============================================================================
// Domain-Specific Grounding Implementations
// ============================================================================

/// `GroundsTo` implementations for domain types from external crates.
///
/// Contains 85+ implementations connecting domain types (chemistry, quantum,
/// transfer, PV, guardian, etc.) to T1 primitives.
pub mod grounding;

// ============================================================================
// Bridge to domain_discovery PrimitiveTier
// ============================================================================

/// Bridge between `GroundingTier` (alias for `Tier`) and `PrimitiveTier`.
pub mod bridge {
    use super::*;
    use crate::domain_discovery::primitives::PrimitiveTier;

    /// Converts a `GroundingTier` to `PrimitiveTier`.
    #[must_use]
    pub fn to_primitive_tier(tier: GroundingTier) -> PrimitiveTier {
        match tier {
            GroundingTier::T1Universal => PrimitiveTier::T1Universal,
            GroundingTier::T2Primitive | GroundingTier::T2Composite => PrimitiveTier::T2CrossDomain,
            GroundingTier::T3DomainSpecific => PrimitiveTier::T3DomainSpecific,
            _ => PrimitiveTier::T3DomainSpecific,
        }
    }

    /// Infers `GroundingTier` from `PrimitiveTier` (lossy - T2 variants collapse).
    #[must_use]
    pub fn from_primitive_tier(tier: PrimitiveTier) -> GroundingTier {
        match tier {
            PrimitiveTier::T1Universal => GroundingTier::T1Universal,
            PrimitiveTier::T2CrossDomain => GroundingTier::T2Composite,
            PrimitiveTier::T3DomainSpecific => GroundingTier::T3DomainSpecific,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_16_primitives() {
        let all = LexPrimitiva::all();
        assert_eq!(all.len(), 16);

        use std::collections::HashSet;
        let symbols: HashSet<_> = all.iter().map(|p| p.symbol()).collect();
        assert_eq!(symbols.len(), 16);
    }

    #[test]
    fn test_primitive_symbols() {
        assert_eq!(LexPrimitiva::Sequence.symbol(), "σ");
        assert_eq!(LexPrimitiva::Mapping.symbol(), "μ");
        assert_eq!(LexPrimitiva::Void.symbol(), "∅");
        assert_eq!(LexPrimitiva::Sum.symbol(), "Σ");
    }

    #[test]
    fn test_root_primitives() {
        assert!(LexPrimitiva::Quantity.is_root());
        assert!(LexPrimitiva::Causality.is_root());
        assert!(!LexPrimitiva::Sequence.is_root());
    }

    #[test]
    fn test_composition_pure() {
        let pure = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert!(pure.is_pure());

        let composite =
            PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        assert!(!composite.is_pure());
    }

    #[test]
    fn test_grounding_tier_classification() {
        let t1 = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert_eq!(GroundingTier::classify(&t1), GroundingTier::T1Universal);

        let t2p = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        assert_eq!(GroundingTier::classify(&t2p), GroundingTier::T2Primitive);

        let t2c = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
        ]);
        assert_eq!(GroundingTier::classify(&t2c), GroundingTier::T2Composite);

        let t3 = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ]);
        assert_eq!(
            GroundingTier::classify(&t3),
            GroundingTier::T3DomainSpecific
        );
    }

    #[test]
    fn test_stdlib_groundings() {
        assert_eq!(u32::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert!(u32::is_pure_primitive());

        assert_eq!(
            <Option<i32>>::dominant_primitive(),
            Some(LexPrimitiva::Void)
        );

        assert_eq!(
            <Result<(), ()>>::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );

        assert_eq!(<()>::dominant_primitive(), Some(LexPrimitiva::Void));
    }

    #[test]
    fn test_display_formatting() {
        let comp = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ]);
        let display = format!("{comp}");
        assert_eq!(display, "[σ + μ + ς]");
    }

    #[test]
    fn test_bridge_tier_conversion() {
        use bridge::*;

        let grounding = GroundingTier::T1Universal;
        let primitive = to_primitive_tier(grounding);
        assert_eq!(
            primitive,
            crate::domain_discovery::primitives::PrimitiveTier::T1Universal
        );

        let back = from_primitive_tier(primitive);
        assert_eq!(back, GroundingTier::T1Universal);
    }
}
