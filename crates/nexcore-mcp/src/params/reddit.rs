//! Reddit Parameters (Social Listening & Signal Detection)
//!
//! Fetching posts, subreddit info, searching, and detecting signals on Reddit.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for reddit_hot_posts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditHotPostsParams {
    /// Subreddit name.
    pub subreddit: String,
    /// Max posts to fetch.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_new_posts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditNewPostsParams {
    /// Subreddit name.
    pub subreddit: String,
    /// Max posts to fetch.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_subreddit_info.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditSubredditInfoParams {
    /// Subreddit name.
    pub subreddit: String,
}

/// Parameters for reddit_detect_signals.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditDetectSignalsParams {
    /// Subreddit name.
    pub subreddit: String,
    /// Entity name to detect signals for.
    pub entity: String,
    /// Max posts to analyze.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_search_entity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditSearchEntityParams {
    /// Subreddit name.
    pub subreddit: String,
    /// Search query string.
    pub query: String,
    /// Max posts to search.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

fn default_reddit_limit() -> u32 {
    25
}
