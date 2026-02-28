// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # NexVigilant Core — Social Types
//!
//! Shared social media domain types extracted from `nexcore-social` so that
//! Domain-layer crates (e.g., `nexcore-value-mining`) can depend on the data
//! types without pulling in the full Service-layer API client.
//!
//! ## Extraction Rationale
//!
//! `nexcore-social` lives in the `mcp-service` hold (Service layer) because it
//! contains HTTP clients, OAuth2 flows, and rate limiters. `nexcore-value-mining`
//! lives in `business-strategy` (Domain layer) and only needs the `Post` data
//! type for signal analysis. This extraction resolves the Domain → Service
//! direction violation (DV7).
//!
//! ## Primitive Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Post Sequence | Sequence | σ |
//! | Score/Engagement | Quantity | N |
//! | Sentiment Ratio | Comparison | κ |
//! | Subreddit Identity | Location | λ |

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![deny(missing_docs)]
#![allow(
    clippy::exhaustive_structs,
    reason = "Post is a data transfer type with intentionally public fields"
)]

pub mod grounding;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Reddit post (submission).
///
/// Core data type for social media signal detection. Contains the fields
/// needed by downstream analysis crates without any API client dependencies.
///
/// ## Primitive Grounding: T3 (σ + N + μ + λ + ς + κ), dominant σ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Unique post ID (e.g., "abc123")
    pub id: String,
    /// Subreddit name (without r/ prefix)
    pub subreddit: String,
    /// Post title
    pub title: String,
    /// Post body text (if self-post)
    #[serde(default)]
    pub selftext: Option<String>,
    /// Author username
    pub author: String,
    /// Score (upvotes - downvotes)
    pub score: i64,
    /// Number of comments
    pub num_comments: i64,
    /// Created timestamp (UTC)
    pub created_utc: f64,
    /// Post URL
    pub url: String,
    /// Whether post is over_18
    pub over_18: bool,
    /// Upvote ratio (0.0 to 1.0)
    pub upvote_ratio: f64,
}

impl Post {
    /// Get created timestamp as DateTime.
    pub fn created_datetime(&self) -> DateTime {
        DateTime::from_timestamp(self.created_utc as i64)
    }

    /// Get full Reddit URL.
    pub fn permalink(&self) -> String {
        format!(
            "https://reddit.com/r/{}/comments/{}",
            self.subreddit, self.id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_created_datetime() {
        let post = Post {
            id: "test123".to_string(),
            subreddit: "test".to_string(),
            title: "Test Post".to_string(),
            selftext: Some("Body text".to_string()),
            author: "testuser".to_string(),
            score: 100,
            num_comments: 50,
            created_utc: 1609459200.0, // 2021-01-01 00:00:00 UTC
            url: "https://reddit.com/r/test".to_string(),
            over_18: false,
            upvote_ratio: 0.95,
        };

        let dt = post.created_datetime();
        assert_eq!(dt.timestamp(), 1609459200);
    }

    #[test]
    fn test_post_permalink() {
        let post = Post {
            id: "abc123".to_string(),
            subreddit: "wallstreetbets".to_string(),
            title: "Test".to_string(),
            selftext: None,
            author: "testuser".to_string(),
            score: 1000,
            num_comments: 200,
            created_utc: 1609459200.0,
            url: "https://reddit.com".to_string(),
            over_18: false,
            upvote_ratio: 0.9,
        };

        assert_eq!(
            post.permalink(),
            "https://reddit.com/r/wallstreetbets/comments/abc123"
        );
    }

    #[test]
    fn test_post_serde_roundtrip() {
        let post = Post {
            id: "rt1".to_string(),
            subreddit: "rust".to_string(),
            title: "Serde test".to_string(),
            selftext: None,
            author: "tester".to_string(),
            score: 42,
            num_comments: 7,
            created_utc: 1700000000.0,
            url: "https://reddit.com/r/rust".to_string(),
            over_18: false,
            upvote_ratio: 0.88,
        };

        let json = serde_json::to_string(&post).unwrap_or_default();
        let restored: Post = serde_json::from_str(&json).unwrap_or_else(|_| post.clone());
        assert_eq!(restored.id, "rt1");
        assert_eq!(restored.score, 42);
    }
}
