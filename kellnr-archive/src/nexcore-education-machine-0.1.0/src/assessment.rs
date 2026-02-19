//! Bayesian mastery assessment engine.
//!
//! # Core Formula
//! ```text
//! P(M|D) = alpha / (alpha + beta)
//! ```
//! - Correct answer at difficulty d: alpha += d
//! - Incorrect answer at difficulty d: beta += (1 - d)
//!
//! # T1 Grounding
//! - κ (comparison): Questions compare learner response to expected answer
//! - N (quantity): Alpha/beta are numeric evidence accumulators
//! - ∂ (boundary): Thresholds partition mastery into verdicts

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::EduError;
use crate::types::{DEFAULT_ALPHA, DEFAULT_BETA, Difficulty, MasteryLevel, MasteryVerdict};

/// Bayesian prior state for mastery estimation.
///
/// Uses Beta(alpha, beta) distribution where:
/// - alpha = accumulated evidence for mastery
/// - beta = accumulated evidence against mastery
///
/// Tier: T2-P (N — Quantity, Mutable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianPrior {
    /// Evidence for mastery (correct responses weighted by difficulty).
    pub alpha: f64,
    /// Evidence against mastery (incorrect responses weighted by inverse difficulty).
    pub beta: f64,
}

impl BayesianPrior {
    /// Create with default priors (uninformative: alpha=1, beta=1).
    #[must_use]
    pub fn default_prior() -> Self {
        Self {
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
        }
    }

    /// Create with custom priors.
    #[must_use]
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self {
            alpha: alpha.max(0.01),
            beta: beta.max(0.01),
        }
    }

    /// Compute P(M|D) = alpha / (alpha + beta).
    #[must_use]
    pub fn mastery_probability(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Update with a correct answer at given difficulty.
    pub fn update_correct(&mut self, difficulty: Difficulty) {
        self.alpha += difficulty.value();
    }

    /// Update with an incorrect answer at given difficulty.
    pub fn update_incorrect(&mut self, difficulty: Difficulty) {
        self.beta += 1.0 - difficulty.value();
    }

    /// Total evidence accumulated.
    #[must_use]
    pub fn total_evidence(&self) -> f64 {
        self.alpha + self.beta
    }

    /// Confidence in the estimate (higher evidence = more confident).
    /// Returns a value in [0, 1] that asymptotically approaches 1.
    #[must_use]
    pub fn confidence(&self) -> f64 {
        let total = self.total_evidence();
        // Logistic growth: rapid increase from 2 (default) to ~20 evidence
        1.0 - 1.0 / (1.0 + total / 5.0)
    }
}

impl Default for BayesianPrior {
    fn default() -> Self {
        Self::default_prior()
    }
}

impl fmt::Display for BayesianPrior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "P(M|D)={:.2} [α={:.1}, β={:.1}]",
            self.mastery_probability(),
            self.alpha,
            self.beta,
        )
    }
}

/// A single assessment question.
///
/// Tier: T2-P (κ — Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Unique question identifier.
    pub id: String,
    /// The question text.
    pub prompt: String,
    /// Expected correct answer.
    pub expected_answer: String,
    /// Difficulty level.
    pub difficulty: Difficulty,
    /// Concept being tested.
    pub concept: String,
}

/// Result of answering a single question.
///
/// Tier: T2-P (κ — Comparison)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResult {
    /// Question ID.
    pub question_id: String,
    /// Whether the answer was correct.
    pub correct: bool,
    /// The learner's answer.
    pub given_answer: String,
    /// Question difficulty.
    pub difficulty: Difficulty,
}

/// A complete assessment (set of questions).
///
/// Tier: T2-C (κ + σ — Comparison dominant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    /// Subject being assessed.
    pub subject_id: String,
    /// Questions in order.
    pub questions: Vec<Question>,
}

impl Assessment {
    /// Create a new assessment for a subject.
    #[must_use]
    pub fn new(subject_id: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            questions: Vec::new(),
        }
    }

    /// Add a question.
    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question);
    }

    /// Number of questions.
    #[must_use]
    pub fn question_count(&self) -> usize {
        self.questions.len()
    }
}

/// Result of a complete assessment.
///
/// Tier: T2-C (N + κ + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    /// Subject assessed.
    pub subject_id: String,
    /// Individual question results.
    pub results: Vec<QuestionResult>,
    /// Updated mastery level after assessment.
    pub mastery: MasteryLevel,
    /// Verdict.
    pub verdict: MasteryVerdict,
    /// Number correct.
    pub correct_count: usize,
    /// Total questions.
    pub total_count: usize,
}

/// Run a complete assessment: apply all question results to a prior and produce a verdict.
///
/// # Errors
/// Returns `EduError::EmptyAssessment` if results is empty.
pub fn evaluate_assessment(
    subject_id: &str,
    prior: &mut BayesianPrior,
    results: &[QuestionResult],
) -> Result<AssessmentResult, EduError> {
    if results.is_empty() {
        return Err(EduError::EmptyAssessment);
    }

    let mut correct_count = 0;
    for r in results {
        if r.correct {
            prior.update_correct(r.difficulty);
            correct_count += 1;
        } else {
            prior.update_incorrect(r.difficulty);
        }
    }

    let prob = prior.mastery_probability();
    let mastery = MasteryLevel::new(prob).unwrap_or(MasteryLevel::ZERO);
    let verdict = MasteryVerdict::from_level(prob);

    Ok(AssessmentResult {
        subject_id: subject_id.to_string(),
        results: results.to_vec(),
        mastery,
        verdict,
        correct_count,
        total_count: results.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── BayesianPrior ────────────────────────────────────────────────
    #[test]
    fn default_prior_is_uninformative() {
        let prior = BayesianPrior::default_prior();
        assert!((prior.mastery_probability() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn correct_increases_alpha() {
        let mut prior = BayesianPrior::default_prior();
        let before = prior.alpha;
        prior.update_correct(Difficulty::MEDIUM);
        assert!(prior.alpha > before);
    }

    #[test]
    fn incorrect_increases_beta() {
        let mut prior = BayesianPrior::default_prior();
        let before = prior.beta;
        prior.update_incorrect(Difficulty::MEDIUM);
        assert!(prior.beta > before);
    }

    #[test]
    fn many_correct_approaches_mastery() {
        let mut prior = BayesianPrior::default_prior();
        for _ in 0..20 {
            prior.update_correct(Difficulty::HARD);
        }
        assert!(prior.mastery_probability() > 0.85);
    }

    #[test]
    fn many_incorrect_approaches_remediate() {
        let mut prior = BayesianPrior::default_prior();
        for _ in 0..20 {
            prior.update_incorrect(Difficulty::EASY);
        }
        assert!(prior.mastery_probability() < 0.50);
    }

    #[test]
    fn hard_correct_contributes_more() {
        let mut prior_hard = BayesianPrior::default_prior();
        let mut prior_easy = BayesianPrior::default_prior();
        prior_hard.update_correct(Difficulty::HARD);
        prior_easy.update_correct(Difficulty::EASY);
        assert!(prior_hard.mastery_probability() > prior_easy.mastery_probability());
    }

    #[test]
    fn confidence_increases_with_evidence() {
        let prior_low = BayesianPrior::default_prior();
        let mut prior_high = BayesianPrior::default_prior();
        for _ in 0..10 {
            prior_high.update_correct(Difficulty::MEDIUM);
        }
        assert!(prior_high.confidence() > prior_low.confidence());
    }

    #[test]
    fn prior_display() {
        let prior = BayesianPrior::default_prior();
        let display = prior.to_string();
        assert!(display.contains("P(M|D)="));
    }

    #[test]
    fn prior_minimum_values() {
        let prior = BayesianPrior::new(-5.0, -10.0);
        assert!(prior.alpha >= 0.01);
        assert!(prior.beta >= 0.01);
    }

    // ── Assessment ──────────────────────────────────────────────────
    #[test]
    fn empty_assessment_errors() {
        let mut prior = BayesianPrior::default_prior();
        let result = evaluate_assessment("test", &mut prior, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn all_correct_gives_mastered() {
        let mut prior = BayesianPrior::new(5.0, 1.0); // start with some evidence
        let results: Vec<QuestionResult> = (0..10)
            .map(|i| QuestionResult {
                question_id: format!("q{i}"),
                correct: true,
                given_answer: "correct".to_string(),
                difficulty: Difficulty::MEDIUM,
            })
            .collect();
        let result = evaluate_assessment("test", &mut prior, &results);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.verdict, MasteryVerdict::Mastered);
            assert_eq!(r.correct_count, 10);
            assert_eq!(r.total_count, 10);
        }
    }

    #[test]
    fn all_incorrect_gives_remediate() {
        let mut prior = BayesianPrior::new(1.0, 5.0);
        let results: Vec<QuestionResult> = (0..10)
            .map(|i| QuestionResult {
                question_id: format!("q{i}"),
                correct: false,
                given_answer: "wrong".to_string(),
                difficulty: Difficulty::EASY,
            })
            .collect();
        let result = evaluate_assessment("test", &mut prior, &results);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.verdict, MasteryVerdict::Remediate);
            assert_eq!(r.correct_count, 0);
        }
    }

    #[test]
    fn mixed_results() {
        let mut prior = BayesianPrior::default_prior();
        let results = vec![
            QuestionResult {
                question_id: "q1".to_string(),
                correct: true,
                given_answer: "a".to_string(),
                difficulty: Difficulty::MEDIUM,
            },
            QuestionResult {
                question_id: "q2".to_string(),
                correct: false,
                given_answer: "b".to_string(),
                difficulty: Difficulty::MEDIUM,
            },
        ];
        let result = evaluate_assessment("test", &mut prior, &results);
        assert!(result.is_ok());
        if let Ok(r) = result {
            assert_eq!(r.correct_count, 1);
            assert_eq!(r.total_count, 2);
        }
    }

    #[test]
    fn assessment_builder() {
        let mut assess = Assessment::new("rust-101");
        assess.add_question(Question {
            id: "q1".to_string(),
            prompt: "What is ownership?".to_string(),
            expected_answer: "A memory management model".to_string(),
            difficulty: Difficulty::MEDIUM,
            concept: "ownership".to_string(),
        });
        assert_eq!(assess.question_count(), 1);
        assert_eq!(assess.subject_id, "rust-101");
    }
}
