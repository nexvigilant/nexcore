// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit API types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// Reddit post (submission).
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
    pub fn created_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_utc as i64, 0).unwrap_or_else(|| DateTime::UNIX_EPOCH)
    }

    /// Get full Reddit URL.
    pub fn permalink(&self) -> String {
        format!(
            "https://reddit.com/r/{}/comments/{}",
            self.subreddit, self.id
        )
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
    pub fn created_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_utc as i64, 0).unwrap_or_else(|| DateTime::UNIX_EPOCH)
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
    fn test_config_creation() {
        let config =
            RedditConfig::new("client123", "secret456", "user789", "pass", "nexcore:0.1.0");

        assert_eq!(config.client_id, "client123");
        assert_eq!(config.client_secret, "secret456");
        assert_eq!(config.username, "user789");
    }
}
