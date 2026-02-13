// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Engagement signal detection (ROR analog).
//!
//! ## Algorithm
//!
//! ```text
//! EngagementROR = (observed_engagement / total_posts) / (baseline_engagement / baseline_posts)
//! ```
//!
//! ## Primitive Grounding
//!
//! ν + N (frequency + quantity)

use crate::error::MiningResult;
use crate::signals::SignalDetector;
use crate::types::{Baseline, SignalType, ValueSignal};
use chrono::Utc;
use nexcore_social::Post;

/// Engagement signal detector using ROR algorithm.
#[derive(Debug, Clone)]
pub struct EngagementDetector {
    /// ROR threshold for signal (default: 3.0).
    pub ror_threshold: f64,
    /// Minimum sample size (default: 10).
    pub min_samples: usize,
}

impl Default for EngagementDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl EngagementDetector {
    /// Create a new engagement detector with default thresholds.
    pub fn new() -> Self {
        Self {
            ror_threshold: 3.0,
            min_samples: 10,
        }
    }

    /// Calculate engagement score for a post.
    fn post_engagement(&self, post: &Post) -> f64 {
        (post.score.abs() + post.num_comments) as f64
    }
}

impl SignalDetector for EngagementDetector {
    fn detect(
        &self,
        posts: &[Post],
        baseline: &Baseline,
        entity: &str,
    ) -> MiningResult<Vec<ValueSignal>> {
        let relevant: Vec<_> = posts
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

        let total_engagement: f64 = relevant.iter().map(|p| self.post_engagement(p)).sum();
        let avg_engagement = total_engagement / relevant.len() as f64;

        // Calculate ROR
        let baseline_avg = baseline.avg_engagement.max(1.0);
        let ror = avg_engagement / baseline_avg;

        let mut signals = Vec::new();

        if ror >= self.ror_threshold {
            let confidence = ((ror - 1.0) / 5.0).clamp(0.5, 1.0);

            if confidence >= self.confidence_threshold() {
                let window_start = relevant
                    .iter()
                    .map(|p| p.created_datetime())
                    .min()
                    .unwrap_or_else(Utc::now);
                let window_end = relevant
                    .iter()
                    .map(|p| p.created_datetime())
                    .max()
                    .unwrap_or_else(Utc::now);

                let signal = ValueSignal::new(
                    SignalType::Engagement,
                    entity,
                    &baseline.source,
                    ror,
                    confidence,
                    window_start,
                    window_end,
                    relevant.len(),
                )
                .with_metadata(serde_json::json!({
                    "avg_engagement": avg_engagement,
                    "baseline_engagement": baseline_avg,
                    "ror": ror,
                }));

                signals.push(signal);
            }
        }

        Ok(signals)
    }

    fn signal_type(&self) -> SignalType {
        SignalType::Engagement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_detector_creation() {
        let detector = EngagementDetector::new();
        assert_eq!(detector.ror_threshold, 3.0);
        assert_eq!(detector.signal_type(), SignalType::Engagement);
    }
}
