//! Submit assessment answers and get verdict

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Answer submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerSubmission {
    /// Question ID
    pub question_id: String,
    /// Given answer
    pub answer: String,
}

/// Assessment result from the engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResultData {
    /// Subject assessed
    pub subject_id: String,
    /// Questions correct
    pub correct_count: usize,
    /// Total questions
    pub total_count: usize,
    /// Updated mastery [0, 1]
    pub mastery: f64,
    /// Verdict: Mastered, Developing, Remediate
    pub verdict: String,
    /// Verdict CSS color class
    pub verdict_color: String,
}

/// Questions for an assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionData {
    /// Question ID
    pub id: String,
    /// Question prompt
    pub prompt: String,
    /// Concept being tested
    pub concept: String,
    /// Difficulty label
    pub difficulty: String,
}

/// Fetch assessment questions for a subject
#[server]
pub async fn get_assessment_questions(subject_id: String) -> Result<Vec<QuestionData>, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!(
        "http://localhost:3030/api/v1/education/subjects/{}/assess",
        subject_id
    );
    let response = client.get(&url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<Vec<QuestionData>>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            Ok(vec![
                QuestionData {
                    id: "q1".into(),
                    prompt: "What is the dominant primitive for HashMap<K, V>?".into(),
                    concept: "mapping".into(),
                    difficulty: "Medium".into(),
                },
                QuestionData {
                    id: "q2".into(),
                    prompt: "How many unique primitives classify a type as T3?".into(),
                    concept: "tier-classification".into(),
                    difficulty: "Easy".into(),
                },
                QuestionData {
                    id: "q3".into(),
                    prompt: "Which primitive represents ordered progression?".into(),
                    concept: "sequence".into(),
                    difficulty: "Easy".into(),
                },
            ])
        }
    }
}

/// Submit assessment answers
#[server]
pub async fn submit_assessment(
    subject_id: String,
    answers: String,
) -> Result<AssessmentResultData, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!(
        "http://localhost:3030/api/v1/education/subjects/{}/assess",
        subject_id
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(answers.clone())
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<AssessmentResultData>()
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))
        }
        _ => {
            // Mock result
            Ok(AssessmentResultData {
                subject_id,
                correct_count: 2,
                total_count: 3,
                mastery: 0.72,
                verdict: "Developing".into(),
                verdict_color: "text-yellow-400".into(),
            })
        }
    }
}
