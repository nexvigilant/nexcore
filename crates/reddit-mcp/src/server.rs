// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Reddit MCP Server implementation.

use crate::params::*;
use crate::tools;
use nexcore_error::{Result, nexerror};
use nexcore_social::{RedditClient, RedditConfig};
use nexcore_value_mining::Baseline;
use parking_lot::RwLock;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ServerInfo};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::{info, warn};

/// Reddit MCP Server state.
#[derive(Clone)]
pub struct RedditServer {
    /// Reddit API client (initialized on first use).
    client: Arc<RwLock<Option<RedditClient>>>,
    /// Baseline statistics per subreddit.
    baselines: Arc<RwLock<HashMap<String, Baseline>>>,
    /// Reddit config from environment.
    config: Option<RedditConfig>,
    /// Tool router
    tool_router: ToolRouter<Self>,
}

impl RedditServer {
    /// Create a new Reddit MCP server.
    pub fn new() -> Result<Self> {
        // Try to load config from environment
        let config = Self::load_config_from_env();

        if config.is_none() {
            warn!(
                "Reddit credentials not found in environment. Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD"
            );
        }

        Ok(Self {
            client: Arc::new(RwLock::new(None)),
            baselines: Arc::new(RwLock::new(HashMap::new())),
            config,
            tool_router: Self::tool_router(),
        })
    }

    /// Load Reddit config from environment variables.
    fn load_config_from_env() -> Option<RedditConfig> {
        let client_id = env::var("REDDIT_CLIENT_ID").ok()?;
        let client_secret = env::var("REDDIT_CLIENT_SECRET").ok()?;
        let username = env::var("REDDIT_USERNAME").ok()?;
        let password = env::var("REDDIT_PASSWORD").ok()?;

        Some(RedditConfig::new(
            client_id,
            client_secret,
            username,
            password,
            "reddit-mcp:1.0.0 (by /u/nexvigilant)",
        ))
    }

    /// Get or create authenticated Reddit client.
    pub async fn get_client(&self) -> Result<RedditClient> {
        // Need to authenticate (always create fresh due to token expiry)
        let config = self
            .config
            .clone()
            .ok_or_else(|| nexerror!("Reddit credentials not configured"))?;

        let mut client = RedditClient::new(config)
            .map_err(|e| nexcore_error::NexError::msg(e.to_string()))?;
        client
            .authenticate()
            .await
            .map_err(|e| nexcore_error::NexError::msg(e.to_string()))?;

        info!("Authenticated with Reddit API");

        // Store for future use (track state)
        {
            let mut guard = self.client.write();
            if let Some(cfg) = self.config.clone() {
                if let Ok(c) = RedditClient::new(cfg) {
                    *guard = Some(c);
                }
            }
        }

        Ok(client)
    }

    /// Get or create baseline for a subreddit.
    pub fn get_baseline(&self, subreddit: &str) -> Baseline {
        let guard = self.baselines.read();
        guard
            .get(subreddit)
            .cloned()
            .unwrap_or_else(|| Baseline::new(subreddit))
    }

    /// Update baseline for a subreddit.
    pub fn update_baseline(&self, baseline: Baseline) {
        let mut guard = self.baselines.write();
        guard.insert(baseline.source.clone(), baseline);
    }

    /// Check if credentials are configured.
    pub fn is_configured(&self) -> bool {
        self.config.is_some()
    }

    /// Get rate limit status.
    pub fn rate_limit_available(&self) -> u64 {
        let guard = self.client.read();
        guard
            .as_ref()
            .map(|c| c.rate_limit_available())
            .unwrap_or(60)
    }
}

#[tool_router]
impl RedditServer {
    /// Check Reddit API configuration status and rate limits.
    #[tool(description = "Check Reddit API status, configuration, and rate limits")]
    async fn reddit_status(&self) -> Result<CallToolResult, McpError> {
        Ok(tools::status::status(self).await)
    }

    /// Authenticate with Reddit API (required before other operations).
    #[tool(description = "Authenticate with Reddit API using configured credentials")]
    async fn reddit_authenticate(&self) -> Result<CallToolResult, McpError> {
        Ok(tools::auth::authenticate(self).await)
    }

    /// Get hot (trending) posts from a subreddit.
    #[tool(description = "Get hot/trending posts from a subreddit")]
    async fn reddit_hot_posts(
        &self,
        Parameters(params): Parameters<HotPostsParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tools::posts::hot_posts(self, params).await)
    }

    /// Get new (recent) posts from a subreddit.
    #[tool(description = "Get new/recent posts from a subreddit")]
    async fn reddit_new_posts(
        &self,
        Parameters(params): Parameters<NewPostsParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tools::posts::new_posts(self, params).await)
    }

    /// Get subreddit information and statistics.
    #[tool(description = "Get subreddit metadata including subscriber count and description")]
    async fn reddit_subreddit_info(
        &self,
        Parameters(params): Parameters<SubredditInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tools::subreddit::info(self, params).await)
    }

    /// Detect value signals in subreddit posts for a specific entity.
    #[tool(
        description = "Detect economic value signals (sentiment, trend, engagement, virality, controversy) in subreddit posts"
    )]
    async fn reddit_detect_signals(
        &self,
        Parameters(params): Parameters<DetectSignalsParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tools::signals::detect(self, params).await)
    }

    /// Search for posts mentioning a specific entity/keyword.
    #[tool(description = "Search for posts mentioning a specific entity or keyword")]
    async fn reddit_search_entity(
        &self,
        Parameters(params): Parameters<SearchEntityParams>,
    ) -> Result<CallToolResult, McpError> {
        Ok(tools::search::search_entity(self, params).await)
    }
}

impl ServerHandler for RedditServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                r#"Reddit MCP Server - Reddit API integration with value signal detection.

## Tools (7)

### Status & Auth
- reddit_status: Check API status and rate limits
- reddit_authenticate: Authenticate with Reddit API

### Posts
- reddit_hot_posts: Get hot/trending posts from a subreddit
- reddit_new_posts: Get new/recent posts from a subreddit
- reddit_subreddit_info: Get subreddit metadata

### Value Mining
- reddit_detect_signals: Detect economic value signals in posts
- reddit_search_entity: Search for posts mentioning an entity"#
                    .to_string(),
            ),
            ..Default::default()
        }
    }
}
