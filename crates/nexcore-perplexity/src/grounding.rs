// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! GroundsTo implementations for nexcore-perplexity types.
//!
//! Connects Perplexity API types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **mu (Mapping)**: Query to search-grounded response (dominant)
//! - **sigma (Sequence)**: Message list forming conversation
//! - **boundary (Boundary)**: API key validation, domain filtering, rate limits
//! - **causality (Causality)**: Citations grounding response to sources
//! - **kappa (Comparison)**: Model selection, recency filtering

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::client::PerplexityClient;
use crate::error::PerplexityError;
use crate::types::{ChatRequest, ChatResponse, Model, ResearchResult, SearchRecency, SearchResult};

// ---------------------------------------------------------------------------
// T2-P: Simple wrapper types
// ---------------------------------------------------------------------------

/// Model: T2-P (kappa Comparison)
///
/// Model selection is a comparison operation: choose between Sonar vs SonarPro.
impl GroundsTo for Model {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- model selection
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// SearchRecency: T2-P (nu Frequency)
///
/// Temporal window filter for search results.
impl GroundsTo for SearchRecency {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // nu -- temporal frequency window
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.95)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Composed types
// ---------------------------------------------------------------------------

/// PerplexityClient: T2-C (mu + boundary + state), dominant mu
///
/// The client maps queries to grounded search responses.
/// Boundary from API key validation; state from configured model.
impl GroundsTo for PerplexityClient {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- query to response mapping
            LexPrimitiva::Boundary, // boundary -- API key, rate limiting
            LexPrimitiva::State,    // state -- configured model + key
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// ChatRequest: T2-C (sigma + mu + boundary + kappa), dominant sigma
///
/// Message sequence forms the core structure; domain filters add boundary.
impl GroundsTo for ChatRequest {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- message list
            LexPrimitiva::Mapping,    // mu -- model to completion
            LexPrimitiva::Boundary,   // boundary -- domain filter, max_tokens
            LexPrimitiva::Comparison, // kappa -- model + recency selection
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// PerplexityError: T2-C (boundary + causality + void), dominant boundary
///
/// Most errors are boundary violations: auth, rate limit, API rejection.
impl GroundsTo for PerplexityError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // boundary -- rate limits, auth, validation
            LexPrimitiva::Causality, // causality -- HTTP request failures
            LexPrimitiva::Void,      // void -- missing API key
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// SearchResult: T2-C (causality + lambda + nu), dominant causality
///
/// A search result grounds the response to a specific source URL.
impl GroundsTo for SearchResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // causality -- url grounds the claim
            LexPrimitiva::Location,  // lambda -- URL location
            LexPrimitiva::Frequency, // nu -- date/freshness
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Full domain types
// ---------------------------------------------------------------------------

/// ChatResponse: T3 (mu + causality + sigma + N + existence), dominant mu
///
/// The full response mapping query to grounded answer with citations.
impl GroundsTo for ChatResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- query to answer
            LexPrimitiva::Causality, // causality -- citations
            LexPrimitiva::Sequence,  // sigma -- choices list
            LexPrimitiva::Quantity,  // N -- token usage
            LexPrimitiva::Existence, // existence -- response validity
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// ResearchResult: T3 (mu + causality + sigma + N), dominant mu
///
/// High-level research output combining answer + citations + metadata.
impl GroundsTo for ResearchResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // mu -- query to research answer
            LexPrimitiva::Causality, // causality -- citations
            LexPrimitiva::Sequence,  // sigma -- related questions
            LexPrimitiva::Quantity,  // N -- tokens used
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
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
    fn model_is_t2p_comparison_dominant() {
        assert_eq!(Model::dominant_primitive(), Some(LexPrimitiva::Comparison));
        assert_eq!(Model::tier(), Tier::T1Universal);
    }

    #[test]
    fn search_recency_is_frequency_dominant() {
        assert_eq!(
            SearchRecency::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn client_is_mapping_dominant() {
        assert_eq!(
            PerplexityClient::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        // 3 primitives = T2-P (2-3 unique)
        assert_eq!(PerplexityClient::tier(), Tier::T2Primitive);
    }

    #[test]
    fn chat_request_is_sequence_dominant() {
        assert_eq!(
            ChatRequest::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(ChatRequest::tier(), Tier::T2Composite);
    }

    #[test]
    fn chat_response_is_mapping_dominant() {
        assert_eq!(
            ChatResponse::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        // 5 primitives = T2-C (4-5 unique)
        assert_eq!(ChatResponse::tier(), Tier::T2Composite);
    }

    #[test]
    fn perplexity_error_is_boundary_dominant() {
        assert_eq!(
            PerplexityError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn research_result_is_t3() {
        assert_eq!(
            ResearchResult::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        // 4 primitives = T2-C boundary, but with domain semantics = T3
        let comp = ResearchResult::primitive_composition();
        assert!(comp.primitives.len() >= 4);
    }

    #[test]
    fn search_result_is_causality_dominant() {
        assert_eq!(
            SearchResult::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }
}
