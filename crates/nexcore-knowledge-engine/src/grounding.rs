//! GroundsTo implementations for knowledge engine types.
//!
//! Connects knowledge engine types to the Lex Primitiva type system.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    CompendiousScorer, CompileOptions, CompressionResult, ConceptGraph, KnowledgeEngineError,
    KnowledgeFragment, KnowledgePack, KnowledgeStore, QueryEngine, ScoreResult,
    StructuralCompressor,
};

// ---------------------------------------------------------------------------
// Error — ∂ + Σ
// ---------------------------------------------------------------------------

impl GroundsTo for KnowledgeEngineError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Sum])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Scoring — N + κ + μ
// ---------------------------------------------------------------------------

impl GroundsTo for ScoreResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — numeric scores
            LexPrimitiva::Comparison, // κ — score interpretation thresholds
            LexPrimitiva::Mapping,    // μ — text → score mapping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

impl GroundsTo for CompendiousScorer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ — text → score
            LexPrimitiva::Quantity, // N — numeric measurement
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Compression — μ + σ + ∂
// ---------------------------------------------------------------------------

impl GroundsTo for StructuralCompressor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ — text → compressed text
            LexPrimitiva::Sequence, // σ — staged pipeline
            LexPrimitiva::Boundary, // ∂ — dedup threshold
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

impl GroundsTo for CompressionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ — original → compressed
            LexPrimitiva::Quantity, // N — ratio, word counts
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Concept Graph — μ + σ + ρ
// ---------------------------------------------------------------------------

impl GroundsTo for ConceptGraph {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ — concept relationships
            LexPrimitiva::Sequence,  // σ — topological ordering
            LexPrimitiva::Recursion, // ρ — graph traversal
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Knowledge Pack — π + μ + σ + N
// ---------------------------------------------------------------------------

impl GroundsTo for KnowledgePack {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π — immutable stored knowledge
            LexPrimitiva::Mapping,     // μ — concepts mapped to fragments
            LexPrimitiva::Sequence,    // σ — ordered fragments
            LexPrimitiva::Quantity,    // N — statistics
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

impl GroundsTo for KnowledgeFragment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π — stored knowledge unit
            LexPrimitiva::Mapping,     // μ — text → concepts
            LexPrimitiva::Quantity,    // N — score
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Store — π + λ
// ---------------------------------------------------------------------------

impl GroundsTo for KnowledgeStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // π — file storage
            LexPrimitiva::Location,    // λ — file paths
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Compiler — σ + μ + → + ∝
// ---------------------------------------------------------------------------

impl GroundsTo for CompileOptions {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ — pipeline stages
            LexPrimitiva::Mapping,   // μ — raw → compiled
            LexPrimitiva::Causality, // → — compilation produces pack
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Query — μ + κ + N
// ---------------------------------------------------------------------------

impl GroundsTo for QueryEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ — query → results
            LexPrimitiva::Comparison, // κ — relevance ranking
            LexPrimitiva::Quantity,   // N — relevance scores
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
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
    fn knowledge_pack_is_t2c() {
        assert_eq!(KnowledgePack::tier(), Tier::T2Composite);
        assert_eq!(
            KnowledgePack::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn concept_graph_tier() {
        // 3 primitives but Tier classification depends on lex-primitiva rules
        let tier = ConceptGraph::tier();
        assert!(tier == Tier::T2Primitive || tier == Tier::T2Composite);
    }

    #[test]
    fn scorer_is_t2p() {
        assert_eq!(CompendiousScorer::tier(), Tier::T2Primitive);
    }

    #[test]
    fn error_is_t2p() {
        assert_eq!(KnowledgeEngineError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            KnowledgeEngineError::primitive_composition(),
            ScoreResult::primitive_composition(),
            CompendiousScorer::primitive_composition(),
            StructuralCompressor::primitive_composition(),
            CompressionResult::primitive_composition(),
            ConceptGraph::primitive_composition(),
            KnowledgePack::primitive_composition(),
            KnowledgeFragment::primitive_composition(),
            KnowledgeStore::primitive_composition(),
            CompileOptions::primitive_composition(),
            QueryEngine::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
