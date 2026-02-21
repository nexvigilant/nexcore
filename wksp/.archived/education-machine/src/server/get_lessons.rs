//! Fetch lessons for a subject

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Lesson summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonSummary {
    /// Lesson ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Difficulty label
    pub difficulty: String,
    /// Number of steps
    pub step_count: usize,
    /// Whether completed
    pub completed: bool,
}

/// Lesson detail with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonDetail {
    /// Lesson ID
    pub id: String,
    /// Title
    pub title: String,
    /// Subject ID
    pub subject_id: String,
    /// Difficulty
    pub difficulty: String,
    /// Steps with content
    pub steps: Vec<LessonStepData>,
}

/// Step data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonStepData {
    /// Step title
    pub title: String,
    /// Content type: "text", "decomposition", "exercise"
    pub content_type: String,
    /// Body text
    pub body: String,
    /// Optional hints
    pub hints: Vec<String>,
}

/// Fetch lessons for a subject
#[server]
pub async fn get_lessons(subject_id: String) -> Result<Vec<LessonSummary>, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!(
        "http://localhost:3030/api/v1/education/subjects/{}/lessons",
        subject_id
    );
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<LessonSummary>>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Mock data
            Ok(vec![
                LessonSummary {
                    id: "l1".into(),
                    title: "Introduction".into(),
                    description: "Overview and key concepts".into(),
                    difficulty: "Easy".into(),
                    step_count: 3,
                    completed: true,
                },
                LessonSummary {
                    id: "l2".into(),
                    title: "Core Concepts".into(),
                    description: "Deep dive into fundamentals".into(),
                    difficulty: "Medium".into(),
                    step_count: 5,
                    completed: false,
                },
                LessonSummary {
                    id: "l3".into(),
                    title: "Advanced Patterns".into(),
                    description: "Complex applications and edge cases".into(),
                    difficulty: "Hard".into(),
                    step_count: 4,
                    completed: false,
                },
            ])
        }
    }
}

/// Fetch a single lesson detail
#[server]
pub async fn get_lesson_detail(lesson_id: String) -> Result<LessonDetail, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!(
        "http://localhost:3030/api/v1/education/lessons/{}",
        lesson_id
    );
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<LessonDetail>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            Ok(LessonDetail {
                id: lesson_id,
                title: "Introduction to Concepts".into(),
                subject_id: "demo".into(),
                difficulty: "Medium".into(),
                steps: vec![
                    LessonStepData {
                        title: "What are Primitives?".into(),
                        content_type: "text".into(),
                        body: "Primitives are the irreducible building blocks of any domain. Every complex concept can be decomposed into a composition of T1 universal primitives.".into(),
                        hints: vec![],
                    },
                    LessonStepData {
                        title: "Decompose: HashMap".into(),
                        content_type: "decomposition".into(),
                        body: "Identify the dominant primitive: HashMap is fundamentally a Mapping (mu) type that associates keys to values.".into(),
                        hints: vec!["Think about what operation HashMap enables".into()],
                    },
                    LessonStepData {
                        title: "Practice Exercise".into(),
                        content_type: "exercise".into(),
                        body: "What is the dominant primitive for Vec<T>? Classify its tier.".into(),
                        hints: vec!["Vec maintains order".into(), "Think about what defines a sequence".into()],
                    },
                ],
            })
        }
    }
}
