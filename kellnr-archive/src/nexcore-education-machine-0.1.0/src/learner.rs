//! Learner state tracking.
//!
//! # T1 Grounding
//! - ς (state): Learner's overall learning state (Mutable)
//! - σ (sequence): Enrollment history is ordered
//! - μ (mapping): Maps subjects to enrollment states
//! - κ (comparison): Assessment records enable comparison
//! - N (quantity): Mastery levels are numeric
//! - ρ (recursion): Learning cycles loop through phases
//!
//! Tier: T3 (ς dominant — full domain entity)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::assessment::BayesianPrior;
use crate::types::{CompetencyLevel, LearningPhase, MasteryLevel, MasteryVerdict};

/// Assessment record — snapshot of a past assessment.
///
/// Tier: T2-P (N + κ — Quantity + Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRecord {
    /// When this assessment occurred (epoch seconds).
    pub timestamp: f64,
    /// Mastery level at time of assessment.
    pub mastery: MasteryLevel,
    /// Verdict rendered.
    pub verdict: MasteryVerdict,
    /// Questions correct out of total.
    pub correct: usize,
    /// Total questions.
    pub total: usize,
}

/// Enrollment in a specific subject.
///
/// Tier: T2-C (ς + μ — State + Mapping, Mutable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    /// Subject identifier.
    pub subject_id: String,
    /// Current learning phase.
    pub phase: LearningPhase,
    /// Bayesian prior for this subject.
    pub prior: BayesianPrior,
    /// Assessment history (ordered by time).
    pub assessments: Vec<AssessmentRecord>,
    /// When enrolled (epoch seconds).
    pub enrolled_at: f64,
    /// Current lesson index in the curriculum.
    pub current_lesson_index: usize,
}

impl Enrollment {
    /// Create a new enrollment.
    #[must_use]
    pub fn new(subject_id: impl Into<String>, enrolled_at: f64) -> Self {
        Self {
            subject_id: subject_id.into(),
            phase: LearningPhase::Discover,
            prior: BayesianPrior::default_prior(),
            assessments: Vec::new(),
            enrolled_at,
            current_lesson_index: 0,
        }
    }

    /// Current mastery probability.
    #[must_use]
    pub fn mastery_probability(&self) -> f64 {
        self.prior.mastery_probability()
    }

    /// Current verdict.
    #[must_use]
    pub fn current_verdict(&self) -> MasteryVerdict {
        MasteryVerdict::from_level(self.mastery_probability())
    }

    /// Current competency level.
    #[must_use]
    pub fn competency(&self) -> CompetencyLevel {
        CompetencyLevel::from_mastery(self.mastery_probability())
    }

    /// Record an assessment result.
    pub fn record_assessment(
        &mut self,
        timestamp: f64,
        mastery: MasteryLevel,
        verdict: MasteryVerdict,
        correct: usize,
        total: usize,
    ) {
        self.assessments.push(AssessmentRecord {
            timestamp,
            mastery,
            verdict,
            correct,
            total,
        });
    }

    /// Number of assessments taken.
    #[must_use]
    pub fn assessment_count(&self) -> usize {
        self.assessments.len()
    }

    /// Advance to next lesson in curriculum.
    pub fn advance_lesson(&mut self) {
        self.current_lesson_index += 1;
    }
}

impl fmt::Display for Enrollment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] mastery={:.0}% ({})",
            self.subject_id,
            self.phase,
            self.mastery_probability() * 100.0,
            self.competency(),
        )
    }
}

/// A learner — the central domain entity.
///
/// Tier: T3 (ς dominant — σμρςNκ composition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learner {
    /// Unique learner identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Subject enrollments.
    pub enrollments: Vec<Enrollment>,
    /// When the learner was created (epoch seconds).
    pub created_at: f64,
}

impl Learner {
    /// Create a new learner.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, created_at: f64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            enrollments: Vec::new(),
            created_at,
        }
    }

    /// Enroll in a subject.
    pub fn enroll(&mut self, subject_id: impl Into<String>, timestamp: f64) {
        let sid = subject_id.into();
        // Only enroll if not already enrolled
        if !self.enrollments.iter().any(|e| e.subject_id == sid) {
            self.enrollments.push(Enrollment::new(sid, timestamp));
        }
    }

    /// Get enrollment for a subject.
    #[must_use]
    pub fn enrollment(&self, subject_id: &str) -> Option<&Enrollment> {
        self.enrollments.iter().find(|e| e.subject_id == subject_id)
    }

    /// Get mutable enrollment for a subject.
    pub fn enrollment_mut(&mut self, subject_id: &str) -> Option<&mut Enrollment> {
        self.enrollments
            .iter_mut()
            .find(|e| e.subject_id == subject_id)
    }

    /// Number of active enrollments.
    #[must_use]
    pub fn enrollment_count(&self) -> usize {
        self.enrollments.len()
    }

    /// Average mastery across all enrollments.
    #[must_use]
    pub fn average_mastery(&self) -> f64 {
        if self.enrollments.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .enrollments
            .iter()
            .map(|e| e.mastery_probability())
            .sum();
        sum / self.enrollments.len() as f64
    }

    /// Count subjects at each verdict level.
    #[must_use]
    pub fn verdict_counts(&self) -> (usize, usize, usize) {
        let mut mastered = 0;
        let mut developing = 0;
        let mut remediate = 0;
        for e in &self.enrollments {
            match e.current_verdict() {
                MasteryVerdict::Mastered => mastered += 1,
                MasteryVerdict::Developing => developing += 1,
                MasteryVerdict::Remediate => remediate += 1,
            }
        }
        (mastered, developing, remediate)
    }
}

impl fmt::Display for Learner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (m, d, r) = self.verdict_counts();
        write!(
            f,
            "{} ({} subjects: {} mastered, {} developing, {} remediate)",
            self.name,
            self.enrollment_count(),
            m,
            d,
            r,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_learner() {
        let learner = Learner::new("l1", "Alice", 1000.0);
        assert_eq!(learner.id, "l1");
        assert_eq!(learner.name, "Alice");
        assert_eq!(learner.enrollment_count(), 0);
    }

    #[test]
    fn enroll_in_subject() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("rust-101", 1001.0);
        assert_eq!(learner.enrollment_count(), 1);
        assert!(learner.enrollment("rust-101").is_some());
    }

    #[test]
    fn no_duplicate_enrollment() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("rust-101", 1001.0);
        learner.enroll("rust-101", 1002.0); // duplicate
        assert_eq!(learner.enrollment_count(), 1);
    }

    #[test]
    fn enrollment_starts_at_discover() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("rust-101", 1001.0);
        let enrollment = learner.enrollment("rust-101");
        assert!(enrollment.is_some());
        if let Some(e) = enrollment {
            assert_eq!(e.phase, LearningPhase::Discover);
        }
    }

    #[test]
    fn average_mastery_empty() {
        let learner = Learner::new("l1", "Alice", 1000.0);
        assert!((learner.average_mastery() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_mastery_with_enrollments() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("subj1", 1001.0);
        learner.enroll("subj2", 1002.0);
        // Default prior: 0.5 each, so average = 0.5
        assert!((learner.average_mastery() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn verdict_counts() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("subj1", 1001.0);
        learner.enroll("subj2", 1002.0);
        // Both start with default prior (0.5) = Developing
        let (m, d, r) = learner.verdict_counts();
        assert_eq!(m, 0);
        assert_eq!(d, 2);
        assert_eq!(r, 0);
    }

    #[test]
    fn enrollment_record_assessment() {
        let mut enrollment = Enrollment::new("rust-101", 1000.0);
        let mastery = MasteryLevel::new(0.75).unwrap_or(MasteryLevel::ZERO);
        enrollment.record_assessment(1001.0, mastery, MasteryVerdict::Developing, 3, 4);
        assert_eq!(enrollment.assessment_count(), 1);
    }

    #[test]
    fn enrollment_advance_lesson() {
        let mut enrollment = Enrollment::new("rust-101", 1000.0);
        assert_eq!(enrollment.current_lesson_index, 0);
        enrollment.advance_lesson();
        assert_eq!(enrollment.current_lesson_index, 1);
    }

    #[test]
    fn learner_display() {
        let mut learner = Learner::new("l1", "Alice", 1000.0);
        learner.enroll("subj1", 1001.0);
        let display = learner.to_string();
        assert!(display.contains("Alice"));
        assert!(display.contains("1 subjects"));
    }

    #[test]
    fn enrollment_display() {
        let enrollment = Enrollment::new("rust-101", 1000.0);
        let display = enrollment.to_string();
        assert!(display.contains("rust-101"));
        assert!(display.contains("Discover"));
    }
}
