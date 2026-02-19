// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! MCP tool parameter types.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for reddit_hot_posts tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HotPostsParams {
    /// Subreddit name (without r/ prefix).
    pub subreddit: String,
    /// Number of posts to fetch (max 100, default 25).
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Parameters for reddit_new_posts tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NewPostsParams {
    /// Subreddit name (without r/ prefix).
    pub subreddit: String,
    /// Number of posts to fetch (max 100, default 25).
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Parameters for reddit_subreddit_info tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SubredditInfoParams {
    /// Subreddit name (without r/ prefix).
    pub subreddit: String,
}

/// Parameters for reddit_detect_signals tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DetectSignalsParams {
    /// Subreddit to scan.
    pub subreddit: String,
    /// Entity to detect signals for (e.g., "TSLA", "Bitcoin").
    pub entity: String,
    /// Number of posts to analyze (default 50).
    #[serde(default = "default_signal_limit")]
    pub limit: u32,
    /// Signal types to detect (default: all).
    /// Options: "sentiment", "trend", "engagement", "virality", "controversy"
    #[serde(default)]
    pub signal_types: Vec<String>,
}

/// Parameters for reddit_search_entity tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SearchEntityParams {
    /// Subreddit to search.
    pub subreddit: String,
    /// Entity/keyword to search for.
    pub query: String,
    /// Number of posts to fetch (default 25).
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Empty parameters for tools that don't need input.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EmptyParams {}

fn default_limit() -> u32 {
    25
}

fn default_signal_limit() -> u32 {
    50
}
