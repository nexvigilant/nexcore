//! Spaced repetition review queue

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Review item due for study
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    /// Item ID
    pub id: String,
    /// Display title
    pub title: String,
    /// Subject it belongs to
    pub subject_name: String,
    /// Current retrievability [0, 1]
    pub retrievability: f64,
    /// Hours until due (negative = overdue)
    pub hours_until_due: f64,
    /// Review count
    pub review_count: u32,
}

/// Fetch items due for review
#[server]
pub async fn get_review_queue() -> Result<Vec<ReviewItem>, ServerFnError> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3030/api/v1/education/reviews")
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<ReviewItem>>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Mock data
            Ok(vec![
                ReviewItem {
                    id: "r1".into(),
                    title: "Ownership Basics".into(),
                    subject_name: "Rust Fundamentals".into(),
                    retrievability: 0.72,
                    hours_until_due: -2.5,
                    review_count: 3,
                },
                ReviewItem {
                    id: "r2".into(),
                    title: "PRR Formula".into(),
                    subject_name: "PV Signal Detection".into(),
                    retrievability: 0.85,
                    hours_until_due: 1.0,
                    review_count: 1,
                },
                ReviewItem {
                    id: "r3".into(),
                    title: "Sequence Primitive".into(),
                    subject_name: "Lex Primitiva".into(),
                    retrievability: 0.45,
                    hours_until_due: -12.0,
                    review_count: 0,
                },
            ])
        }
    }
}
