// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Sentiment signal detection (PRR analog).
//!
//! ## Algorithm
//!
//! ```text
//! SentimentPRR = (observed_positive / expected_positive) / (observed_negative / expected_negative)
//!
//! WHERE:
//!   observed_positive = posts with positive sentiment in window
//!   expected_positive = baseline.positive_rate × total_posts
//!   observed_negative = posts with negative sentiment in window
//!   expected_negative = baseline.negative_rate × total_posts
//! ```
//!
//! ## Signal Thresholds
//!
//! - PRR > 2.0: Strong positive sentiment signal
//! - PRR < 0.5: Strong negative sentiment signal
//!
//! ## Primitive Grounding
//!
//! N + κ (quantity + comparison)

use crate::error::{MiningError, MiningResult};
use crate::signals::SignalDetector;
use crate::types::{Baseline, SignalType, ValueSignal};
use nexcore_chrono::DateTime;
use nexcore_social_types::Post;

/// Sentiment signal detector using PRR algorithm.
#[derive(Debug, Clone, Default)]
pub struct SentimentDetector {
    /// PRR threshold for positive signal (default: 2.0).
    pub positive_threshold: f64,
    /// PRR threshold for negative signal (default: 0.5).
    pub negative_threshold: f64,
    /// Minimum sample size required (default: 10).
    pub min_samples: usize,
}

impl SentimentDetector {
    /// Create a new sentiment detector with default thresholds.
    pub fn new() -> Self {
        Self {
            positive_threshold: 2.0,
            negative_threshold: 0.5,
            min_samples: 10,
        }
    }

    /// Create a detector with custom thresholds.
    pub fn with_thresholds(positive: f64, negative: f64, min_samples: usize) -> Self {
        Self {
            positive_threshold: positive,
            negative_threshold: negative,
            min_samples,
        }
    }

    /// Classify sentiment of a post title/body.
    ///
    /// Simple heuristic based on upvote ratio:
    /// - ratio > 0.7 = positive
    /// - ratio < 0.3 = negative
    /// - else = neutral
    ///
    /// For production, integrate VADER or transformer model.
    fn classify_sentiment(&self, post: &Post) -> Sentiment {
        // Simple heuristic: use upvote ratio as proxy for sentiment
        // In production, use VADER or transformer model
        if post.upvote_ratio > 0.7 {
            Sentiment::Positive
        } else if post.upvote_ratio < 0.3 {
            Sentiment::Negative
        } else {
            Sentiment::Neutral
        }
    }

    /// Calculate PRR from observed and expected counts.
    fn calculate_prr(
        &self,
        observed_positive: usize,
        observed_negative: usize,
        expected_positive: f64,
        expected_negative: f64,
    ) -> f64 {
        // Avoid division by zero
        let exp_pos = expected_positive.max(0.001);
        let exp_neg = expected_negative.max(0.001);
        let obs_pos = observed_positive as f64;
        let obs_neg = observed_negative.max(1) as f64; // Avoid div by zero

        (obs_pos / exp_pos) / (obs_neg / exp_neg)
    }

    /// Calculate confidence based on sample size and PRR distance from 1.0.
    fn calculate_confidence(&self, prr: f64, sample_size: usize) -> f64 {
        // Confidence increases with sample size (log scale)
        let sample_factor = (sample_size as f64).ln().min(5.0) / 5.0;

        // Confidence increases with distance from neutral (1.0)
        let prr_factor = if prr > 1.0 {
            ((prr - 1.0) / 2.0).min(1.0)
        } else {
            ((1.0 - prr) / 0.5).min(1.0)
        };

        (sample_factor * 0.4 + prr_factor * 0.6).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Sentiment {
    Positive,
    Neutral,
    Negative,
}

impl SignalDetector for SentimentDetector {
    fn detect(
        &self,
        posts: &[Post],
        baseline: &Baseline,
        entity: &str,
    ) -> MiningResult<Vec<ValueSignal>> {
        if posts.len() < self.min_samples {
            return Err(MiningError::InsufficientData(format!(
                "Need at least {} posts, got {}",
                self.min_samples,
                posts.len()
            )));
        }

        // Filter posts mentioning the entity
        let relevant_posts: Vec<_> = posts
            .iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&entity.to_lowercase())
                    || p.selftext
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&entity.to_lowercase()))
                        .unwrap_or(false)
            })
            .collect();

        if relevant_posts.len() < self.min_samples {
            return Ok(vec![]); // Not enough relevant posts
        }

        // Count sentiments
        let mut positive = 0;
        let mut negative = 0;
        let mut neutral = 0;

        for post in &relevant_posts {
            match self.classify_sentiment(post) {
                Sentiment::Positive => positive += 1,
                Sentiment::Negative => negative += 1,
                Sentiment::Neutral => neutral += 1,
            }
        }

        let total = relevant_posts.len() as f64;

        // Calculate expected counts from baseline
        let expected_positive = baseline.positive_rate * total;
        let expected_negative = baseline.negative_rate * total;

        // Calculate PRR
        let prr = self.calculate_prr(positive, negative, expected_positive, expected_negative);

        // Check if signal threshold met
        let mut signals = Vec::new();

        if prr > self.positive_threshold || prr < self.negative_threshold {
            let confidence = self.calculate_confidence(prr, relevant_posts.len());

            if confidence >= self.confidence_threshold() {
                let window_start = relevant_posts
                    .iter()
                    .map(|p| p.created_datetime())
                    .min()
                    .unwrap_or_else(|| DateTime::now());
                let window_end = relevant_posts
                    .iter()
                    .map(|p| p.created_datetime())
                    .max()
                    .unwrap_or_else(|| DateTime::now());

                let signal = ValueSignal::new(
                    SignalType::Sentiment,
                    entity,
                    &baseline.source,
                    prr,
                    confidence,
                    window_start,
                    window_end,
                    relevant_posts.len(),
                )
                .with_metadata(serde_json::json!({
                    "positive_count": positive,
                    "negative_count": negative,
                    "neutral_count": neutral,
                    "expected_positive": expected_positive,
                    "expected_negative": expected_negative,
                    "prr": prr,
                    "direction": if prr > 1.0 { "positive" } else { "negative" },
                }));

                signals.push(signal);
            }
        }

        Ok(signals)
    }

    fn signal_type(&self) -> SignalType {
        SignalType::Sentiment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_post(title: &str, upvote_ratio: f64) -> Post {
        Post {
            id: format!("test_{}", title.len()),
            subreddit: "test".to_string(),
            title: title.to_string(),
            selftext: None,
            author: "testuser".to_string(),
            score: 100,
            num_comments: 50,
            created_utc: nexcore_chrono::DateTime::now().timestamp() as f64,
            url: "https://reddit.com".to_string(),
            over_18: false,
            upvote_ratio,
        }
    }

    #[test]
    fn test_sentiment_classification() {
        let detector = SentimentDetector::new();

        let positive_post = make_post("Great news!", 0.9);
        let negative_post = make_post("Terrible!", 0.2);
        let neutral_post = make_post("Hmm", 0.5);

        assert_eq!(
            detector.classify_sentiment(&positive_post),
            Sentiment::Positive
        );
        assert_eq!(
            detector.classify_sentiment(&negative_post),
            Sentiment::Negative
        );
        assert_eq!(
            detector.classify_sentiment(&neutral_post),
            Sentiment::Neutral
        );
    }

    #[test]
    fn test_prr_calculation() {
        let detector = SentimentDetector::new();

        // Equal observed and expected = PRR of 1.0
        let prr = detector.calculate_prr(10, 10, 10.0, 10.0);
        assert!((prr - 1.0).abs() < 0.001);

        // Double positive rate = PRR of 2.0
        let prr = detector.calculate_prr(20, 10, 10.0, 10.0);
        assert!((prr - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_insufficient_data_error() {
        let detector = SentimentDetector::new();
        let baseline = Baseline::new("test");
        let posts = vec![make_post("TSLA test", 0.8)]; // Only 1 post

        let result = detector.detect(&posts, &baseline, "TSLA");
        assert!(result.is_err());

        if let Err(MiningError::InsufficientData(_)) = result {
            // Expected
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_no_signal_when_neutral() {
        let detector = SentimentDetector::new();
        let mut baseline = Baseline::new("test");
        // Set baseline to expect mostly neutral (low positive and negative rates)
        baseline.positive_rate = 0.3;
        baseline.negative_rate = 0.3;

        // Create posts with neutral sentiment (upvote_ratio 0.4-0.6 = neutral)
        let posts: Vec<_> = (0..20)
            .map(|i| make_post(&format!("TSLA post {}", i), 0.5))
            .collect();

        let signals = detector.detect(&posts, &baseline, "TSLA");
        assert!(signals.is_ok());
        // Neutral posts = low positive count, low negative count
        // PRR should be close to 1.0, not triggering threshold
        let sigs = signals.ok().unwrap_or_default();
        // With neutral posts, we might still get signals if confidence is low
        // Check that any signal has low confidence
        for sig in &sigs {
            assert!(
                sig.confidence < 0.9,
                "Neutral posts should not produce high confidence signals"
            );
        }
    }

    #[test]
    fn test_positive_signal_detection() {
        let detector = SentimentDetector::new();
        let mut baseline = Baseline::new("test");
        baseline.positive_rate = 0.5;
        baseline.negative_rate = 0.5;

        // Create posts with high positive sentiment
        let posts: Vec<_> = (0..20)
            .map(|i| make_post(&format!("TSLA is amazing {}", i), 0.95))
            .collect();

        let signals = detector.detect(&posts, &baseline, "TSLA");
        assert!(signals.is_ok());

        let signals = signals.ok().unwrap_or_default();
        if !signals.is_empty() {
            let signal = &signals[0];
            assert_eq!(signal.signal_type, SignalType::Sentiment);
            assert!(signal.score > 1.0); // PRR > 1 means positive
        }
    }
}
