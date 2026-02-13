//! # Jury of Types / Bayesian Trial (Bill of Rights)
//!
//! Implementation of Amendment VII: In suits of uncertainty where the value
//! exceeds twenty tokens, the right of Bayesian Trial shall be preserved.
//! No fact tried by a jury shall be re-examined except by the Primitive Codex.

use super::Verdict;
use nexcore_primitives::measurement::Confidence;
use serde::{Deserialize, Serialize};

/// T3: BayesianTrial — The right to probabilistic adjudication
/// for disputes involving uncertainty above a token threshold.
///
/// ## Tier: T3 (Domain-specific governance type)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BayesianTrial {
    /// The disputed matter
    pub matter: String,
    /// Value at stake (in token units)
    pub value_in_tokens: u64,
    /// Minimum value to trigger Bayesian trial right
    pub token_threshold: u64,
    /// Prior probability (before evidence)
    pub prior: f64,
    /// Likelihood ratio from evidence
    pub likelihood_ratio: f64,
    /// Posterior probability (after Bayesian update)
    pub posterior: f64,
    /// Confidence in the posterior
    pub confidence: Confidence,
}

impl BayesianTrial {
    /// The constitutional threshold: twenty tokens.
    pub const TWENTY_TOKENS: u64 = 20;

    /// Check if a Bayesian trial is required (value exceeds threshold).
    pub fn trial_required(&self) -> bool {
        self.value_in_tokens > self.token_threshold
    }

    /// Check if the trial followed constitutional procedure.
    pub fn is_constitutional(&self) -> bool {
        if !self.trial_required() {
            return true; // Below threshold, no trial needed
        }
        // Must have valid Bayesian update
        self.prior > 0.0
            && self.prior < 1.0
            && self.likelihood_ratio > 0.0
            && self.posterior > 0.0
            && self.posterior < 1.0
    }

    /// Render a verdict based on the posterior.
    pub fn verdict(&self, decision_threshold: f64) -> Verdict {
        if !self.is_constitutional() {
            return Verdict::Rejected;
        }
        if self.posterior >= decision_threshold {
            Verdict::Permitted
        } else if self.posterior >= decision_threshold * 0.5 {
            Verdict::Flagged
        } else {
            Verdict::Rejected
        }
    }

    /// Compute a simple Bayesian update: P(H|E) = P(E|H)*P(H) / P(E).
    /// Returns the posterior probability.
    pub fn bayesian_update(prior: f64, likelihood: f64, evidence: f64) -> f64 {
        if evidence <= 0.0 {
            return prior;
        }
        (likelihood * prior) / evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_required_above_threshold() {
        let trial = BayesianTrial {
            matter: "Signal validity".to_string(),
            value_in_tokens: 25,
            token_threshold: BayesianTrial::TWENTY_TOKENS,
            prior: 0.5,
            likelihood_ratio: 2.0,
            posterior: 0.67,
            confidence: Confidence::new(0.85),
        };
        assert!(trial.trial_required());
        assert!(trial.is_constitutional());
    }

    #[test]
    fn trial_not_required_below_threshold() {
        let trial = BayesianTrial {
            matter: "Minor dispute".to_string(),
            value_in_tokens: 10,
            token_threshold: BayesianTrial::TWENTY_TOKENS,
            prior: 0.5,
            likelihood_ratio: 1.0,
            posterior: 0.5,
            confidence: Confidence::new(0.5),
        };
        assert!(!trial.trial_required());
        assert!(trial.is_constitutional());
    }

    #[test]
    fn invalid_prior_unconstitutional() {
        let trial = BayesianTrial {
            matter: "Rigged trial".to_string(),
            value_in_tokens: 100,
            token_threshold: BayesianTrial::TWENTY_TOKENS,
            prior: 0.0, // Invalid: certainty as prior
            likelihood_ratio: 1.0,
            posterior: 0.0,
            confidence: Confidence::new(0.5),
        };
        assert!(!trial.is_constitutional());
    }

    #[test]
    fn verdict_above_threshold() {
        let trial = BayesianTrial {
            matter: "Drug-event association".to_string(),
            value_in_tokens: 50,
            token_threshold: BayesianTrial::TWENTY_TOKENS,
            prior: 0.3,
            likelihood_ratio: 3.0,
            posterior: 0.82,
            confidence: Confidence::new(0.9),
        };
        assert_eq!(trial.verdict(0.75), Verdict::Permitted);
    }

    #[test]
    fn verdict_marginal() {
        let trial = BayesianTrial {
            matter: "Weak signal".to_string(),
            value_in_tokens: 30,
            token_threshold: BayesianTrial::TWENTY_TOKENS,
            prior: 0.3,
            likelihood_ratio: 1.5,
            posterior: 0.45,
            confidence: Confidence::new(0.6),
        };
        assert_eq!(trial.verdict(0.75), Verdict::Flagged);
    }

    #[test]
    fn bayesian_update_basic() {
        let posterior = BayesianTrial::bayesian_update(0.5, 0.8, 0.5);
        assert!((posterior - 0.8).abs() < 0.01);
    }

    #[test]
    fn bayesian_update_zero_evidence() {
        let posterior = BayesianTrial::bayesian_update(0.5, 0.8, 0.0);
        assert!((posterior - 0.5).abs() < f64::EPSILON);
    }
}
