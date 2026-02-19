//! # Predictor — The Oracle's prediction engine
//!
//! Combines first-order (bigram) and second-order (trigram) Markov chains
//! with Bayesian smoothing for robust event prediction.
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Primitives |
//! |------|------|------------|
//! | `Predictor` | T3 | σ+→+ν+κ+N+π (the full Oracle) |
//! | `Prediction` | T2-C | →+N+κ (causality with quantified confidence) |

use serde::{Deserialize, Serialize};

use crate::event::EventSequence;
use crate::matrix::TransitionMatrix;

/// A prediction: what event is most likely to occur next?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// The predicted event kind.
    pub event: String,
    /// Confidence in this prediction (0.0 - 1.0).
    pub confidence: f64,
    /// Alternative predictions with their confidences.
    pub alternatives: Vec<(String, f64)>,
    /// Entropy of the prediction distribution (lower = more certain).
    pub entropy: f64,
    /// How many observations this prediction is based on.
    pub evidence_count: u64,
}

/// The Oracle's prediction engine.
///
/// Uses a combination of:
/// - First-order transitions (bigrams): P(next | current)
/// - Second-order transitions (trigrams): P(next | prev, current)
/// - Bayesian smoothing with a uniform prior
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Predictor {
    /// First-order transition matrix.
    pub(crate) order1: TransitionMatrix,
    /// Second-order transitions: "prev|current" → next
    pub(crate) order2: TransitionMatrix,
    /// Total events ingested.
    pub(crate) total_events: u64,
    /// Smoothing parameter (Laplace smoothing). Higher = more uniform prior.
    pub(crate) alpha: f64,
}

impl Predictor {
    /// Create a new Predictor with default smoothing (alpha = 1.0).
    pub fn new() -> Self {
        Self {
            order1: TransitionMatrix::new(),
            order2: TransitionMatrix::new(),
            total_events: 0,
            alpha: 1.0,
        }
    }

    /// Create a Predictor with custom smoothing parameter.
    pub fn with_alpha(alpha: f64) -> Self {
        Self {
            alpha: alpha.max(0.0),
            ..Self::new()
        }
    }

    /// Ingest an event sequence, learning all transitions.
    pub fn ingest(&mut self, sequence: &EventSequence) {
        let kinds: Vec<&str> = sequence.kinds().collect();

        // Learn first-order transitions (bigrams)
        self.order1.learn_sequence(&kinds);

        // Learn second-order transitions (trigrams)
        for window in kinds.windows(3) {
            let compound_key = format!("{}|{}", window[0], window[1]);
            self.order2.record(&compound_key, window[2]);
        }

        self.total_events += sequence.len() as u64;
    }

    /// Predict the next event given the current state.
    ///
    /// If both current and previous are provided, uses second-order prediction
    /// blended with first-order for robustness.
    pub fn predict(&self, current: &str, previous: Option<&str>) -> Prediction {
        let n_states = self.order1.state_count().max(1);

        // First-order prediction
        let order1_preds = self.order1.top_predictions(current, n_states);
        let order1_total = self.order1.row_total(current);

        // Second-order prediction (if previous state available)
        let (order2_preds, order2_total) = if let Some(prev) = previous {
            let compound = format!("{prev}|{current}");
            let preds = self.order2.top_predictions(&compound, n_states);
            let total = self.order2.row_total(&compound);
            (preds, total)
        } else {
            (Vec::new(), 0)
        };

        // Blend: if we have second-order data, weight it 0.7 / first-order 0.3
        // Otherwise use first-order only
        let blended = if order2_total > 0 && !order2_preds.is_empty() {
            blend_predictions(&order1_preds, &order2_preds, 0.3, 0.7)
        } else {
            order1_preds
        };

        // Apply Laplace smoothing
        let smoothed = self.smooth(&blended, n_states);

        // Build prediction
        if smoothed.is_empty() {
            return Prediction {
                event: String::new(),
                confidence: 0.0,
                alternatives: Vec::new(),
                entropy: 0.0,
                evidence_count: 0,
            };
        }

        let top = &smoothed[0];
        let alternatives: Vec<(String, f64)> = smoothed[1..].to_vec();
        let entropy = self.order1.entropy(current);
        let evidence = order1_total + order2_total;

        Prediction {
            event: top.0.clone(),
            confidence: top.1,
            alternatives,
            entropy,
            evidence_count: evidence,
        }
    }

    /// Apply Laplace smoothing to prediction probabilities.
    fn smooth(&self, predictions: &[(String, f64)], n_states: usize) -> Vec<(String, f64)> {
        if predictions.is_empty() || self.alpha == 0.0 {
            return predictions.to_vec();
        }

        let total_mass: f64 = predictions.iter().map(|(_, p)| *p).sum();
        if total_mass == 0.0 {
            return predictions.to_vec();
        }

        let denom = total_mass + self.alpha * n_states as f64;
        predictions
            .iter()
            .map(|(name, p)| {
                let smoothed = (p + self.alpha / n_states as f64) / denom;
                (name.clone(), smoothed)
            })
            .collect()
    }

    /// Get the overall predictability score (0.0 = chaotic, 1.0 = deterministic).
    ///
    /// Based on average entropy: predictability = 1 - (avg_entropy / max_entropy)
    pub fn predictability(&self) -> f64 {
        let n = self.order1.state_count();
        if n <= 1 {
            return 1.0;
        }
        let max_entropy = (n as f64).log2();
        if max_entropy == 0.0 {
            return 1.0;
        }
        let avg = self.order1.average_entropy();
        (1.0 - avg / max_entropy).clamp(0.0, 1.0)
    }

    /// Total events ingested.
    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    /// Number of unique event kinds seen.
    pub fn vocabulary_size(&self) -> usize {
        self.order1.state_count()
    }
}

/// Blend two prediction lists with given weights.
fn blend_predictions(
    a: &[(String, f64)],
    b: &[(String, f64)],
    weight_a: f64,
    weight_b: f64,
) -> Vec<(String, f64)> {
    let mut combined: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for (name, prob) in a {
        *combined.entry(name.clone()).or_insert(0.0) += prob * weight_a;
    }
    for (name, prob) in b {
        *combined.entry(name.clone()).or_insert(0.0) += prob * weight_b;
    }

    let mut result: Vec<(String, f64)> = combined.into_iter().collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sequence(kinds: &[&str]) -> EventSequence {
        let mut seq = EventSequence::new();
        for k in kinds {
            seq.push_kind(*k);
        }
        seq
    }

    #[test]
    fn predict_simple_pattern() {
        let mut oracle = Predictor::new();
        // Learn: read → edit → build → test (repeat)
        for _ in 0..10 {
            oracle.ingest(&make_sequence(&["read", "edit", "build", "test"]));
        }

        let pred = oracle.predict("read", None);
        assert_eq!(pred.event, "edit");
        // With Laplace smoothing across 4 states, confidence is reduced from raw 1.0
        assert!(pred.confidence > 0.2);
    }

    #[test]
    fn predict_with_alternatives() {
        let mut oracle = Predictor::new();
        // After "read", sometimes "edit" (7x), sometimes "build" (3x)
        for _ in 0..7 {
            oracle.ingest(&make_sequence(&["read", "edit"]));
        }
        for _ in 0..3 {
            oracle.ingest(&make_sequence(&["read", "build"]));
        }

        let pred = oracle.predict("read", None);
        assert_eq!(pred.event, "edit");
        assert!(!pred.alternatives.is_empty());
    }

    #[test]
    fn predict_unknown_state() {
        let oracle = Predictor::new();
        let pred = oracle.predict("never_seen", None);
        assert!(pred.event.is_empty());
        assert_eq!(pred.confidence, 0.0);
    }

    #[test]
    fn second_order_prediction() {
        let mut oracle = Predictor::new();
        // Pattern: read → edit → build (always)
        // But: grep → edit → test (always)
        // So after "edit", the prediction depends on what came before
        for _ in 0..10 {
            oracle.ingest(&make_sequence(&["read", "edit", "build"]));
        }
        for _ in 0..10 {
            oracle.ingest(&make_sequence(&["grep", "edit", "test"]));
        }

        let pred_after_read = oracle.predict("edit", Some("read"));
        let pred_after_grep = oracle.predict("edit", Some("grep"));

        // With second-order context, predictions should differ
        assert_eq!(pred_after_read.event, "build");
        assert_eq!(pred_after_grep.event, "test");
    }

    #[test]
    fn predictability_deterministic() {
        let mut oracle = Predictor::new();
        // Perfectly deterministic: a → b → a → b...
        for _ in 0..50 {
            oracle.ingest(&make_sequence(&["a", "b"]));
        }
        assert!(oracle.predictability() > 0.9);
    }

    #[test]
    fn predictability_chaotic() {
        let mut oracle = Predictor::new();
        // After "start", equally likely to go to a, b, c, d
        for _ in 0..25 {
            oracle.ingest(&make_sequence(&["start", "a"]));
            oracle.ingest(&make_sequence(&["start", "b"]));
            oracle.ingest(&make_sequence(&["start", "c"]));
            oracle.ingest(&make_sequence(&["start", "d"]));
        }
        // Should be less predictable (but not 0 because other states are deterministic)
        assert!(oracle.predictability() < 0.9);
    }

    #[test]
    fn vocabulary_size() {
        let mut oracle = Predictor::new();
        oracle.ingest(&make_sequence(&["a", "b", "c"]));
        assert_eq!(oracle.vocabulary_size(), 3);
    }

    #[test]
    fn alpha_zero_no_smoothing() {
        let mut oracle = Predictor::with_alpha(0.0);
        oracle.ingest(&make_sequence(&["a", "b"]));
        let pred = oracle.predict("a", None);
        assert_eq!(pred.event, "b");
        assert!((pred.confidence - 1.0).abs() < 1e-10);
    }

    #[test]
    fn serde_roundtrip() {
        let mut oracle = Predictor::new();
        oracle.ingest(&make_sequence(&["x", "y", "z"]));
        let json = serde_json::to_string(&oracle).unwrap_or_default();
        let restored: Predictor = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(restored.total_events(), 3);
    }
}
