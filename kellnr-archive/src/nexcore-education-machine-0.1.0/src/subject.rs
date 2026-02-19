//! Domain-agnostic subject representation.
//!
//! # T1 Grounding
//! - σ (sequence): Lessons form an ordered curriculum
//! - μ (mapping): Subject maps concepts to lessons
//!
//! Tier: T2-C (σ + μ)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Reference to a lesson within a subject.
///
/// Tier: T2-P (σ + μ — ordered mapping)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonRef {
    /// Unique lesson identifier.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Ordinal position in curriculum.
    pub order: usize,
}

/// A subject (domain of study).
///
/// Tier: T2-C (σ + μ — Sequence dominant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Brief description.
    pub description: String,
    /// Ordered lesson references forming the curriculum.
    pub lessons: Vec<LessonRef>,
    /// Domain tags for categorization.
    pub tags: Vec<String>,
}

impl Subject {
    /// Create a new subject.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            lessons: Vec::new(),
            tags: Vec::new(),
        }
    }

    /// Add a lesson reference to the curriculum.
    pub fn add_lesson(&mut self, id: impl Into<String>, title: impl Into<String>) {
        let order = self.lessons.len();
        self.lessons.push(LessonRef {
            id: id.into(),
            title: title.into(),
            order,
        });
    }

    /// Add a tag.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Number of lessons in curriculum.
    #[must_use]
    pub fn lesson_count(&self) -> usize {
        self.lessons.len()
    }

    /// Get lesson by index.
    #[must_use]
    pub fn lesson_at(&self, index: usize) -> Option<&LessonRef> {
        self.lessons.get(index)
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} lessons)", self.name, self.lessons.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_subject() {
        let subj = Subject::new(
            "rust-101",
            "Rust Fundamentals",
            "Learn the Rust programming language",
        );
        assert_eq!(subj.id, "rust-101");
        assert_eq!(subj.name, "Rust Fundamentals");
        assert_eq!(subj.lesson_count(), 0);
    }

    #[test]
    fn add_lessons() {
        let mut subj = Subject::new("math", "Mathematics", "Core math");
        subj.add_lesson("l1", "Algebra");
        subj.add_lesson("l2", "Calculus");
        assert_eq!(subj.lesson_count(), 2);
        assert_eq!(subj.lessons[0].order, 0);
        assert_eq!(subj.lessons[1].order, 1);
    }

    #[test]
    fn lesson_at() {
        let mut subj = Subject::new("pv", "PV", "Pharmacovigilance");
        subj.add_lesson("l1", "Signal Detection");
        assert!(subj.lesson_at(0).is_some());
        assert!(subj.lesson_at(1).is_none());
    }

    #[test]
    fn display_subject() {
        let mut subj = Subject::new("bio", "Biology", "Life sciences");
        subj.add_lesson("l1", "Cells");
        assert!(subj.to_string().contains("1 lessons"));
    }

    #[test]
    fn add_tags() {
        let mut subj = Subject::new("cs", "CS", "Computer science");
        subj.add_tag("programming");
        subj.add_tag("algorithms");
        assert_eq!(subj.tags.len(), 2);
    }
}
