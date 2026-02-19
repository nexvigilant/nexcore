//! # Accuracy — Self-tracking prediction performance
//!
//! The Oracle tracks its own prediction accuracy over time,
//! enabling meta-learning: knowing when to trust its predictions.
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Primitives |
//! |------|------|------------|
//! | `AccuracyTracker` | T2-C | ν+N+κ+π (frequency of correct comparisons, persisted) |
//! | `AccuracyReport` | T2-P | N+κ (quantified comparison) |

use serde::{Deserialize, Serialize};

/// Tracks prediction accuracy with a sliding window.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccuracyTracker {
    /// Recent predictions: (predicted, actual, was_correct)
    history: Vec<PredictionOutcome>,
    /// Maximum history size (sliding window).
    window_size: usize,
    /// Lifetime correct count.
    lifetime_correct: u64,
    /// Lifetime total count.
    lifetime_total: u64,
}

/// A single prediction outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionOutcome {
    /// What was predicted.
    pub predicted: String,
    /// What actually happened.
    pub actual: String,
    /// Whether the prediction was correct.
    pub correct: bool,
    /// The confidence of the prediction when it was made.
    pub confidence: f64,
}

/// A snapshot of prediction accuracy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyReport {
    /// Recent accuracy (sliding window).
    pub recent_accuracy: f64,
    /// Lifetime accuracy.
    pub lifetime_accuracy: f64,
    /// Number of predictions in the window.
    pub window_count: usize,
    /// Total lifetime predictions.
    pub lifetime_count: u64,
    /// Calibration: average confidence on correct predictions.
    pub avg_confidence_correct: f64,
    /// Calibration: average confidence on incorrect predictions.
    pub avg_confidence_incorrect: f64,
    /// Brier score (lower is better, 0 = perfect).
    pub brier_score: f64,
}

impl AccuracyTracker {
    /// Create a new tracker with the given window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            history: Vec::new(),
            window_size: window_size.max(1),
            lifetime_correct: 0,
            lifetime_total: 0,
        }
    }

    /// Record a prediction outcome.
    pub fn record(&mut self, predicted: &str, actual: &str, confidence: f64) {
        let correct = predicted == actual;

        self.history.push(PredictionOutcome {
            predicted: predicted.to_string(),
            actual: actual.to_string(),
            correct,
            confidence,
        });

        // Maintain sliding window
        if self.history.len() > self.window_size {
            self.history.remove(0);
        }

        self.lifetime_total += 1;
        if correct {
            self.lifetime_correct += 1;
        }
    }

    /// Get a full accuracy report.
    pub fn report(&self) -> AccuracyReport {
        let window_correct = self.history.iter().filter(|o| o.correct).count();
        let recent_accuracy = if self.history.is_empty() {
            0.0
        } else {
            window_correct as f64 / self.history.len() as f64
        };

        let lifetime_accuracy = if self.lifetime_total == 0 {
            0.0
        } else {
            self.lifetime_correct as f64 / self.lifetime_total as f64
        };

        // Calibration metrics
        let correct_outcomes: Vec<&PredictionOutcome> =
            self.history.iter().filter(|o| o.correct).collect();
        let incorrect_outcomes: Vec<&PredictionOutcome> =
            self.history.iter().filter(|o| !o.correct).collect();

        let avg_confidence_correct = if correct_outcomes.is_empty() {
            0.0
        } else {
            correct_outcomes.iter().map(|o| o.confidence).sum::<f64>()
                / correct_outcomes.len() as f64
        };

        let avg_confidence_incorrect = if incorrect_outcomes.is_empty() {
            0.0
        } else {
            incorrect_outcomes.iter().map(|o| o.confidence).sum::<f64>()
                / incorrect_outcomes.len() as f64
        };

        // Brier score: mean squared error of probability estimates
        let brier_score = if self.history.is_empty() {
            0.0
        } else {
            self.history
                .iter()
                .map(|o| {
                    let outcome = if o.correct { 1.0 } else { 0.0 };
                    (o.confidence - outcome).powi(2)
                })
                .sum::<f64>()
                / self.history.len() as f64
        };

        AccuracyReport {
            recent_accuracy,
            lifetime_accuracy,
            window_count: self.history.len(),
            lifetime_count: self.lifetime_total,
            avg_confidence_correct,
            avg_confidence_incorrect,
            brier_score,
        }
    }

    /// Is the Oracle currently well-calibrated?
    /// Well-calibrated = confident when correct, uncertain when wrong.
    pub fn is_calibrated(&self) -> bool {
        let report = self.report();
        if report.window_count < 10 {
            return false; // Not enough data
        }
        // Confidence on correct should be higher than on incorrect
        report.avg_confidence_correct > report.avg_confidence_incorrect
    }

    /// Lifetime total predictions.
    pub fn lifetime_total(&self) -> u64 {
        self.lifetime_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tracker() {
        let tracker = AccuracyTracker::new(100);
        let report = tracker.report();
        assert_eq!(report.recent_accuracy, 0.0);
        assert_eq!(report.lifetime_accuracy, 0.0);
        assert_eq!(report.brier_score, 0.0);
    }

    #[test]
    fn perfect_predictions() {
        let mut tracker = AccuracyTracker::new(100);
        for _ in 0..10 {
            tracker.record("edit", "edit", 0.9);
        }
        let report = tracker.report();
        assert!((report.recent_accuracy - 1.0).abs() < 1e-10);
        assert!((report.lifetime_accuracy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mixed_predictions() {
        let mut tracker = AccuracyTracker::new(100);
        tracker.record("edit", "edit", 0.8); // correct
        tracker.record("build", "test", 0.6); // wrong
        tracker.record("read", "read", 0.9); // correct
        tracker.record("build", "build", 0.7); // correct

        let report = tracker.report();
        assert!((report.recent_accuracy - 0.75).abs() < 1e-10);
        assert_eq!(report.window_count, 4);
    }

    #[test]
    fn sliding_window() {
        let mut tracker = AccuracyTracker::new(3);
        // Fill window with wrong predictions
        tracker.record("a", "b", 0.5);
        tracker.record("a", "b", 0.5);
        tracker.record("a", "b", 0.5);
        assert!((tracker.report().recent_accuracy - 0.0).abs() < 1e-10);

        // Now push correct ones — old wrong ones slide out
        tracker.record("x", "x", 0.9);
        tracker.record("y", "y", 0.9);
        tracker.record("z", "z", 0.9);
        assert!((tracker.report().recent_accuracy - 1.0).abs() < 1e-10);

        // But lifetime still reflects all 6
        assert!((tracker.report().lifetime_accuracy - 0.5).abs() < 1e-10);
    }

    #[test]
    fn brier_score_perfect() {
        let mut tracker = AccuracyTracker::new(100);
        // Perfect calibration: high confidence + correct
        tracker.record("a", "a", 1.0);
        tracker.record("b", "b", 1.0);
        let report = tracker.report();
        assert!((report.brier_score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn brier_score_worst() {
        let mut tracker = AccuracyTracker::new(100);
        // Worst calibration: high confidence + wrong
        tracker.record("a", "b", 1.0);
        tracker.record("c", "d", 1.0);
        let report = tracker.report();
        assert!((report.brier_score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_check() {
        let mut tracker = AccuracyTracker::new(100);
        // Not enough data
        assert!(!tracker.is_calibrated());

        // Well-calibrated: high confidence when correct, low when wrong
        for _ in 0..8 {
            tracker.record("a", "a", 0.9); // correct, high confidence
        }
        for _ in 0..4 {
            tracker.record("a", "b", 0.3); // wrong, low confidence
        }
        assert!(tracker.is_calibrated());
    }

    #[test]
    fn serde_roundtrip() {
        let mut tracker = AccuracyTracker::new(50);
        tracker.record("a", "a", 0.8);
        tracker.record("b", "c", 0.6);
        let json = serde_json::to_string(&tracker).unwrap_or_default();
        let restored: AccuracyTracker = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(restored.lifetime_total(), 2);
    }
}
