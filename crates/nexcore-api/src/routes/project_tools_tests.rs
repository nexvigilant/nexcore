//! Tests for project_tools endpoints (3 handlers → 7 error-path tests)
//!
//! Happy-path tests require MCP server runtime (integration test territory).
//! These tests cover all error paths: missing project, wrong circle, FORBIDDEN.

#![cfg(test)]


use super::circles;
use super::project_tools::*;
use super::projects;
use crate::ApiState;
use crate::persistence::Persistence;
use crate::persistence::firestore::MockPersistence;
use crate::routes::skills::SkillAppState;
use axum::extract::{Json, Path, State};
use std::sync::Arc;

fn test_state() -> ApiState {
    ApiState {
        persistence: Arc::new(Persistence::Mock(MockPersistence::new())),
        skill_state: SkillAppState::default(),
    }
}

/// Create circle with founder. Returns circle_id.
async fn seed_circle(state: &ApiState) -> String {
    circles::create_circle(
        State(state.clone()),
        Json(circles::CreateCircleRequest {
            name: format!("Tools-{}", nexcore_id::NexId::v4()),
            description: "Test".to_string(),
            mission: None,
            formation: None,
            tenant_id: None,
            created_by: "user-1".to_string(),
            visibility: None,
            join_policy: None,
            circle_type: None,
            therapeutic_areas: None,
            tags: None,
        }),
    )
    .await
    .expect("circle")
    .0
    .id
}

/// Add a Member-role user (Member < Researcher → FORBIDDEN for tools).
async fn add_member_user(state: &ApiState, circle_id: &str) {
    circles::join_circle(
        State(state.clone()),
        Path(circle_id.to_string()),
        Json(circles::JoinRequest {
            user_id: "member-user".to_string(),
        }),
    )
    .await
    .expect("join");
}

fn signal_req(user: &str) -> SignalDetectRequest {
    SignalDetectRequest {
        drug_count: 100,
        event_count: 50,
        drug_event_count: 10,
        total_count: 10000,
        user_id: user.to_string(),
    }
}

// ============================================================================
// Signal Detect — Error Paths
// ============================================================================

#[tokio::test]
async fn test_signal_detect_project_not_found() {
    let state = test_state();
    let cid = seed_circle(&state).await;

    let result = signal_detect(
        State(state),
        Path((cid, "nonexistent".to_string())),
        Json(signal_req("user-1")),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_signal_detect_requires_researcher() {
    let state = test_state();
    let cid = seed_circle(&state).await;
    add_member_user(&state, &cid).await;

    let result = signal_detect(
        State(state),
        Path((cid, "any-project".to_string())),
        Json(signal_req("member-user")),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_signal_detect_wrong_circle() {
    let state = test_state();
    let cid = seed_circle(&state).await;

    // Create project in cid
    let project = projects::create_project(
        State(state.clone()),
        Path(cid.clone()),
        Json(projects::CreateProjectRequest {
            name: "P".to_string(),
            description: "D".to_string(),
            project_type: None,
            loop_method: None,
            therapeutic_area: None,
            drug_names: None,
            indications: None,
            data_sources: None,
            target_completion: None,
            lead_user_id: "user-1".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("project")
    .0;

    // Try to access from wrong circle — user-1 is not a member there
    let result = signal_detect(
        State(state),
        Path(("wrong-circle".to_string(), project.id)),
        Json(signal_req("user-1")),
    )
    .await;
    assert!(result.is_err());
}

// ============================================================================
// FAERS Query — Error Paths
// ============================================================================

#[tokio::test]
async fn test_faers_query_project_not_found() {
    let state = test_state();
    let cid = seed_circle(&state).await;

    let result = faers_query(
        State(state),
        Path((cid, "nonexistent".to_string())),
        Json(FaersQueryRequest {
            query: "aspirin".to_string(),
            limit: None,
            user_id: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_faers_query_requires_researcher() {
    let state = test_state();
    let cid = seed_circle(&state).await;
    add_member_user(&state, &cid).await;

    let result = faers_query(
        State(state),
        Path((cid, "any-project".to_string())),
        Json(FaersQueryRequest {
            query: "aspirin".to_string(),
            limit: None,
            user_id: "member-user".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

// ============================================================================
// Literature Search — Error Paths
// ============================================================================

#[tokio::test]
async fn test_literature_search_project_not_found() {
    let state = test_state();
    let cid = seed_circle(&state).await;

    let result = literature_search(
        State(state),
        Path((cid, "nonexistent".to_string())),
        Json(LiteratureSearchRequest {
            query: "pharmacovigilance".to_string(),
            category: None,
            limit: None,
            user_id: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_literature_search_requires_researcher() {
    let state = test_state();
    let cid = seed_circle(&state).await;
    add_member_user(&state, &cid).await;

    let result = literature_search(
        State(state),
        Path((cid, "any-project".to_string())),
        Json(LiteratureSearchRequest {
            query: "pharmacovigilance".to_string(),
            category: None,
            limit: None,
            user_id: "member-user".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}
