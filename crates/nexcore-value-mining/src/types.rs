// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Core types for value mining.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Type of value signal detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Sentiment signal (PRR analog) - unexpected positive/negative sentiment.
    Sentiment,
    /// Trend signal (IC analog) - directional momentum over time.
    Trend,
    /// Engagement signal (ROR analog) - unusual engagement rate.
    Engagement,
    /// Virality signal (EBGM analog) - exponential growth detection.
    Virality,
    /// Controversy signal (Chi² analog) - high sentiment variance.
    Controversy,
}

impl SignalType {
    /// Get the PV algorithm analog for this signal type.
    pub fn pv_analog(&self) -> &'static str {
        match self {
            Self::Sentiment => "PRR",
            Self::Trend => "IC",
            Self::Engagement => "ROR",
            Self::Virality => "EBGM",
            Self::Controversy => "Chi²",
        }
    }

    /// Get the T1 primitive grounding for this signal type.
    pub fn primitives(&self) -> &'static str {
        match self {
            Self::Sentiment => "N + κ (quantity + comparison)",
            Self::Trend => "σ + → (sequence + causality)",
            Self::Engagement => "ν + N (frequency + quantity)",
            Self::Virality => "∂ + ∝ (boundary + irreversibility)",
            Self::Controversy => "κ + ς (comparison + state)",
        }
    }
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentiment => write!(f, "Sentiment"),
            Self::Trend => write!(f, "Trend"),
            Self::Engagement => write!(f, "Engagement"),
            Self::Virality => write!(f, "Virality"),
            Self::Controversy => write!(f, "Controversy"),
        }
    }
}

/// Strength/confidence of a detected signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
    /// Weak signal (score 0.5-0.7).
    Weak,
    /// Moderate signal (score 0.7-0.85).
    Moderate,
    /// Strong signal (score 0.85-0.95).
    Strong,
    /// Very strong signal (score > 0.95).
    VeryStrong,
}

impl SignalStrength {
    /// Create SignalStrength from confidence score (0.0 to 1.0).
    pub fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.95 {
            Self::VeryStrong
        } else if confidence >= 0.85 {
            Self::Strong
        } else if confidence >= 0.7 {
            Self::Moderate
        } else {
            Self::Weak
        }
    }

    /// Get minimum confidence threshold for this strength.
    pub fn min_confidence(&self) -> f64 {
        match self {
            Self::Weak => 0.5,
            Self::Moderate => 0.7,
            Self::Strong => 0.85,
            Self::VeryStrong => 0.95,
        }
    }
}

impl std::fmt::Display for SignalStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weak => write!(f, "Weak"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Strong => write!(f, "Strong"),
            Self::VeryStrong => write!(f, "VeryStrong"),
        }
    }
}

/// Economic/social value detection with PV algorithm analogs.
///
/// Tier: T3 (N + ν — quantity with frequency)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSignal {
    /// Unique signal ID.
    pub id: String,
    /// Type of signal.
    pub signal_type: SignalType,
    /// Entity this signal relates to (e.g., stock ticker, product name).
    pub entity: String,
    /// Source (e.g., subreddit name).
    pub source: String,
    /// Raw score from detection algorithm.
    pub score: f64,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f64,
    /// Signal strength classification.
    pub strength: SignalStrength,
    /// Start of detection window.
    pub window_start: DateTime,
    /// End of detection window.
    pub window_end: DateTime,
    /// When signal was detected.
    pub detected_at: DateTime,
    /// Number of data points used in detection.
    pub sample_size: usize,
    /// Additional metadata (JSON).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Backward-compatible alias.
#[deprecated(note = "use ValueSignal — F2 equivocation fix")]
pub type Signal = ValueSignal;

impl ValueSignal {
    /// Create a new signal.
    pub fn new(
        signal_type: SignalType,
        entity: impl Into<String>,
        source: impl Into<String>,
        score: f64,
        confidence: f64,
        window_start: DateTime,
        window_end: DateTime,
        sample_size: usize,
    ) -> Self {
        let id = format!(
            "{}-{}-{}",
            signal_type,
            nexcore_chrono::DateTime::now().timestamp_millis(),
            rand_suffix()
        );

        Self {
            id,
            signal_type,
            entity: entity.into(),
            source: source.into(),
            score,
            confidence,
            strength: SignalStrength::from_confidence(confidence),
            window_start,
            window_end,
            detected_at: DateTime::now(),
            sample_size,
            metadata: serde_json::Value::Null,
        }
    }

    /// Add metadata to the signal.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if signal meets minimum confidence threshold.
    pub fn is_actionable(&self) -> bool {
        self.confidence >= 0.7
    }
}

/// Baseline statistics for a source (e.g., subreddit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    /// Source identifier.
    pub source: String,
    /// Average positive sentiment rate (0.0 to 1.0).
    pub positive_rate: f64,
    /// Average negative sentiment rate (0.0 to 1.0).
    pub negative_rate: f64,
    /// Average engagement per post.
    pub avg_engagement: f64,
    /// Average posts per hour.
    pub posts_per_hour: f64,
    /// When baseline was computed.
    pub computed_at: DateTime,
    /// Number of samples used to compute baseline.
    pub sample_count: usize,
}

impl Baseline {
    /// Create a new baseline.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            positive_rate: 0.5,
            negative_rate: 0.5,
            avg_engagement: 100.0,
            posts_per_hour: 10.0,
            computed_at: DateTime::now(),
            sample_count: 0,
        }
    }

    /// Update baseline with new observations.
    pub fn update(
        &mut self,
        positive_rate: f64,
        negative_rate: f64,
        avg_engagement: f64,
        posts_per_hour: f64,
        new_samples: usize,
    ) {
        // Exponential moving average with decay factor
        let alpha = 0.1; // Weight for new observations
        let old_weight = 1.0 - alpha;
        let new_weight = alpha;

        self.positive_rate = self.positive_rate * old_weight + positive_rate * new_weight;
        self.negative_rate = self.negative_rate * old_weight + negative_rate * new_weight;
        self.avg_engagement = self.avg_engagement * old_weight + avg_engagement * new_weight;
        self.posts_per_hour = self.posts_per_hour * old_weight + posts_per_hour * new_weight;
        self.sample_count += new_samples;
        self.computed_at = DateTime::now();
    }

    /// Update baseline from posts data.
    pub fn update_from_posts(&mut self, posts: &[nexcore_social::Post]) {
        if posts.is_empty() {
            return;
        }

        // Calculate statistics from posts
        let total = posts.len() as f64;
        let positive_count = posts.iter().filter(|p| p.upvote_ratio > 0.6).count() as f64;
        let negative_count = posts.iter().filter(|p| p.upvote_ratio < 0.4).count() as f64;

        let positive_rate = positive_count / total;
        let negative_rate = negative_count / total;

        let avg_engagement = posts
            .iter()
            .map(|p| p.score as f64 + p.num_comments as f64)
            .sum::<f64>()
            / total;

        // Estimate posts per hour from time range if possible
        let posts_per_hour = if posts.len() > 1 {
            let min_time = posts
                .iter()
                .map(|p| p.created_utc)
                .fold(f64::INFINITY, f64::min);
            let max_time = posts
                .iter()
                .map(|p| p.created_utc)
                .fold(f64::NEG_INFINITY, f64::max);
            let hours = (max_time - min_time) / 3600.0;
            if hours > 0.0 {
                posts.len() as f64 / hours
            } else {
                self.posts_per_hour
            }
        } else {
            self.posts_per_hour
        };

        self.update(
            positive_rate,
            negative_rate,
            avg_engagement,
            posts_per_hour,
            posts.len(),
        );
    }
}

/// Generate a random suffix for IDs.
fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{:08x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_display() {
        assert_eq!(SignalType::Sentiment.to_string(), "Sentiment");
        assert_eq!(SignalType::Trend.to_string(), "Trend");
        assert_eq!(SignalType::Engagement.to_string(), "Engagement");
        assert_eq!(SignalType::Virality.to_string(), "Virality");
        assert_eq!(SignalType::Controversy.to_string(), "Controversy");
    }

    #[test]
    fn test_signal_type_pv_analog() {
        assert_eq!(SignalType::Sentiment.pv_analog(), "PRR");
        assert_eq!(SignalType::Trend.pv_analog(), "IC");
        assert_eq!(SignalType::Engagement.pv_analog(), "ROR");
        assert_eq!(SignalType::Virality.pv_analog(), "EBGM");
        assert_eq!(SignalType::Controversy.pv_analog(), "Chi²");
    }

    #[test]
    fn test_signal_strength_from_confidence() {
        assert_eq!(SignalStrength::from_confidence(0.4), SignalStrength::Weak);
        assert_eq!(
            SignalStrength::from_confidence(0.75),
            SignalStrength::Moderate
        );
        assert_eq!(SignalStrength::from_confidence(0.9), SignalStrength::Strong);
        assert_eq!(
            SignalStrength::from_confidence(0.98),
            SignalStrength::VeryStrong
        );
    }

    #[test]
    fn test_signal_creation() {
        let signal = ValueSignal::new(
            SignalType::Sentiment,
            "TSLA",
            "wallstreetbets",
            3.5,
            0.85,
            DateTime::now(),
            DateTime::now(),
            100,
        );

        assert_eq!(signal.signal_type, SignalType::Sentiment);
        assert_eq!(signal.entity, "TSLA");
        assert_eq!(signal.source, "wallstreetbets");
        assert_eq!(signal.strength, SignalStrength::Strong);
        assert!(signal.is_actionable());
    }

    #[test]
    fn test_baseline_update() {
        let mut baseline = Baseline::new("wallstreetbets");
        assert_eq!(baseline.positive_rate, 0.5);

        baseline.update(0.8, 0.2, 500.0, 50.0, 100);

        // Should have moved toward new values
        assert!(baseline.positive_rate > 0.5);
        assert!(baseline.positive_rate < 0.8);
        assert_eq!(baseline.sample_count, 100);
    }
}
