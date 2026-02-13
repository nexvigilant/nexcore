// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Subreddit info tool implementation.

use crate::params::SubredditInfoParams;
use crate::server::RedditServer;
use rmcp::model::{CallToolResult, Content};

/// Get subreddit information.
pub async fn info(server: &RedditServer, params: SubredditInfoParams) -> CallToolResult {
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

    match client.get_subreddit(&params.subreddit).await {
        Ok(subreddit) => {
            let result = serde_json::json!({
                "success": true,
                "subreddit": {
                    "name": subreddit.display_name,
                    "description": subreddit.public_description,
                    "subscribers": subreddit.subscribers,
                    "url": format!("https://reddit.com/r/{}", subreddit.display_name)
                }
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch subreddit: {}", e)
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
    }
}
