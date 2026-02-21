//! Fetch available subjects

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Subject summary for listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSummary {
    /// Subject ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Number of lessons
    pub lesson_count: usize,
    /// Current mastery level [0, 1]
    pub mastery: f64,
    /// Current phase
    pub phase: String,
    /// Tags
    pub tags: Vec<String>,
}

/// Fetch all available subjects
#[server]
pub async fn get_subjects() -> Result<Vec<SubjectSummary>, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3030/api/v1/education/subjects")
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<SubjectSummary>>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Mock data
            Ok(vec![
                SubjectSummary {
                    id: "rust-fundamentals".into(),
                    name: "Rust Fundamentals".into(),
                    description: "Core Rust: ownership, borrowing, lifetimes, traits".into(),
                    lesson_count: 8,
                    mastery: 0.92,
                    phase: "Master".into(),
                    tags: vec!["programming".into(), "rust".into()],
                },
                SubjectSummary {
                    id: "pv-signal-detection".into(),
                    name: "PV Signal Detection".into(),
                    description: "PRR, ROR, IC, EBGM disproportionality methods".into(),
                    lesson_count: 6,
                    mastery: 0.68,
                    phase: "Practice".into(),
                    tags: vec!["pharmacovigilance".into(), "statistics".into()],
                },
                SubjectSummary {
                    id: "lex-primitiva".into(),
                    name: "Lex Primitiva".into(),
                    description: "16 T1 universal primitives and type grounding".into(),
                    lesson_count: 4,
                    mastery: 0.35,
                    phase: "Extract".into(),
                    tags: vec!["type-theory".into(), "primitives".into()],
                },
            ])
        }
    }
}
