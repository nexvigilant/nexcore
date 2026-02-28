// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # GroundsTo implementations for social media types
//!
//! Connects social media domain types to the Lex Primitiva type system.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::Post;

/// Post: T3 (σ + N + μ + λ + ς + κ), dominant σ
///
/// Reddit post (submission). Sequence-dominant: posts form a stream.
impl GroundsTo for Post {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ -- post stream ordering
            LexPrimitiva::Quantity,   // N -- score, num_comments, upvote_ratio
            LexPrimitiva::Mapping,    // μ -- JSON → typed fields
            LexPrimitiva::Location,   // λ -- subreddit, URL identity
            LexPrimitiva::State,      // ς -- point-in-time content
            LexPrimitiva::Comparison, // κ -- upvote_ratio comparison
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn post_is_t3() {
        assert_eq!(Post::tier(), Tier::T3DomainSpecific);
        assert_eq!(Post::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }
}
