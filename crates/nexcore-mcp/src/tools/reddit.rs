//! Reddit tools — API integration with value signal detection.
//!
//! Consolidated from `reddit-mcp` satellite server.
//! 7 tools: status, authenticate, hot_posts, new_posts, subreddit_info, detect_signals, search_entity.
//!
//! Uses OnceLock-based lazy state for Reddit client and baselines.
//!
//! Tier: T3 (μ Mapping + σ Sequence + ∂ Boundary + ς State + ν Frequency)

use std::collections::HashMap;
use std::sync::OnceLock;

use nexcore_social::{Post, RedditClient, RedditConfig};
use nexcore_value_mining::{
    Baseline, ControversyDetector, EngagementDetector, SentimentDetector, SignalDetector,
    TrendDetector, ViralityDetector,
};
use parking_lot::RwLock as SyncRwLock;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::params::{
    RedditDetectSignalsParams, RedditHotPostsParams, RedditNewPostsParams,
    RedditSearchEntityParams, RedditSubredditInfoParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};

// ============================================================================
// Lazy state
// ============================================================================

struct RedditState {
    config: Option<RedditConfig>,
    baselines: HashMap<String, Baseline>,
}

static STATE: OnceLock<SyncRwLock<RedditState>> = OnceLock::new();

fn state() -> &'static SyncRwLock<RedditState> {
    STATE.get_or_init(|| {
        let config = load_config_from_env();
        if config.is_none() {
            warn!("Reddit credentials not configured. Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD");
        }
        SyncRwLock::new(RedditState {
            config,
            baselines: HashMap::new(),
        })
    })
}

fn load_config_from_env() -> Option<RedditConfig> {
    let client_id = std::env::var("REDDIT_CLIENT_ID").ok()?;
    let client_secret = std::env::var("REDDIT_CLIENT_SECRET").ok()?;
    let username = std::env::var("REDDIT_USERNAME").ok()?;
    let password = std::env::var("REDDIT_PASSWORD").ok()?;
    Some(RedditConfig::new(
        client_id,
        client_secret,
        username,
        password,
        "nexcore-mcp-reddit:1.0.0 (by /u/nexvigilant)",
    ))
}

async fn get_client() -> Result<RedditClient, McpError> {
    let config = {
        let guard = state().read();
        guard.config.clone()
    };
    let config = config
        .ok_or_else(|| McpError::new(ErrorCode(500), "Reddit credentials not configured", None))?;
    let mut client = RedditClient::new(config)
        .map_err(|e| McpError::new(ErrorCode(500), format!("Reddit client: {e}"), None))?;
    client
        .authenticate()
        .await
        .map_err(|e| McpError::new(ErrorCode(500), format!("Reddit auth: {e}"), None))?;
    Ok(client)
}

fn is_configured() -> bool {
    state().read().config.is_some()
}

fn get_baseline(subreddit: &str) -> Baseline {
    let guard = state().read();
    guard
        .baselines
        .get(subreddit)
        .cloned()
        .unwrap_or_else(|| Baseline::new(subreddit))
}

fn update_baseline(baseline: Baseline) {
    let mut guard = state().write();
    guard.baselines.insert(baseline.source.clone(), baseline);
}

// ============================================================================
// Tool implementations
// ============================================================================

/// Check Reddit API status and rate limits.
pub fn reddit_status() -> Result<CallToolResult, McpError> {
    let configured = is_configured();
    let result = json!({
        "configured": configured,
        "status": if configured { "ready" } else { "credentials_missing" },
        "message": if configured {
            "Reddit API configured. Use reddit_authenticate to connect."
        } else {
            "Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD environment variables."
        },
        "tools": [
            "reddit_status", "reddit_authenticate", "reddit_hot_posts",
            "reddit_new_posts", "reddit_subreddit_info", "reddit_detect_signals",
            "reddit_search_entity"
        ]
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Authenticate with Reddit API.
pub async fn reddit_authenticate() -> Result<CallToolResult, McpError> {
    if !is_configured() {
        let result = json!({
            "success": false,
            "error": "Reddit credentials not configured",
            "help": "Set REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD"
        });
        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }
    match get_client().await {
        Ok(_) => {
            let result = json!({
                "success": true,
                "message": "Successfully authenticated with Reddit API"
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Authentication failed: {}", e.message)
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Get hot/trending posts from a subreddit.
pub async fn reddit_hot_posts(params: RedditHotPostsParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let limit = params.limit.min(100);
    match client.get_hot_posts(&params.subreddit, limit).await {
        Ok(posts) => {
            let result = json!({
                "success": true,
                "subreddit": params.subreddit,
                "count": posts.len(),
                "posts": posts.iter().map(format_post).collect::<Vec<_>>()
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to fetch posts: {e}")
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Get new/recent posts from a subreddit.
pub async fn reddit_new_posts(params: RedditNewPostsParams) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let limit = params.limit.min(100);
    match client.get_new_posts(&params.subreddit, limit).await {
        Ok(posts) => {
            let result = json!({
                "success": true,
                "subreddit": params.subreddit,
                "count": posts.len(),
                "posts": posts.iter().map(format_post).collect::<Vec<_>>()
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to fetch posts: {e}")
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Get subreddit metadata.
pub async fn reddit_subreddit_info(
    params: RedditSubredditInfoParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    match client.get_subreddit(&params.subreddit).await {
        Ok(sub) => {
            let result = json!({
                "success": true,
                "subreddit": {
                    "name": sub.display_name,
                    "description": sub.public_description,
                    "subscribers": sub.subscribers,
                    "url": format!("https://reddit.com/r/{}", sub.display_name)
                }
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to fetch subreddit: {e}")
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Detect value signals in subreddit posts.
pub async fn reddit_detect_signals(
    params: RedditDetectSignalsParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let limit = params.limit.min(100);
    let posts = match client.get_hot_posts(&params.subreddit, limit).await {
        Ok(p) => p,
        Err(e) => {
            let result =
                json!({ "success": false, "error": format!("Failed to fetch posts: {e}") });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }
    };

    let baseline = get_baseline(&params.subreddit);

    let detectors: Vec<Box<dyn SignalDetector>> = vec![
        Box::new(SentimentDetector::new()),
        Box::new(TrendDetector::new()),
        Box::new(EngagementDetector::new()),
        Box::new(ViralityDetector::new()),
        Box::new(ControversyDetector::new()),
    ];

    let mut all_signals = Vec::new();
    let mut detector_results = Vec::new();

    for detector in &detectors {
        match detector.detect(&posts, &baseline, &params.entity) {
            Ok(signals) => {
                let signal_type = format!("{:?}", detector.signal_type());
                detector_results.push(json!({
                    "type": signal_type,
                    "count": signals.len(),
                    "signals": signals.iter().map(|s| json!({
                        "id": s.id,
                        "score": s.score,
                        "confidence": s.confidence,
                        "strength": format!("{:?}", s.strength)
                    })).collect::<Vec<_>>()
                }));
                all_signals.extend(signals);
            }
            Err(e) => {
                detector_results.push(json!({
                    "type": format!("{:?}", detector.signal_type()),
                    "error": e.to_string()
                }));
            }
        }
    }

    let mut updated_baseline = baseline.clone();
    updated_baseline.update_from_posts(&posts);
    update_baseline(updated_baseline);

    let result = json!({
        "success": true,
        "entity": params.entity,
        "subreddit": params.subreddit,
        "posts_analyzed": posts.len(),
        "total_signals": all_signals.len(),
        "detectors": detector_results,
        "summary": {
            "strongest_signal": all_signals.iter()
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
                .map(|s| json!({
                    "type": format!("{:?}", s.signal_type),
                    "score": s.score,
                    "confidence": s.confidence
                }))
        }
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Search for posts mentioning a specific entity.
pub async fn reddit_search_entity(
    params: RedditSearchEntityParams,
) -> Result<CallToolResult, McpError> {
    let client = get_client().await?;
    let limit = params.limit.min(100);
    let posts = match client.get_hot_posts(&params.subreddit, limit).await {
        Ok(p) => p,
        Err(e) => {
            let result =
                json!({ "success": false, "error": format!("Failed to fetch posts: {e}") });
            return Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]));
        }
    };

    let entity_lower = params.query.to_lowercase();
    let matching_posts: Vec<&Post> = posts
        .iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&entity_lower)
                || p.selftext
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&entity_lower))
                    .unwrap_or(false)
        })
        .collect();

    let result = json!({
        "success": true,
        "entity": params.query,
        "subreddit": params.subreddit,
        "total_searched": posts.len(),
        "matches_found": matching_posts.len(),
        "posts": matching_posts.iter().map(|p| json!({
            "id": p.id,
            "title": p.title,
            "author": p.author,
            "score": p.score,
            "num_comments": p.num_comments,
            "upvote_ratio": p.upvote_ratio,
            "created_utc": p.created_utc,
            "url": p.url,
            "selftext_preview": p.selftext.as_ref().map(|s| truncate(s, 200)),
            "permalink": p.permalink()
        })).collect::<Vec<_>>()
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Helpers
// ============================================================================

fn format_post(post: &Post) -> serde_json::Value {
    json!({
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
