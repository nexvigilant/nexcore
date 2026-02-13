//! # FSRS (Free Spaced Repetition Scheduler)
//!
//! Optimal review scheduling algorithm based on the forgetting curve.
//!
//! ## Core Concept
//!
//! FSRS uses a power-law forgetting curve to model memory decay:
//!
//! ```text
//! R(t) = (1 + t/(9s))^(-1)
//! ```
//!
//! Where:
//! - `R` = Retrievability (probability of recall)
//! - `t` = Time since last review (days)
//! - `s` = Stability (memory strength)
//!
//! ## Usage
//!
//! ```
//! use nexcore_vigilance::foundation::algorithms::scheduling::{FsrsScheduler, Card, Rating};
//!
//! let scheduler = FsrsScheduler::new(None, 0.9);
//! let mut card = Card::default();
//!
//! // Simulate a review
//! let retrievability = scheduler.forgetting_curve(1.0, 1.0);
//! assert!((retrievability - 0.9).abs() < 0.01); // ~90% at t=1, s=1
//! ```

use serde::{Deserialize, Serialize};

/// Default 17-parameter set for FSRS-4.5
pub const DEFAULT_PARAMETERS: [f64; 17] = [
    0.4, 0.6, 2.4, 5.8,  // Initial stability for Again, Hard, Good, Easy
    4.93, // Initial difficulty
    0.94, // Difficulty adjustment multiplier
    0.86, // Difficulty update factor
    0.01, // Mean reversion weight
    1.49, // Stability increase base
    0.14, // Stability difficulty factor
    0.94, // Stability retrievability factor
    2.18, // Again stability base
    0.05, // Again difficulty factor
    0.34, // Again stability power
    1.26, // Again retrievability factor
    0.29, // Reserved
    2.61, // Reserved
];

/// Card learning state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CardState {
    /// New card, never reviewed
    #[default]
    New,
    /// Currently being learned
    Learning,
    /// In review cycle
    Review,
    /// Being relearned after lapse
    Relearning,
}

/// Review rating (user feedback).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    /// Complete failure, reset
    Again = 1,
    /// Recalled with difficulty
    Hard = 2,
    /// Recalled correctly
    Good = 3,
    /// Recalled easily
    Easy = 4,
}

impl Rating {
    /// Convert to f64 for calculations.
    pub fn as_f64(&self) -> f64 {
        *self as usize as f64
    }
}

/// Flashcard state for scheduling.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Card {
    /// Current memory stability (days)
    pub stability: f64,
    /// Current difficulty (1.0-10.0)
    pub difficulty: f64,
    /// Days since last review
    pub elapsed_days: u64,
    /// Days until next scheduled review
    pub scheduled_days: u64,
    /// Total number of reviews
    pub reps: u32,
    /// Number of times card was forgotten
    pub lapses: u32,
    /// Current learning state
    pub state: CardState,
}

/// Result of a review operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    /// Updated card state
    pub card: Card,
    /// Rating given
    pub rating: Rating,
    /// Days until next review
    pub scheduled_days: u64,
    /// Retrievability at review time
    pub retrievability: f64,
}

/// FSRS Scheduler with configurable parameters.
#[derive(Debug, Clone)]
pub struct FsrsScheduler {
    /// 17 algorithm parameters
    pub parameters: [f64; 17],
    /// Target retention rate (0.7-0.97 recommended)
    pub desired_retention: f64,
}

impl Default for FsrsScheduler {
    fn default() -> Self {
        Self {
            parameters: DEFAULT_PARAMETERS,
            desired_retention: 0.9,
        }
    }
}

impl FsrsScheduler {
    /// Create a new scheduler with optional custom parameters.
    ///
    /// # Arguments
    ///
    /// * `parameters` - Optional 17-element parameter array (uses defaults if None)
    /// * `desired_retention` - Target retention rate (0.0-1.0)
    pub fn new(parameters: Option<[f64; 17]>, desired_retention: f64) -> Self {
        Self {
            parameters: parameters.unwrap_or(DEFAULT_PARAMETERS),
            desired_retention: desired_retention.clamp(0.7, 0.99),
        }
    }

    /// Calculate retrievability using the power-law forgetting curve.
    ///
    /// ```text
    /// R(t) = (1 + t/(9s))^(-1)
    /// ```
    ///
    /// At t=0: R=1.0 (perfect recall)
    /// At t=9s: R=0.5 (50% retention - half-life)
    ///
    /// # Arguments
    ///
    /// * `elapsed_days` - Days since last review
    /// * `stability` - Current memory stability
    ///
    /// # Returns
    ///
    /// Retrievability (0.0-1.0)
    pub fn forgetting_curve(&self, elapsed_days: f64, stability: f64) -> f64 {
        if stability <= 0.0 {
            return 0.0;
        }
        (1.0 + elapsed_days / (9.0 * stability)).powf(-1.0)
    }

    /// Calculate initial stability based on first rating.
    pub fn init_stability(&self, rating: Rating) -> f64 {
        self.parameters[rating as usize - 1]
    }

    /// Calculate initial difficulty based on first rating.
    pub fn init_difficulty(&self, rating: Rating) -> f64 {
        self.parameters[4] - self.parameters[5] * (rating.as_f64() - 3.0)
    }

    /// Update difficulty after a review.
    ///
    /// Uses mean reversion to prevent runaway values.
    pub fn next_difficulty(&self, current_difficulty: f64, rating: Rating) -> f64 {
        let next_d = current_difficulty - self.parameters[6] * (rating.as_f64() - 3.0);
        self.mean_reversion(self.parameters[4], next_d)
            .clamp(1.0, 10.0)
    }

    /// Update stability after a review.
    ///
    /// Different formulas for "Again" vs other ratings.
    pub fn next_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
        rating: Rating,
    ) -> f64 {
        if rating == Rating::Again {
            // Stability reset formula for failures
            self.parameters[11]
                * difficulty.powf(-self.parameters[12])
                * ((stability + 1.0).powf(self.parameters[13]) - 1.0)
                * ((1.0 - retrievability) * self.parameters[14]).exp()
        } else {
            // Stability increase formula for recalls
            stability
                * (1.0
                    + (self.parameters[8].exp()
                        * (11.0 - difficulty)
                        * stability.powf(-self.parameters[9])
                        * (((1.0 - retrievability) * self.parameters[10]).exp() - 1.0)))
        }
    }

    /// Calculate optimal interval to achieve desired retention.
    ///
    /// Derived from forgetting curve: R = (1 + t/(9s))^(-1)
    /// Solving for t: t = 9s * (R^(-1) - 1) = 9s * (1/R - 1)
    ///
    /// Since we use 0.1 as the base in calculation:
    /// interval = round(s / 0.1 * (1/R - 1))
    pub fn next_interval(&self, stability: f64) -> f64 {
        (stability / 0.1 * (self.desired_retention.recip() - 1.0))
            .round()
            .max(1.0)
    }

    /// Apply mean reversion to prevent extreme values.
    fn mean_reversion(&self, initial: f64, current: f64) -> f64 {
        self.parameters[7] * initial + (1.0 - self.parameters[7]) * current
    }

    /// Review a card and return updated state.
    ///
    /// # Arguments
    ///
    /// * `card` - Current card state
    /// * `rating` - User's rating
    /// * `elapsed_days` - Days since last review (or 0 for new cards)
    ///
    /// # Returns
    ///
    /// Updated card and review result
    pub fn review(&self, card: &Card, rating: Rating, elapsed_days: u64) -> ReviewResult {
        let mut new_card = card.clone();
        let elapsed = elapsed_days as f64;

        let retrievability = if card.state == CardState::New {
            // New card: initialize
            new_card.difficulty = self.init_difficulty(rating);
            new_card.stability = self.init_stability(rating);
            new_card.state = CardState::Learning;
            1.0
        } else {
            // Existing card: update
            let r = self.forgetting_curve(elapsed, card.stability);
            new_card.difficulty = self.next_difficulty(card.difficulty, rating);
            new_card.stability = self.next_stability(card.difficulty, card.stability, r, rating);

            if rating == Rating::Again {
                new_card.lapses += 1;
                new_card.state = CardState::Relearning;
            } else {
                new_card.state = CardState::Review;
            }
            r
        };

        let interval = self.next_interval(new_card.stability);
        new_card.scheduled_days = interval as u64;
        new_card.elapsed_days = elapsed_days;
        new_card.reps += 1;

        ReviewResult {
            card: new_card,
            rating,
            scheduled_days: interval as u64,
            retrievability,
        }
    }

    /// Predict retrievability at a future time.
    pub fn predict_retrievability(&self, card: &Card, days_from_now: u64) -> f64 {
        let total_elapsed = card.elapsed_days + days_from_now;
        self.forgetting_curve(total_elapsed as f64, card.stability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forgetting_curve_at_zero() {
        let scheduler = FsrsScheduler::default();
        let r = scheduler.forgetting_curve(0.0, 1.0);
        assert!((r - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_forgetting_curve_half_life() {
        // At t = 9s, R should be 0.5 (half-life)
        let scheduler = FsrsScheduler::default();
        let s = 1.0;
        let t = 9.0 * s; // 9 days for s=1
        let r = scheduler.forgetting_curve(t, s);
        assert!((r - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_forgetting_curve_decay() {
        let scheduler = FsrsScheduler::default();
        let r1 = scheduler.forgetting_curve(1.0, 1.0);
        let r2 = scheduler.forgetting_curve(5.0, 1.0);
        let r3 = scheduler.forgetting_curve(10.0, 1.0);

        // Retrievability should decrease over time
        assert!(r1 > r2);
        assert!(r2 > r3);
    }

    #[test]
    fn test_init_stability() {
        let scheduler = FsrsScheduler::default();

        // Higher ratings should give higher initial stability
        let s_again = scheduler.init_stability(Rating::Again);
        let s_hard = scheduler.init_stability(Rating::Hard);
        let s_good = scheduler.init_stability(Rating::Good);
        let s_easy = scheduler.init_stability(Rating::Easy);

        assert!(s_again < s_hard);
        assert!(s_hard < s_good);
        assert!(s_good < s_easy);
    }

    #[test]
    fn test_init_difficulty() {
        let scheduler = FsrsScheduler::default();

        // Higher ratings should give lower initial difficulty
        let d_again = scheduler.init_difficulty(Rating::Again);
        let d_good = scheduler.init_difficulty(Rating::Good);
        let d_easy = scheduler.init_difficulty(Rating::Easy);

        assert!(d_easy < d_good);
        assert!(d_good < d_again);
    }

    #[test]
    fn test_review_new_card() {
        let scheduler = FsrsScheduler::default();
        let card = Card::default();

        let result = scheduler.review(&card, Rating::Good, 0);

        assert_eq!(result.card.state, CardState::Learning);
        assert!(result.card.stability > 0.0);
        assert!(result.card.difficulty > 0.0);
        assert_eq!(result.card.reps, 1);
    }

    #[test]
    fn test_review_again_increases_lapses() {
        let scheduler = FsrsScheduler::default();
        let mut card = Card::default();
        card.state = CardState::Review;
        card.stability = 10.0;
        card.difficulty = 5.0;

        let result = scheduler.review(&card, Rating::Again, 5);

        assert_eq!(result.card.lapses, 1);
        assert_eq!(result.card.state, CardState::Relearning);
    }

    #[test]
    fn test_interval_increases_with_stability() {
        let scheduler = FsrsScheduler::default();

        let i1 = scheduler.next_interval(1.0);
        let i2 = scheduler.next_interval(10.0);
        let i3 = scheduler.next_interval(100.0);

        assert!(i1 < i2);
        assert!(i2 < i3);
    }

    #[test]
    fn test_difficulty_mean_reversion() {
        let scheduler = FsrsScheduler::default();

        // Multiple "Easy" ratings should decrease difficulty but not below 1.0
        let mut d = 8.0;
        for _ in 0..20 {
            d = scheduler.next_difficulty(d, Rating::Easy);
        }
        assert!(d >= 1.0);

        // Multiple "Again" ratings should increase difficulty but not above 10.0
        let mut d = 3.0;
        for _ in 0..20 {
            d = scheduler.next_difficulty(d, Rating::Again);
        }
        assert!(d <= 10.0);
    }
}
