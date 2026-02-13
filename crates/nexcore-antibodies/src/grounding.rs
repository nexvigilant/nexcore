//! # GroundsTo implementations for nexcore-antibodies types
//!
//! Connects the adaptive immune layer to the Lex Primitiva type system.
//!
//! ## κ (Comparison) Focus
//!
//! The core operation is epitope-paratope binding — structural comparison.
//! The crate maps biological immune recognition to software threat detection:
//! antigen detection (∃), boundary defense (∂), and causal response (→).

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    AffinityScore, Antibody, AntibodyError, AntibodyRepertoire, Antigen, BindingResult, Epitope,
    ImmunoglobulinClass, NeutralizationAction, Paratope, ThreatSeverity,
};

// ---------------------------------------------------------------------------
// Classification types — Σ dominant
// ---------------------------------------------------------------------------

/// ImmunoglobulinClass: T2-P (Σ · κ), dominant Σ
///
/// Five-variant sum type classifying immune response behavior.
/// Sum-dominant: the type IS a categorical alternation (IgG|IgM|IgA|IgD|IgE).
impl GroundsTo for ImmunoglobulinClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — five-variant enum
            LexPrimitiva::Comparison, // κ — threshold ordering between classes
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// ThreatSeverity: T2-P (κ · Σ), dominant κ
///
/// Ordinal severity classification: Low < Medium < High < Critical.
/// Comparison-dominant: the purpose is ordered severity comparison.
impl GroundsTo for ThreatSeverity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — ordinal comparison between levels
            LexPrimitiva::Sum,        // Σ — four-variant enum
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Measurement types — N dominant
// ---------------------------------------------------------------------------

/// AffinityScore: T2-P (N · ∂), dominant N
///
/// Binding strength in [0.0, 1.0]. Clamped numeric measurement.
/// Quantity-dominant: the score IS a numeric measurement.
impl GroundsTo for AffinityScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — f64 binding strength
            LexPrimitiva::Boundary, // ∂ — clamped to [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Recognition types — κ dominant
// ---------------------------------------------------------------------------

/// Epitope: T2-P (κ · λ), dominant κ
///
/// Matchable feature on an antigen — the "key" in lock-and-key binding.
/// Comparison-dominant: the epitope exists to be compared against paratopes.
impl GroundsTo for Epitope {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — matchable signature
            LexPrimitiva::Location,   // λ — identity within domain
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Paratope: T2-P (κ · μ · N), dominant κ
///
/// The antibody's recognition site — the "lock" that matches epitope "keys".
/// Comparison-dominant: the matching logic IS comparison.
impl GroundsTo for Paratope {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — matching rule
            LexPrimitiva::Mapping,    // μ — pattern → match function
            LexPrimitiva::Quantity,   // N — specificity count
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Composite types — multi-primitive
// ---------------------------------------------------------------------------

/// Antigen: T2-C (∃ · ∂ · σ · κ · N), dominant ∃
///
/// A detected threat with its matchable features. The antigen EXISTS
/// as a discovered entity carrying epitopes for recognition.
/// Existence-dominant: detection of the threat IS the core event.
impl GroundsTo for Antigen {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ — threat detected/exists
            LexPrimitiva::Boundary,   // ∂ — severity classification
            LexPrimitiva::Sequence,   // σ — ordered epitope list
            LexPrimitiva::Comparison, // κ — severity comparison
            LexPrimitiva::Quantity,   // N — severity level
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// NeutralizationAction: T2-C (→ · ∂ · Σ), dominant →
///
/// Response action upon successful binding. Each variant causes a
/// different system state change.
/// Causality-dominant: the action IS a cause → effect relationship.
impl GroundsTo for NeutralizationAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → — action causes system response
            LexPrimitiva::Boundary,  // ∂ — action severity boundaries
            LexPrimitiva::Sum,       // Σ — five-variant alternation
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// BindingResult: T2-C (κ · → · N · ∃), dominant κ
///
/// Result of an antibody-antigen binding attempt.
/// Comparison-dominant: the result IS the outcome of a match comparison.
impl GroundsTo for BindingResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — binding comparison result
            LexPrimitiva::Causality,  // → — binding → action
            LexPrimitiva::Quantity,   // N — affinity score
            LexPrimitiva::Existence,  // ∃ — bound or not
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3 domain types
// ---------------------------------------------------------------------------

/// Antibody: T3 (κ · ∂ · ∃ · → · Σ · μ), dominant κ
///
/// Complete recognition + neutralization unit.
/// The full adaptive immune response: paratope binds epitope,
/// Ig class determines behavior, action neutralizes threat.
/// Comparison-dominant: the antibody's purpose IS structural matching.
impl GroundsTo for Antibody {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — paratope matching
            LexPrimitiva::Boundary,   // ∂ — threshold-gated binding
            LexPrimitiva::Existence,  // ∃ — threat recognition
            LexPrimitiva::Causality,  // → — binding → response
            LexPrimitiva::Sum,        // Σ — Ig class variant
            LexPrimitiva::Mapping,    // μ — pattern → action mapping
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// AntibodyRepertoire: T3 (π · μ · κ · σ · ∃), dominant π
///
/// Persistent indexed collection of all known antibodies.
/// Persistence-dominant: the repertoire IS the memory of past encounters.
impl GroundsTo for AntibodyRepertoire {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π — persisted immune memory
            LexPrimitiva::Mapping,     // μ — paratope → antibody index
            LexPrimitiva::Comparison,  // κ — binding lookup
            LexPrimitiva::Sequence,    // σ — ordered results
            LexPrimitiva::Existence,   // ∃ — antibody existence check
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types — ∂ dominant
// ---------------------------------------------------------------------------

/// AntibodyError: T2-C (∂ · Σ · N · κ), dominant ∂
///
/// Error variants representing boundary violations in the immune system.
/// Boundary-dominant: errors ARE boundary conditions (unrecognized, insufficient).
impl GroundsTo for AntibodyError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — violated constraints
            LexPrimitiva::Sum,        // Σ — error variant alternation
            LexPrimitiva::Quantity,   // N — affinity score/threshold
            LexPrimitiva::Comparison, // κ — threshold comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
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
    fn immunoglobulin_class_is_sum_dominant() {
        let comp = ImmunoglobulinClass::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(ImmunoglobulinClass::tier(), Tier::T2Primitive);
    }

    #[test]
    fn threat_severity_is_comparison_dominant() {
        let comp = ThreatSeverity::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn affinity_score_is_quantity_dominant() {
        let comp = AffinityScore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Boundary));
    }

    #[test]
    fn epitope_is_comparison_dominant() {
        let comp = Epitope::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn paratope_includes_mapping() {
        let comp = Paratope::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn antigen_is_existence_dominant() {
        let comp = Antigen::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert_eq!(Antigen::tier(), Tier::T2Composite);
    }

    #[test]
    fn antibody_is_t3() {
        // 6 primitives = T3 (domain-specific)
        assert_eq!(Antibody::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            Antibody::primitive_composition().dominant,
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn repertoire_is_persistence_dominant() {
        let comp = AntibodyRepertoire::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(AntibodyRepertoire::tier(), Tier::T2Composite);
    }

    #[test]
    fn binding_result_includes_causality() {
        let comp = BindingResult::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Causality));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn neutralization_action_is_causality_dominant() {
        let comp = NeutralizationAction::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn error_is_boundary_dominant() {
        let comp = AntibodyError::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }
}
