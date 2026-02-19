//! Spaced repetition engine using FSRS-inspired scheduling.
//!
//! # Core Concept
//! Retrievability R(t) = e^(-t / S) where:
//! - t = time since last review (hours)
//! - S = stability (how long until R drops to target threshold)
//!
//! # T1 Grounding
//! - ρ (recursion): Review cycles repeat recursively
//! - ν (frequency): Scheduling manages review frequency
//! - N (quantity): Stability, interval are numeric
//! - ς (state): ReviewState tracks mutable study progress
//!
//! Tier: T2-C (ρ + ν + N + ς — Recursion dominant)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::{DEFAULT_INTERVAL_HOURS, Grade, STABILITY_DECAY};

/// Target retrievability threshold — schedule review when R drops below this.
pub const TARGET_RETRIEVABILITY: f64 = 0.90;

/// Minimum stability (hours) to prevent scheduling too frequently.
pub const MIN_STABILITY: f64 = 1.0;

/// Maximum stability cap (hours) — ~6 months.
pub const MAX_STABILITY: f64 = 4320.0;

/// Review state for a single item (concept/lesson).
///
/// Tier: T2-C (ρ + ν + N + ς — Recursion dominant, Mutable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewState {
    /// Item identifier (lesson or concept ID).
    pub item_id: String,
    /// Current stability in hours.
    pub stability: f64,
    /// Number of reviews completed.
    pub review_count: u32,
    /// Last review timestamp (epoch seconds).
    pub last_review: f64,
    /// Next scheduled review (epoch seconds).
    pub next_review: f64,
    /// Current interval in hours.
    pub interval_hours: f64,
}

impl ReviewState {
    /// Create a new review state for an item.
    #[must_use]
    pub fn new(item_id: impl Into<String>, now: f64) -> Self {
        Self {
            item_id: item_id.into(),
            stability: DEFAULT_INTERVAL_HOURS,
            review_count: 0,
            last_review: now,
            next_review: now + DEFAULT_INTERVAL_HOURS * 3600.0,
            interval_hours: DEFAULT_INTERVAL_HOURS,
        }
    }

    /// Compute current retrievability R(t) = e^(-t/S).
    ///
    /// `now` is the current time in epoch seconds.
    #[must_use]
    pub fn retrievability(&self, now: f64) -> f64 {
        let elapsed_hours = (now - self.last_review) / 3600.0;
        if elapsed_hours <= 0.0 || self.stability <= 0.0 {
            return 1.0;
        }
        (-elapsed_hours / self.stability).exp()
    }

    /// Whether this item is due for review.
    #[must_use]
    pub fn due_for_review(&self, now: f64) -> bool {
        now >= self.next_review
    }

    /// Schedule the next review after grading.
    pub fn schedule_review(&mut self, grade: Grade, now: f64) {
        self.review_count += 1;
        self.last_review = now;

        // Update stability based on grade
        let factor = grade.stability_factor();
        self.stability =
            (self.stability * factor * STABILITY_DECAY).clamp(MIN_STABILITY, MAX_STABILITY);

        // Compute new interval from stability
        // interval = -S * ln(target_R)
        self.interval_hours = -self.stability * TARGET_RETRIEVABILITY.ln();

        // Schedule next review
        self.next_review = now + self.interval_hours * 3600.0;
    }

    /// Hours until next review.
    #[must_use]
    pub fn hours_until_review(&self, now: f64) -> f64 {
        (self.next_review - now) / 3600.0
    }

    /// Whether retrievability has dropped below target.
    #[must_use]
    pub fn needs_reinforcement(&self, now: f64) -> bool {
        self.retrievability(now) < TARGET_RETRIEVABILITY
    }
}

impl fmt::Display for ReviewState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [S={:.1}h, reviews={}, interval={:.1}h]",
            self.item_id, self.stability, self.review_count, self.interval_hours,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_review_state() {
        let state = ReviewState::new("lesson-1", 1000.0);
        assert_eq!(state.item_id, "lesson-1");
        assert_eq!(state.review_count, 0);
        assert!((state.stability - DEFAULT_INTERVAL_HOURS).abs() < f64::EPSILON);
    }

    #[test]
    fn retrievability_at_zero_time() {
        let state = ReviewState::new("l1", 1000.0);
        let r = state.retrievability(1000.0);
        assert!((r - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn retrievability_decays_over_time() {
        let state = ReviewState::new("l1", 0.0);
        let r_early = state.retrievability(3600.0); // 1 hour later
        let r_late = state.retrievability(86400.0); // 24 hours later
        assert!(r_early > r_late);
    }

    #[test]
    fn retrievability_stays_in_range() {
        let state = ReviewState::new("l1", 0.0);
        let r = state.retrievability(1_000_000.0); // very long time
        assert!(r >= 0.0);
        assert!(r <= 1.0);
    }

    #[test]
    fn due_for_review() {
        let state = ReviewState::new("l1", 0.0);
        assert!(!state.due_for_review(0.0));
        // After interval passes
        let after_interval = state.next_review + 1.0;
        assert!(state.due_for_review(after_interval));
    }

    #[test]
    fn schedule_review_updates_state() {
        let mut state = ReviewState::new("l1", 0.0);
        let review_time = 86400.0; // 24 hours later
        state.schedule_review(Grade::Good, review_time);
        assert_eq!(state.review_count, 1);
        assert!((state.last_review - review_time).abs() < f64::EPSILON);
        assert!(state.next_review > review_time);
    }

    #[test]
    fn easy_grade_increases_stability() {
        let mut state1 = ReviewState::new("l1", 0.0);
        let mut state2 = ReviewState::new("l2", 0.0);
        state1.schedule_review(Grade::Easy, 86400.0);
        state2.schedule_review(Grade::Hard, 86400.0);
        // Easy should have higher stability than Hard
        assert!(state1.stability > state2.stability);
    }

    #[test]
    fn again_grade_decreases_stability() {
        let mut state = ReviewState::new("l1", 0.0);
        let initial_stability = state.stability;
        state.schedule_review(Grade::Again, 86400.0);
        assert!(state.stability < initial_stability);
    }

    #[test]
    fn stability_clamped() {
        let mut state = ReviewState::new("l1", 0.0);
        // Many "Again" grades should hit minimum
        for i in 0..50 {
            state.schedule_review(Grade::Again, (i as f64 + 1.0) * 3600.0);
        }
        assert!(state.stability >= MIN_STABILITY);
    }

    #[test]
    fn review_state_display() {
        let state = ReviewState::new("lesson-1", 0.0);
        let display = state.to_string();
        assert!(display.contains("lesson-1"));
        assert!(display.contains("reviews=0"));
    }
}
