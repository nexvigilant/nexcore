// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit API client implementation.

use crate::error::{SocialError, SocialResult};
use crate::ratelimit::RateLimiter;
use crate::reddit::types::{
    Comment, ListingResponse, OAuthResponse, Post, RedditConfig, Subreddit,
};
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use tracing::{debug, warn};

/// Reddit API client with rate limiting.
///
/// Rate limit: 60 requests per minute (Reddit free tier).
pub struct RedditClient {
    config: RedditConfig,
    client: Client,
    access_token: Option<String>,
    rate_limiter: Arc<RateLimiter>,
}

impl RedditClient {
    /// Reddit API rate limit: 60 requests per minute.
    const REQUESTS_PER_MINUTE: u64 = 60;

    /// Create new Reddit client with automatic rate limiting.
    ///
    /// Rate limit is enforced at 60 requests/minute per Reddit API requirements.
    pub fn new(config: RedditConfig) -> SocialResult<Self> {
        let client = Client::builder().user_agent(&config.user_agent).build()?;

        Ok(Self {
            config,
            client,
            access_token: None,
            rate_limiter: Arc::new(RateLimiter::new(Self::REQUESTS_PER_MINUTE)),
        })
    }

    /// Get available rate limit tokens.
    pub fn rate_limit_available(&self) -> u64 {
        self.rate_limiter.available()
    }

    /// Wait for rate limit if necessary, then proceed.
    async fn wait_for_rate_limit(&self) {
        self.rate_limiter.acquire().await;
    }

    /// Authenticate with Reddit OAuth2.
    ///
    /// Must be called before making API requests.
    pub async fn authenticate(&mut self) -> SocialResult<()> {
        debug!("Authenticating with Reddit OAuth2");

        let response = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(&self.config.client_id, Some(&self.config.client_secret))
            .form(&[
                ("grant_type", "password"),
                ("username", &self.config.username),
                ("password", &self.config.password),
            ])
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SocialError::AuthError(format!(
                "OAuth2 failed: {}",
                error_text
            )));
        }

        let oauth: OAuthResponse = response.json().await?;
        self.access_token = Some(oauth.access_token);

        debug!("Successfully authenticated with Reddit");
        Ok(())
    }

    /// Ensure client is authenticated.
    fn ensure_authenticated(&self) -> SocialResult<&str> {
        self.access_token.as_deref().ok_or_else(|| {
            SocialError::AuthError("Not authenticated - call authenticate() first".to_string())
        })
    }

    /// Get hot posts from a subreddit.
    ///
    /// # Arguments
    ///
    /// * `subreddit` - Subreddit name (without r/ prefix)
    /// * `limit` - Number of posts to fetch (max 100)
    ///
    /// # Rate Limiting
    ///
    /// This method automatically waits for rate limit tokens before making requests.
    pub async fn get_hot_posts(&self, subreddit: &str, limit: u32) -> SocialResult<Vec<Post>> {
        let token = self.ensure_authenticated()?;
        let limit = limit.min(100);

        // Wait for rate limit token
        self.wait_for_rate_limit().await;
        debug!("Fetching {} hot posts from r/{}", limit, subreddit);

        let url = format!(
            "https://oauth.reddit.com/r/{}/hot?limit={}",
            subreddit, limit
        );

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(SocialError::RateLimitError(
                "Reddit API rate limit exceeded".to_string(),
            ));
        }

        if response.status() == StatusCode::NOT_FOUND {
            return Err(SocialError::NotFound(format!(
                "Subreddit r/{} not found",
                subreddit
            )));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SocialError::ApiError(error_text));
        }

        let listing: ListingResponse<Post> = response.json().await?;
        let posts: Vec<Post> = listing.data.children.into_iter().map(|c| c.data).collect();

        debug!("Successfully fetched {} posts", posts.len());
        Ok(posts)
    }

    /// Get new posts from a subreddit.
    ///
    /// # Arguments
    ///
    /// * `subreddit` - Subreddit name (without r/ prefix)
    /// * `limit` - Number of posts to fetch (max 100)
    ///
    /// # Rate Limiting
    ///
    /// This method automatically waits for rate limit tokens before making requests.
    pub async fn get_new_posts(&self, subreddit: &str, limit: u32) -> SocialResult<Vec<Post>> {
        let token = self.ensure_authenticated()?;
        let limit = limit.min(100);

        // Wait for rate limit token
        self.wait_for_rate_limit().await;
        debug!("Fetching {} new posts from r/{}", limit, subreddit);

        let url = format!(
            "https://oauth.reddit.com/r/{}/new?limit={}",
            subreddit, limit
        );

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(SocialError::RateLimitError(
                "Reddit API rate limit exceeded".to_string(),
            ));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SocialError::ApiError(error_text));
        }

        let listing: ListingResponse<Post> = response.json().await?;
        let posts: Vec<Post> = listing.data.children.into_iter().map(|c| c.data).collect();

        debug!("Successfully fetched {} new posts", posts.len());
        Ok(posts)
    }

    /// Get subreddit metadata.
    ///
    /// # Rate Limiting
    ///
    /// This method automatically waits for rate limit tokens before making requests.
    pub async fn get_subreddit(&self, subreddit: &str) -> SocialResult<Subreddit> {
        let token = self.ensure_authenticated()?;

        // Wait for rate limit token
        self.wait_for_rate_limit().await;
        debug!("Fetching metadata for r/{}", subreddit);

        let url = format!("https://oauth.reddit.com/r/{}/about", subreddit);

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(SocialError::NotFound(format!(
                "Subreddit r/{} not found",
                subreddit
            )));
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SocialError::ApiError(error_text));
        }

        #[derive(serde::Deserialize)]
        struct SubredditWrapper {
            data: Subreddit,
        }

        let wrapper: SubredditWrapper = response.json().await?;
        Ok(wrapper.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = RedditConfig::new(
            "test_client",
            "test_secret",
            "test_user",
            "test_pass",
            "nexcore-test:0.1.0",
        );

        let client = RedditClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_has_rate_limiter() {
        let config = RedditConfig::new(
            "test_client",
            "test_secret",
            "test_user",
            "test_pass",
            "nexcore-test:0.1.0",
        );

        let client = RedditClient::new(config)
            .ok()
            .unwrap_or_else(|| panic!("Failed to create client"));

        // Should have 60 tokens available (Reddit API limit)
        assert_eq!(client.rate_limit_available(), 60);
    }

    #[tokio::test]
    async fn test_unauthenticated_request_fails() {
        let config = RedditConfig::new(
            "test_client",
            "test_secret",
            "test_user",
            "test_pass",
            "nexcore-test:0.1.0",
        );

        let client = RedditClient::new(config)
            .ok()
            .unwrap_or_else(|| panic!("Failed to create client"));

        let result = client.get_hot_posts("wallstreetbets", 10).await;
        assert!(result.is_err());

        if let Err(SocialError::AuthError(_)) = result {
            // Expected
        } else {
            panic!("Expected AuthError");
        }
    }
}
