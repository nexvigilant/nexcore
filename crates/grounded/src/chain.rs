//! EvidenceChain: gradient-like confidence propagation tracking.
//!
//! Tracks how confidence flows through a chain of reasoning steps,
//! analogous to gradient flow through a computation graph.
//!
//! Grounds to: σ(Sequence) + →(Causality) + N(Quantity)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::confidence::Confidence;

/// A single step in an evidence chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStep {
    /// Description of this reasoning step.
    pub description: String,
    /// Confidence at this step.
    pub confidence: Confidence,
    /// How much this step contributed to (or degraded) overall confidence.
    /// Positive = strengthened, negative = weakened.
    pub delta: f64,
    /// When this step was recorded.
    pub recorded_at: DateTime,
}

/// A chain of evidence steps tracking confidence propagation.
///
/// Like backpropagation through a neural network, this tracks how
/// confidence flows (and degrades) through a reasoning chain.
/// Each step either strengthens or weakens the overall confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceChain {
    /// What claim this chain supports.
    pub claim: String,
    /// Ordered steps in the chain.
    steps: Vec<EvidenceStep>,
    /// Current aggregate confidence.
    current_confidence: Confidence,
}

impl EvidenceChain {
    /// Start a new evidence chain for a claim.
    pub fn new(claim: impl Into<String>, initial_confidence: Confidence) -> Self {
        let claim = claim.into();
        Self {
            steps: vec![EvidenceStep {
                description: format!("initial prior for: {claim}"),
                confidence: initial_confidence,
                delta: 0.0,
                recorded_at: DateTime::now(),
            }],
            claim,
            current_confidence: initial_confidence,
        }
    }

    /// Add a step that strengthens confidence (multiplicative).
    pub fn strengthen(&mut self, description: impl Into<String>, factor: Confidence) {
        let old = self.current_confidence.value();
        // Bayesian-ish update: move toward 1.0 proportional to factor
        let new_val = 1.0 - (1.0 - old) * (1.0 - factor.value());
        let new_confidence = Confidence::new(new_val).unwrap_or(Confidence::CERTAIN);
        let delta = new_confidence.value() - old;

        self.steps.push(EvidenceStep {
            description: description.into(),
            confidence: new_confidence,
            delta,
            recorded_at: DateTime::now(),
        });
        self.current_confidence = new_confidence;
    }

    /// Add a step that weakens confidence (multiplicative).
    pub fn weaken(&mut self, description: impl Into<String>, factor: Confidence) {
        let old = self.current_confidence.value();
        // Move toward 0.0 proportional to factor
        let new_val = old * (1.0 - factor.value());
        let new_confidence = Confidence::new(new_val).unwrap_or(Confidence::NONE);
        let delta = new_confidence.value() - old;

        self.steps.push(EvidenceStep {
            description: description.into(),
            confidence: new_confidence,
            delta,
            recorded_at: DateTime::now(),
        });
        self.current_confidence = new_confidence;
    }

    /// Add a step with an explicit new confidence value.
    pub fn update(&mut self, description: impl Into<String>, new_confidence: Confidence) {
        let delta = new_confidence.value() - self.current_confidence.value();
        self.steps.push(EvidenceStep {
            description: description.into(),
            confidence: new_confidence,
            delta,
            recorded_at: DateTime::now(),
        });
        self.current_confidence = new_confidence;
    }

    /// Current confidence after all steps.
    pub fn confidence(&self) -> Confidence {
        self.current_confidence
    }

    /// All steps in the chain.
    pub fn steps(&self) -> &[EvidenceStep] {
        &self.steps
    }

    /// Number of evidence steps (including initial).
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Whether the chain has only the initial step.
    pub fn is_empty(&self) -> bool {
        self.steps.len() <= 1
    }

    /// Total positive evidence (sum of positive deltas).
    pub fn total_support(&self) -> f64 {
        self.steps
            .iter()
            .filter(|s| s.delta > 0.0)
            .map(|s| s.delta)
            .sum()
    }

    /// Total negative evidence (sum of negative deltas, as positive number).
    pub fn total_opposition(&self) -> f64 {
        self.steps
            .iter()
            .filter(|s| s.delta < 0.0)
            .map(|s| -s.delta)
            .sum()
    }

    /// Net evidence direction: positive = supported, negative = weakened.
    pub fn net_evidence(&self) -> f64 {
        self.current_confidence.value()
            - self
                .steps
                .first()
                .map(|s| s.confidence.value())
                .unwrap_or(0.5)
    }
}

impl std::fmt::Display for EvidenceChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Evidence Chain: {}", self.claim)?;
        for (i, step) in self.steps.iter().enumerate() {
            let arrow = if step.delta > 0.0 {
                "+"
            } else if step.delta < 0.0 {
                "-"
            } else {
                "="
            };
            writeln!(
                f,
                "  [{i}] {arrow} {} (conf: {}, delta: {:.4})",
                step.description, step.confidence, step.delta
            )?;
        }
        writeln!(f, "  Final: {}", self.current_confidence)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(v: f64) -> Confidence {
        Confidence::new(v).unwrap_or(Confidence::NONE)
    }

    #[test]
    fn chain_basic() {
        let chain = EvidenceChain::new("test claim", c(0.5));
        assert_eq!(chain.len(), 1);
        assert!(chain.is_empty()); // only initial prior, no evidence yet
        assert_eq!(chain.confidence(), c(0.5));
    }

    #[test]
    fn chain_strengthen() {
        let mut chain = EvidenceChain::new("gravity exists", c(0.5));
        chain.strengthen("apple fell down", c(0.8));

        // strengthen formula: new = 1 - (1 - old) * (1 - factor)
        // = 1 - (1 - 0.5) * (1 - 0.8) = 1 - 0.5 * 0.2 = 0.9
        assert!(
            (chain.confidence().value() - 0.9).abs() < 1e-10,
            "expected 0.9, got {}",
            chain.confidence().value()
        );
        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());

        // Delta should be +0.4 (from 0.5 to 0.9)
        let step = &chain.steps()[1];
        assert!((step.delta - 0.4).abs() < 1e-10, "delta should be +0.4");
    }

    #[test]
    fn chain_weaken() {
        let mut chain = EvidenceChain::new("perpetual motion works", c(0.5));
        chain.weaken("thermodynamics says no", c(0.9));

        // weaken formula: new = old * (1 - factor) = 0.5 * (1 - 0.9) = 0.05
        assert!(
            (chain.confidence().value() - 0.05).abs() < 1e-10,
            "expected 0.05, got {}",
            chain.confidence().value()
        );
        assert_eq!(chain.len(), 2);

        // Delta should be -0.45 (from 0.5 to 0.05)
        let step = &chain.steps()[1];
        assert!(
            (step.delta - (-0.45)).abs() < 1e-10,
            "delta should be -0.45"
        );
    }

    #[test]
    fn chain_mixed_evidence() {
        let mut chain = EvidenceChain::new("new drug is effective", c(0.5));
        chain.strengthen("phase 1 trial positive", c(0.3));
        chain.strengthen("phase 2 trial positive", c(0.4));
        chain.weaken("adverse event reported", c(0.2));
        chain.strengthen("phase 3 confirms efficacy", c(0.6));

        assert_eq!(chain.len(), 5);
        assert!(chain.total_support() > 0.0);
        assert!(chain.total_opposition() > 0.0);
        // Net should be positive given more support than opposition
        assert!(chain.net_evidence() > 0.0);
        // Final confidence should be > initial 0.5
        assert!(chain.confidence().value() > 0.5);
    }

    #[test]
    fn chain_update_explicit() {
        let mut chain = EvidenceChain::new("test", c(0.5));
        chain.update("new evidence", c(0.8));
        assert!((chain.confidence().value() - 0.8).abs() < 1e-10);
        let step = &chain.steps()[1];
        assert!((step.delta - 0.3).abs() < 1e-10, "delta should be +0.3");
    }

    #[test]
    fn chain_net_evidence_negative() {
        let mut chain = EvidenceChain::new("weak claim", c(0.5));
        chain.weaken("contradicted", c(0.8));
        assert!(
            chain.net_evidence() < 0.0,
            "weakened chain has negative net evidence"
        );
    }

    #[test]
    fn chain_display() {
        let mut chain = EvidenceChain::new("test", c(0.5));
        chain.strengthen("evidence A", c(0.5));
        let display = format!("{chain}");
        assert!(display.contains("Evidence Chain: test"));
        assert!(display.contains("evidence A"));
    }
}
