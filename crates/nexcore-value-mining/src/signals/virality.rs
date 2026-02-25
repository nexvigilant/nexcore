// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Virality signal detection (EBGM analog).
//!
//! ## Algorithm
//!
//! ```text
//! Virality = growth_rate > threshold AND acceleration > 0
//!
//! WHERE:
//!   growth_rate = (engagement[t] - engagement[t-1]) / engagement[t-1]
//!   acceleration = growth_rate[t] - growth_rate[t-1]
//! ```
//!
//! ## Primitive Grounding
//!
//! ∂ + ∝ (boundary + irreversibility)

use crate::error::MiningResult;
use crate::signals::SignalDetector;
use crate::types::{Baseline, SignalType, ValueSignal};
use nexcore_chrono::DateTime;
use nexcore_social::Post;

/// Virality signal detector (exponential growth detection).
#[derive(Debug, Clone)]
pub struct ViralityDetector {
    /// Growth rate threshold (default: 2.0 = 200%).
    pub growth_threshold: f64,
    /// Minimum time windows with sustained growth (default: 3).
    pub sustained_windows: usize,
    /// Minimum sample size (default: 10).
    pub min_samples: usize,
}

impl Default for ViralityDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ViralityDetector {
    /// Create a new virality detector with default thresholds.
    pub fn new() -> Self {
        Self {
            growth_threshold: 2.0,
            sustained_windows: 3,
            min_samples: 10,
        }
    }
}

impl SignalDetector for ViralityDetector {
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

        // Sort by time
        let mut sorted = relevant.clone();
        sorted.sort_by(|a, b| {
            a.created_utc
                .partial_cmp(&b.created_utc)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate growth rates between time windows
        // For simplicity, split into chunks and compare engagement
        let chunk_size = sorted.len() / 4;
        if chunk_size < 2 {
            return Ok(vec![]);
        }

        let mut growth_rates = Vec::new();
        for i in 0..4 {
            let start = i * chunk_size;
            let end = ((i + 1) * chunk_size).min(sorted.len());
            let chunk_engagement: f64 = sorted[start..end]
                .iter()
                .map(|p| (p.score.abs() + p.num_comments) as f64)
                .sum();
            growth_rates.push(chunk_engagement);
        }

        // Check for exponential growth pattern
        let mut is_viral = true;
        let mut total_growth = 1.0;

        for i in 1..growth_rates.len() {
            let prev = growth_rates[i - 1].max(1.0);
            let curr = growth_rates[i];
            let rate = curr / prev;
            total_growth *= rate;

            if rate < 1.1 {
                // Must be growing at least 10% per window
                is_viral = false;
                break;
            }
        }

        let mut signals = Vec::new();

        if is_viral && total_growth >= self.growth_threshold {
            let confidence = (total_growth / 10.0).clamp(0.6, 1.0);

            if confidence >= self.confidence_threshold() {
                let window_start = sorted
                    .first()
                    .map(|p| p.created_datetime())
                    .unwrap_or_else(|| DateTime::now());
                let window_end = sorted
                    .last()
                    .map(|p| p.created_datetime())
                    .unwrap_or_else(|| DateTime::now());

                let signal = ValueSignal::new(
                    SignalType::Virality,
                    entity,
                    &baseline.source,
                    total_growth,
                    confidence,
                    window_start,
                    window_end,
                    relevant.len(),
                )
                .with_metadata(serde_json::json!({
                    "total_growth": total_growth,
                    "growth_rates": growth_rates,
                }));

                signals.push(signal);
            }
        }

        Ok(signals)
    }

    fn signal_type(&self) -> SignalType {
        SignalType::Virality
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virality_detector_creation() {
        let detector = ViralityDetector::new();
        assert_eq!(detector.growth_threshold, 2.0);
        assert_eq!(detector.signal_type(), SignalType::Virality);
    }
}
