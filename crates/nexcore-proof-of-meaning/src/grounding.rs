//! GroundsTo implementations for proof-of-meaning types.
//!
//! Maps semantic chemistry types to Lex Primitiva T1 primitives.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::chromatography::{Band, Chromatogram, SeparationQuality};
use crate::distillation::{DistillationResult, Fraction, MassBalance};
use crate::element::{Atom, Bond, Compound, ElementClass, Relation, StoichiometryResult};
use crate::pipeline::{ProofStep, ProofTrail, SemanticEquivalenceProof, TransformationMethod};
use crate::registry::{AtomStatus, RegisteredAtom};
use crate::spectrum::{EquivalenceInterpretation, Probe, SpectralDistance, SpectralLine, Spectrum};
use crate::titration::{
    EquivalencePoint, EquivalenceProof, EquivalenceVerdict, TitrationCurve, TitrationPoint,
};

// =============================================================================
// Foundation types (T1 / T2-P)
// =============================================================================

/// Atom: T2-P (∃ + κ) — an existent concept with comparison identity.
/// Dominant: Existence — an atom IS a crystallized meaning.
impl GroundsTo for Atom {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Existence, 0.95)
    }
}

/// ElementClass: T1 (κ) — pure comparison/classification.
impl GroundsTo for ElementClass {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// Relation: T2-P (→ + κ) — causal or structural link between atoms.
impl GroundsTo for Relation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Causality, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Causality, 0.90)
    }
}

/// Bond: T2-C (→ + κ + N + ∃) — a measured causal link.
impl GroundsTo for Bond {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// Compound: T3 (∃ + κ + → + Σ + N + ∂) — a bonded assembly with conservation.
impl GroundsTo for Compound {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// StoichiometryResult: T2-P (N + κ) — quantitative conservation check.
impl GroundsTo for StoichiometryResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// =============================================================================
// Distillation types
// =============================================================================

/// Fraction: T2-C (∃ + N + ν + σ) — a separated atom with order and volatility.
impl GroundsTo for Fraction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// DistillationResult: T3 (σ + Σ + N + ∂ + ∃ + ν) — full separation trail.
impl GroundsTo for DistillationResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Sum,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// MassBalance: T2-P (N + κ) — conservation verification.
impl GroundsTo for MassBalance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.95)
    }
}

// =============================================================================
// Chromatography types
// =============================================================================

/// Band: T2-C (κ + N + μ + ∃) — a classified atom with measured affinity.
impl GroundsTo for Band {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Chromatogram: T3 (σ + κ + μ + N + ∂ + ∃) — full separation result.
impl GroundsTo for Chromatogram {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SeparationQuality: T1 (κ) — quality classification.
impl GroundsTo for SeparationQuality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

// =============================================================================
// Spectroscopy types
// =============================================================================

/// Probe: T2-C (∂ + κ + ∃ + μ) — a boundary test for semantic identity.
impl GroundsTo for Probe {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// SpectralLine: T2-P (N + μ) — a measured response.
impl GroundsTo for SpectralLine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Spectrum: T3 (σ + N + μ + ∃ + ν + κ) — full fingerprint.
impl GroundsTo for Spectrum {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// SpectralDistance: T2-P (N + κ) — measured semantic distance.
impl GroundsTo for SpectralDistance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// EquivalenceInterpretation: T1 (κ) — classification verdict.
impl GroundsTo for EquivalenceInterpretation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// =============================================================================
// Titration types
// =============================================================================

/// TitrationPoint: T2-C (N + κ + → + ∃) — a measured reaction step.
impl GroundsTo for TitrationPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// TitrationCurve: T3 (σ + N + κ + → + ∃ + Σ) — full titration trail.
impl GroundsTo for TitrationCurve {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// EquivalencePoint: T2-C (κ + N + → + ∃) — a proven semantic match.
impl GroundsTo for EquivalencePoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// EquivalenceProof: T3 (κ + N + → + σ + ∃ + Σ + ∂) — formal equivalence proof.
impl GroundsTo for EquivalenceProof {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
            LexPrimitiva::Sum,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// EquivalenceVerdict: T1 (κ) — final classification.
impl GroundsTo for EquivalenceVerdict {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// =============================================================================
// Pipeline types
// =============================================================================

/// ProofStep: T2-C (σ + κ + → + ∂) — one verified transformation.
impl GroundsTo for ProofStep {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// TransformationMethod: T1 (κ) — method classification.
impl GroundsTo for TransformationMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ProofTrail: T3 (σ + κ + → + ∂ + ∃ + π) — auditable transformation chain.
impl GroundsTo for ProofTrail {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

/// SemanticEquivalenceProof: T3 (κ + σ + → + ∂ + ∃ + π + N + Σ) — the apex proof.
impl GroundsTo for SemanticEquivalenceProof {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Sequence,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

// =============================================================================
// Registry types
// =============================================================================

/// RegisteredAtom: T2-C (∃ + κ + π + ∂) — a crystallized, persistent atom.
impl GroundsTo for RegisteredAtom {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Persistence,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.90)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// AtomStatus: T2-P (ς + ∂) — lifecycle state with transitions.
impl GroundsTo for AtomStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::State, 0.90)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}
