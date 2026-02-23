//! # Transition Matrix — Markov chain state transitions
//!
//! Learns P(next | current) from observed event sequences.
//! The matrix is the Oracle's memory of causal patterns.
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Primitives |
//! |------|------|------------|
//! | `TransitionMatrix` | T2-C | →(Causality) + N(Quantity) + μ(Mapping) + ν(Frequency) |
//! | `TransitionRow` | T2-P | μ(Mapping) + N(Quantity) |

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A first-order Markov transition matrix.
///
/// Tracks how often event A is followed by event B.
/// P(B | A) = count(A→B) / sum(count(A→*))
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransitionMatrix {
    /// counts[from][to] = number of observed transitions
    counts: HashMap<String, HashMap<String, u64>>,
    /// Total transitions observed
    total_transitions: u64,
}

impl TransitionMatrix {
    /// Create an empty transition matrix.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a transition from one event to another.
    pub fn record(&mut self, from: &str, to: &str) {
        self.counts
            .entry(from.to_string())
            .or_default()
            .entry(to.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        self.total_transitions += 1;
    }

    /// Learn from an event sequence (extracts all consecutive pairs).
    pub fn learn_sequence(&mut self, kinds: &[&str]) {
        for window in kinds.windows(2) {
            self.record(window[0], window[1]);
        }
    }

    /// Get the probability P(to | from).
    pub fn probability(&self, from: &str, to: &str) -> f64 {
        let row_total = self.row_total(from);
        if row_total == 0 {
            return 0.0;
        }
        let count = self
            .counts
            .get(from)
            .and_then(|row| row.get(to))
            .copied()
            .unwrap_or(0);
        count as f64 / row_total as f64
    }

    /// Get the total count of transitions from a given event.
    pub fn row_total(&self, from: &str) -> u64 {
        self.counts
            .get(from)
            .map(|row| row.values().sum())
            .unwrap_or(0)
    }

    /// Get the top-N most likely next events given current state.
    pub fn top_predictions(&self, from: &str, n: usize) -> Vec<(String, f64)> {
        let row_total = self.row_total(from);
        if row_total == 0 {
            return Vec::new();
        }

        let mut predictions: Vec<(String, f64)> = self
            .counts
            .get(from)
            .map(|row| {
                row.iter()
                    .map(|(to, &count)| (to.clone(), count as f64 / row_total as f64))
                    .collect()
            })
            .unwrap_or_default();

        predictions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predictions.truncate(n);
        predictions
    }

    /// Get all unique states (event kinds) seen.
    pub fn states(&self) -> Vec<&str> {
        let mut all: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for (from, row) in &self.counts {
            all.insert(from.as_str());
            for to in row.keys() {
                all.insert(to.as_str());
            }
        }
        let mut sorted: Vec<&str> = all.into_iter().collect();
        sorted.sort_unstable();
        sorted
    }

    /// Total number of transitions recorded.
    pub fn total_transitions(&self) -> u64 {
        self.total_transitions
    }

    /// Number of unique states.
    pub fn state_count(&self) -> usize {
        self.states().len()
    }

    /// Entropy of the transition distribution from a given state.
    /// High entropy = unpredictable. Low entropy = deterministic.
    ///
    /// H(X | from) = -Σ P(x) log2(P(x))
    pub fn entropy(&self, from: &str) -> f64 {
        let row_total = self.row_total(from);
        if row_total == 0 {
            return 0.0;
        }

        self.counts
            .get(from)
            .map(|row| {
                row.values()
                    .map(|&count| {
                        let p = count as f64 / row_total as f64;
                        if p > 0.0 { -p * p.log2() } else { 0.0 }
                    })
                    .sum()
            })
            .unwrap_or(0.0)
    }

    /// Average entropy across all states (weighted by row frequency).
    /// This is the overall predictability of the system.
    pub fn average_entropy(&self) -> f64 {
        if self.total_transitions == 0 {
            return 0.0;
        }

        let weighted_sum: f64 = self
            .counts
            .keys()
            .map(|from| {
                let weight = self.row_total(from) as f64 / self.total_transitions as f64;
                weight * self.entropy(from)
            })
            .sum();

        weighted_sum
    }

    /// Merge another matrix into this one (additive).
    pub fn merge(&mut self, other: &TransitionMatrix) {
        for (from, row) in &other.counts {
            for (to, &count) in row {
                *self
                    .counts
                    .entry(from.clone())
                    .or_default()
                    .entry(to.clone())
                    .or_insert(0) += count;
            }
        }
        self.total_transitions += other.total_transitions;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix() {
        let m = TransitionMatrix::new();
        assert_eq!(m.total_transitions(), 0);
        assert_eq!(m.state_count(), 0);
        assert_eq!(m.probability("a", "b"), 0.0);
    }

    #[test]
    fn single_transition() {
        let mut m = TransitionMatrix::new();
        m.record("a", "b");
        assert_eq!(m.probability("a", "b"), 1.0);
        assert_eq!(m.probability("a", "c"), 0.0);
        assert_eq!(m.total_transitions(), 1);
    }

    #[test]
    fn multiple_transitions() {
        let mut m = TransitionMatrix::new();
        m.record("a", "b");
        m.record("a", "b");
        m.record("a", "c");
        assert!((m.probability("a", "b") - 2.0 / 3.0).abs() < 1e-10);
        assert!((m.probability("a", "c") - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn learn_sequence() {
        let mut m = TransitionMatrix::new();
        m.learn_sequence(&["read", "edit", "build", "test"]);
        assert_eq!(m.probability("read", "edit"), 1.0);
        assert_eq!(m.probability("edit", "build"), 1.0);
        assert_eq!(m.probability("build", "test"), 1.0);
        assert_eq!(m.total_transitions(), 3);
    }

    #[test]
    fn top_predictions() {
        let mut m = TransitionMatrix::new();
        for _ in 0..10 {
            m.record("start", "read");
        }
        for _ in 0..5 {
            m.record("start", "write");
        }
        for _ in 0..2 {
            m.record("start", "build");
        }

        let top = m.top_predictions("start", 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "read");
        assert!(top[0].1 > top[1].1);
    }

    #[test]
    fn entropy_deterministic() {
        let mut m = TransitionMatrix::new();
        m.record("a", "b");
        m.record("a", "b");
        m.record("a", "b");
        // All transitions go to b → entropy = 0
        assert!((m.entropy("a") - 0.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_uniform() {
        let mut m = TransitionMatrix::new();
        m.record("a", "b");
        m.record("a", "c");
        // Uniform over 2 → entropy = 1.0 bit
        assert!((m.entropy("a") - 1.0).abs() < 1e-10);
    }

    #[test]
    fn entropy_unknown_state() {
        let m = TransitionMatrix::new();
        assert!((m.entropy("unknown") - 0.0).abs() < 1e-10);
    }

    #[test]
    fn merge_matrices() {
        let mut m1 = TransitionMatrix::new();
        m1.record("a", "b");
        m1.record("a", "b");

        let mut m2 = TransitionMatrix::new();
        m2.record("a", "b");
        m2.record("a", "c");

        m1.merge(&m2);
        assert_eq!(m1.total_transitions(), 4);
        assert!((m1.probability("a", "b") - 3.0 / 4.0).abs() < 1e-10);
        assert!((m1.probability("a", "c") - 1.0 / 4.0).abs() < 1e-10);
    }

    #[test]
    fn states_sorted() {
        let mut m = TransitionMatrix::new();
        m.record("c", "a");
        m.record("b", "a");
        let states = m.states();
        assert_eq!(states, vec!["a", "b", "c"]);
    }

    #[test]
    fn top_predictions_empty() {
        let m = TransitionMatrix::new();
        assert!(m.top_predictions("unknown", 5).is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let mut m = TransitionMatrix::new();
        m.record("read", "edit");
        m.record("edit", "build");
        let json = serde_json::to_string(&m).unwrap_or_default();
        let restored: TransitionMatrix = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(restored.total_transitions(), 2);
        assert_eq!(restored.probability("read", "edit"), 1.0);
    }
}
