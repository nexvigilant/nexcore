// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — Social
//!
//! Social media API clients for economic value signal detection.
//!
//! ## Supported Platforms
//!
//! - Reddit (OAuth2 app authentication)
//!
//! ## Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | API Request | Causality | → |
//! | Response Mapping | Mapping | μ |
//! | Rate Limiting | Boundary + Frequency | ∂ + ν |
//! | Post Sequence | Sequence | σ |
//! | Sentiment Score | Quantity | N |
//! | Timestamp | Sequence + Quantity | σ + N |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

pub mod error;
pub mod grounding;
pub mod ratelimit;
pub mod reddit;

// Re-exports
pub use error::{SocialError, SocialResult};
pub use ratelimit::RateLimiter;
pub use reddit::{Comment, Post, RedditClient, RedditConfig, Subreddit};

/// Prelude for common imports.
pub mod prelude {
    //! Common imports for social media API usage.
    pub use crate::error::{SocialError, SocialResult};
    pub use crate::reddit::{Comment, Post, RedditClient, RedditConfig};
}
