// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Signal detection tool implementation.

use crate::params::DetectSignalsParams;
use crate::server::RedditServer;
use nexcore_value_mining::{
    ControversyDetector, EngagementDetector, SentimentDetector, SignalDetector, TrendDetector,
    ViralityDetector,
};
use rmcp::model::{CallToolResult, Content};

/// Detect value signals in subreddit posts.
pub async fn detect(server: &RedditServer, params: DetectSignalsParams) -> CallToolResult {
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

    // Fetch posts
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

    // Get or create baseline
    let baseline = server.get_baseline(&params.subreddit);

    // Run all detectors
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
                detector_results.push(serde_json::json!({
                    "type": signal_type,
                    "count": signals.len(),
                    "signals": signals.iter().map(|s| serde_json::json!({
                        "id": s.id,
                        "score": s.score,
                        "confidence": s.confidence,
                        "strength": format!("{:?}", s.strength)
                    })).collect::<Vec<_>>()
                }));
                all_signals.extend(signals);
            }
            Err(e) => {
                detector_results.push(serde_json::json!({
                    "type": format!("{:?}", detector.signal_type()),
                    "error": e.to_string()
                }));
            }
        }
    }

    // Update baseline with new data
    let mut updated_baseline = baseline.clone();
    updated_baseline.update_from_posts(&posts);
    server.update_baseline(updated_baseline);

    let result = serde_json::json!({
        "success": true,
        "entity": params.entity,
        "subreddit": params.subreddit,
        "posts_analyzed": posts.len(),
        "total_signals": all_signals.len(),
        "detectors": detector_results,
        "summary": {
            "strongest_signal": all_signals.iter()
                .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
                .map(|s| serde_json::json!({
                    "type": format!("{:?}", s.signal_type),
                    "score": s.score,
                    "confidence": s.confidence
                }))
        }
    });

    CallToolResult::success(vec![Content::text(result.to_string())])
}
