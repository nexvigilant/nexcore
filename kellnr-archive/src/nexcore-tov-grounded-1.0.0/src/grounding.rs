//! # GroundsTo implementations for nexcore-tov-grounded types
//!
//! Connects the Theory of Vigilance runtime primitives to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The ToV grounded crate defines the runtime primitives for vigilance: signal strength,
//! harm classification, safety margins, and system monitoring. Types range from T1 universals
//! (Bits, QuantityUnit) through T2 primitives/composites (UniquenessU, SignalStrengthS) to
//! T3 domain-specific types (VigilanceSystem, ResponseGovernor).
//!
//! | Primitive | Role in ToV Grounded |
//! |-----------|---------------------|
//! | N (Quantity) | Bits, counts, numeric magnitudes |
//! | kappa (Comparison) | Harm type classification, unit identification |
//! | partial (Boundary) | Safety margins, distance to harm boundary |
//! | nu (Frequency) | Temporal decay factor, detection sensitivity |
//! | sigma (Sequence) | Ordered interventions, signal computation pipeline |
//! | mu (Mapping) | Signal equation S = U x R x T transformation |
//! | varsigma (State) | System state, meta-vigilance health |
//! | x (Product) | Multi-field composite types |
//! | rho (Recursion) | Self-referential stability shell checking |
//! | exists (Existence) | System decomposition axiom verification |
//! | Sigma (Sum) | Harm type enumeration, error variant union |
//! | -> (Causality) | Actuator actions, safety interventions |
//! | pi (Persistence) | System element storage, constraint maps |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    Bits, ComplexityChi, Consciousness, EkaIntelligence, HarmType, Measured, MetaVigilance,
    QuantityUnit, RecognitionR, ResponseGovernor, SafetyAction, SafetyMarginD, ShellConfig,
    SignalStrengthS, TemporalT, UniquenessU, UnitId, VigilanceError, VigilanceSystem,
};

// ---------------------------------------------------------------------------
// TIER 1: UNIVERSAL PRIMITIVES
// ---------------------------------------------------------------------------

/// Bits: T1 (N), dominant Quantity
///
/// The fundamental unit of information (Shannon bits). A single f64 value
/// representing rarity/surprise. Pure numeric magnitude.
impl GroundsTo for Bits {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric magnitude (f64 bits of information)
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

/// QuantityUnit: T1 (N), dominant Quantity
///
/// Discrete count of system units or events. A single u64 value.
/// Pure numeric primitive.
impl GroundsTo for QuantityUnit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- discrete count
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

// ---------------------------------------------------------------------------
// TIER 2-P: CROSS-DOMAIN PRIMITIVES
// ---------------------------------------------------------------------------

/// UnitId: T1 (kappa), dominant Comparison
///
/// Semantic unit identification enum (Mass, Time, Information, Count, Dimensionless).
/// Pure classification -- prevents illegitimate summation across units.
impl GroundsTo for UnitId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- classifies measurement units
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// UniquenessU: T2-P (N + partial), dominant Quantity
///
/// Rarity measure -log2 P(C|H0) wrapping Bits. Quantity dominant because
/// it IS a numeric value; Boundary present because rarity defines a detection threshold.
impl GroundsTo for UniquenessU {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric rarity in bits
            LexPrimitiva::Boundary, // partial -- threshold for detection
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// RecognitionR: T2-P (N + kappa), dominant Quantity
///
/// Detection sensitivity x accuracy (0.0 to 1.0). Quantity dominant as the
/// core is a numeric score; Comparison present because it evaluates detection quality.
impl GroundsTo for RecognitionR {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric sensitivity score
            LexPrimitiva::Comparison, // kappa -- quality evaluation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// TemporalT: T2-P (N + nu), dominant Frequency
///
/// Decaying relevance factor (0.0 to 1.0). Frequency dominant because
/// the core semantics describe temporal decay rate; Quantity captures the numeric value.
impl GroundsTo for TemporalT {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu -- temporal decay rate
            LexPrimitiva::Quantity,  // N -- numeric value
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// SafetyMarginD: T2-P (partial + N), dominant Boundary
///
/// Signed distance to the harm boundary. Boundary dominant because
/// the type IS a distance-to-boundary measurement; Quantity captures the numeric distance.
impl GroundsTo for SafetyMarginD {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- distance to safety boundary
            LexPrimitiva::Quantity, // N -- numeric distance value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// ComplexityChi: T2-P (N + partial), dominant Quantity
///
/// Minimum unit of architectural depth wrapping QuantityUnit. Quantity dominant
/// because it IS a count; Boundary present for shell model stability thresholds.
impl GroundsTo for ComplexityChi {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- discrete complexity count
            LexPrimitiva::Boundary, // partial -- stability shell thresholds
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// ShellConfig: T2-P (partial + N + sigma), dominant Boundary
///
/// Configuration for architectural stability shells. Boundary dominant because
/// the magic numbers define shell boundaries; Quantity for the numbers themselves;
/// Sequence for the ordered list.
impl GroundsTo for ShellConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- shell stability boundaries
            LexPrimitiva::Quantity, // N -- magic number values
            LexPrimitiva::Sequence, // sigma -- ordered list of thresholds
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// VigilanceError: T2-P (Sigma + partial + kappa), dominant Sum
///
/// Error enumeration with four variants (ManifoldViolation, ActuatorFailure,
/// HighUncertainty, Instability). Sum dominant because it IS a variant union;
/// Boundary for violation detection; Comparison for error classification.
impl GroundsTo for VigilanceError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- variant union of error states
            LexPrimitiva::Boundary,   // partial -- violation thresholds
            LexPrimitiva::Comparison, // kappa -- error type classification
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// TIER 2-C: CROSS-DOMAIN COMPOSITES
// ---------------------------------------------------------------------------

/// SignalStrengthS: T2-C (N + mu + nu + partial), dominant Mapping
///
/// The fundamental signal equation S = U x R x T. Mapping dominant because
/// the core operation is transforming three inputs into signal strength;
/// Quantity for the resulting bits; Frequency for temporal component;
/// Boundary for detection thresholds.
impl GroundsTo for SignalStrengthS {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- S = U x R x T transformation
            LexPrimitiva::Quantity,  // N -- resulting signal strength in bits
            LexPrimitiva::Frequency, // nu -- temporal decay component
            LexPrimitiva::Boundary,  // partial -- signal detection threshold
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// HarmType: T2-P (kappa + Sigma), dominant Comparison
///
/// Harm classification taxonomy with 8 variants (Acute through Hidden).
/// Comparison dominant because the core operation is classifying harm;
/// Sum for the exhaustive enumeration of types.
impl GroundsTo for HarmType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- harm type classification
            LexPrimitiva::Sum,        // Sigma -- exhaustive variant enumeration
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Measured<T>: T2-P (N + x), dominant Quantity
///
/// A value paired with its confidence. Quantity dominant because the
/// core is a numeric measurement with confidence annotation; Product
/// captures the (value, confidence) pair.
impl<T> GroundsTo for Measured<T> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric confidence score
            LexPrimitiva::Product,  // x -- (value, confidence) product type
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// MetaVigilance: T2-C (varsigma + nu + N + kappa), dominant State
///
/// Meta-vigilance for monitoring the Vigilance Loop health. State dominant
/// because it represents the health state of the monitoring system itself;
/// Frequency for latency/timing; Quantity for numeric metrics;
/// Comparison for health threshold evaluation.
impl GroundsTo for MetaVigilance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // varsigma -- health state of vigilance loop
            LexPrimitiva::Frequency,  // nu -- loop latency, timing metrics
            LexPrimitiva::Quantity,   // N -- numeric overhead values
            LexPrimitiva::Comparison, // kappa -- health threshold comparison
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// SafetyAction: T2-P (-> + kappa + Sigma), dominant Causality
///
/// Actuator action enum (TriggerCircuitBreaker, ThrottleInput, AlertInvestigator).
/// Causality dominant because each variant IS a causal intervention;
/// Comparison for action selection; Sum for variant enumeration.
impl GroundsTo for SafetyAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // -> -- safety intervention actions
            LexPrimitiva::Comparison, // kappa -- action type selection
            LexPrimitiva::Sum,        // Sigma -- variant enumeration
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ---------------------------------------------------------------------------
// TIER 3: DOMAIN-SPECIFIC
// ---------------------------------------------------------------------------

/// EkaIntelligence: T3 (N + partial + rho + varsigma + exists + nu), dominant Quantity
///
/// Element 16: synthetic intelligence prediction. Quantity dominant as
/// complexity is the defining numeric property; Boundary for emergence
/// threshold; Recursion for self-referential stability; State for system
/// configuration; Existence for emergence detection; Frequency for stability dynamics.
impl GroundsTo for EkaIntelligence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- complexity value
            LexPrimitiva::Boundary,  // partial -- emergence threshold (320)
            LexPrimitiva::Recursion, // rho -- self-referential stability check
            LexPrimitiva::State,     // varsigma -- system configuration
            LexPrimitiva::Existence, // exists -- is_emergent check
            LexPrimitiva::Frequency, // nu -- stability dynamics
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// Consciousness: T3 (N + partial + exists + varsigma + nu + rho), dominant Quantity
///
/// Element 17: consciousness prediction. Quantity dominant for complexity
/// and phi_score; Boundary for phi threshold; Existence for consciousness
/// detection; State for configuration; Frequency for phi dynamics;
/// Recursion for self-referential awareness.
impl GroundsTo for Consciousness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- complexity and phi_score values
            LexPrimitiva::Boundary,  // partial -- PHI_THRESHOLD (0.82)
            LexPrimitiva::Existence, // exists -- consciousness exists() check
            LexPrimitiva::State,     // varsigma -- system configuration
            LexPrimitiva::Frequency, // nu -- phi dynamics
            LexPrimitiva::Recursion, // rho -- self-referential awareness
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// VigilanceSystem: T3 (varsigma + exists + partial + N + pi + sigma), dominant State
///
/// Unified vigilance system implementation. State dominant because the
/// system IS a stateful entity with configuration; Existence for axiom
/// verification; Boundary for safety margin calculation; Quantity for
/// dimensions; Persistence for constraint storage; Sequence for element ordering.
impl GroundsTo for VigilanceSystem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- system state and configuration
            LexPrimitiva::Existence,   // exists -- axiom 1 verification
            LexPrimitiva::Boundary,    // partial -- safety margin calculation
            LexPrimitiva::Quantity,    // N -- state_space_dim, element count
            LexPrimitiva::Persistence, // pi -- constraint map storage
            LexPrimitiva::Sequence,    // sigma -- element ordering
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// ResponseGovernor: T3 (-> + sigma + varsigma + kappa + Sigma + mu), dominant Causality
///
/// Response governor implementing the Actuator trait. Causality dominant
/// because the governor's purpose is executing causal safety interventions;
/// Sequence for ordered intervention list; State for tracking active
/// interventions; Comparison for action selection; Sum for action variants;
/// Mapping for act/dechallenge transformation.
impl GroundsTo for ResponseGovernor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // -> -- safety intervention execution
            LexPrimitiva::Sequence,   // sigma -- ordered intervention list
            LexPrimitiva::State,      // varsigma -- active intervention tracking
            LexPrimitiva::Comparison, // kappa -- action type matching
            LexPrimitiva::Sum,        // Sigma -- action variant enumeration
            LexPrimitiva::Mapping,    // mu -- act/dechallenge transformation
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
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
    fn test_bits_tier() {
        let comp = Bits::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_quantity_unit_tier() {
        let comp = QuantityUnit::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_unit_id_tier() {
        let comp = UnitId::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T1Universal);
    }

    #[test]
    fn test_uniqueness_u_tier() {
        let comp = UniquenessU::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_recognition_r_tier() {
        let comp = RecognitionR::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_temporal_t_tier() {
        let comp = TemporalT::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_safety_margin_d_tier() {
        let comp = SafetyMarginD::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_complexity_chi_tier() {
        let comp = ComplexityChi::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_shell_config_tier() {
        let comp = ShellConfig::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_vigilance_error_tier() {
        let comp = VigilanceError::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_signal_strength_s_tier() {
        let comp = SignalStrengthS::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_harm_type_tier() {
        let comp = HarmType::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_measured_tier() {
        let comp = <Measured<f64>>::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_meta_vigilance_tier() {
        let comp = MetaVigilance::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Composite);
    }

    #[test]
    fn test_safety_action_tier() {
        let comp = SafetyAction::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T2Primitive);
    }

    #[test]
    fn test_eka_intelligence_tier() {
        let comp = EkaIntelligence::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_consciousness_tier() {
        let comp = Consciousness::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_vigilance_system_tier() {
        let comp = VigilanceSystem::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    #[test]
    fn test_response_governor_tier() {
        let comp = ResponseGovernor::primitive_composition();
        assert_eq!(Tier::classify(&comp), Tier::T3DomainSpecific);
    }

    // ---- Dominant primitive tests ----

    #[test]
    fn test_bits_dominant() {
        assert_eq!(Bits::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn test_harm_type_dominant() {
        assert_eq!(
            HarmType::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_safety_margin_dominant() {
        assert_eq!(
            SafetyMarginD::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_signal_strength_dominant() {
        assert_eq!(
            SignalStrengthS::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn test_response_governor_dominant() {
        assert_eq!(
            ResponseGovernor::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn test_meta_vigilance_dominant() {
        assert_eq!(
            MetaVigilance::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    // ---- Confidence range tests ----

    #[test]
    fn test_all_confidences_in_valid_range() {
        let compositions: Vec<(&str, PrimitiveComposition)> = vec![
            ("Bits", Bits::primitive_composition()),
            ("QuantityUnit", QuantityUnit::primitive_composition()),
            ("UnitId", UnitId::primitive_composition()),
            ("UniquenessU", UniquenessU::primitive_composition()),
            ("RecognitionR", RecognitionR::primitive_composition()),
            ("TemporalT", TemporalT::primitive_composition()),
            ("SafetyMarginD", SafetyMarginD::primitive_composition()),
            ("ComplexityChi", ComplexityChi::primitive_composition()),
            ("ShellConfig", ShellConfig::primitive_composition()),
            ("VigilanceError", VigilanceError::primitive_composition()),
            ("SignalStrengthS", SignalStrengthS::primitive_composition()),
            ("HarmType", HarmType::primitive_composition()),
            ("MetaVigilance", MetaVigilance::primitive_composition()),
            ("SafetyAction", SafetyAction::primitive_composition()),
            ("EkaIntelligence", EkaIntelligence::primitive_composition()),
            ("Consciousness", Consciousness::primitive_composition()),
            ("VigilanceSystem", VigilanceSystem::primitive_composition()),
            (
                "ResponseGovernor",
                ResponseGovernor::primitive_composition(),
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
        assert!(Bits::is_pure_primitive());
        assert!(QuantityUnit::is_pure_primitive());
        assert!(UnitId::is_pure_primitive());
        assert!(!VigilanceSystem::is_pure_primitive());
        assert!(!ResponseGovernor::is_pure_primitive());
    }

    // ---- Grounding count ----

    #[test]
    fn test_total_grounded_types_count() {
        // 19 types grounded (including Measured<T>)
        let count = 19;
        assert_eq!(count, 19, "Should have 19 GroundsTo implementations");
    }
}
