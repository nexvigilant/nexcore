//! Belief state management for Minesweeper-PV cells.
//!
//! ## Safety Axioms
//!
//! The BeliefState implements Bayesian belief propagation with the following
//! constraints from the Tree of Vigilance:
//!
//! - Beliefs are updated via likelihood ratios (multiplicative on odds)
//! - Uncertainty (sigma) decreases monotonically with information gain
//! - Status transitions follow the Signal Confirmation Axiom

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::evidence::Evidence;
use crate::minesweeper::types::{
    BASE_SIGNAL_RATE, CONFIRM_THRESHOLD, CONFIRM_UNCERTAINTY, CellStatus, FLAG_THRESHOLD,
    TemporalWindow, UNFLAG_THRESHOLD,
};

/// Bayesian belief state for a cell.
///
/// The belief state tracks both the point estimate (theta) and uncertainty (sigma).
/// Updates follow Bayesian inference using likelihood ratios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefState {
    /// Probability estimate that this is a true signal (0.0 to 1.0)
    pub theta: f64,
    /// Uncertainty measure (higher = more uncertain)
    pub sigma: f64,
}

impl Default for BeliefState {
    fn default() -> Self {
        Self {
            theta: BASE_SIGNAL_RATE,
            sigma: 1.0,
        }
    }
}

impl BeliefState {
    /// Create a new belief state with custom prior
    #[must_use]
    pub fn with_prior(theta: f64) -> Self {
        Self {
            theta: theta.clamp(0.0001, 0.9999),
            sigma: 1.0,
        }
    }

    /// Calculate confidence (inverse of uncertainty)
    ///
    /// Confidence ranges from 0 (no confidence) to approaching 1 (high confidence)
    #[must_use]
    pub fn confidence(&self) -> f64 {
        1.0 / (1.0 + self.sigma)
    }

    /// Calculate odds from probability
    ///
    /// Returns `f64::INFINITY` if theta >= 1.0
    #[must_use]
    pub fn odds(&self) -> f64 {
        if self.theta >= 1.0 {
            f64::INFINITY
        } else {
            self.theta / (1.0 - self.theta)
        }
    }

    /// Update theta from posterior odds
    ///
    /// Clamps the result to (0.0001, 0.9999) to avoid numerical issues
    pub fn update_from_odds(&mut self, posterior_odds: f64) {
        if posterior_odds.is_infinite() {
            self.theta = 0.9999;
        } else {
            self.theta = (posterior_odds / (1.0 + posterior_odds)).clamp(0.0001, 0.9999);
        }
    }

    /// Update belief with a likelihood ratio
    ///
    /// Returns the information gain (in bits)
    pub fn update(&mut self, likelihood_ratio: f64) -> f64 {
        let prior_odds = self.odds();
        let posterior_odds = prior_odds * likelihood_ratio;
        self.update_from_odds(posterior_odds);

        // Calculate information gain and reduce uncertainty
        let information_gain = if likelihood_ratio > 0.0 {
            likelihood_ratio.log2().abs()
        } else {
            0.0
        };

        // Reduce uncertainty proportional to information gain
        self.sigma *= (-0.3 * information_gain).exp();

        information_gain
    }
}

/// A cell in the Minesweeper-PV grid.
///
/// Each cell represents a drug-event-population combination with:
/// - Unique identifier (drug|event|population)
/// - Bayesian belief state
/// - Investigation status
/// - Evidence history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Unique identifier: "drug|event|population"
    pub id: String,
    /// Drug name
    pub drug: String,
    /// Adverse event (MedDRA preferred term)
    pub event: String,
    /// Population subset (e.g., "all", "elderly", "pediatric")
    pub population: String,
    /// Time-to-onset window categorization
    pub tto_window: TemporalWindow,
    /// Current belief state
    pub belief: BeliefState,
    /// Current investigation status
    pub status: CellStatus,
    /// Number of evidence updates applied
    pub evidence_count: usize,
    /// Last update timestamp
    pub last_updated: DateTime,
}

impl Cell {
    /// Create a new cell with default belief state
    #[must_use]
    pub fn new(
        drug: impl Into<String>,
        event: impl Into<String>,
        population: impl Into<String>,
    ) -> Self {
        let drug = drug.into();
        let event = event.into();
        let population = population.into();
        let id = format!("{}|{}|{}", drug, event, population);

        Self {
            id,
            drug,
            event,
            population,
            tto_window: TemporalWindow::default(),
            belief: BeliefState::default(),
            status: CellStatus::default(),
            evidence_count: 0,
            last_updated: DateTime::now(),
        }
    }

    /// Create a new cell with custom prior probability
    #[must_use]
    pub fn with_prior(
        drug: impl Into<String>,
        event: impl Into<String>,
        population: impl Into<String>,
        prior: f64,
    ) -> Self {
        let mut cell = Self::new(drug, event, population);
        cell.belief = BeliefState::with_prior(prior);
        cell
    }

    /// Create a new cell with time-to-onset window
    #[must_use]
    pub fn with_tto(
        drug: impl Into<String>,
        event: impl Into<String>,
        population: impl Into<String>,
        tto_window: TemporalWindow,
    ) -> Self {
        let mut cell = Self::new(drug, event, population);
        cell.tto_window = tto_window;
        cell
    }

    /// Update belief state with new evidence
    ///
    /// Returns the likelihood ratio that was applied
    pub fn update_belief(&mut self, evidence: &Evidence) -> f64 {
        let lr = evidence.compute_likelihood_ratio();
        self.belief.update(lr);
        self.evidence_count += 1;
        self.last_updated = DateTime::now();
        self.update_status();
        lr
    }

    /// Update cell status based on current belief state
    fn update_status(&mut self) {
        // Once confirmed, don't change status
        if self.status == CellStatus::Confirmed {
            return;
        }

        // Check for confirmation (high belief + low uncertainty)
        if self.belief.theta > CONFIRM_THRESHOLD && self.belief.sigma < CONFIRM_UNCERTAINTY {
            self.status = CellStatus::Confirmed;
            return;
        }

        // Check for flagging (moderate belief)
        if self.belief.theta > FLAG_THRESHOLD {
            if self.status != CellStatus::Investigated {
                self.status = CellStatus::Flagged;
            }
            return;
        }

        // Check for deferral (low belief after being flagged)
        if self.belief.theta < UNFLAG_THRESHOLD && self.status == CellStatus::Flagged {
            self.status = CellStatus::Deferred;
        }
    }

    /// Mark cell as investigated
    pub fn mark_investigated(&mut self) {
        if self.status != CellStatus::Confirmed {
            self.status = CellStatus::Investigated;
        }
    }

    /// Get current probability estimate
    #[must_use]
    pub fn theta(&self) -> f64 {
        self.belief.theta
    }

    /// Get current uncertainty
    #[must_use]
    pub fn sigma(&self) -> f64 {
        self.belief.sigma
    }

    /// Get current confidence
    #[must_use]
    pub fn confidence(&self) -> f64 {
        self.belief.confidence()
    }

    /// Check if cell is flagged or confirmed (needs attention)
    #[must_use]
    pub fn needs_attention(&self) -> bool {
        matches!(self.status, CellStatus::Flagged | CellStatus::Confirmed)
    }

    /// Check if cell can be investigated (not confirmed)
    #[must_use]
    pub fn can_investigate(&self) -> bool {
        !matches!(self.status, CellStatus::Confirmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belief_state_default() {
        let belief = BeliefState::default();
        assert!((belief.theta - BASE_SIGNAL_RATE).abs() < f64::EPSILON);
        assert!((belief.sigma - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_belief_odds_probability() {
        let mut belief = BeliefState::with_prior(0.5);
        assert!((belief.odds() - 1.0).abs() < 1e-10);

        belief.theta = 0.8;
        assert!((belief.odds() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_belief_update() {
        let mut belief = BeliefState::with_prior(0.5);
        let initial_theta = belief.theta;

        // Update with LR > 1 should increase theta
        belief.update(2.0);
        assert!(belief.theta > initial_theta);

        // Update with LR < 1 should decrease theta
        let high_theta = belief.theta;
        belief.update(0.5);
        assert!(belief.theta < high_theta);
    }

    #[test]
    fn test_belief_uncertainty_reduction() {
        let mut belief = BeliefState::default();
        let initial_sigma = belief.sigma;

        // Strong evidence should reduce uncertainty
        belief.update(4.0);
        assert!(belief.sigma < initial_sigma);
    }

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new("aspirin", "gi_bleeding", "all");
        assert_eq!(cell.id, "aspirin|gi_bleeding|all");
        assert_eq!(cell.status, CellStatus::Unknown);
    }

    #[test]
    fn test_cell_belief_update() {
        let mut cell = Cell::new("aspirin", "gi_bleeding", "all");
        let evidence = Evidence::builder()
            .prr(3.0)
            .chi2(8.0)
            .count(5)
            .temporal_pattern("strong")
            .mechanism("established")
            .build();

        let initial_theta = cell.theta();
        cell.update_belief(&evidence);

        assert!(cell.theta() > initial_theta);
        assert_eq!(cell.evidence_count, 1);
    }

    #[test]
    fn test_cell_status_flagging() {
        let mut cell = Cell::new("drug_x", "event_y", "all");
        cell.belief.theta = 0.35; // Above FLAG_THRESHOLD
        cell.update_status();

        assert_eq!(cell.status, CellStatus::Flagged);
    }

    #[test]
    fn test_cell_status_confirmation() {
        let mut cell = Cell::new("drug_x", "event_y", "all");
        cell.belief.theta = 0.85; // Above CONFIRM_THRESHOLD
        cell.belief.sigma = 0.2; // Below CONFIRM_UNCERTAINTY
        cell.update_status();

        assert_eq!(cell.status, CellStatus::Confirmed);
    }

    #[test]
    fn test_cell_status_deferral() {
        let mut cell = Cell::new("drug_x", "event_y", "all");

        // First flag it
        cell.status = CellStatus::Flagged;

        // Then set low probability
        cell.belief.theta = 0.1; // Below UNFLAG_THRESHOLD
        cell.update_status();

        assert_eq!(cell.status, CellStatus::Deferred);
    }

    #[test]
    fn test_confirmed_status_locked() {
        let mut cell = Cell::new("drug_x", "event_y", "all");
        cell.status = CellStatus::Confirmed;

        // Even with low theta, status shouldn't change
        cell.belief.theta = 0.1;
        cell.update_status();

        assert_eq!(cell.status, CellStatus::Confirmed);
    }
}
