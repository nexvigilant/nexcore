//! Fetch learner state from nexcore-api

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Learner state for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnerState {
    /// Learner name
    pub name: String,
    /// Number of enrolled subjects
    pub enrollment_count: usize,
    /// Average mastery across all subjects
    pub average_mastery: f64,
    /// Subjects mastered
    pub mastered_count: usize,
    /// Subjects developing
    pub developing_count: usize,
    /// Subjects needing remediation
    pub remediate_count: usize,
    /// Reviews due today
    pub reviews_due: usize,
}

/// Fetch current learner state
#[server]
pub async fn get_learner() -> Result<LearnerState, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3030/api/v1/education/learner")
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<LearnerState>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Mock data for development
            Ok(LearnerState {
                name: "Learner".into(),
                enrollment_count: 3,
                average_mastery: 0.62,
                mastered_count: 1,
                developing_count: 1,
                remediate_count: 1,
                reviews_due: 5,
            })
        }
    }
}
