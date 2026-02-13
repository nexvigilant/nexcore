//! Learning content representation.
//!
//! # T1 Grounding
//! - μ (mapping): Lesson maps concepts to content
//! - σ (sequence): Steps form an ordered progression
//!
//! Tier: T2-C (μ + σ — Mapping dominant)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::Difficulty;

/// Mapping from a domain concept to its T1/T2/T3 primitive decomposition.
///
/// Tier: T2-P (μ — Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveMapping {
    /// The domain concept being decomposed.
    pub concept: String,
    /// Tier classification (T1, T2-P, T2-C, T3).
    pub tier: String,
    /// Primitive symbols involved (e.g., ["σ", "μ", "ρ"]).
    pub primitives: Vec<String>,
    /// Dominant primitive.
    pub dominant: String,
}

/// Type of lesson content.
///
/// Tier: T2-P (Σ — Sum, content categorization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LessonContent {
    /// Textual explanation.
    Text {
        /// The content body.
        body: String,
    },
    /// Primitive decomposition exercise.
    Decomposition {
        /// Concept to decompose.
        concept: String,
        /// Expected primitive mapping.
        expected: PrimitiveMapping,
    },
    /// Practice exercise.
    Exercise {
        /// Exercise prompt.
        prompt: String,
        /// Hints available.
        hints: Vec<String>,
        /// Reference solution.
        solution: String,
    },
}

/// A single step within a lesson.
///
/// Tier: T2-P (σ — Sequence position)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonStep {
    /// Step title.
    pub title: String,
    /// Content for this step.
    pub content: LessonContent,
    /// Ordinal position.
    pub order: usize,
}

/// A complete lesson.
///
/// Tier: T2-C (μ + σ — Mapping dominant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    /// Unique identifier.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Brief description.
    pub description: String,
    /// Subject this lesson belongs to.
    pub subject_id: String,
    /// Difficulty level.
    pub difficulty: Difficulty,
    /// Ordered steps.
    pub steps: Vec<LessonStep>,
    /// Primitive mappings taught in this lesson.
    pub mappings: Vec<PrimitiveMapping>,
    /// Prerequisites (lesson IDs).
    pub prerequisites: Vec<String>,
}

impl Lesson {
    /// Create a new lesson.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        subject_id: impl Into<String>,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            subject_id: subject_id.into(),
            difficulty,
            steps: Vec::new(),
            mappings: Vec::new(),
            prerequisites: Vec::new(),
        }
    }

    /// Add a text step.
    pub fn add_text_step(&mut self, title: impl Into<String>, body: impl Into<String>) {
        let order = self.steps.len();
        self.steps.push(LessonStep {
            title: title.into(),
            content: LessonContent::Text { body: body.into() },
            order,
        });
    }

    /// Add a primitive decomposition step.
    pub fn add_decomposition_step(
        &mut self,
        title: impl Into<String>,
        concept: impl Into<String>,
        expected: PrimitiveMapping,
    ) {
        let order = self.steps.len();
        self.steps.push(LessonStep {
            title: title.into(),
            content: LessonContent::Decomposition {
                concept: concept.into(),
                expected,
            },
            order,
        });
    }

    /// Add a practice exercise step.
    pub fn add_exercise_step(
        &mut self,
        title: impl Into<String>,
        prompt: impl Into<String>,
        solution: impl Into<String>,
    ) {
        let order = self.steps.len();
        self.steps.push(LessonStep {
            title: title.into(),
            content: LessonContent::Exercise {
                prompt: prompt.into(),
                hints: Vec::new(),
                solution: solution.into(),
            },
            order,
        });
    }

    /// Add a primitive mapping taught.
    pub fn add_mapping(&mut self, mapping: PrimitiveMapping) {
        self.mappings.push(mapping);
    }

    /// Number of steps.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

impl fmt::Display for Lesson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}, {} steps]",
            self.title,
            self.difficulty,
            self.steps.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_lesson() {
        let lesson = Lesson::new("l1", "Ownership", "rust-101", Difficulty::MEDIUM);
        assert_eq!(lesson.id, "l1");
        assert_eq!(lesson.title, "Ownership");
        assert_eq!(lesson.subject_id, "rust-101");
        assert_eq!(lesson.step_count(), 0);
    }

    #[test]
    fn add_text_step() {
        let mut lesson = Lesson::new("l1", "Ownership", "rust-101", Difficulty::EASY);
        lesson.add_text_step("Introduction", "Rust uses an ownership model...");
        assert_eq!(lesson.step_count(), 1);
        assert_eq!(lesson.steps[0].order, 0);
    }

    #[test]
    fn add_exercise_step() {
        let mut lesson = Lesson::new("l2", "Borrowing", "rust-101", Difficulty::MEDIUM);
        lesson.add_exercise_step(
            "Practice",
            "Write a function that borrows",
            "fn borrow(s: &str) {}",
        );
        assert_eq!(lesson.step_count(), 1);
        if let LessonContent::Exercise { prompt, .. } = &lesson.steps[0].content {
            assert!(prompt.contains("borrows"));
        }
    }

    #[test]
    fn add_decomposition_step() {
        let mut lesson = Lesson::new("l3", "Primitives", "meta", Difficulty::HARD);
        let mapping = PrimitiveMapping {
            concept: "HashMap".to_string(),
            tier: "T1".to_string(),
            primitives: vec!["μ".to_string()],
            dominant: "μ".to_string(),
        };
        lesson.add_decomposition_step("Decompose HashMap", "HashMap", mapping);
        assert_eq!(lesson.step_count(), 1);
    }

    #[test]
    fn lesson_display() {
        let mut lesson = Lesson::new("l1", "Ownership", "rust-101", Difficulty::MEDIUM);
        lesson.add_text_step("Intro", "text");
        let display = lesson.to_string();
        assert!(display.contains("Ownership"));
        assert!(display.contains("1 steps"));
    }

    #[test]
    fn multiple_steps_ordering() {
        let mut lesson = Lesson::new("l1", "Multi", "test", Difficulty::EASY);
        lesson.add_text_step("Step 1", "First");
        lesson.add_text_step("Step 2", "Second");
        lesson.add_exercise_step("Step 3", "Do this", "answer");
        assert_eq!(lesson.steps[0].order, 0);
        assert_eq!(lesson.steps[1].order, 1);
        assert_eq!(lesson.steps[2].order, 2);
    }
}
