// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Course Validation Engine
//!
//! Deterministic validation checks for academy course content.
//! Migrated from `studio/backend/app/services/course_validation_service.py`.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | Validator | ∂ | Boundary that separates valid/invalid |
//! | Issue | ∅ | Void detected in content |
//! | Score | N | Quantified validation result |
//! | Rating | κ | Comparison against threshold |

use serde::{Deserialize, Serialize};

use crate::formatting::AcademyCourse;

// ---------------------------------------------------------------------------
// Validation Issue Types
// ---------------------------------------------------------------------------

/// Category of validation check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCategory {
    /// Firestore schema compliance.
    Structure,
    /// Content quality standards.
    Content,
    /// WCAG 2.1 AA accessibility.
    Accessibility,
    /// Component auto-detection syntax.
    Components,
    /// Quiz and assessment rules.
    Assessment,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    /// Informational — no action required.
    Info,
    /// Warning — should be addressed before publishing.
    Warning,
    /// Error — must be fixed before publishing.
    Error,
}

/// A single validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    /// Unique issue identifier.
    pub id: String,
    /// Which category this check belongs to.
    pub category: ValidationCategory,
    /// Severity level.
    pub severity: ValidationSeverity,
    /// Human-readable description.
    pub message: String,
    /// Location in the course (e.g., "module[0].lesson[2]").
    pub location: Option<String>,
    /// Suggestion for fixing the issue.
    pub suggestion: Option<String>,
}

/// Quality rating based on validation score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityRating {
    /// Score >= 95.
    Excellent,
    /// Score >= 85.
    Good,
    /// Score >= 70.
    Acceptable,
    /// Score >= 50.
    NeedsWork,
    /// Score < 50.
    Critical,
}

impl QualityRating {
    /// Determine rating from a score (0–100).
    pub fn from_score(score: u32) -> Self {
        match score {
            95..=100 => Self::Excellent,
            85..=94 => Self::Good,
            70..=84 => Self::Acceptable,
            50..=69 => Self::NeedsWork,
            _ => Self::Critical,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Acceptable => "Acceptable",
            Self::NeedsWork => "Needs Work",
            Self::Critical => "Critical",
        }
    }
}

/// Status of overall validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    /// All checks passed.
    Pass,
    /// Non-blocking issues found.
    Warning,
    /// Blocking issues found.
    Fail,
}

/// Complete validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    /// Overall status.
    pub status: ValidationStatus,
    /// Course ID validated.
    pub course_id: String,
    /// Course title.
    pub course_title: String,
    /// Total issues found.
    pub total_issues: usize,
    /// Error count.
    pub errors: usize,
    /// Warning count.
    pub warnings: usize,
    /// Info count.
    pub infos: usize,
    /// Validation score (0–100).
    pub validation_score: u32,
    /// Quality rating.
    pub rating: QualityRating,
    /// Individual issues.
    pub issues: Vec<ValidationIssue>,
    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
}

// ---------------------------------------------------------------------------
// Validation Checks
// ---------------------------------------------------------------------------

/// Run all structure validation checks.
fn validate_structure(course: &AcademyCourse) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // S1: Course must have an ID
    if course.course_id.is_empty() {
        issues.push(ValidationIssue {
            id: "S1".into(),
            category: ValidationCategory::Structure,
            severity: ValidationSeverity::Error,
            message: "Course ID is empty".into(),
            location: Some("courseId".into()),
            suggestion: Some("Provide a unique course identifier".into()),
        });
    }

    // S2: Course must have a title
    if course.title.is_empty() {
        issues.push(ValidationIssue {
            id: "S2".into(),
            category: ValidationCategory::Structure,
            severity: ValidationSeverity::Error,
            message: "Course title is empty".into(),
            location: Some("title".into()),
            suggestion: Some("Add a descriptive course title".into()),
        });
    }

    // S3: Course must have at least one module with lessons
    let has_lessons = course.modules.iter().any(|m| !m.lessons.is_empty());
    if !has_lessons {
        issues.push(ValidationIssue {
            id: "S3".into(),
            category: ValidationCategory::Structure,
            severity: ValidationSeverity::Error,
            message: "Course has no lessons in any module".into(),
            location: Some("modules".into()),
            suggestion: Some("Add at least one lesson to a module".into()),
        });
    }

    // S4: Module order must be sequential
    for (i, module) in course.modules.iter().enumerate() {
        if module.order != (i + 1) as u32 {
            issues.push(ValidationIssue {
                id: "S4".into(),
                category: ValidationCategory::Structure,
                severity: ValidationSeverity::Warning,
                message: format!(
                    "Module '{}' has order {} but is at position {}",
                    module.title,
                    module.order,
                    i + 1
                ),
                location: Some(format!("modules[{i}].order")),
                suggestion: Some("Fix module ordering to be sequential".into()),
            });
        }
    }

    // S5: Lesson order must be sequential within each module
    for (mi, module) in course.modules.iter().enumerate() {
        for (li, lesson) in module.lessons.iter().enumerate() {
            if lesson.order != (li + 1) as u32 {
                issues.push(ValidationIssue {
                    id: "S5".into(),
                    category: ValidationCategory::Structure,
                    severity: ValidationSeverity::Warning,
                    message: format!(
                        "Lesson '{}' has order {} but is at position {}",
                        lesson.title,
                        lesson.order,
                        li + 1
                    ),
                    location: Some(format!("modules[{mi}].lessons[{li}].order")),
                    suggestion: Some("Fix lesson ordering to be sequential".into()),
                });
            }
        }
    }

    // S6: Description must not be empty
    if course.description.is_empty() {
        issues.push(ValidationIssue {
            id: "S6".into(),
            category: ValidationCategory::Structure,
            severity: ValidationSeverity::Warning,
            message: "Course description is empty".into(),
            location: Some("description".into()),
            suggestion: Some("Add a course description".into()),
        });
    }

    issues
}

/// Run content quality checks.
fn validate_content(course: &AcademyCourse) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for (mi, module) in course.modules.iter().enumerate() {
        for (li, lesson) in module.lessons.iter().enumerate() {
            let loc = format!("modules[{mi}].lessons[{li}]");

            // C1: Lesson must have content
            if lesson.content.is_empty() {
                issues.push(ValidationIssue {
                    id: "C1".into(),
                    category: ValidationCategory::Content,
                    severity: ValidationSeverity::Error,
                    message: format!("Lesson '{}' has no content", lesson.title),
                    location: Some(format!("{loc}.content")),
                    suggestion: Some("Add lesson content".into()),
                });
            }

            // C2: Lesson should have learning objectives
            if lesson.learning_objectives.is_empty() {
                issues.push(ValidationIssue {
                    id: "C2".into(),
                    category: ValidationCategory::Content,
                    severity: ValidationSeverity::Warning,
                    message: format!("Lesson '{}' has no learning objectives", lesson.title),
                    location: Some(format!("{loc}.learningObjectives")),
                    suggestion: Some("Add at least one learning objective".into()),
                });
            }

            // C3: Duration should be reasonable (5–60 min)
            if lesson.duration < 5 || lesson.duration > 60 {
                issues.push(ValidationIssue {
                    id: "C3".into(),
                    category: ValidationCategory::Content,
                    severity: ValidationSeverity::Warning,
                    message: format!(
                        "Lesson '{}' duration {} min is outside recommended range (5-60)",
                        lesson.title, lesson.duration
                    ),
                    location: Some(format!("{loc}.duration")),
                    suggestion: Some("Adjust content length for 5-60 minute lessons".into()),
                });
            }

            // C4: Title should not be empty
            if lesson.title.is_empty() {
                issues.push(ValidationIssue {
                    id: "C4".into(),
                    category: ValidationCategory::Content,
                    severity: ValidationSeverity::Error,
                    message: "Lesson has empty title".into(),
                    location: Some(format!("{loc}.title")),
                    suggestion: Some("Add a descriptive lesson title".into()),
                });
            }
        }
    }

    issues
}

/// Run assessment validation checks.
fn validate_assessments(course: &AcademyCourse) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for (mi, module) in course.modules.iter().enumerate() {
        for (li, lesson) in module.lessons.iter().enumerate() {
            if let Some(assessment) = &lesson.assessment {
                let loc = format!("modules[{mi}].lessons[{li}].assessment");

                // A1: Passing score must be in valid range
                if assessment.passing_score > 100 {
                    issues.push(ValidationIssue {
                        id: "A1".into(),
                        category: ValidationCategory::Assessment,
                        severity: ValidationSeverity::Error,
                        message: format!(
                            "Assessment passing score {} exceeds 100",
                            assessment.passing_score
                        ),
                        location: Some(format!("{loc}.passingScore")),
                        suggestion: Some("Set passing score between 0 and 100".into()),
                    });
                }

                // A2: Must allow at least one attempt
                if assessment.max_attempts == 0 {
                    issues.push(ValidationIssue {
                        id: "A2".into(),
                        category: ValidationCategory::Assessment,
                        severity: ValidationSeverity::Error,
                        message: "Assessment allows 0 attempts".into(),
                        location: Some(format!("{loc}.maxAttempts")),
                        suggestion: Some("Set max attempts to at least 1".into()),
                    });
                }
            }
        }
    }

    issues
}

/// Validate a complete academy course.
///
/// Runs structure, content, and assessment checks. Returns a full report
/// with score, rating, and actionable issues.
pub fn validate_course(course: &AcademyCourse) -> ValidationReport {
    let mut all_issues = Vec::new();
    all_issues.extend(validate_structure(course));
    all_issues.extend(validate_content(course));
    all_issues.extend(validate_assessments(course));

    let errors = all_issues
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Error)
        .count();
    let warnings = all_issues
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Warning)
        .count();
    let infos = all_issues
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Info)
        .count();

    // Score: start at 100, deduct 10 per error, 3 per warning, 1 per info
    let deductions = (errors * 10 + warnings * 3 + infos) as u32;
    let validation_score = 100u32.saturating_sub(deductions);

    let status = if errors > 0 {
        ValidationStatus::Fail
    } else if warnings > 0 {
        ValidationStatus::Warning
    } else {
        ValidationStatus::Pass
    };

    let rating = QualityRating::from_score(validation_score);

    let mut recommendations = Vec::new();
    if errors > 0 {
        recommendations.push(format!("Fix {errors} error(s) before publishing"));
    }
    if warnings > 0 {
        recommendations.push(format!("Address {warnings} warning(s) to improve quality"));
    }
    if validation_score < 85 {
        recommendations.push("Consider adding learning objectives to all lessons".into());
    }

    ValidationReport {
        status,
        course_id: course.course_id.clone(),
        course_title: course.title.clone(),
        total_issues: all_issues.len(),
        errors,
        warnings,
        infos,
        validation_score,
        rating,
        issues: all_issues,
        recommendations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatting::{AcademyCourse, CourseMetadata, Lesson, Module};

    fn minimal_course() -> AcademyCourse {
        AcademyCourse {
            course_id: "test-001".into(),
            title: "Test Course".into(),
            description: "A test".into(),
            topic: "Testing".into(),
            domain: "QA".into(),
            target_audience: "Engineers".into(),
            modules: vec![Module {
                title: "Knowledge".into(),
                description: "Learn knowledge".into(),
                order: 1,
                lessons: vec![Lesson {
                    title: "Lesson 1".into(),
                    description: "First lesson".into(),
                    content: "<p>Content here</p>".into(),
                    video_url: None,
                    video_provider: None,
                    video_id: None,
                    duration: 10,
                    order: 1,
                    assessment: None,
                    learning_objectives: vec!["Understand testing".into()],
                }],
            }],
            metadata: CourseMetadata {
                total_lessons: 1,
                total_modules: 1,
                estimated_duration: 10,
                generated_at: "2026-03-22T00:00:00Z".into(),
                quality_score: None,
            },
        }
    }

    #[test]
    fn valid_course_passes() {
        let report = validate_course(&minimal_course());
        assert_eq!(report.status, ValidationStatus::Pass);
        assert_eq!(report.errors, 0);
        assert!(report.validation_score >= 95);
    }

    #[test]
    fn empty_course_id_is_error() {
        let mut course = minimal_course();
        course.course_id = String::new();
        let report = validate_course(&course);
        assert_eq!(report.status, ValidationStatus::Fail);
        assert!(report.issues.iter().any(|i| i.id == "S1"));
    }

    #[test]
    fn empty_content_is_error() {
        let mut course = minimal_course();
        course.modules[0].lessons[0].content = String::new();
        let report = validate_course(&course);
        assert!(report.issues.iter().any(|i| i.id == "C1"));
    }

    #[test]
    fn quality_rating_thresholds() {
        assert_eq!(QualityRating::from_score(100), QualityRating::Excellent);
        assert_eq!(QualityRating::from_score(95), QualityRating::Excellent);
        assert_eq!(QualityRating::from_score(85), QualityRating::Good);
        assert_eq!(QualityRating::from_score(70), QualityRating::Acceptable);
        assert_eq!(QualityRating::from_score(50), QualityRating::NeedsWork);
        assert_eq!(QualityRating::from_score(49), QualityRating::Critical);
    }
}
