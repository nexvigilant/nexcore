// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Posts tool implementations.

use crate::params::{HotPostsParams, NewPostsParams};
use crate::server::RedditServer;
use nexcore_social::Post;
use rmcp::model::{CallToolResult, Content};

/// Get hot posts from a subreddit.
pub async fn hot_posts(server: &RedditServer, params: HotPostsParams) -> CallToolResult {
    let client = match server.get_client().await {
        Ok(c) => c,
        Err(e) => {
            return CallToolResult::success(vec![Content::text(
                serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get client: {}", e)
                })
                .to_string(),
            )]);
        }
    };

    let limit = params.limit.min(100);

    match client.get_hot_posts(&params.subreddit, limit).await {
        Ok(posts) => {
            let result = serde_json::json!({
                "success": true,
                "subreddit": params.subreddit,
                "count": posts.len(),
                "posts": posts.iter().map(format_post).collect::<Vec<_>>()
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch posts: {}", e)
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
    }
}

/// Get new posts from a subreddit.
pub async fn new_posts(server: &RedditServer, params: NewPostsParams) -> CallToolResult {
    let client = match server.get_client().await {
        Ok(c) => c,
        Err(e) => {
            return CallToolResult::success(vec![Content::text(
                serde_json::json!({
                    "success": false,
                    "error": format!("Failed to get client: {}", e)
                })
                .to_string(),
            )]);
        }
    };

    let limit = params.limit.min(100);

    match client.get_new_posts(&params.subreddit, limit).await {
        Ok(posts) => {
            let result = serde_json::json!({
                "success": true,
                "subreddit": params.subreddit,
                "count": posts.len(),
                "posts": posts.iter().map(format_post).collect::<Vec<_>>()
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch posts: {}", e)
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
    }
}

/// Format a post for JSON output.
fn format_post(post: &Post) -> serde_json::Value {
    serde_json::json!({
        "id": post.id,
        "title": post.title,
        "author": post.author,
        "score": post.score,
        "num_comments": post.num_comments,
        "upvote_ratio": post.upvote_ratio,
        "created_utc": post.created_utc,
        "url": post.url,
        "selftext": post.selftext.as_ref().map(|s| truncate(s, 500)),
        "permalink": post.permalink()
    })
}

/// Truncate string to max length.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
