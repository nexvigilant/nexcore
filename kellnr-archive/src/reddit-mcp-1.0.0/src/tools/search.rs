// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Entity search tool implementation.

use crate::params::SearchEntityParams;
use crate::server::RedditServer;
use nexcore_social::Post;
use rmcp::model::{CallToolResult, Content};

/// Search for posts mentioning a specific entity.
pub async fn search_entity(server: &RedditServer, params: SearchEntityParams) -> CallToolResult {
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

    // Fetch posts from subreddit
    let limit = params.limit.min(100);
    let posts = match client.get_hot_posts(&params.subreddit, limit).await {
        Ok(p) => p,
        Err(e) => {
            return CallToolResult::success(vec![Content::text(
                serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch posts: {}", e)
                })
                .to_string(),
            )]);
        }
    };

    // Filter posts mentioning the entity
    let entity_lower = params.query.to_lowercase();
    let matching_posts: Vec<&Post> = posts
        .iter()
        .filter(|p| {
            let title_match = p.title.to_lowercase().contains(&entity_lower);
            let text_match = p
                .selftext
                .as_ref()
                .map(|t| t.to_lowercase().contains(&entity_lower))
                .unwrap_or(false);
            title_match || text_match
        })
        .collect();

    let result = serde_json::json!({
        "success": true,
        "entity": params.query,
        "subreddit": params.subreddit,
        "total_searched": posts.len(),
        "matches_found": matching_posts.len(),
        "posts": matching_posts.iter().map(|p| serde_json::json!({
            "id": p.id,
            "title": p.title,
            "author": p.author,
            "score": p.score,
            "num_comments": p.num_comments,
            "upvote_ratio": p.upvote_ratio,
            "created_utc": p.created_utc,
            "url": p.url,
            "selftext_preview": p.selftext.as_ref().map(|s| truncate(s, 200)),
            "permalink": p.permalink(),
            "match_locations": find_match_locations(p, &entity_lower)
        })).collect::<Vec<_>>()
    });

    CallToolResult::success(vec![Content::text(result.to_string())])
}

/// Find where entity matches occur in post.
fn find_match_locations(post: &Post, entity: &str) -> Vec<String> {
    let mut locations = Vec::new();

    if post.title.to_lowercase().contains(entity) {
        locations.push("title".to_string());
    }

    if let Some(ref text) = post.selftext {
        if text.to_lowercase().contains(entity) {
            locations.push("selftext".to_string());
        }
    }

    locations
}

/// Truncate string to max length.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
