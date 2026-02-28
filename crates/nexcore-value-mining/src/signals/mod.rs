// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Signal detection algorithms.
//!
//! Each detector implements the `SignalDetector` trait and maps to a
//! pharmacovigilance algorithm (PRR, ROR, IC, EBGM, Chi²).

mod controversy;
mod engagement;
mod sentiment;
mod trend;
mod virality;

pub use controversy::ControversyDetector;
pub use engagement::EngagementDetector;
pub use sentiment::SentimentDetector;
pub use trend::TrendDetector;
pub use virality::ViralityDetector;

use crate::error::MiningResult;
use crate::types::{Baseline, ValueSignal};
use nexcore_social_types::Post;

/// Trait for signal detection algorithms.
///
/// Each implementation applies a specific detection algorithm to social media data.
pub trait SignalDetector: Send + Sync {
    /// Detect signals from a batch of posts.
    ///
    /// # Arguments
    ///
    /// * `posts` - Posts to analyze
    /// * `baseline` - Baseline statistics for comparison
    /// * `entity` - Entity to detect signals for (e.g., "TSLA", "Bitcoin")
    ///
    /// # Returns
    ///
    /// Vector of detected signals (may be empty if no signals found).
    fn detect(
        &self,
        posts: &[Post],
        baseline: &Baseline,
        entity: &str,
    ) -> MiningResult<Vec<ValueSignal>>;

    /// Get the signal type this detector produces.
    fn signal_type(&self) -> crate::types::SignalType;

    /// Get the confidence threshold for this detector.
    fn confidence_threshold(&self) -> f64 {
        0.7 // Default: moderate confidence
    }
}
