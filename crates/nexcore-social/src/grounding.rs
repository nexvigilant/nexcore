//! # GroundsTo implementations for nexcore-social types
//!
//! Connects social media API types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **→ (Causality)**: API request → response
//! - **μ (Mapping)**: response → typed data
//! - **∂ (Boundary)**: rate limiting

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::SocialError;
use crate::ratelimit::RateLimiter;
use crate::reddit::{Comment, RedditConfig, Subreddit};

// ---------------------------------------------------------------------------
// T2-P: Configuration types
// ---------------------------------------------------------------------------

/// RedditConfig: T2-P (ς + ∂), dominant ς
///
/// OAuth2 configuration state. State-dominant: credential snapshot.
impl GroundsTo for RedditConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- credential state
            LexPrimitiva::Boundary, // ∂ -- auth boundary
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// RateLimiter: T2-C (∂ + ν + N + ς), dominant ∂
///
/// Token bucket rate limiter. Boundary-dominant: enforces request limits.
impl GroundsTo for RateLimiter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- capacity limit
            LexPrimitiva::Frequency, // ν -- refill rate
            LexPrimitiva::Quantity,  // N -- token count
            LexPrimitiva::State,     // ς -- current token state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Domain types
// ---------------------------------------------------------------------------

// Post GroundsTo impl is in nexcore-social-types::grounding (canonical location)

/// Comment: T3 (σ + ρ + N + μ + λ + ς), dominant ρ
///
/// Reddit comment. Recursion-dominant: comments nest in threads.
impl GroundsTo for Comment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- nested thread structure
            LexPrimitiva::Sequence,  // σ -- temporal ordering
            LexPrimitiva::Quantity,  // N -- score
            LexPrimitiva::Mapping,   // μ -- JSON → typed fields
            LexPrimitiva::Location,  // λ -- link_id, parent reference
            LexPrimitiva::State,     // ς -- content snapshot
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
    }
}

/// Subreddit: T2-C (λ + N + ς + Σ), dominant λ
///
/// Subreddit metadata. Location-dominant: community identity.
impl GroundsTo for Subreddit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- display_name identity
            LexPrimitiva::Quantity, // N -- subscriber count
            LexPrimitiva::State,    // ς -- current description
            LexPrimitiva::Sum,      // Σ -- subscriber aggregation
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// SocialError: T2-C (∂ + → + ∅ + ν), dominant ∂
///
/// Social API errors: HTTP failures, auth errors, rate limits.
impl GroundsTo for SocialError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- rate limits, auth boundary
            LexPrimitiva::Causality, // → -- request failures
            LexPrimitiva::Void,      // ∅ -- not found
            LexPrimitiva::Frequency, // ν -- rate limit tracking
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
    fn reddit_config_is_state_dominant() {
        assert_eq!(
            RedditConfig::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn rate_limiter_is_boundary_dominant() {
        assert_eq!(
            RateLimiter::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(RateLimiter::tier(), Tier::T2Composite);
    }

    // post_is_t3 test moved to nexcore-social-types::grounding

    #[test]
    fn comment_is_recursion_dominant() {
        assert_eq!(Comment::dominant_primitive(), Some(LexPrimitiva::Recursion));
    }

    #[test]
    fn subreddit_is_location_dominant() {
        assert_eq!(
            Subreddit::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
    }

    #[test]
    fn social_error_is_boundary_dominant() {
        assert_eq!(
            SocialError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }
}
