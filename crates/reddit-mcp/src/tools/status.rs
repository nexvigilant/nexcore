// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Status tool implementation.

use crate::server::RedditServer;
use rmcp::model::{CallToolResult, Content};

/// Check Reddit API status.
pub async fn status(server: &RedditServer) -> CallToolResult {
    let configured = server.is_configured();
    let rate_limit = server.rate_limit_available();

    let result = serde_json::json!({
        "configured": configured,
        "rate_limit_available": rate_limit,
        "rate_limit_max": 60,
        "status": if configured { "ready" } else { "credentials_missing" },
        "message": if configured {
            "Reddit API configured. Use reddit_authenticate to connect."
        } else {
            "Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD environment variables."
        },
        "tools": [
            "reddit_status",
            "reddit_authenticate",
            "reddit_hot_posts",
            "reddit_new_posts",
            "reddit_subreddit_info",
            "reddit_detect_signals",
            "reddit_search_entity"
        ]
    });

    CallToolResult::success(vec![Content::text(result.to_string())])
}
