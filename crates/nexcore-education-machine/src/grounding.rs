//! # GroundsTo implementations for nexcore-education-machine types
//!
//! Connects the education engine to the Lex Primitiva type system.
//!
//! ## Primitive Foundation: Education = σμρςNκ (T3, dominant σ)
//!
//! ## Dominant Primitive Distribution
//!
//! - `MasteryLevel`, `Difficulty`, `BayesianPrior` — Quantity (N) because
//!   they wrap numeric measurements.
//! - `LearningPhase`, `CompetencyLevel`, `MasteryVerdict`, `Grade` — Sum (Σ)
//!   because they categorize/partition a domain.
//! - `Question` — Comparison (κ) because questions compare learner answer
//!   to expected answer.
//! - `Subject` — Sequence (σ) because a subject's curriculum is ordered.
//! - `Lesson` — Mapping (μ) because lessons map concepts to content.
//! - `Enrollment` — State (ς) because enrollment tracks mutable learning state.
//! - `Assessment` — Comparison (κ) because assessment compares performance.
//! - `ReviewState` — Recursion (ρ) because review cycles repeat recursively.
//! - `Learner` — State (ς) full T3 domain entity.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::assessment::{Assessment, AssessmentResult, BayesianPrior, Question, QuestionResult};
use crate::learner::{AssessmentRecord, Enrollment, Learner};
use crate::lesson::{Lesson, LessonContent, LessonStep, PrimitiveMapping};
use crate::spaced_repetition::ReviewState;
use crate::state_machine::PhaseTransition;
use crate::subject::{LessonRef, Subject};
use crate::types::{
    CompetencyLevel, Difficulty, Grade, LearningPhase, MasteryLevel, MasteryVerdict,
};

// ---------------------------------------------------------------------------
// Quantity-dominant newtypes — N
// ---------------------------------------------------------------------------

/// MasteryLevel: T2-P (N), dominant N
///
/// P(M|D) ∈ [0, 1] — a single numeric measure of mastery.
impl GroundsTo for MasteryLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — mastery probability is numeric
            LexPrimitiva::Boundary, // ∂ — clamped to [0, 1]
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// Difficulty: T2-P (N), dominant N
///
/// Difficulty level ∈ [0, 1] — numeric measure of question/lesson hardness.
impl GroundsTo for Difficulty {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — numeric difficulty value
            LexPrimitiva::Boundary, // ∂ — clamped to [0, 1]
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// BayesianPrior: T2-P (N + ς), dominant N
///
/// Beta(alpha, beta) distribution parameters — accumulators of evidence.
impl GroundsTo for BayesianPrior {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — alpha, beta are numeric accumulators
            LexPrimitiva::State,    // ς — mutable evidence accumulation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Sum-dominant enums — Σ (categorization)
// ---------------------------------------------------------------------------

/// LearningPhase: T2-P (Σ), dominant Σ
///
/// Five phases: Discover → Extract → Practice → Assess → Master.
/// Sum-dominant because it partitions the learning process into discrete stages.
impl GroundsTo for LearningPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — 5 discrete categories
            LexPrimitiva::Sequence, // σ — ordered progression
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// CompetencyLevel: T2-P (Σ), dominant Σ (Modal)
///
/// Dreyfus model: Novice → Beginner → Intermediate → Advanced → Expert.
impl GroundsTo for CompetencyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — 5 discrete levels
            LexPrimitiva::Comparison, // κ — derived from mastery comparison
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// MasteryVerdict: T2-P (Σ), dominant Σ
///
/// Mastered / Developing / Remediate — assessment outcome classification.
impl GroundsTo for MasteryVerdict {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — 3 verdict categories
            LexPrimitiva::Comparison, // κ — threshold comparison to determine verdict
        ])
        .with_dominant(LexPrimitiva::Sum, 0.90)
    }
}

/// Grade: T2-P (Σ), dominant Σ
///
/// Again / Hard / Good / Easy — self-reported response quality.
impl GroundsTo for Grade {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — 4 grade categories
            LexPrimitiva::Quantity, // N — numeric score (0-3)
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Comparison-dominant types — κ
// ---------------------------------------------------------------------------

/// Question: T2-P (κ), dominant κ
///
/// A question compares learner response to expected answer.
impl GroundsTo for Question {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — comparing answer to expected
            LexPrimitiva::Quantity,   // N — difficulty is numeric
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// QuestionResult: T2-P (κ + N), dominant κ
impl GroundsTo for QuestionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — correct/incorrect comparison
            LexPrimitiva::Quantity,   // N — difficulty level
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Assessment: T2-C (κ + σ + N + Σ), dominant κ
///
/// A complete set of questions comparing learner understanding.
impl GroundsTo for Assessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — assessment compares performance
            LexPrimitiva::Sequence,   // σ — ordered questions
            LexPrimitiva::Quantity,   // N — difficulty values
            LexPrimitiva::Sum,        // Σ — aggregated verdict
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// AssessmentResult: T2-C (κ + N + Σ + ∂), dominant κ
impl GroundsTo for AssessmentResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — comparison outcomes
            LexPrimitiva::Quantity,   // N — mastery score, counts
            LexPrimitiva::Sum,        // Σ — verdict category
            LexPrimitiva::Boundary,   // ∂ — mastery thresholds
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Sequence/Mapping-dominant types — σ, μ
// ---------------------------------------------------------------------------

/// Subject: T2-C (σ + μ + Σ), dominant σ
///
/// Ordered curriculum mapping concepts to lessons.
impl GroundsTo for Subject {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ — ordered curriculum
            LexPrimitiva::Mapping,  // μ — concepts → lessons
            LexPrimitiva::Sum,      // Σ — categorization via tags
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// LessonRef: T2-P (σ + μ), dominant σ
impl GroundsTo for LessonRef {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ — position in curriculum
            LexPrimitiva::Mapping,  // μ — ID → lesson
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// Lesson: T2-C (μ + σ + N + κ), dominant μ
///
/// Maps concepts to content via ordered steps.
impl GroundsTo for Lesson {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ — concepts → content
            LexPrimitiva::Sequence,   // σ — ordered steps
            LexPrimitiva::Quantity,   // N — difficulty
            LexPrimitiva::Comparison, // κ — exercises compare responses
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// LessonStep: T2-P (σ + μ), dominant σ
impl GroundsTo for LessonStep {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ — step order
            LexPrimitiva::Mapping,  // μ — content mapping
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// LessonContent: T2-P (μ + Σ), dominant μ
impl GroundsTo for LessonContent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // μ — content maps concepts to presentation
            LexPrimitiva::Sum,     // Σ — Text/Decomposition/Exercise variants
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// PrimitiveMapping: T2-P (μ + κ), dominant μ
impl GroundsTo for PrimitiveMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ — concept → primitives
            LexPrimitiva::Comparison, // κ — tier classification
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

// ---------------------------------------------------------------------------
// State-dominant types — ς
// ---------------------------------------------------------------------------

/// Enrollment: T2-C (ς + μ + N + κ), dominant ς (Mutable)
///
/// Tracks a learner's mutable state within a subject.
impl GroundsTo for Enrollment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — phase, prior are mutable
            LexPrimitiva::Mapping,    // μ — subject → learning state
            LexPrimitiva::Quantity,   // N — mastery probability
            LexPrimitiva::Comparison, // κ — phase transitions, verdicts
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// AssessmentRecord: T2-P (N + κ), dominant N
impl GroundsTo for AssessmentRecord {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N — mastery, correct, total
            LexPrimitiva::Comparison, // κ — verdict comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Recursion-dominant types — ρ
// ---------------------------------------------------------------------------

/// ReviewState: T2-C (ρ + ν + N + ς), dominant ρ (Mutable)
///
/// Spaced repetition review cycle — recursively scheduling reviews.
impl GroundsTo for ReviewState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ — review cycles repeat
            LexPrimitiva::Frequency, // ν — scheduling frequency
            LexPrimitiva::Quantity,  // N — stability, interval
            LexPrimitiva::State,     // ς — mutable review state
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// FSM transition record
// ---------------------------------------------------------------------------

/// PhaseTransition: T2-C (ς + σ + κ + μ), dominant ς
impl GroundsTo for PhaseTransition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — state change record
            LexPrimitiva::Sequence,   // σ — ordered in time
            LexPrimitiva::Comparison, // κ — transition validation
            LexPrimitiva::Mapping,    // μ — from → to mapping
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

// ---------------------------------------------------------------------------
// T3 Domain Entity — Learner
// ---------------------------------------------------------------------------

/// Learner: T3 (ς + σ + μ + ρ + N + κ), dominant ς (Mutable)
///
/// The central domain entity composing all education primitives.
/// Education = σμρςNκ (6 unique primitives = T3).
impl GroundsTo for Learner {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς — overall learning state
            LexPrimitiva::Sequence,   // σ — curriculum progression
            LexPrimitiva::Mapping,    // μ — subject → enrollment mapping
            LexPrimitiva::Recursion,  // ρ — review cycles
            LexPrimitiva::Quantity,   // N — mastery levels
            LexPrimitiva::Comparison, // κ — assessment comparisons
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ═══════════════════════════════════════════════════════════════════
    // Tier Classification Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn mastery_level_is_t2p() {
        assert_eq!(MasteryLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn difficulty_is_t2p() {
        assert_eq!(Difficulty::tier(), Tier::T2Primitive);
    }

    #[test]
    fn bayesian_prior_is_t2p() {
        assert_eq!(BayesianPrior::tier(), Tier::T2Primitive);
    }

    #[test]
    fn learning_phase_is_t2p() {
        assert_eq!(LearningPhase::tier(), Tier::T2Primitive);
    }

    #[test]
    fn competency_level_is_t2p() {
        assert_eq!(CompetencyLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn mastery_verdict_is_t2p() {
        assert_eq!(MasteryVerdict::tier(), Tier::T2Primitive);
    }

    #[test]
    fn grade_is_t2p() {
        assert_eq!(Grade::tier(), Tier::T2Primitive);
    }

    #[test]
    fn question_is_t2p() {
        assert_eq!(Question::tier(), Tier::T2Primitive);
    }

    #[test]
    fn question_result_is_t2p() {
        assert_eq!(QuestionResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn lesson_ref_is_t2p() {
        assert_eq!(LessonRef::tier(), Tier::T2Primitive);
    }

    #[test]
    fn lesson_step_is_t2p() {
        assert_eq!(LessonStep::tier(), Tier::T2Primitive);
    }

    #[test]
    fn lesson_content_is_t2p() {
        assert_eq!(LessonContent::tier(), Tier::T2Primitive);
    }

    #[test]
    fn primitive_mapping_is_t2p() {
        assert_eq!(PrimitiveMapping::tier(), Tier::T2Primitive);
    }

    #[test]
    fn assessment_record_is_t2p() {
        assert_eq!(AssessmentRecord::tier(), Tier::T2Primitive);
    }

    #[test]
    fn subject_is_t2c() {
        assert_eq!(Subject::tier(), Tier::T2Primitive);
    }

    #[test]
    fn lesson_is_t2c() {
        assert_eq!(Lesson::tier(), Tier::T2Composite);
    }

    #[test]
    fn enrollment_is_t2c() {
        assert_eq!(Enrollment::tier(), Tier::T2Composite);
    }

    #[test]
    fn assessment_is_t2c() {
        assert_eq!(Assessment::tier(), Tier::T2Composite);
    }

    #[test]
    fn assessment_result_is_t2c() {
        assert_eq!(AssessmentResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn review_state_is_t2c() {
        assert_eq!(ReviewState::tier(), Tier::T2Composite);
    }

    #[test]
    fn phase_transition_is_t2c() {
        assert_eq!(PhaseTransition::tier(), Tier::T2Composite);
    }

    #[test]
    fn learner_is_t3() {
        assert_eq!(Learner::tier(), Tier::T3DomainSpecific);
    }

    // ═══════════════════════════════════════════════════════════════════
    // Dominant Primitive Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn mastery_level_dominant_is_quantity() {
        let comp = MasteryLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn difficulty_dominant_is_quantity() {
        let comp = Difficulty::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn learning_phase_dominant_is_sum() {
        let comp = LearningPhase::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn competency_dominant_is_sum() {
        let comp = CompetencyLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn verdict_dominant_is_sum() {
        let comp = MasteryVerdict::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn grade_dominant_is_sum() {
        let comp = Grade::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
    }

    #[test]
    fn question_dominant_is_comparison() {
        let comp = Question::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn subject_dominant_is_sequence() {
        let comp = Subject::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn lesson_dominant_is_mapping() {
        let comp = Lesson::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn enrollment_dominant_is_state() {
        let comp = Enrollment::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn review_state_dominant_is_recursion() {
        let comp = ReviewState::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
    }

    #[test]
    fn learner_dominant_is_state() {
        let comp = Learner::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Composition Content Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn learner_contains_all_six_primitives() {
        let comp = Learner::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::State));
        assert!(comp.primitives.contains(&LexPrimitiva::Sequence));
        assert!(comp.primitives.contains(&LexPrimitiva::Mapping));
        assert!(comp.primitives.contains(&LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Quantity));
        assert!(comp.primitives.contains(&LexPrimitiva::Comparison));
    }

    #[test]
    fn review_state_contains_frequency() {
        let comp = ReviewState::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Frequency));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Confidence Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn all_confidences_in_valid_range() {
        let compositions = [
            MasteryLevel::primitive_composition(),
            Difficulty::primitive_composition(),
            BayesianPrior::primitive_composition(),
            LearningPhase::primitive_composition(),
            CompetencyLevel::primitive_composition(),
            MasteryVerdict::primitive_composition(),
            Grade::primitive_composition(),
            Question::primitive_composition(),
            QuestionResult::primitive_composition(),
            Subject::primitive_composition(),
            LessonRef::primitive_composition(),
            Lesson::primitive_composition(),
            LessonStep::primitive_composition(),
            LessonContent::primitive_composition(),
            PrimitiveMapping::primitive_composition(),
            Assessment::primitive_composition(),
            AssessmentResult::primitive_composition(),
            Enrollment::primitive_composition(),
            AssessmentRecord::primitive_composition(),
            ReviewState::primitive_composition(),
            PhaseTransition::primitive_composition(),
            Learner::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(
                comp.confidence >= 0.80,
                "Confidence {} below 0.80",
                comp.confidence
            );
            assert!(
                comp.confidence <= 0.95,
                "Confidence {} above 0.95",
                comp.confidence
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // StateMode Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn mutable_types_have_mutable_state_mode() {
        assert_eq!(BayesianPrior::state_mode(), Some(StateMode::Mutable));
        assert_eq!(Enrollment::state_mode(), Some(StateMode::Mutable));
        assert_eq!(ReviewState::state_mode(), Some(StateMode::Mutable));
        assert_eq!(Learner::state_mode(), Some(StateMode::Mutable));
    }

    #[test]
    fn modal_types_have_modal_state_mode() {
        assert_eq!(CompetencyLevel::state_mode(), Some(StateMode::Modal));
        assert_eq!(PhaseTransition::state_mode(), Some(StateMode::Modal));
    }

    #[test]
    fn stateless_types_have_no_state_mode() {
        assert_eq!(MasteryLevel::state_mode(), None);
        assert_eq!(Difficulty::state_mode(), None);
        assert_eq!(Question::state_mode(), None);
        assert_eq!(Grade::state_mode(), None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // Tier matches primitive count cross-check
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn tier_matches_unique_primitive_count() {
        // T2-P: 2-3 unique primitives
        assert!(MasteryLevel::primitive_composition().unique().len() <= 3);
        assert!(Difficulty::primitive_composition().unique().len() <= 3);
        assert!(LearningPhase::primitive_composition().unique().len() <= 3);
        assert!(Grade::primitive_composition().unique().len() <= 3);
        assert!(Question::primitive_composition().unique().len() <= 3);

        // T2-C: 4-5 unique primitives
        let lesson_len = Lesson::primitive_composition().unique().len();
        assert!(lesson_len >= 4 && lesson_len <= 5);

        let enroll_len = Enrollment::primitive_composition().unique().len();
        assert!(enroll_len >= 4 && enroll_len <= 5);

        let review_len = ReviewState::primitive_composition().unique().len();
        assert!(review_len >= 4 && review_len <= 5);

        // T3: 6+ unique primitives
        assert!(Learner::primitive_composition().unique().len() >= 6);
    }
}
