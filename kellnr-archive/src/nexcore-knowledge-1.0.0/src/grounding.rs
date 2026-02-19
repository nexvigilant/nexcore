//! # GroundsTo implementations for nexcore-knowledge types
//!
//! Connects KSB (Knowledge-Skills-Behaviors) types to the Lex Primitiva type system.
//!
//! ## Persistence (pi) Focus
//!
//! Knowledge IS persistent structured information. The crate indexes 628 PV
//! articles across 15 domains, making Persistence the dominant primitive
//! for the domain types, with Mapping for search/retrieval operations.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{KnowledgeError, KsbArticle, KsbDomain, KsbIndex, ProficiencyLevel, SearchResult};

// ---------------------------------------------------------------------------
// Classification enums -- Sigma (Sum) dominant
// ---------------------------------------------------------------------------

/// KsbDomain: T1 (Sigma), pure sum
///
/// Fifteen-variant enum classifying PV knowledge domains (D01-D15).
impl GroundsTo for KsbDomain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ProficiencyLevel: T2-P (Sigma + kappa), dominant Sigma
///
/// Ordinal proficiency: Foundation, Intermediate, Advanced, Expert.
/// Sum-dominant: categorical alternation.
/// Comparison is secondary (levels are ordered).
impl GroundsTo for ProficiencyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Sigma -- level variant
            LexPrimitiva::Comparison, // kappa -- ordinal comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Knowledge types -- pi (Persistence) dominant
// ---------------------------------------------------------------------------

/// KsbArticle: T2-C (pi + Sigma + mu + lambda), dominant pi
///
/// A single KSB article with domain, title, content, proficiency.
/// Persistence-dominant: an article IS stored knowledge.
/// Sum is secondary (domain classification).
/// Mapping is tertiary (article maps concepts to explanations).
/// Location is quaternary (file path location).
impl GroundsTo for KsbArticle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- stored knowledge
            LexPrimitiva::Sum,         // Sigma -- domain classification
            LexPrimitiva::Mapping,     // mu -- concept -> explanation
            LexPrimitiva::Location,    // lambda -- file path
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// KsbIndex: T2-C (pi + mu + sigma + N), dominant pi
///
/// Index of all KSB articles with search capabilities.
/// Persistence-dominant: the index IS a persistent knowledge store.
/// Mapping is secondary (ID -> article lookup).
/// Sequence is tertiary (scanning traversal).
/// Quantity is quaternary (article counts).
impl GroundsTo for KsbIndex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- persistent knowledge store
            LexPrimitiva::Mapping,     // mu -- ID -> article lookup
            LexPrimitiva::Sequence,    // sigma -- scan traversal
            LexPrimitiva::Quantity,    // N -- article counts
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.80)
    }
}

/// SearchResult: T2-P (mu + kappa + N), dominant mu
///
/// Result of searching articles: article reference with relevance score.
/// Mapping-dominant: it maps a query to matched articles.
/// Comparison is secondary (relevance ranking).
/// Quantity is tertiary (relevance score).
impl GroundsTo for SearchResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- query -> result mapping
            LexPrimitiva::Comparison, // kappa -- relevance ranking
            LexPrimitiva::Quantity,   // N -- relevance score
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// KnowledgeError: T2-P (partial + Sigma), dominant partial
impl GroundsTo for KnowledgeError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant
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
    fn ksb_domain_is_t1() {
        assert_eq!(KsbDomain::tier(), Tier::T1Universal);
        assert!(KsbDomain::is_pure_primitive());
    }

    #[test]
    fn proficiency_level_is_t2p() {
        assert_eq!(ProficiencyLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn ksb_article_is_t2c() {
        assert_eq!(KsbArticle::tier(), Tier::T2Composite);
        assert_eq!(
            KsbArticle::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn ksb_index_is_t2c() {
        assert_eq!(KsbIndex::tier(), Tier::T2Composite);
        assert_eq!(
            KsbIndex::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
    }

    #[test]
    fn search_result_is_t2p() {
        assert_eq!(SearchResult::tier(), Tier::T2Primitive);
        assert_eq!(
            SearchResult::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            KsbDomain::primitive_composition(),
            ProficiencyLevel::primitive_composition(),
            KsbArticle::primitive_composition(),
            KsbIndex::primitive_composition(),
            SearchResult::primitive_composition(),
            KnowledgeError::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
