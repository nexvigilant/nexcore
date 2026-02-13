//! # Skill Maturation Primitives
//!
//! Five types completing the skill maturation feedback loop:
//! ```text
//! PRACTICE(REPETITION + ADAPTATION) → measure CONSISTENCY → score TRANSFER
//! ```
//!
//! ## Tier Classification
//!
//! | Type | Tier | Codex |
//! |------|------|-------|
//! | [`Repetition`] | T2-P | I (QUANTIFY), IV (WRAP) |
//! | [`Adaptation`] | T2-P | I (QUANTIFY), IV (WRAP) |
//! | [`Practice`] | T2-C | IX (MEASURE) |
//! | [`Consistency`] | T2-C | IX (MEASURE) |
//! | [`Transfer`] | T2-C | IX (MEASURE) |

use nexcore_constants::{Confidence, Measured, Tier};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// ERROR TYPE
// ═══════════════════════════════════════════════════════════════════════════

/// Errors during maturation assessment.
#[derive(Debug, thiserror::Error)]
pub enum MaturationError {
    /// Not enough data points for computation.
    #[error("insufficient data: needed {needed}, have {have}")]
    InsufficientData {
        /// Minimum required.
        needed: usize,
        /// Actually provided.
        have: usize,
    },
    /// Goal value outside valid range.
    #[error("invalid goal: {0} (must be in [0.0, 1.0])")]
    InvalidGoal(f64),
    /// Referenced skill not found.
    #[error("skill not found: {0}")]
    SkillNotFound(String),
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-P: REPETITION
// ═══════════════════════════════════════════════════════════════════════════

/// Practice volume tracker (successes + failures).
///
/// Tier: T2-P (cross-domain primitive — applicable to any learning domain).
///
/// Codex I: `From<Repetition> for f64` → success rate.
/// Codex II: `tier() -> Tier::T2Primitive`.
/// Codex IV: Wraps `(u32, u32)` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repetition {
    reps: u32,
    lapses: u32,
}

impl Repetition {
    /// Zero practice.
    pub const ZERO: Self = Self { reps: 0, lapses: 0 };
    /// Novice threshold (< 3 attempts).
    pub const NOVICE: Self = Self { reps: 2, lapses: 1 };

    /// Create from known counts.
    #[must_use]
    pub const fn new(reps: u32, lapses: u32) -> Self {
        Self { reps, lapses }
    }

    /// Record a successful attempt.
    #[must_use]
    pub const fn record_success(self) -> Self {
        Self {
            reps: self.reps.saturating_add(1),
            lapses: self.lapses,
        }
    }

    /// Record a failed attempt.
    #[must_use]
    pub const fn record_failure(self) -> Self {
        Self {
            reps: self.reps,
            lapses: self.lapses.saturating_add(1),
        }
    }

    /// Total attempts (reps + lapses).
    #[must_use]
    pub const fn total_attempts(self) -> u32 {
        self.reps.saturating_add(self.lapses)
    }

    /// Proportion of successful attempts as [`Confidence`].
    #[must_use]
    pub fn success_rate(self) -> Confidence {
        let total = self.total_attempts();
        if total == 0 {
            return Confidence::NONE;
        }
        #[allow(clippy::cast_precision_loss)]
        Confidence::new(self.reps as f64 / total as f64)
    }

    /// Fewer than 3 total attempts.
    #[must_use]
    pub const fn is_novice(self) -> bool {
        self.total_attempts() < 3
    }

    /// 10 or more total attempts.
    #[must_use]
    pub const fn is_practiced(self) -> bool {
        self.total_attempts() >= 10
    }

    /// Codex II: Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Primitive
    }
}

impl Default for Repetition {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Codex I: Quantify as success rate.
impl From<Repetition> for f64 {
    fn from(r: Repetition) -> Self {
        r.success_rate().value()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-P: ADAPTATION
// ═══════════════════════════════════════════════════════════════════════════

/// Improvement rate in range [-1.0, 1.0].
///
/// Tier: T2-P (cross-domain primitive — tracks learning trajectory).
///
/// Negative = regressing, zero = plateau, positive = improving.
/// Unlike [`Confidence`], this supports negative values for regression.
///
/// Codex I: `From<Adaptation> for f64` → raw rate.
/// Codex IV: Wraps `f64` with clamped range.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Adaptation {
    rate: f64,
}

impl Adaptation {
    /// Maximum improvement.
    pub const IMPROVING: Self = Self { rate: 1.0 };
    /// No change.
    pub const PLATEAU: Self = Self { rate: 0.0 };
    /// Maximum regression.
    pub const REGRESSING: Self = Self { rate: -1.0 };

    /// Create with clamping to [-1.0, 1.0].
    #[must_use]
    pub fn new(rate: f64) -> Self {
        Self {
            rate: rate.clamp(-1.0, 1.0),
        }
    }

    /// Raw adaptation rate.
    #[must_use]
    pub const fn rate(self) -> f64 {
        self.rate
    }

    /// Rate > 0.05 (measurable improvement).
    #[must_use]
    pub fn is_improving(self) -> bool {
        self.rate > 0.05
    }

    /// |rate| ≤ 0.05 (stalled).
    #[must_use]
    pub fn is_plateaued(self) -> bool {
        self.rate.abs() <= 0.05
    }

    /// Rate < -0.05 (measurable decline).
    #[must_use]
    pub fn is_regressing(self) -> bool {
        self.rate < -0.05
    }

    /// Compute adaptation from a sequence of outcome scores.
    ///
    /// Uses simple linear regression slope, normalized to [-1.0, 1.0].
    /// Returns [`Adaptation::PLATEAU`] for fewer than 2 outcomes.
    #[must_use]
    pub fn from_outcomes(outcomes: &[f64]) -> Self {
        if outcomes.len() < 2 {
            return Self::PLATEAU;
        }

        let n = outcomes.len() as f64;
        let mut sum_x = 0.0_f64;
        let mut sum_y = 0.0_f64;
        let mut sum_xy = 0.0_f64;
        let mut sum_xx = 0.0_f64;

        for (i, &y) in outcomes.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let sum_x_sq = sum_x * sum_x;
        let denom = n * sum_xx - sum_x_sq;
        if denom.abs() < f64::EPSILON {
            return Self::PLATEAU;
        }

        let sum_xy_cross = sum_x * sum_y;
        let slope = (n * sum_xy - sum_xy_cross) / denom;

        // Normalize: slope per step relative to value range
        let max_val = outcomes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_val = outcomes.iter().cloned().fold(f64::INFINITY, f64::min);
        let range = max_val - min_val;

        let normalized = if range.abs() < f64::EPSILON {
            0.0
        } else {
            (slope * (n - 1.0)) / range
        };

        Self::new(normalized)
    }

    /// Codex II: Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Primitive
    }
}

impl Default for Adaptation {
    fn default() -> Self {
        Self::PLATEAU
    }
}

/// Codex I: Quantify as raw rate.
impl From<Adaptation> for f64 {
    fn from(a: Adaptation) -> Self {
        a.rate
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-C: PRACTICE
// ═══════════════════════════════════════════════════════════════════════════

/// Deliberate repetition toward a goal.
///
/// Tier: T2-C (composed from Repetition + Adaptation + Confidence + Measured).
///
/// Codex IX: `current_level` is `Measured<Confidence>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Practice {
    /// Identifier for the skill being practiced.
    pub skill_id: String,
    /// Volume of practice.
    pub repetition: Repetition,
    /// Improvement trajectory.
    pub adaptation: Adaptation,
    /// Target competency level.
    pub goal: Confidence,
    /// Current observed level with uncertainty.
    pub current_level: Measured<Confidence>,
    /// Unix epoch seconds of last practice.
    pub last_practiced_epoch: u64,
}

impl Practice {
    /// Start practicing a new skill toward a goal.
    ///
    /// Returns `Err` if goal is outside [0.0, 1.0] (though Confidence clamps,
    /// we reject clearly invalid inputs like NaN/Inf).
    pub fn new(skill_id: impl Into<String>, goal: f64) -> Result<Self, MaturationError> {
        if goal.is_nan() || goal.is_infinite() {
            return Err(MaturationError::InvalidGoal(goal));
        }
        Ok(Self {
            skill_id: skill_id.into(),
            repetition: Repetition::ZERO,
            adaptation: Adaptation::PLATEAU,
            goal: Confidence::new(goal),
            current_level: Measured::new(Confidence::NONE, Confidence::LOW),
            last_practiced_epoch: 0,
        })
    }

    /// Record an attempt outcome in [0.0, 1.0].
    ///
    /// Updates repetition (success if outcome ≥ 0.5), adaptation, and current level.
    pub fn record_attempt(&mut self, outcome: f64, outcomes_history: &[f64]) {
        if outcome >= 0.5 {
            self.repetition = self.repetition.record_success();
        } else {
            self.repetition = self.repetition.record_failure();
        }

        self.adaptation = Adaptation::from_outcomes(outcomes_history);

        // Confidence in the level estimate grows with sample size
        #[allow(clippy::cast_precision_loss)]
        let sample_confidence = (self.repetition.total_attempts() as f64 / 30.0).min(1.0);
        self.current_level = Measured::new(
            Confidence::new(outcome),
            Confidence::new(sample_confidence),
        );
    }

    /// Progress toward goal as [`Confidence`] ratio.
    #[must_use]
    pub fn progress(&self) -> Confidence {
        let goal_val = self.goal.value();
        if goal_val < f64::EPSILON {
            return Confidence::PERFECT;
        }
        Confidence::new(self.current_level.value.value() / goal_val)
    }

    /// Whether current level meets or exceeds goal.
    #[must_use]
    pub fn has_reached_goal(&self) -> bool {
        self.current_level.value.value() >= self.goal.value()
    }

    /// Builder: set a new goal.
    #[must_use]
    pub fn with_goal(mut self, goal: Confidence) -> Self {
        self.goal = goal;
        self
    }

    /// Codex II: Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Composite
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-C: CONSISTENCY
// ═══════════════════════════════════════════════════════════════════════════

/// Outcome reliability across attempts.
///
/// Tier: T2-C (composed from Confidence + statistics).
///
/// Score = 1.0 − normalized standard deviation.
/// Higher = more consistent outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Consistency {
    score: Confidence,
    sample_size: u32,
    variance: f64,
}

impl Consistency {
    /// Compute from a sequence of outcomes in [0.0, 1.0].
    ///
    /// Returns `Err(InsufficientData)` if empty.
    pub fn from_outcomes(outcomes: &[f64]) -> Result<Self, MaturationError> {
        if outcomes.is_empty() {
            return Err(MaturationError::InsufficientData {
                needed: 1,
                have: 0,
            });
        }

        let n = outcomes.len() as f64;
        let mean = outcomes.iter().sum::<f64>() / n;
        let variance = if outcomes.len() == 1 {
            0.0
        } else {
            outcomes.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)
        };
        let std_dev = variance.sqrt();

        // Normalize: for [0,1] outcomes, max std_dev ≈ 0.5
        let normalized_std = (std_dev / 0.5).min(1.0);

        #[allow(clippy::cast_possible_truncation)]
        Ok(Self {
            score: Confidence::new(1.0 - normalized_std),
            sample_size: outcomes.len() as u32,
            variance: variance.clamp(0.0, 1.0),
        })
    }

    /// Consistency score (1.0 = perfectly consistent).
    #[must_use]
    pub const fn score(&self) -> Confidence {
        self.score
    }

    /// Number of observations.
    #[must_use]
    pub const fn sample_size(&self) -> u32 {
        self.sample_size
    }

    /// Outcome variance [0.0, 1.0].
    #[must_use]
    pub const fn variance(&self) -> f64 {
        self.variance
    }

    /// Score ≥ 0.8.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.score.value() >= 0.8
    }

    /// Score < 0.4.
    #[must_use]
    pub fn is_unreliable(&self) -> bool {
        self.score.value() < 0.4
    }

    /// Sample size ≥ 30 (CLT threshold).
    #[must_use]
    pub const fn is_significant(&self) -> bool {
        self.sample_size >= 30
    }

    /// Codex II: Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Composite
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T2-C: TRANSFER
// ═══════════════════════════════════════════════════════════════════════════

/// Cross-context portability of a skill.
///
/// Tier: T2-C (composed from Confidence + Measured + context pair).
///
/// Theoretical confidence = structural×0.4 + functional×0.4 + contextual×0.2.
/// This formula is inlined (no stem-core dependency).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Where the skill was learned.
    pub source_context: String,
    /// Where the skill is applied.
    pub target_context: String,
    /// Structural similarity between contexts.
    pub structural: Confidence,
    /// Functional overlap between contexts.
    pub functional: Confidence,
    /// Contextual factors (culture, tooling, etc.).
    pub contextual: Confidence,
    /// Observed transfer success with uncertainty.
    pub observed_success: Measured<Confidence>,
    /// Number of transfer attempts.
    pub attempts: u32,
}

impl Transfer {
    /// Create a new transfer assessment with default confidences.
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source_context: source.into(),
            target_context: target.into(),
            structural: Confidence::UNCERTAIN,
            functional: Confidence::UNCERTAIN,
            contextual: Confidence::UNCERTAIN,
            observed_success: Measured::new(Confidence::NONE, Confidence::NONE),
            attempts: 0,
        }
    }

    /// Predicted transfer confidence: s×0.4 + f×0.4 + c×0.2.
    #[must_use]
    pub fn theoretical_confidence(&self) -> Confidence {
        let val = self.structural.value() * 0.4
            + self.functional.value() * 0.4
            + self.contextual.value() * 0.2;
        Confidence::new(val)
    }

    /// Record a transfer attempt.
    pub fn record_attempt(&mut self, success: bool) {
        self.attempts = self.attempts.saturating_add(1);

        // Running success proportion
        let prev = self.observed_success.value.value();
        #[allow(clippy::cast_precision_loss)]
        let n = self.attempts as f64;
        let outcome = if success { 1.0 } else { 0.0 };
        let new_rate = prev + (outcome - prev) / n;

        // Confidence grows with observations
        let obs_confidence = (n / 30.0).min(1.0);

        self.observed_success = Measured::new(
            Confidence::new(new_rate),
            Confidence::new(obs_confidence),
        );
    }

    /// Best available transfer estimate (observed if enough data, else theoretical).
    #[must_use]
    pub fn effective_transfer(&self) -> Measured<Confidence> {
        if self.attempts >= 5 {
            self.observed_success
        } else {
            // Low confidence in purely theoretical estimate
            Measured::new(self.theoretical_confidence(), Confidence::LOW)
        }
    }

    /// Which dimension limits the transfer most.
    #[must_use]
    pub fn limiting_factor(&self) -> &str {
        let s = self.structural.value();
        let f = self.functional.value();
        let c = self.contextual.value();

        if s <= f && s <= c {
            "structural"
        } else if f <= c {
            "functional"
        } else {
            "contextual"
        }
    }

    /// Codex II: Tier classification.
    #[must_use]
    pub const fn tier() -> Tier {
        Tier::T2Composite
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Positive ──────────────────────────────────────────────────────────

    #[test]
    fn repetition_tracks_success_and_failure() {
        let r = Repetition::ZERO.record_success().record_success().record_failure();
        assert_eq!(r.reps, 2);
        assert_eq!(r.lapses, 1);
        assert_eq!(r.total_attempts(), 3);
    }

    #[test]
    fn repetition_success_rate() {
        let r = Repetition::new(7, 3);
        let rate = r.success_rate();
        assert!((rate.value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn adaptation_from_improving_outcomes() {
        let outcomes = vec![0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let a = Adaptation::from_outcomes(&outcomes);
        assert!(a.is_improving(), "rate={}", a.rate());
    }

    #[test]
    fn adaptation_from_declining_outcomes() {
        let outcomes = vec![0.9, 0.8, 0.7, 0.6, 0.5, 0.4];
        let a = Adaptation::from_outcomes(&outcomes);
        assert!(a.is_regressing(), "rate={}", a.rate());
    }

    #[test]
    fn adaptation_plateau_detection() {
        let outcomes = vec![0.5, 0.5, 0.5, 0.5];
        let a = Adaptation::from_outcomes(&outcomes);
        assert!(a.is_plateaued(), "rate={}", a.rate());
    }

    #[test]
    fn practice_records_attempt() {
        let mut p = Practice::new("rust-lifetimes", 0.9).unwrap();
        p.record_attempt(0.6, &[0.4, 0.5, 0.6]);
        assert_eq!(p.repetition.reps, 1);
        assert_eq!(p.repetition.lapses, 0);
    }

    #[test]
    fn practice_progress_toward_goal() {
        let mut p = Practice::new("testing", 0.8).unwrap();
        p.record_attempt(0.4, &[0.4]);
        let progress = p.progress().value();
        assert!((progress - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn consistency_from_uniform_outcomes() {
        let outcomes = vec![0.8, 0.8, 0.8, 0.8, 0.8];
        let c = Consistency::from_outcomes(&outcomes).unwrap();
        assert!(c.score().value() > 0.95, "score={}", c.score().value());
        assert!(c.is_consistent());
    }

    #[test]
    fn consistency_from_varied_outcomes() {
        let outcomes = vec![0.1, 0.9, 0.2, 0.8, 0.1, 0.9];
        let c = Consistency::from_outcomes(&outcomes).unwrap();
        assert!(c.score().value() < 0.5, "score={}", c.score().value());
    }

    #[test]
    fn transfer_theoretical_confidence() {
        let mut t = Transfer::new("pharmacology", "toxicology");
        t.structural = Confidence::new(0.9);
        t.functional = Confidence::new(0.8);
        t.contextual = Confidence::new(0.7);
        let tc = t.theoretical_confidence();
        // 0.9*0.4 + 0.8*0.4 + 0.7*0.2 = 0.36 + 0.32 + 0.14 = 0.82
        assert!((tc.value() - 0.82).abs() < 1e-10);
    }

    #[test]
    fn transfer_records_attempts() {
        let mut t = Transfer::new("rust", "go");
        t.record_attempt(true);
        t.record_attempt(true);
        t.record_attempt(false);
        assert_eq!(t.attempts, 3);
        // 2/3 success
        assert!((t.observed_success.value.value() - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn serde_round_trip_repetition() {
        let r = Repetition::new(5, 2);
        let json = serde_json::to_string(&r).unwrap();
        let back: Repetition = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn serde_round_trip_adaptation() {
        let a = Adaptation::new(0.42);
        let json = serde_json::to_string(&a).unwrap();
        let back: Adaptation = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn serde_round_trip_practice() {
        let p = Practice::new("test-skill", 0.8).unwrap();
        let json = serde_json::to_string(&p).unwrap();
        let _back: Practice = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn serde_round_trip_consistency() {
        let c = Consistency::from_outcomes(&[0.5, 0.6, 0.7]).unwrap();
        let json = serde_json::to_string(&c).unwrap();
        let back: Consistency = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn serde_round_trip_transfer() {
        let t = Transfer::new("a", "b");
        let json = serde_json::to_string(&t).unwrap();
        let _back: Transfer = serde_json::from_str(&json).unwrap();
    }

    // ── Negative ──────────────────────────────────────────────────────────

    #[test]
    fn adaptation_clamps_out_of_range() {
        assert!((Adaptation::new(5.0).rate() - 1.0).abs() < f64::EPSILON);
        assert!((Adaptation::new(-3.0).rate() - (-1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn consistency_from_empty_returns_error() {
        let result = Consistency::from_outcomes(&[]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MaturationError::InsufficientData { needed: 1, have: 0 }
        ));
    }

    #[test]
    fn practice_nan_goal_handled() {
        let result = Practice::new("x", f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn transfer_zero_attempts_no_observed() {
        let t = Transfer::new("a", "b");
        assert_eq!(t.attempts, 0);
        assert!((t.observed_success.value.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn repetition_default_is_zero() {
        let r = Repetition::default();
        assert_eq!(r.total_attempts(), 0);
        assert!((r.success_rate().value() - 0.0).abs() < f64::EPSILON);
    }

    // ── Edge ──────────────────────────────────────────────────────────────

    #[test]
    fn adaptation_single_outcome_is_plateau() {
        let a = Adaptation::from_outcomes(&[0.5]);
        assert!(a.is_plateaued());
    }

    #[test]
    fn consistency_single_observation() {
        let c = Consistency::from_outcomes(&[0.7]).unwrap();
        // Single observation → zero variance → perfect consistency
        assert!((c.score().value() - 1.0).abs() < f64::EPSILON);
        assert_eq!(c.sample_size(), 1);
    }

    #[test]
    fn repetition_u32_max_saturates() {
        let r = Repetition::new(u32::MAX, 0).record_success();
        assert_eq!(r.reps, u32::MAX);
    }

    #[test]
    fn practice_goal_already_reached() {
        let mut p = Practice::new("trivial", 0.3).unwrap();
        p.record_attempt(0.9, &[0.9]);
        assert!(p.has_reached_goal());
    }

    #[test]
    fn transfer_same_context() {
        let mut t = Transfer::new("rust", "rust");
        t.structural = Confidence::PERFECT;
        t.functional = Confidence::PERFECT;
        t.contextual = Confidence::PERFECT;
        assert!((t.theoretical_confidence().value() - 1.0).abs() < f64::EPSILON);
    }

    // ── Tier Classification ───────────────────────────────────────────────

    #[test]
    fn tier_classifications_correct() {
        assert_eq!(Repetition::tier(), Tier::T2Primitive);
        assert_eq!(Adaptation::tier(), Tier::T2Primitive);
        assert_eq!(Practice::tier(), Tier::T2Composite);
        assert_eq!(Consistency::tier(), Tier::T2Composite);
        assert_eq!(Transfer::tier(), Tier::T2Composite);
    }
}
