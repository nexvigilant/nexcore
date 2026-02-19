//! # GroundsTo implementations for nexcore-insight types
//!
//! Maps each composite type to its Lex Primitiva T1 composition.

use crate::composites::{
    Compression, Connection, Novelty, NoveltyReason, Pattern, Recognition, Suddenness,
};
use crate::engine::InsightEngine;
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

// ── Pattern = sigma + kappa + mu ─────────────────────────────────────────────

impl GroundsTo for Pattern {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }

    /// Pattern confidence mutates via record_occurrence() — reversible.
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ── Recognition = kappa + exists + sigma ─────────────────────────────────────

impl GroundsTo for Recognition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.9)
    }
}

// ── Novelty = void + exists + sigma ──────────────────────────────────────────

impl GroundsTo for Novelty {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,
            LexPrimitiva::Existence,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

// ── NoveltyReason = void + comparison (T2-P) ─────────────────────────────────

impl GroundsTo for NoveltyReason {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Void, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Void, 0.9)
    }
}

// ── Connection = mu + kappa + state ──────────────────────────────────────────

impl GroundsTo for Connection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }

    /// Connection strength is mutable — reversible state change.
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ── Compression = N + mu + kappa ─────────────────────────────────────────────

impl GroundsTo for Compression {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.8)
    }
}

// ── Suddenness = sigma + boundary + N + kappa ────────────────────────────────

impl GroundsTo for Suddenness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.8)
    }
}

// ── InsightEngine = full INSIGHT decomposition (T3) ──────────────────────────

impl GroundsTo for InsightEngine {
    fn primitive_composition() -> PrimitiveComposition {
        // INSIGHT = <sigma, kappa, mu, exists, state, void, N, boundary>
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Void,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }

    /// Engine accumulates observations, patterns, events — append-only, irreversible.
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::state_mode::StateMode;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn test_pattern_grounds_to_t2c() {
        let comp = Pattern::primitive_composition();
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(
            Pattern::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_recognition_grounds_to_t2p() {
        let comp = Recognition::primitive_composition();
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(
            Recognition::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_novelty_grounds_to_t2p() {
        let comp = Novelty::primitive_composition();
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(Novelty::dominant_primitive(), Some(LexPrimitiva::Void));
    }

    #[test]
    fn test_novelty_reason_grounds_to_t2p() {
        let comp = NoveltyReason::primitive_composition();
        assert_eq!(comp.unique().len(), 2);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_connection_grounds_to_t2p() {
        let comp = Connection::primitive_composition();
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(
            Connection::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_compression_grounds_to_t2p() {
        let comp = Compression::primitive_composition();
        assert_eq!(comp.unique().len(), 3);
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
        assert_eq!(
            Compression::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn test_suddenness_grounds_to_t2c() {
        let comp = Suddenness::primitive_composition();
        assert_eq!(comp.unique().len(), 4);
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
        assert_eq!(
            Suddenness::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_engine_grounds_to_t3() {
        let comp = InsightEngine::primitive_composition();
        assert_eq!(comp.unique().len(), 8);
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
        assert_eq!(
            InsightEngine::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_pattern_state_mode_mutable() {
        assert_eq!(Pattern::state_mode(), Some(StateMode::Mutable));
    }

    #[test]
    fn test_connection_state_mode_mutable() {
        assert_eq!(Connection::state_mode(), Some(StateMode::Mutable));
    }

    #[test]
    fn test_engine_state_mode_accumulated() {
        assert_eq!(InsightEngine::state_mode(), Some(StateMode::Accumulated));
        // Verify composition also carries the mode
        let comp = InsightEngine::primitive_composition();
        assert_eq!(comp.state_mode, Some(StateMode::Accumulated));
    }

    #[test]
    fn test_stateless_composites_have_no_mode() {
        // Types without ς in their composition should return None
        assert_eq!(Novelty::state_mode(), None);
        assert_eq!(NoveltyReason::state_mode(), None);
        assert_eq!(Recognition::state_mode(), None);
        assert_eq!(Compression::state_mode(), None);
        assert_eq!(Suddenness::state_mode(), None);
    }

    #[test]
    fn test_all_composites_have_valid_confidence() {
        let compositions = vec![
            Pattern::primitive_composition(),
            Recognition::primitive_composition(),
            Novelty::primitive_composition(),
            NoveltyReason::primitive_composition(),
            Connection::primitive_composition(),
            Compression::primitive_composition(),
            Suddenness::primitive_composition(),
            InsightEngine::primitive_composition(),
        ];
        for comp in compositions {
            assert!(
                comp.confidence >= 0.0 && comp.confidence <= 1.0,
                "Invalid confidence {:.2} for composition {:?}",
                comp.confidence,
                comp.primitives,
            );
        }
    }
}
