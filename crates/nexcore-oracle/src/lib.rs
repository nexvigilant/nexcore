//! # nexcore-oracle — Bayesian Event Prediction Engine
//!
//! Learns transition probabilities from event sequences, predicts what
//! happens next, and self-tracks accuracy over time.
//!
//! ## Architecture
//!
//! ```text
//! Events --> TransitionMatrix --> Predictor --> Prediction
//!   (sigma)   (arrow+N+mu+nu)     (full)      (arrow+N+kappa)
//!                                    |
//!                          AccuracyTracker <-- actual outcome
//!                           (nu+N+kappa+pi)
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_oracle::prelude::*;
//!
//! let mut oracle = Oracle::new();
//!
//! // Teach the oracle a pattern
//! let mut seq = EventSequence::new();
//! seq.push_kind("read");
//! seq.push_kind("edit");
//! seq.push_kind("build");
//! seq.push_kind("test");
//! oracle.ingest(&seq);
//!
//! // Predict what comes after "edit"
//! let pred = oracle.predict("edit");
//! assert_eq!(pred.event, "build");
//!
//! // Record what actually happened (self-improving)
//! oracle.observe("edit", "build");
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![warn(missing_docs)]

pub mod accuracy;
pub mod event;
pub mod grounding;
pub mod matrix;
pub mod predictor;

pub use accuracy::{AccuracyReport, AccuracyTracker, PredictionOutcome};
pub use event::{Event, EventKind, EventSequence};
pub use matrix::TransitionMatrix;
pub use predictor::{Prediction, Predictor};

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::Oracle;
    pub use crate::accuracy::{AccuracyReport, AccuracyTracker};
    pub use crate::event::{Event, EventSequence};
    pub use crate::matrix::TransitionMatrix;
    pub use crate::predictor::{Prediction, Predictor};
}

use serde::{Deserialize, Serialize};

/// The unified Oracle — prediction engine with self-tracking accuracy.
///
/// Combines the `Predictor` (Markov chain learner) with an `AccuracyTracker`
/// (self-monitoring performance). The Oracle learns from event sequences,
/// predicts next events, and tracks whether its predictions are correct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oracle {
    predictor: Predictor,
    accuracy: AccuracyTracker,
    /// The last event observed (for context in next prediction).
    last_event: Option<String>,
    /// The second-to-last event (for second-order context).
    prev_event: Option<String>,
}

impl Oracle {
    /// Create a new Oracle with default settings.
    pub fn new() -> Self {
        Self {
            predictor: Predictor::new(),
            accuracy: AccuracyTracker::new(100),
            last_event: None,
            prev_event: None,
        }
    }

    /// Create an Oracle with custom smoothing and accuracy window.
    pub fn with_config(alpha: f64, accuracy_window: usize) -> Self {
        Self {
            predictor: Predictor::with_alpha(alpha),
            accuracy: AccuracyTracker::new(accuracy_window),
            last_event: None,
            prev_event: None,
        }
    }

    /// Ingest an event sequence to learn from.
    pub fn ingest(&mut self, sequence: &EventSequence) {
        self.predictor.ingest(sequence);
        let kinds: Vec<&str> = sequence.kinds().collect();
        if let Some(&last) = kinds.last() {
            self.prev_event = self.last_event.take();
            self.last_event = Some(last.to_string());
        }
    }

    /// Predict the next event given the current state.
    pub fn predict(&self, current: &str) -> Prediction {
        self.predictor.predict(current, self.prev_event.as_deref())
    }

    /// Predict with explicit second-order context.
    pub fn predict_with_context(&self, current: &str, previous: &str) -> Prediction {
        self.predictor.predict(current, Some(previous))
    }

    /// Record what actually happened after a prediction.
    /// This feeds the accuracy tracker for self-improvement.
    pub fn observe(&mut self, predicted_from: &str, actual_next: &str) {
        let pred = self.predict(predicted_from);
        self.accuracy
            .record(&pred.event, actual_next, pred.confidence);
        self.prev_event = self.last_event.take();
        self.last_event = Some(actual_next.to_string());
    }

    /// Get the accuracy report.
    pub fn accuracy_report(&self) -> AccuracyReport {
        self.accuracy.report()
    }

    /// Is the Oracle well-calibrated?
    pub fn is_calibrated(&self) -> bool {
        self.accuracy.is_calibrated()
    }

    /// Overall predictability score (0 = chaotic, 1 = deterministic).
    pub fn predictability(&self) -> f64 {
        self.predictor.predictability()
    }

    /// Total events ingested.
    pub fn total_events(&self) -> u64 {
        self.predictor.total_events()
    }

    /// Number of unique event kinds.
    pub fn vocabulary_size(&self) -> usize {
        self.predictor.vocabulary_size()
    }

    /// Get the underlying predictor for advanced usage.
    pub fn predictor(&self) -> &Predictor {
        &self.predictor
    }

    /// Get the underlying accuracy tracker.
    pub fn accuracy_tracker(&self) -> &AccuracyTracker {
        &self.accuracy
    }
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seq(kinds: &[&str]) -> EventSequence {
        let mut seq = EventSequence::new();
        for k in kinds {
            seq.push_kind(*k);
        }
        seq
    }

    #[test]
    fn oracle_basic_workflow() {
        let mut oracle = Oracle::new();
        for _ in 0..20 {
            oracle.ingest(&make_seq(&["read", "edit", "build", "test"]));
        }
        let pred = oracle.predict("edit");
        assert_eq!(pred.event, "build");
        assert!(pred.confidence > 0.0);
        oracle.observe("edit", "build");
        assert_eq!(oracle.accuracy_report().lifetime_count, 1);
    }

    #[test]
    fn oracle_second_order() {
        let mut oracle = Oracle::new();
        for _ in 0..20 {
            oracle.ingest(&make_seq(&["read", "edit", "build"]));
            oracle.ingest(&make_seq(&["grep", "edit", "test"]));
        }
        assert_eq!(oracle.predict_with_context("edit", "read").event, "build");
        assert_eq!(oracle.predict_with_context("edit", "grep").event, "test");
    }

    #[test]
    fn oracle_accuracy_tracking() {
        let mut oracle = Oracle::new();
        for _ in 0..50 {
            oracle.ingest(&make_seq(&["a", "b", "c"]));
        }
        for _ in 0..10 {
            oracle.observe("a", "b");
        }
        let report = oracle.accuracy_report();
        assert_eq!(report.lifetime_count, 10);
        assert!((report.lifetime_accuracy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn oracle_predictability() {
        let mut oracle = Oracle::new();
        for _ in 0..50 {
            oracle.ingest(&make_seq(&["a", "b"]));
        }
        assert!(oracle.predictability() > 0.8);
    }

    #[test]
    fn oracle_vocabulary() {
        let mut oracle = Oracle::new();
        oracle.ingest(&make_seq(&["read", "edit", "build", "test", "deploy"]));
        assert_eq!(oracle.vocabulary_size(), 5);
    }

    #[test]
    fn oracle_empty() {
        let oracle = Oracle::new();
        let pred = oracle.predict("nothing");
        assert!(pred.event.is_empty());
        assert_eq!(oracle.total_events(), 0);
    }

    #[test]
    fn oracle_serde_roundtrip() {
        let mut oracle = Oracle::new();
        oracle.ingest(&make_seq(&["x", "y", "z"]));
        let json = serde_json::to_string(&oracle).unwrap_or_default();
        assert!(!json.is_empty());
        let restored: Oracle = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(restored.total_events(), 3);
    }

    #[test]
    fn oracle_default() {
        let oracle = Oracle::default();
        assert_eq!(oracle.total_events(), 0);
    }

    #[test]
    fn oracle_custom_config() {
        let oracle = Oracle::with_config(0.5, 200);
        assert_eq!(oracle.total_events(), 0);
    }

    #[test]
    fn oracle_real_world_tool_sequence() {
        let mut oracle = Oracle::new();
        let dev_cycle = [
            "read",
            "grep",
            "read",
            "edit",
            "edit",
            "cargo_check",
            "edit",
            "cargo_test",
            "read",
            "edit",
            "cargo_check",
        ];
        for _ in 0..30 {
            oracle.ingest(&make_seq(&dev_cycle));
        }
        let pred = oracle.predict("edit");
        assert!(
            pred.event == "edit" || pred.event == "cargo_check" || pred.event == "cargo_test",
            "unexpected prediction: {}",
            pred.event
        );
        assert!(pred.confidence > 0.0);
        let pred2 = oracle.predict("cargo_check");
        assert!(!pred2.event.is_empty());
    }
}
