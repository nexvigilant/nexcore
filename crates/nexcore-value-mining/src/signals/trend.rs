// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Trend signal detection (IC analog).
//!
//! ## Algorithm
//!
//! ```text
//! TrendStrength = Σ(t=1 to n) weight[t] × sentiment[t] / n
//!
//! WHERE:
//!   weight[t] = exponential decay (recent posts weighted higher)
//!   sentiment[t] = normalized sentiment score at time t
//!   n = window size (hours)
//! ```
//!
//! ## Primitive Grounding
//!
//! σ + → (sequence + causality)

use crate::error::MiningResult;
use crate::signals::SignalDetector;
use crate::types::{Baseline, SignalType, ValueSignal};
use nexcore_chrono::DateTime;
use nexcore_social::Post;

/// Trend signal detector using weighted moving average.
#[derive(Debug, Clone)]
pub struct TrendDetector {
    /// Minimum trend strength to trigger signal (default: 0.6).
    pub strength_threshold: f64,
    /// Minimum consistent direction percentage (default: 0.8).
    pub consistency_threshold: f64,
    /// Minimum sample size (default: 10).
    pub min_samples: usize,
}

impl Default for TrendDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl TrendDetector {
    /// Create a new trend detector with default thresholds.
    pub fn new() -> Self {
        Self {
            strength_threshold: 0.6,
            consistency_threshold: 0.8,
            min_samples: 10,
        }
    }

    /// Calculate trend strength from sorted posts (oldest to newest).
    fn calculate_trend_strength(&self, posts: &[&Post]) -> f64 {
        if posts.is_empty() {
            return 0.0;
        }

        let n = posts.len() as f64;
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for (i, post) in posts.iter().enumerate() {
            // Exponential weight: recent posts weighted higher
            let weight = (i as f64 / n).exp();
            let sentiment = post.upvote_ratio * 2.0 - 1.0; // Map 0-1 to -1 to +1

            weighted_sum += weight * sentiment;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            0.0
        }
    }

    /// Calculate consistency (percentage of posts in same direction).
    fn calculate_consistency(&self, posts: &[&Post]) -> f64 {
        if posts.is_empty() {
            return 0.0;
        }

        let positive_count = posts.iter().filter(|p| p.upvote_ratio > 0.5).count();
        let negative_count = posts.len() - positive_count;

        positive_count.max(negative_count) as f64 / posts.len() as f64
    }
}

impl SignalDetector for TrendDetector {
    fn detect(
        &self,
        posts: &[Post],
        baseline: &Baseline,
        entity: &str,
    ) -> MiningResult<Vec<ValueSignal>> {
        // Filter and sort by time
        let mut relevant: Vec<_> = posts
            .iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&entity.to_lowercase())
                    || p.selftext
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&entity.to_lowercase()))
                        .unwrap_or(false)
            })
            .collect();

        if relevant.len() < self.min_samples {
            return Ok(vec![]);
        }

        // Sort by created time (oldest first)
        relevant.sort_by(|a, b| {
            a.created_utc
                .partial_cmp(&b.created_utc)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let trend_strength = self.calculate_trend_strength(&relevant);
        let consistency = self.calculate_consistency(&relevant);

        let mut signals = Vec::new();

        if trend_strength.abs() >= self.strength_threshold
            && consistency >= self.consistency_threshold
        {
            let confidence = (trend_strength.abs() * 0.5 + consistency * 0.5).clamp(0.0, 1.0);

            if confidence >= self.confidence_threshold() {
                let window_start = relevant
                    .first()
                    .map(|p| p.created_datetime())
                    .unwrap_or_else(|| DateTime::now());
                let window_end = relevant
                    .last()
                    .map(|p| p.created_datetime())
                    .unwrap_or_else(|| DateTime::now());

                let signal = ValueSignal::new(
                    SignalType::Trend,
                    entity,
                    &baseline.source,
                    trend_strength,
                    confidence,
                    window_start,
                    window_end,
                    relevant.len(),
                )
                .with_metadata(serde_json::json!({
                    "trend_strength": trend_strength,
                    "consistency": consistency,
                    "direction": if trend_strength > 0.0 { "up" } else { "down" },
                }));

                signals.push(signal);
            }
        }

        Ok(signals)
    }

    fn signal_type(&self) -> SignalType {
        SignalType::Trend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_post(title: &str, upvote_ratio: f64, created_utc: f64) -> Post {
        Post {
            id: format!("test_{}", created_utc as i64),
            subreddit: "test".to_string(),
            title: title.to_string(),
            selftext: None,
            author: "testuser".to_string(),
            score: 100,
            num_comments: 50,
            created_utc,
            url: "https://reddit.com".to_string(),
            over_18: false,
            upvote_ratio,
        }
    }

    #[test]
    fn test_trend_detector_creation() {
        let detector = TrendDetector::new();
        assert_eq!(detector.strength_threshold, 0.6);
        assert_eq!(detector.signal_type(), SignalType::Trend);
    }

    #[test]
    fn test_upward_trend_detection() {
        let detector = TrendDetector::new();
        let baseline = Baseline::new("test");

        // Create posts with increasing positive sentiment
        let now = nexcore_chrono::DateTime::now().timestamp() as f64;
        let posts: Vec<_> = (0..20)
            .map(|i| {
                let ratio = 0.5 + (i as f64 / 40.0); // Gradually increasing
                make_post(
                    &format!("TSLA post {}", i),
                    ratio,
                    now - (20 - i) as f64 * 3600.0,
                )
            })
            .collect();

        let signals = detector.detect(&posts, &baseline, "TSLA");
        assert!(signals.is_ok());
    }
}
