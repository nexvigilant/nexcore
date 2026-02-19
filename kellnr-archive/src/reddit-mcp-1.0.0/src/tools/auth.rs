// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Authentication tool implementation.

use crate::server::RedditServer;
use rmcp::model::{CallToolResult, Content};

/// Authenticate with Reddit API.
pub async fn authenticate(server: &RedditServer) -> CallToolResult {
    if !server.is_configured() {
        return CallToolResult::success(vec![Content::text(serde_json::json!({
            "success": false,
            "error": "Reddit credentials not configured",
            "help": "Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD"
        }).to_string())]);
    }

    match server.get_client().await {
        Ok(_) => {
            let result = serde_json::json!({
                "success": true,
                "message": "Successfully authenticated with Reddit API",
                "rate_limit_available": server.rate_limit_available()
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
        Err(e) => {
            let result = serde_json::json!({
                "success": false,
                "error": format!("Authentication failed: {}", e)
            });
            CallToolResult::success(vec![Content::text(result.to_string())])
        }
    }
}
