//! # GroundsTo Implementations
//!
//! Maps every public type in this crate to its T1 Lex Primitiva composition.
//!
//! ## Grounding Table
//!
//! | Type | Primitives | Tier | Dominant |
//! |------|-----------|------|----------|
//! | `IncidentSeverity` | Sigma (Sum) | T1 | Sigma 1.0 |
//! | `IncidentSignature` | mu + kappa | T2-P | mu 0.80 |
//! | `Incident` | pi + arrow + sigma_seq + varsigma + kappa | T2-C | pi 0.55 |
//! | `PlaybookStep` | arrow + varsigma | T2-P | arrow 0.80 |
//! | `Playbook` | sigma_seq + mu + pi + kappa | T2-C | sigma_seq 0.60 |
//! | `PlaybookMatch` | kappa + mu | T2-P | kappa 0.80 |
//! | `MemoryConfig` | varsigma + N | T2-P | varsigma 0.80 |
//! | `MemoryError` | Sigma (Sum) | T1 | Sigma 1.0 |
//! | `MemoryStats` | N + Sigma | T2-P | N 0.80 |
//! | `MemoryStore` | pi + mu + varsigma | T2-P | pi 0.70 |
//! | `SimilarIncident` | kappa + N | T2-P | kappa 0.80 |

use crate::incident::{Incident, IncidentSeverity, IncidentSignature};
use crate::playbook::{Playbook, PlaybookMatch, PlaybookStep};
use crate::store::{MemoryConfig, MemoryError, MemoryStats, MemoryStore, SimilarIncident};
use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

// =============================================================================
// T1 — Pure enum types
// =============================================================================

impl GroundsTo for IncidentSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Sum, 1.0)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

impl GroundsTo for MemoryError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// =============================================================================
// T2-P — 2-3 primitive combinations
// =============================================================================

impl GroundsTo for IncidentSignature {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for PlaybookStep {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

impl GroundsTo for PlaybookMatch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

impl GroundsTo for MemoryConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::State, 0.80)
    }
}

impl GroundsTo for MemoryStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for MemoryStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

impl GroundsTo for SimilarIncident {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// =============================================================================
// T2-C — 4-5 primitive composites
// =============================================================================

impl GroundsTo for Incident {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.55)
    }
}

impl GroundsTo for Playbook {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Persistence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ── T1 types ────────────────────────────────────────────────────────────

    #[test]
    fn incident_severity_is_t1() {
        assert_eq!(IncidentSeverity::tier(), Tier::T1Universal);
        assert_eq!(
            IncidentSeverity::dominant_primitive(),
            Some(LexPrimitiva::Sum),
        );
        assert!(IncidentSeverity::is_pure_primitive());
    }

    #[test]
    fn memory_error_is_t1() {
        assert_eq!(MemoryError::tier(), Tier::T1Universal);
        assert_eq!(
            MemoryError::dominant_primitive(),
            Some(LexPrimitiva::Sum),
        );
        assert!(MemoryError::is_pure_primitive());
    }

    // ── T2-P types ──────────────────────────────────────────────────────────

    #[test]
    fn incident_signature_is_t2p() {
        assert_eq!(IncidentSignature::tier(), Tier::T2Primitive);
        assert_eq!(
            IncidentSignature::dominant_primitive(),
            Some(LexPrimitiva::Mapping),
        );
        let comp = IncidentSignature::primitive_composition();
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn playbook_step_is_t2p() {
        assert_eq!(PlaybookStep::tier(), Tier::T2Primitive);
        assert_eq!(
            PlaybookStep::dominant_primitive(),
            Some(LexPrimitiva::Causality),
        );
    }

    #[test]
    fn playbook_match_is_t2p() {
        assert_eq!(PlaybookMatch::tier(), Tier::T2Primitive);
        assert_eq!(
            PlaybookMatch::dominant_primitive(),
            Some(LexPrimitiva::Comparison),
        );
    }

    #[test]
    fn memory_config_is_t2p() {
        assert_eq!(MemoryConfig::tier(), Tier::T2Primitive);
        assert_eq!(
            MemoryConfig::dominant_primitive(),
            Some(LexPrimitiva::State),
        );
    }

    #[test]
    fn memory_stats_is_t2p() {
        assert_eq!(MemoryStats::tier(), Tier::T2Primitive);
        assert_eq!(
            MemoryStats::dominant_primitive(),
            Some(LexPrimitiva::Quantity),
        );
    }

    #[test]
    fn memory_store_is_t2p() {
        assert_eq!(MemoryStore::tier(), Tier::T2Primitive);
        assert_eq!(
            MemoryStore::dominant_primitive(),
            Some(LexPrimitiva::Persistence),
        );
    }

    #[test]
    fn similar_incident_is_t2p() {
        assert_eq!(SimilarIncident::tier(), Tier::T2Primitive);
        assert_eq!(
            SimilarIncident::dominant_primitive(),
            Some(LexPrimitiva::Comparison),
        );
    }

    // ── T2-C composites ─────────────────────────────────────────────────────

    #[test]
    fn incident_is_t2c() {
        assert_eq!(Incident::tier(), Tier::T2Composite);
        assert_eq!(
            Incident::dominant_primitive(),
            Some(LexPrimitiva::Persistence),
        );
        let comp = Incident::primitive_composition();
        assert_eq!(comp.unique().len(), 5);
        assert!((comp.confidence - 0.55).abs() < f64::EPSILON);
    }

    #[test]
    fn playbook_is_t2c() {
        assert_eq!(Playbook::tier(), Tier::T2Composite);
        assert_eq!(
            Playbook::dominant_primitive(),
            Some(LexPrimitiva::Sequence),
        );
        let comp = Playbook::primitive_composition();
        assert_eq!(comp.unique().len(), 4);
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    // ── Cross-cutting ───────────────────────────────────────────────────────

    #[test]
    fn all_types_have_valid_dominant() {
        // Every type's dominant must appear in its own composition.
        fn check<T: GroundsTo>() {
            let comp = T::primitive_composition();
            if let Some(dom) = comp.dominant {
                assert!(
                    comp.primitives.contains(&dom),
                    "Dominant {:?} not in primitives",
                    dom,
                );
            }
        }
        check::<IncidentSeverity>();
        check::<IncidentSignature>();
        check::<Incident>();
        check::<PlaybookStep>();
        check::<Playbook>();
        check::<PlaybookMatch>();
        check::<MemoryConfig>();
        check::<MemoryError>();
        check::<MemoryStats>();
        check::<MemoryStore>();
        check::<SimilarIncident>();
    }

    #[test]
    fn confidence_in_range() {
        fn check<T: GroundsTo>() {
            let comp = T::primitive_composition();
            assert!(
                (0.0..=1.0).contains(&comp.confidence),
                "Confidence {} out of range",
                comp.confidence,
            );
        }
        check::<IncidentSeverity>();
        check::<IncidentSignature>();
        check::<Incident>();
        check::<PlaybookStep>();
        check::<Playbook>();
        check::<PlaybookMatch>();
        check::<MemoryConfig>();
        check::<MemoryError>();
        check::<MemoryStats>();
        check::<MemoryStore>();
        check::<SimilarIncident>();
    }
}
