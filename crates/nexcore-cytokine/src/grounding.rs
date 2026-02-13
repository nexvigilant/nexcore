//! # GroundsTo implementations for nexcore-cytokine types
//!
//! Connects cytokine signaling types to the Lex Primitiva type system.
//!
//! ## → (Causality) Focus
//!
//! Cytokines ARE causal signals: emission → receptor activation → response.
//! The crate maps biological immune signaling to inter-component messaging.
//! Each family maps to a different T1 primitive in its purpose.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::types::{Cytokine, CytokineFamily, Scope, ThreatLevel};

// ---------------------------------------------------------------------------
// Classification types — Σ dominant
// ---------------------------------------------------------------------------

/// CytokineFamily: T2-P (Σ · →), dominant Σ
///
/// Nine-variant sum type classifying signal families.
/// Sum-dominant: the type IS a categorical alternation (IL1|IL2|...|Custom).
/// Each variant maps to a different T1 primitive in its biological purpose.
impl GroundsTo for CytokineFamily {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ — variant alternation
            LexPrimitiva::Causality, // → — each family causes specific response
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// ThreatLevel: T2-P (N · κ), dominant N
///
/// Ordinal signal concentration: Trace < Low < Medium < High < Critical.
/// Quantity-dominant: threat level IS a numeric concentration level.
impl GroundsTo for ThreatLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — concentration level
            LexPrimitiva::Comparison, // κ — ordered comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Scope: T2-P (λ · ∂), dominant λ
///
/// Signal propagation scope: Autocrine → Paracrine → Endocrine → Systemic.
/// Location-dominant: scope defines spatial reach of the signal.
impl GroundsTo for Scope {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ — spatial scope
            LexPrimitiva::Boundary, // ∂ — scope boundary
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Composite types — multi-primitive
// ---------------------------------------------------------------------------

/// Cytokine: T3 (→ · Σ · N · λ · ς · π), dominant →
///
/// A typed signal carrying family, severity, scope, payload, and TTL.
/// Causality-dominant: a cytokine IS a causal signal — emission causes
/// receptor activation causes response.
impl GroundsTo for Cytokine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // → — signal causes response
            LexPrimitiva::Sum,         // Σ — family variant
            LexPrimitiva::Quantity,    // N — severity concentration
            LexPrimitiva::Location,    // λ — scope/source/target
            LexPrimitiva::State,       // ς — payload carries state
            LexPrimitiva::Persistence, // π — TTL-based persistence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
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
    fn cytokine_family_is_sum_dominant() {
        let comp = CytokineFamily::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(CytokineFamily::tier(), Tier::T2Primitive);
    }

    #[test]
    fn threat_level_is_quantity_dominant() {
        let comp = ThreatLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn scope_is_location_dominant() {
        let comp = Scope::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn cytokine_is_t3_causality_dominant() {
        // 6 primitives = T3
        assert_eq!(Cytokine::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            Cytokine::primitive_composition().dominant,
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn cytokine_includes_persistence() {
        let comp = Cytokine::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Persistence));
    }
}
