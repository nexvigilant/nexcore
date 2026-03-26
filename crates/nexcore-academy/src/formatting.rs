// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Academy Formatting Engine
//!
//! Deterministic transformation of KSB frameworks + content into
//! Academy course JSON structure.
//! Migrated from `studio/backend/app/services/academy_formatter.py`.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | Duration | N | Quantified reading time |
//! | Module order | σ | K→S→B sequence |
//! | Formatting | μ | KSB → Course structure mapping |
//! | Structure | ∂ | Boundary of valid course JSON |

use serde::{Deserialize, Serialize};

use crate::ksb::{KsbCategory, KsbFramework};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Assumed reading speed in words per minute.
pub const READING_SPEED_WPM: u32 = 200;

/// Buffer minutes added to reading duration.
pub const DURATION_BUFFER_MINUTES: u32 = 2;

/// Maximum lesson duration in minutes.
pub const MAX_DURATION_MINUTES: u32 = 60;

/// Minimum lesson duration in minutes (fallback for empty content).
pub const MIN_DURATION_MINUTES: u32 = 5;

/// Default assessment passing score (0–100).
pub const DEFAULT_PASSING_SCORE: u32 = 70;

/// Default maximum assessment attempts.
pub const DEFAULT_MAX_ATTEMPTS: u32 = 3;

// ---------------------------------------------------------------------------
// Duration Calculation
// ---------------------------------------------------------------------------

/// Estimate reading duration in minutes from word count.
///
/// Formula: `words / 200 WPM + 2 min buffer`, clamped to [5, 60].
pub fn estimate_duration_minutes(word_count: usize) -> u32 {
    if word_count == 0 {
        return MIN_DURATION_MINUTES;
    }
    let raw = (word_count as u32 / READING_SPEED_WPM) + DURATION_BUFFER_MINUTES;
    raw.clamp(MIN_DURATION_MINUTES, MAX_DURATION_MINUTES)
}

/// Count words in content (whitespace-separated).
pub fn word_count(content: &str) -> usize {
    content.split_whitespace().count()
}

// ---------------------------------------------------------------------------
// Course Structure Types
// ---------------------------------------------------------------------------

/// Assessment configuration for a lesson.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assessment {
    /// Minimum score to pass (0–100).
    pub passing_score: u32,
    /// Maximum number of attempts allowed.
    pub max_attempts: u32,
    /// Whether to randomize question order.
    pub randomize_questions: bool,
}

impl Default for Assessment {
    fn default() -> Self {
        Self {
            passing_score: DEFAULT_PASSING_SCORE,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            randomize_questions: false,
        }
    }
}

/// A single lesson within a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    /// Lesson title.
    pub title: String,
    /// Short description.
    pub description: String,
    /// HTML content body.
    pub content: String,
    /// Video URL (if any).
    pub video_url: Option<String>,
    /// Video provider (e.g., "youtube").
    pub video_provider: Option<String>,
    /// Video ID.
    pub video_id: Option<String>,
    /// Estimated duration in minutes.
    pub duration: u32,
    /// Position within module (1-indexed).
    pub order: u32,
    /// Assessment configuration (if present).
    pub assessment: Option<Assessment>,
    /// Learning objectives for this lesson.
    pub learning_objectives: Vec<String>,
}

/// A module (group of lessons by KSB category).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    /// Module title (e.g., "Knowledge").
    pub title: String,
    /// Module description.
    pub description: String,
    /// Position within course (1-indexed).
    pub order: u32,
    /// Lessons in this module.
    pub lessons: Vec<Lesson>,
}

/// Course metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseMetadata {
    /// Total number of lessons.
    pub total_lessons: usize,
    /// Total number of modules.
    pub total_modules: usize,
    /// Total estimated duration in minutes.
    pub estimated_duration: u32,
    /// ISO 8601 generation timestamp.
    pub generated_at: String,
    /// Quality score (if validated).
    pub quality_score: Option<f64>,
}

/// Complete academy course structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyCourse {
    /// Unique course identifier.
    pub course_id: String,
    /// Course title.
    pub title: String,
    /// Course description.
    pub description: String,
    /// Topic covered.
    pub topic: String,
    /// Domain context.
    pub domain: String,
    /// Target audience.
    pub target_audience: String,
    /// Modules (K, S, B).
    pub modules: Vec<Module>,
    /// Course metadata.
    pub metadata: CourseMetadata,
}

// ---------------------------------------------------------------------------
// Content for Lessons
// ---------------------------------------------------------------------------

/// Content generated for a KSB component.
pub struct LessonContent {
    /// Component ID this content belongs to.
    pub component_id: String,
    /// HTML content body.
    pub html: String,
    /// Learning objectives.
    pub learning_objectives: Vec<String>,
}

// ---------------------------------------------------------------------------
// Formatting Engine
// ---------------------------------------------------------------------------

/// Generate course title from topic and domain.
///
/// If the domain is already part of the topic, just use the topic.
pub fn generate_title(topic: &str, domain: &str) -> String {
    if topic.to_lowercase().contains(&domain.to_lowercase()) || domain.is_empty() {
        topic.to_string()
    } else {
        format!("{topic} ({domain})")
    }
}

/// Generate course description.
pub fn generate_description(topic: &str, domain: &str, target_audience: &str) -> String {
    let mut parts = vec![format!("A comprehensive course on {topic}.")];
    if !target_audience.is_empty() {
        parts.push(format!("Designed for {target_audience}."));
    }
    if !domain.is_empty() {
        parts.push(format!(
            "Covers essential concepts and practical applications in {domain}."
        ));
    }
    parts.push(
        "This course includes structured lessons with learning objectives, \
         rich content, and assessments to validate understanding."
            .into(),
    );
    parts.join(" ")
}

/// Parameters for formatting a course.
pub struct FormatCourseParams<'a> {
    /// Unique course identifier.
    pub course_id: &'a str,
    /// KSB framework to format.
    pub framework: &'a KsbFramework,
    /// Content for each KSB component.
    pub content: &'a [LessonContent],
    /// Target audience description.
    pub target_audience: &'a str,
    /// ISO 8601 generation timestamp.
    pub generated_at: &'a str,
    /// Quality score (if validated).
    pub quality_score: Option<f64>,
}

/// Format a KSB framework + content into an academy course.
///
/// Each KSB category becomes a module. Each component becomes a lesson.
/// Content is matched by component ID.
pub fn format_course(params: &FormatCourseParams<'_>) -> AcademyCourse {
    let content_map: std::collections::BTreeMap<&str, &LessonContent> = params
        .content
        .iter()
        .map(|c| (c.component_id.as_str(), c))
        .collect();

    let mut modules = Vec::with_capacity(3);
    let mut total_lessons = 0u32;
    let mut total_duration = 0u32;

    for (module_order, category) in KsbCategory::ALL.iter().enumerate() {
        let components = params.framework.by_category(*category);
        let mut lessons = Vec::with_capacity(components.len());

        for (lesson_order, component) in components.iter().enumerate() {
            let (html, objectives) = match content_map.get(component.id.as_str()) {
                Some(lc) => (lc.html.clone(), lc.learning_objectives.clone()),
                None => (
                    format!("<p>{}</p>", component.description),
                    vec![format!("Understand {}", component.title)],
                ),
            };

            let duration = estimate_duration_minutes(word_count(&html));
            total_duration += duration;

            lessons.push(Lesson {
                title: component.title.clone(),
                description: component.description.clone(),
                content: html,
                video_url: None,
                video_provider: None,
                video_id: None,
                duration,
                order: (lesson_order + 1) as u32,
                assessment: None,
                learning_objectives: objectives,
            });
        }

        total_lessons += lessons.len() as u32;

        modules.push(Module {
            title: category.label().to_string(),
            description: category.module_description(),
            order: (module_order + 1) as u32,
            lessons,
        });
    }

    AcademyCourse {
        course_id: params.course_id.to_string(),
        title: generate_title(&params.framework.topic, &params.framework.domain),
        description: generate_description(
            &params.framework.topic,
            &params.framework.domain,
            params.target_audience,
        ),
        topic: params.framework.topic.clone(),
        domain: params.framework.domain.clone(),
        target_audience: params.target_audience.to_string(),
        modules,
        metadata: CourseMetadata {
            total_lessons: total_lessons as usize,
            total_modules: 3,
            estimated_duration: total_duration,
            generated_at: params.generated_at.to_string(),
            quality_score: params.quality_score,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ksb::{KsbComponent, KsbFramework};

    #[test]
    fn duration_estimation() {
        assert_eq!(estimate_duration_minutes(0), MIN_DURATION_MINUTES);
        assert_eq!(
            estimate_duration_minutes(200),
            3_u32.clamp(MIN_DURATION_MINUTES, MAX_DURATION_MINUTES)
        ); // 1 + 2 = 3, clamped to 5
        assert_eq!(estimate_duration_minutes(1000), 7); // 5 + 2
        assert_eq!(estimate_duration_minutes(20000), MAX_DURATION_MINUTES); // 100 + 2, clamped to 60
    }

    #[test]
    fn title_generation() {
        assert_eq!(
            generate_title("Pharmacovigilance", "Pharmaceutical Industry"),
            "Pharmacovigilance (Pharmaceutical Industry)"
        );
        assert_eq!(
            generate_title(
                "Pharmaceutical Industry Overview",
                "Pharmaceutical Industry"
            ),
            "Pharmaceutical Industry Overview"
        );
    }

    #[test]
    fn format_course_structure() {
        let fw = KsbFramework {
            topic: "PV Basics".into(),
            domain: "Pharma".into(),
            knowledge: vec![KsbComponent {
                id: "K1".into(),
                category: crate::ksb::KsbCategory::Knowledge,
                title: "ADR Classification".into(),
                description: "Understand ADR types".into(),
            }],
            skills: vec![],
            behaviors: vec![],
        };

        let content = vec![LessonContent {
            component_id: "K1".into(),
            html: "<p>ADRs are classified into Type A and Type B reactions.</p>".into(),
            learning_objectives: vec!["Classify ADR types".into()],
        }];

        let course = format_course(&FormatCourseParams {
            course_id: "test-001",
            framework: &fw,
            content: &content,
            target_audience: "PharmD graduates",
            generated_at: "2026-03-22T00:00:00Z",
            quality_score: None,
        });

        assert_eq!(course.course_id, "test-001");
        assert_eq!(course.modules.len(), 3); // K, S, B modules always created
        assert_eq!(course.modules[0].lessons.len(), 1);
        assert_eq!(course.modules[1].lessons.len(), 0);
        assert_eq!(course.metadata.total_lessons, 1);
    }
}
