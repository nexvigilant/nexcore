// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit API types.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

// Re-export Post from nexcore-social-types (canonical definition).
pub use nexcore_social_types::Post;

/// Reddit API configuration.
#[derive(Debug, Clone)]
pub struct RedditConfig {
    /// OAuth2 client ID from https://www.reddit.com/prefs/apps
    pub client_id: String,
    /// OAuth2 client secret
    pub client_secret: String,
    /// Reddit username
    pub username: String,
    /// Reddit password
    pub password: String,
    /// User-Agent header (required by Reddit API)
    pub user_agent: String,
}

impl RedditConfig {
    /// Create new Reddit configuration.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            username: username.into(),
            password: password.into(),
            user_agent: user_agent.into(),
        }
    }
}

/// Reddit comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: String,
    /// Parent post ID
    pub link_id: String,
    /// Author username
    pub author: String,
    /// Comment body text
    pub body: String,
    /// Score
    pub score: i64,
    /// Created timestamp (UTC)
    pub created_utc: f64,
}

impl Comment {
    /// Get created timestamp as DateTime.
    pub fn created_datetime(&self) -> DateTime {
        DateTime::from_timestamp(self.created_utc as i64)
    }
}

/// Subreddit metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subreddit {
    /// Subreddit name (without r/ prefix)
    pub display_name: String,
    /// Subscriber count
    pub subscribers: i64,
    /// Description
    pub public_description: String,
}

/// Reddit API OAuth2 response.
#[derive(Debug, Deserialize)]
pub(crate) struct OAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Reddit API listing response wrapper.
#[derive(Debug, Deserialize)]
pub(crate) struct ListingResponse<T> {
    pub data: ListingData<T>,
}

/// Reddit API listing data.
#[derive(Debug, Deserialize)]
pub(crate) struct ListingData<T> {
    pub children: Vec<ListingChild<T>>,
}

/// Reddit API listing child wrapper.
#[derive(Debug, Deserialize)]
pub(crate) struct ListingChild<T> {
    pub data: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config =
            RedditConfig::new("client123", "secret456", "user789", "pass", "nexcore:0.1.0");

        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret456");
        assert_eq!(config.username, "user789");
    }

    #[test]
    fn test_post_reexport_accessible() {
        // Verify Post is accessible via re-export
        let post = Post {
            id: "re1".to_string(),
            subreddit: "test".to_string(),
            title: "Re-export test".to_string(),
            selftext: None,
            author: "tester".to_string(),
            score: 1,
            num_comments: 0,
            created_utc: 1700000000.0,
            url: "https://reddit.com".to_string(),
            over_18: false,
            upvote_ratio: 0.5,
        };
        assert_eq!(post.id, "re1");
    }
}
