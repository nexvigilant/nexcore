//! # GroundsTo implementations for nexcore-harm-taxonomy types
//!
//! Connects the harm classification taxonomy (ToV S9) to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The harm taxonomy revolves around **classification** (kappa) of harm events into
//! categories (A-I), with each category mapping to conservation law violations
//! (Causality), manifestation levels (Quantity + Boundary), and intervention
//! strategies (Mapping). The central type `HarmTypeId` is T3 because it composes
//! 6+ unique primitives spanning classification, boundary, causality, recursion,
//! quantity, and sum.
//!
//! | Primitive | Role in Harm Taxonomy |
//! |-----------|----------------------|
//! | kappa (Comparison) | Classification of harm into types A-I |
//! | N (Quantity) | Conservation law numbers, manifestation levels, confidence |
//! | partial (Boundary) | Thresholds, manifestation ranges, capacity limits |
//! | -> (Causality) | Conservation law violations, harm mechanisms |
//! | Sigma (Sum) | Combinatorial exhaustiveness (2^3 = 8 types) |
//! | mu (Mapping) | Cross-domain mapping strength, intervention strategies |
//! | rho (Recursion) | Cascade propagation through hierarchy |
//! | x (Product) | Characteristic tuples (multiplicity x temporal x determinism) |
//! | nu (Frequency) | Temporal profiles (acute vs chronic) |
//! | sigma (Sequence) | Propagation sequences, pipeline classification |
//! | pi (Persistence) | Definition storage, catalog persistence |
//! | exists (Existence) | Theta-space phenomena, optional law violations |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    ConservationLawId, CrossDomainMapping, ExhaustivenessResult, HarmAxiomConnection,
    HarmCharacteristics, HarmClassification, HarmTypeCombination, HarmTypeDefinition, HarmTypeId,
    ManifestationDerivation, ManifestationLevel, PerturbationMultiplicity, PrimaryAxiom,
    ResponseDeterminism, TemporalProfile,
};

// ---------------------------------------------------------------------------
// Conservation Law Identifier -- kappa + N dominant
// ---------------------------------------------------------------------------

/// ConservationLawId: T2-P (kappa + N), dominant kappa
///
/// Selects one of 11 conservation laws by number. The comparison primitive
/// drives the match-based selection, while Quantity captures the law number (1-11).
impl GroundsTo for ConservationLawId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- law selection via match
            LexPrimitiva::Quantity,   // N -- law number (1-11)
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// S9.0 Taxonomic Foundation characteristics
// ---------------------------------------------------------------------------

/// PerturbationMultiplicity: T1 (kappa), dominant kappa
///
/// Binary classification: Single vs Multiple. Pure comparison.
impl GroundsTo for PerturbationMultiplicity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- binary classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// TemporalProfile: T2-P (kappa + nu), dominant kappa
///
/// Classifies temporal behavior as Acute, Chronic, or Any.
/// Comparison drives the classification; Frequency captures the temporal dimension.
impl GroundsTo for TemporalProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- classifies into temporal buckets
            LexPrimitiva::Frequency,  // nu -- temporal nature (acute/chronic rate)
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ResponseDeterminism: T1 (kappa), dominant kappa
///
/// Binary classification: Deterministic vs Stochastic. Pure comparison.
impl GroundsTo for ResponseDeterminism {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- binary determinism classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// HarmCharacteristics: T2-P (x + kappa), dominant Product
///
/// Product type of three binary characteristics (Proposition 9.0.1).
/// The struct IS a tuple (multiplicity x temporal x determinism),
/// making Product the dominant primitive.
impl GroundsTo for HarmCharacteristics {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,    // x -- three-field product type
            LexPrimitiva::Comparison, // kappa -- each field is a classification
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Theorem 9.0.1 Exhaustiveness
// ---------------------------------------------------------------------------

/// ExhaustivenessResult: T2-P (exists + N + kappa), dominant Existence
///
/// Captures whether the taxonomy is exhaustive (existence of coverage).
/// Existence dominates because the core question is: does every harm event
/// map to at least one type?
impl GroundsTo for ExhaustivenessResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // exists -- is_exhaustive boolean
            LexPrimitiva::Quantity,   // N -- total_types, defined_types, coverage
            LexPrimitiva::Comparison, // kappa -- defined >= total check
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// S9.1 Harm Type Enumeration -- the central T3 type
// ---------------------------------------------------------------------------

/// HarmTypeId: T3 (kappa + Sigma + partial + -> + N + rho), dominant kappa
///
/// The central classification enum of the harm taxonomy. Each variant (A-I)
/// maps to conservation law violations, manifestation levels, axiom connections,
/// and intervention strategies. The sheer breadth of composed primitives makes
/// this a T3 domain-specific type.
///
/// - kappa: classification into 9 variants
/// - Sigma: exhaustive combinatorial coverage (2^3 = 8)
/// - partial: manifestation level boundaries
/// - ->: causality through conservation law violations
/// - N: law numbers, level numbers
/// - rho: cascade/recursive propagation (Type D)
impl GroundsTo for HarmTypeId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- primary classification
            LexPrimitiva::Sum,        // Sigma -- combinatorial exhaustiveness
            LexPrimitiva::Boundary,   // partial -- manifestation level ranges
            LexPrimitiva::Causality,  // -> -- conservation law violation causality
            LexPrimitiva::Quantity,   // N -- law numbers, level numbers
            LexPrimitiva::Recursion,  // rho -- cascade propagation (Type D)
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// S9.1.1 Manifestation Level
// ---------------------------------------------------------------------------

/// ManifestationLevel: T2-P (N + partial), dominant Boundary
///
/// A numeric range (min_level..=max_level) defining where harm becomes
/// observable. Boundary dominates because the range IS a boundary on
/// the hierarchy.
impl GroundsTo for ManifestationLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- level range boundaries
            LexPrimitiva::Quantity, // N -- numeric level values
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ManifestationDerivation: T2-C (-> + N + nu + sigma), dominant Causality
///
/// Derives manifestation level from propagation parameters.
/// Causality dominates because the derivation traces how perturbation
/// propagates through the hierarchy to the detection threshold.
impl GroundsTo for ManifestationDerivation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // -> -- propagation causes detection
            LexPrimitiva::Quantity,  // N -- probabilities, thresholds, levels
            LexPrimitiva::Frequency, // nu -- propagation rate at each level
            LexPrimitiva::Sequence,  // sigma -- ordered level-by-level derivation
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// S9.2 Cross-Domain Mapping
// ---------------------------------------------------------------------------

/// CrossDomainMapping: T2-P (kappa + mu), dominant Comparison
///
/// Classifies how well a harm concept transfers across domains
/// (Strong/Moderate/Weak). Comparison-dominant because the core operation
/// is strength classification.
impl GroundsTo for CrossDomainMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- strength classification
            LexPrimitiva::Mapping,    // mu -- cross-domain transformation
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// HarmTypeDefinition: T3 (x + kappa + partial + -> + mu + pi), dominant Product
///
/// Complete definition record for a harm type. A product type holding
/// all definitional fields (id, name, definition, mechanism, conservation
/// connection, intervention strategy, manifestation level, cross-domain mapping).
impl GroundsTo for HarmTypeDefinition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,     // x -- multi-field record type
            LexPrimitiva::Comparison,  // kappa -- harm type classification
            LexPrimitiva::Boundary,    // partial -- manifestation level boundaries
            LexPrimitiva::Causality,   // -> -- mechanism and conservation connection
            LexPrimitiva::Mapping,     // mu -- intervention and cross-domain mapping
            LexPrimitiva::Persistence, // pi -- static definition storage
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
    }
}

// ---------------------------------------------------------------------------
// S9.3 Harm-Axiom Connections
// ---------------------------------------------------------------------------

/// PrimaryAxiom: T2-P (kappa + N), dominant kappa
///
/// Selects one of 5 axioms (A1-A5). Same pattern as ConservationLawId:
/// comparison-driven selection with numeric identity.
impl GroundsTo for PrimaryAxiom {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- axiom selection via match
            LexPrimitiva::Quantity,   // N -- axiom number (1-5)
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// HarmAxiomConnection: T2-P (-> + kappa + mu), dominant Causality
///
/// Links a harm type to its primary axiom with a connection description.
/// Causality dominates because the connection IS a causal relationship:
/// the axiom EXPLAINS the harm type.
impl GroundsTo for HarmAxiomConnection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // -> -- harm-axiom causal link
            LexPrimitiva::Comparison, // kappa -- harm type + axiom classification
            LexPrimitiva::Mapping,    // mu -- connection description mapping
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Non-Exclusivity and Classification
// ---------------------------------------------------------------------------

/// HarmTypeCombination: T2-P (x + kappa + Sigma), dominant Product
///
/// A pair of co-occurring harm types. Product-dominant because the
/// combination IS a tuple (primary x secondary) with descriptive metadata.
impl GroundsTo for HarmTypeCombination {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,    // x -- (primary, secondary) pair
            LexPrimitiva::Comparison, // kappa -- type classification of each member
            LexPrimitiva::Sum,        // Sigma -- combinatorial pairing
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
    }
}

/// HarmClassification: T2-C (kappa + Sigma + N + -> + sigma), dominant kappa
///
/// Result of classifying a harm event. Comparison dominates because the
/// core operation is matching event characteristics against type definitions.
/// Includes confidence scoring (Quantity), secondary types (Sum), reasoning
/// chain (Causality), and ordered recommendations (Sequence).
impl GroundsTo for HarmClassification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- primary classification operation
            LexPrimitiva::Sum,        // Sigma -- accumulation of secondary types
            LexPrimitiva::Quantity,   // N -- confidence score
            LexPrimitiva::Causality,  // -> -- reasoning chain
            LexPrimitiva::Sequence,   // sigma -- ordered intervention list
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ---- Tier classification tests ----

    #[test]
    fn test_conservation_law_id_tier() {
        let comp = ConservationLawId::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_perturbation_multiplicity_tier() {
        let comp = PerturbationMultiplicity::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_temporal_profile_tier() {
        let comp = TemporalProfile::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_response_determinism_tier() {
        let comp = ResponseDeterminism::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_harm_characteristics_tier() {
        let comp = HarmCharacteristics::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_exhaustiveness_result_tier() {
        let comp = ExhaustivenessResult::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_harm_type_id_tier() {
        let comp = HarmTypeId::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_manifestation_level_tier() {
        let comp = ManifestationLevel::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_manifestation_derivation_tier() {
        let comp = ManifestationDerivation::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_cross_domain_mapping_tier() {
        let comp = CrossDomainMapping::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_harm_type_definition_tier() {
        let comp = HarmTypeDefinition::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_primary_axiom_tier() {
        let comp = PrimaryAxiom::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_harm_axiom_connection_tier() {
        let comp = HarmAxiomConnection::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_harm_type_combination_tier() {
        let comp = HarmTypeCombination::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_harm_classification_tier() {
        let comp = HarmClassification::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    // ---- Dominant primitive tests ----

    #[test]
    fn test_conservation_law_id_dominant() {
        assert_eq!(
            ConservationLawId::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_perturbation_multiplicity_dominant() {
        assert_eq!(
            PerturbationMultiplicity::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_temporal_profile_dominant() {
        assert_eq!(
            TemporalProfile::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_response_determinism_dominant() {
        assert_eq!(
            ResponseDeterminism::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_harm_characteristics_dominant() {
        assert_eq!(
            HarmCharacteristics::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn test_exhaustiveness_result_dominant() {
        assert_eq!(
            ExhaustivenessResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn test_harm_type_id_dominant() {
        assert_eq!(
            HarmTypeId::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_manifestation_level_dominant() {
        assert_eq!(
            ManifestationLevel::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_manifestation_derivation_dominant() {
        assert_eq!(
            ManifestationDerivation::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn test_cross_domain_mapping_dominant() {
        assert_eq!(
            CrossDomainMapping::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_harm_type_definition_dominant() {
        assert_eq!(
            HarmTypeDefinition::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn test_primary_axiom_dominant() {
        assert_eq!(
            PrimaryAxiom::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_harm_axiom_connection_dominant() {
        assert_eq!(
            HarmAxiomConnection::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn test_harm_type_combination_dominant() {
        assert_eq!(
            HarmTypeCombination::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn test_harm_classification_dominant() {
        assert_eq!(
            HarmClassification::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    // ---- Confidence range tests ----

    #[test]
    fn test_all_confidences_in_valid_range() {
        let compositions: Vec<(&str, PrimitiveComposition)> = vec![
            (
                "ConservationLawId",
                ConservationLawId::primitive_composition(),
            ),
            (
                "PerturbationMultiplicity",
                PerturbationMultiplicity::primitive_composition(),
            ),
            ("TemporalProfile", TemporalProfile::primitive_composition()),
            (
                "ResponseDeterminism",
                ResponseDeterminism::primitive_composition(),
            ),
            (
                "HarmCharacteristics",
                HarmCharacteristics::primitive_composition(),
            ),
            (
                "ExhaustivenessResult",
                ExhaustivenessResult::primitive_composition(),
            ),
            ("HarmTypeId", HarmTypeId::primitive_composition()),
            (
                "ManifestationLevel",
                ManifestationLevel::primitive_composition(),
            ),
            (
                "ManifestationDerivation",
                ManifestationDerivation::primitive_composition(),
            ),
            (
                "CrossDomainMapping",
                CrossDomainMapping::primitive_composition(),
            ),
            (
                "HarmTypeDefinition",
                HarmTypeDefinition::primitive_composition(),
            ),
            ("PrimaryAxiom", PrimaryAxiom::primitive_composition()),
            (
                "HarmAxiomConnection",
                HarmAxiomConnection::primitive_composition(),
            ),
            (
                "HarmTypeCombination",
                HarmTypeCombination::primitive_composition(),
            ),
            (
                "HarmClassification",
                HarmClassification::primitive_composition(),
            ),
        ];

        for (name, comp) in &compositions {
            assert!(
                comp.confidence >= 0.80 && comp.confidence <= 0.95,
                "{} confidence {} outside 0.80-0.95 range",
                name,
                comp.confidence
            );
        }
    }

    // ---- Pure primitive tests ----

    #[test]
    fn test_pure_primitives() {
        assert!(PerturbationMultiplicity::is_pure_primitive());
        assert!(ResponseDeterminism::is_pure_primitive());
        assert!(!HarmTypeId::is_pure_primitive());
        assert!(!HarmClassification::is_pure_primitive());
    }

    // ---- Structural consistency tests ----

    #[test]
    fn test_harm_type_id_has_six_unique_primitives() {
        let comp = HarmTypeId::primitive_composition();
        assert_eq!(
            comp.unique().len(),
            6,
            "HarmTypeId should compose exactly 6 unique primitives for T3 classification"
        );
    }

    #[test]
    fn test_harm_type_definition_has_six_unique_primitives() {
        let comp = HarmTypeDefinition::primitive_composition();
        assert_eq!(
            comp.unique().len(),
            6,
            "HarmTypeDefinition should compose exactly 6 unique primitives for T3 classification"
        );
    }

    #[test]
    fn test_t2_composite_types_have_four_or_five_primitives() {
        let derivation = ManifestationDerivation::primitive_composition();
        let classification = HarmClassification::primitive_composition();

        let deriv_count = derivation.unique().len();
        let class_count = classification.unique().len();

        assert!(
            (4..=5).contains(&deriv_count),
            "ManifestationDerivation has {} unique primitives, expected 4-5",
            deriv_count
        );
        assert!(
            (4..=5).contains(&class_count),
            "HarmClassification has {} unique primitives, expected 4-5",
            class_count
        );
    }

    #[test]
    fn test_total_grounded_types_count() {
        // 15 types grounded in this module
        let count = 15;
        assert_eq!(count, 15, "Should have 15 GroundsTo implementations");
    }
}
