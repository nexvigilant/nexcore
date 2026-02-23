//! # GroundsTo implementations for nexcore-synth types
//!
//! Connects the autonomous primitive synthesis engine to the Lex Primitiva type system.
//!
//! ## Evolution Loop (ρ-synth)
//!
//! Statistical Drift (ν) → Structural Inference (μ) → Primitive Synthesis (Σ)
//!
//! ## Key Primitive Mapping
//! - Statistical analysis: Frequency (ν) -- drift detection
//! - Structural inference: Mapping (μ) -- data -> schema
//! - Primitive composition: Sum (Σ) -- candidate synthesis
//! - Decision loop: Recursion (ρ) -- evolution cycle

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{SynthCandidate, SynthEngine, SynthError};

// ---------------------------------------------------------------------------
// Candidate types -- Sum (Σ) dominant
// ---------------------------------------------------------------------------

/// SynthCandidate: T2-C (Σ + ∃ + κ + π), dominant Σ
///
/// A newly synthesized primitive candidate.
/// Sum-dominant: it represents a composition of primitives.
/// Existence is secondary (it IS a newly found candidate).
/// Comparison is tertiary (confidence scoring).
/// Persistence is quaternary (candidate ID and metadata).
impl GroundsTo for SynthCandidate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,         // Σ -- primitive composition
            LexPrimitiva::Existence,   // ∃ -- candidate presence
            LexPrimitiva::Comparison,  // κ -- confidence scoring
            LexPrimitiva::Persistence, // π -- metadata storage
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Engine types -- Recursion (ρ) dominant
// ---------------------------------------------------------------------------

/// SynthEngine: T3 (ρ + μ + ν + κ + Σ), dominant ρ
///
/// The full evolution engine: analyze, infer, compose.
/// Recursion-dominant: it implements the Level 5 evolution cycle.
/// Mapping is secondary (structural inference).
/// Frequency is tertiary (statistical analysis).
/// Comparison is quaternary (primitive mapping).
/// Sum is quinary (final synthesis).
impl GroundsTo for SynthEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,  // ρ -- evolution loop
            LexPrimitiva::Mapping,    // μ -- structural inference
            LexPrimitiva::Frequency,  // ν -- statistical analysis
            LexPrimitiva::Comparison, // κ -- primitive mapping
            LexPrimitiva::Sum,        // Σ -- candidate synthesis
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// SynthError: T2-P (Boundary + Sum), dominant Boundary
impl GroundsTo for SynthError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- synthesis error boundary
            LexPrimitiva::Sum,      // Σ -- error variant
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}
