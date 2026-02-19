//! T2-P newtypes and core enums for the education engine.
//!
//! # T1 Grounding
//! - N (quantity): MasteryLevel, Difficulty wrap f64 measures
//! - Σ (sum): LearningPhase, CompetencyLevel, MasteryVerdict, Grade enumerate categories
//! - κ (comparison): Verdicts compare mastery against thresholds

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::EduError;

// ===========================================================================
// Constants — Bayesian mastery thresholds
// ===========================================================================

/// Mastery threshold: P(M|D) >= 0.85 means MASTERED.
pub const MASTERY_THRESHOLD: f64 = 0.85;

/// Developing threshold: P(M|D) >= 0.50 means DEVELOPING.
pub const DEVELOPING_THRESHOLD: f64 = 0.50;

/// Default Bayesian prior alpha (initial correct evidence).
pub const DEFAULT_ALPHA: f64 = 1.0;

/// Default Bayesian prior beta (initial incorrect evidence).
pub const DEFAULT_BETA: f64 = 1.0;

/// Stability decay factor for spaced repetition.
pub const STABILITY_DECAY: f64 = 0.9;

/// Default review interval in hours.
pub const DEFAULT_INTERVAL_HOURS: f64 = 24.0;

// ===========================================================================
// MasteryLevel — T2-P newtype (N dominant)
// ===========================================================================

/// Mastery probability P(M|D) ∈ [0, 1].
///
/// Tier: T2-P (N — Quantity)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MasteryLevel(f64);

impl MasteryLevel {
    /// Create a new mastery level.
    ///
    /// # Errors
    /// Returns `EduError::InvalidMasteryLevel` if value is not in [0, 1].
    pub fn new(value: f64) -> Result<Self, EduError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(EduError::InvalidMasteryLevel { value })
        }
    }

    /// Inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Classify into a MasteryVerdict.
    #[must_use]
    pub fn verdict(self) -> MasteryVerdict {
        MasteryVerdict::from_level(self.0)
    }

    /// Zero mastery.
    pub const ZERO: Self = Self(0.0);

    /// Full mastery.
    pub const FULL: Self = Self(1.0);
}

impl fmt::Display for MasteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}%", self.0 * 100.0)
    }
}

// ===========================================================================
// Difficulty — T2-P newtype (N dominant)
// ===========================================================================

/// Difficulty level of a question or lesson ∈ [0, 1].
///
/// 0.0 = trivial, 1.0 = extremely hard.
///
/// Tier: T2-P (N — Quantity)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Difficulty(f64);

impl Difficulty {
    /// Create a new difficulty level.
    ///
    /// # Errors
    /// Returns `EduError::InvalidDifficulty` if value is not in [0, 1].
    pub fn new(value: f64) -> Result<Self, EduError> {
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(EduError::InvalidDifficulty { value })
        }
    }

    /// Inner value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Easy difficulty.
    pub const EASY: Self = Self(0.25);

    /// Medium difficulty.
    pub const MEDIUM: Self = Self(0.50);

    /// Hard difficulty.
    pub const HARD: Self = Self(0.75);
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if self.0 < 0.33 {
            "Easy"
        } else if self.0 < 0.66 {
            "Medium"
        } else {
            "Hard"
        };
        write!(f, "{label} ({:.0}%)", self.0 * 100.0)
    }
}

// ===========================================================================
// LearningPhase — 5-phase FSM enum (Σ dominant)
// ===========================================================================

/// The five phases of the learning process.
///
/// Tier: T2-P (Σ — Sum, categorizes learning stages)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningPhase {
    /// Survey domain, identify concepts. (PRIMA Reconnaissance + CEP SEE)
    Discover,
    /// Decompose to T1/T2/T3 primitives. (PRIMA Extraction + CEP DECOMPOSE)
    Extract,
    /// Recursive application with spaced repetition. (PRIMA Anchoring)
    Practice,
    /// Bayesian mastery evaluation P(M|D). (MVM pattern)
    Assess,
    /// Certified competency (verdict >= 0.85).
    Master,
}

impl LearningPhase {
    /// Ordinal position (0-indexed).
    #[must_use]
    pub fn ordinal(self) -> usize {
        match self {
            Self::Discover => 0,
            Self::Extract => 1,
            Self::Practice => 2,
            Self::Assess => 3,
            Self::Master => 4,
        }
    }

    /// All phases in order.
    #[must_use]
    pub fn all() -> &'static [LearningPhase] {
        &[
            Self::Discover,
            Self::Extract,
            Self::Practice,
            Self::Assess,
            Self::Master,
        ]
    }

    /// Next phase in sequence, if any.
    #[must_use]
    pub fn next(self) -> Option<LearningPhase> {
        match self {
            Self::Discover => Some(Self::Extract),
            Self::Extract => Some(Self::Practice),
            Self::Practice => Some(Self::Assess),
            Self::Assess => Some(Self::Master),
            Self::Master => None,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Discover => "Discover",
            Self::Extract => "Extract",
            Self::Practice => "Practice",
            Self::Assess => "Assess",
            Self::Master => "Master",
        }
    }
}

impl fmt::Display for LearningPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ===========================================================================
// CompetencyLevel — T2-P enum (Σ dominant)
// ===========================================================================

/// Competency classification (Dreyfus model simplified).
///
/// Tier: T2-P (Σ — Sum, modal classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompetencyLevel {
    /// No knowledge.
    Novice,
    /// Basic understanding.
    Beginner,
    /// Can apply with guidance.
    Intermediate,
    /// Independent application.
    Advanced,
    /// Teaching-level mastery.
    Expert,
}

impl CompetencyLevel {
    /// Map mastery level to competency.
    #[must_use]
    pub fn from_mastery(mastery: f64) -> Self {
        if mastery >= 0.95 {
            Self::Expert
        } else if mastery >= 0.85 {
            Self::Advanced
        } else if mastery >= 0.65 {
            Self::Intermediate
        } else if mastery >= 0.35 {
            Self::Beginner
        } else {
            Self::Novice
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
        }
    }
}

impl fmt::Display for CompetencyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ===========================================================================
// MasteryVerdict — T2-P enum (Σ dominant)
// ===========================================================================

/// Assessment verdict after Bayesian evaluation.
///
/// Tier: T2-P (Σ — Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MasteryVerdict {
    /// P(M|D) >= 0.85 — concept mastered.
    Mastered,
    /// 0.50 <= P(M|D) < 0.85 — progressing.
    Developing,
    /// P(M|D) < 0.50 — needs remediation.
    Remediate,
}

impl MasteryVerdict {
    /// Classify from mastery level value.
    #[must_use]
    pub fn from_level(level: f64) -> Self {
        if level >= MASTERY_THRESHOLD {
            Self::Mastered
        } else if level >= DEVELOPING_THRESHOLD {
            Self::Developing
        } else {
            Self::Remediate
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mastered => "Mastered",
            Self::Developing => "Developing",
            Self::Remediate => "Remediate",
        }
    }

    /// CSS color class for UI rendering.
    #[must_use]
    pub const fn color_class(&self) -> &'static str {
        match self {
            Self::Mastered => "text-green-400",
            Self::Developing => "text-yellow-400",
            Self::Remediate => "text-red-400",
        }
    }
}

impl fmt::Display for MasteryVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// ===========================================================================
// Grade — T2-P enum (Σ dominant)
// ===========================================================================

/// Self-reported response quality for spaced repetition.
///
/// Tier: T2-P (Σ — Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grade {
    /// Complete failure — no recall.
    Again,
    /// Significant difficulty — partial recall.
    Hard,
    /// Moderate effort — successful recall.
    Good,
    /// Effortless recall — instant response.
    Easy,
}

impl Grade {
    /// Numeric score [0, 3].
    #[must_use]
    pub fn score(self) -> u8 {
        match self {
            Self::Again => 0,
            Self::Hard => 1,
            Self::Good => 2,
            Self::Easy => 3,
        }
    }

    /// Stability multiplier for interval scheduling.
    #[must_use]
    pub fn stability_factor(self) -> f64 {
        match self {
            Self::Again => 0.2,
            Self::Hard => 0.8,
            Self::Good => 1.0,
            Self::Easy => 1.5,
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Again => f.write_str("Again"),
            Self::Hard => f.write_str("Hard"),
            Self::Good => f.write_str("Good"),
            Self::Easy => f.write_str("Easy"),
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── MasteryLevel ────────────────────────────────────────────────
    #[test]
    fn mastery_level_bounds() {
        assert!(MasteryLevel::new(-0.1).is_err());
        assert!(MasteryLevel::new(1.1).is_err());
        assert!(MasteryLevel::new(0.0).is_ok());
        assert!(MasteryLevel::new(1.0).is_ok());
        assert!(MasteryLevel::new(0.5).is_ok());
    }

    #[test]
    fn mastery_level_verdict() {
        let mastered = MasteryLevel::new(0.90).unwrap_or(MasteryLevel::ZERO);
        assert_eq!(mastered.verdict(), MasteryVerdict::Mastered);

        let developing = MasteryLevel::new(0.60).unwrap_or(MasteryLevel::ZERO);
        assert_eq!(developing.verdict(), MasteryVerdict::Developing);

        let remediate = MasteryLevel::new(0.30).unwrap_or(MasteryLevel::ZERO);
        assert_eq!(remediate.verdict(), MasteryVerdict::Remediate);
    }

    #[test]
    fn mastery_level_display() {
        let ml = MasteryLevel::new(0.85).unwrap_or(MasteryLevel::ZERO);
        assert_eq!(ml.to_string(), "85.00%");
    }

    // ── Difficulty ──────────────────────────────────────────────────
    #[test]
    fn difficulty_bounds() {
        assert!(Difficulty::new(-0.1).is_err());
        assert!(Difficulty::new(1.1).is_err());
        assert!(Difficulty::new(0.0).is_ok());
        assert!(Difficulty::new(1.0).is_ok());
    }

    #[test]
    fn difficulty_display() {
        let easy = Difficulty::EASY;
        assert!(easy.to_string().contains("Easy"));

        let hard = Difficulty::HARD;
        assert!(hard.to_string().contains("Hard"));
    }

    #[test]
    fn difficulty_constants() {
        assert!((Difficulty::EASY.value() - 0.25).abs() < f64::EPSILON);
        assert!((Difficulty::MEDIUM.value() - 0.50).abs() < f64::EPSILON);
        assert!((Difficulty::HARD.value() - 0.75).abs() < f64::EPSILON);
    }

    // ── LearningPhase ───────────────────────────────────────────────
    #[test]
    fn learning_phase_ordering() {
        assert_eq!(LearningPhase::Discover.ordinal(), 0);
        assert_eq!(LearningPhase::Master.ordinal(), 4);
    }

    #[test]
    fn learning_phase_next() {
        assert_eq!(LearningPhase::Discover.next(), Some(LearningPhase::Extract));
        assert_eq!(LearningPhase::Extract.next(), Some(LearningPhase::Practice));
        assert_eq!(LearningPhase::Practice.next(), Some(LearningPhase::Assess));
        assert_eq!(LearningPhase::Assess.next(), Some(LearningPhase::Master));
        assert_eq!(LearningPhase::Master.next(), None);
    }

    #[test]
    fn learning_phase_all() {
        let all = LearningPhase::all();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], LearningPhase::Discover);
        assert_eq!(all[4], LearningPhase::Master);
    }

    #[test]
    fn learning_phase_display() {
        assert_eq!(LearningPhase::Discover.to_string(), "Discover");
        assert_eq!(LearningPhase::Master.to_string(), "Master");
    }

    // ── CompetencyLevel ─────────────────────────────────────────────
    #[test]
    fn competency_from_mastery() {
        assert_eq!(CompetencyLevel::from_mastery(0.96), CompetencyLevel::Expert);
        assert_eq!(
            CompetencyLevel::from_mastery(0.90),
            CompetencyLevel::Advanced
        );
        assert_eq!(
            CompetencyLevel::from_mastery(0.70),
            CompetencyLevel::Intermediate
        );
        assert_eq!(
            CompetencyLevel::from_mastery(0.40),
            CompetencyLevel::Beginner
        );
        assert_eq!(CompetencyLevel::from_mastery(0.10), CompetencyLevel::Novice);
    }

    #[test]
    fn competency_display() {
        assert_eq!(CompetencyLevel::Expert.to_string(), "Expert");
        assert_eq!(CompetencyLevel::Novice.to_string(), "Novice");
    }

    // ── MasteryVerdict ──────────────────────────────────────────────
    #[test]
    fn verdict_thresholds() {
        assert_eq!(MasteryVerdict::from_level(0.85), MasteryVerdict::Mastered);
        assert_eq!(MasteryVerdict::from_level(0.84), MasteryVerdict::Developing);
        assert_eq!(MasteryVerdict::from_level(0.50), MasteryVerdict::Developing);
        assert_eq!(MasteryVerdict::from_level(0.49), MasteryVerdict::Remediate);
        assert_eq!(MasteryVerdict::from_level(0.0), MasteryVerdict::Remediate);
    }

    #[test]
    fn verdict_color_class() {
        assert_eq!(MasteryVerdict::Mastered.color_class(), "text-green-400");
        assert_eq!(MasteryVerdict::Developing.color_class(), "text-yellow-400");
        assert_eq!(MasteryVerdict::Remediate.color_class(), "text-red-400");
    }

    // ── Grade ───────────────────────────────────────────────────────
    #[test]
    fn grade_scores() {
        assert_eq!(Grade::Again.score(), 0);
        assert_eq!(Grade::Hard.score(), 1);
        assert_eq!(Grade::Good.score(), 2);
        assert_eq!(Grade::Easy.score(), 3);
    }

    #[test]
    fn grade_stability_factors() {
        assert!((Grade::Again.stability_factor() - 0.2).abs() < f64::EPSILON);
        assert!((Grade::Easy.stability_factor() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn grade_display() {
        assert_eq!(Grade::Again.to_string(), "Again");
        assert_eq!(Grade::Easy.to_string(), "Easy");
    }
}
