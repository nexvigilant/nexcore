// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Controversy signal detection (Chi² analog).
//!
//! ## Algorithm
//!
//! ```text
//! Controversy = sentiment_variance / mean_engagement
//!
//! WHERE:
//!   sentiment_variance = Σ(sentiment[i] - mean_sentiment)² / n
//! ```
//!
//! ## Primitive Grounding
//!
//! κ + ς (comparison + state)

use crate::error::MiningResult;
use crate::signals::SignalDetector;
use crate::types::{Baseline, SignalType, ValueSignal};
use nexcore_chrono::DateTime;
use nexcore_social_types::Post;

/// Controversy signal detector (high variance / polarization).
#[derive(Debug, Clone)]
pub struct ControversyDetector {
    /// Controversy threshold (default: 0.5).
    pub controversy_threshold: f64,
    /// Minimum sample size (default: 10).
    pub min_samples: usize,
}

impl Default for ControversyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ControversyDetector {
    /// Create a new controversy detector with default thresholds.
    pub fn new() -> Self {
        Self {
            controversy_threshold: 0.5,
            min_samples: 10,
        }
    }

    /// Calculate variance of upvote ratios.
    fn calculate_variance(&self, posts: &[&Post]) -> f64 {
        if posts.len() < 2 {
            return 0.0;
        }

        let mean: f64 = posts.iter().map(|p| p.upvote_ratio).sum::<f64>() / posts.len() as f64;

        let variance: f64 = posts
            .iter()
            .map(|p| (p.upvote_ratio - mean).powi(2))
            .sum::<f64>()
            / (posts.len() - 1) as f64;

        variance
    }

    /// Check if distribution is bimodal (clustered at extremes).
    fn is_bimodal(&self, posts: &[&Post]) -> bool {
        let high_count = posts.iter().filter(|p| p.upvote_ratio > 0.7).count();
        let low_count = posts.iter().filter(|p| p.upvote_ratio < 0.3).count();
        let mid_count = posts.len() - high_count - low_count;

        // Bimodal if extreme ends together exceed middle
        high_count + low_count > mid_count && high_count > 0 && low_count > 0
    }
}

impl SignalDetector for ControversyDetector {
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

        let variance = self.calculate_variance(&relevant);
        let is_bimodal = self.is_bimodal(&relevant);

        // Calculate controversy index
        let mean_engagement: f64 = relevant
            .iter()
            .map(|p| (p.score.abs() + p.num_comments) as f64)
            .sum::<f64>()
            / relevant.len() as f64;

        let controversy_index = variance / mean_engagement.max(1.0) * 100.0;

        let mut signals = Vec::new();

        if controversy_index >= self.controversy_threshold && is_bimodal {
            let confidence = (controversy_index / 2.0).clamp(0.5, 1.0);

            if confidence >= self.confidence_threshold() {
                let window_start = relevant
                    .iter()
                    .map(|p| p.created_datetime())
                    .min()
                    .unwrap_or_else(|| DateTime::now());
                let window_end = relevant
                    .iter()
                    .map(|p| p.created_datetime())
                    .max()
                    .unwrap_or_else(|| DateTime::now());

                let signal = ValueSignal::new(
                    SignalType::Controversy,
                    entity,
                    &baseline.source,
                    controversy_index,
                    confidence,
                    window_start,
                    window_end,
                    relevant.len(),
                )
                .with_metadata(serde_json::json!({
                    "variance": variance,
                    "controversy_index": controversy_index,
                    "is_bimodal": is_bimodal,
                    "mean_engagement": mean_engagement,
                }));

                signals.push(signal);
            }
        }

        Ok(signals)
    }

    fn signal_type(&self) -> SignalType {
        SignalType::Controversy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controversy_detector_creation() {
        let detector = ControversyDetector::new();
        assert_eq!(detector.controversy_threshold, 0.5);
        assert_eq!(detector.signal_type(), SignalType::Controversy);
    }
}
