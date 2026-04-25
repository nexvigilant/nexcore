//! Tests for publications & collaboration endpoints (6 handlers → 14 tests)

#![cfg(test)]


use super::circles;
use super::projects;
use super::publications::*;
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

/// Create circle → project → deliverable → approve → return IDs for publishing.
async fn seed_approved_deliverable(state: &ApiState) -> (String, String, String) {
    // Circle (user-1 = Founder = Lead+)
    let circle = circles::create_circle(
        State(state.clone()),
        Json(circles::CreateCircleRequest {
            name: format!("Pub-Test-{}", nexcore_id::NexId::v4()),
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
    .0;
    let cid = circle.id.clone();

    // Project
    let project = projects::create_project(
        State(state.clone()),
        Path(cid.clone()),
        Json(projects::CreateProjectRequest {
            name: "Study".to_string(),
            description: "Study desc".to_string(),
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
    let pid = project.id.clone();

    // Deliverable
    let deliverable = projects::create_deliverable(
        State(state.clone()),
        Path((cid.clone(), pid.clone())),
        Json(projects::CreateDeliverableRequest {
            name: "Final Report".to_string(),
            deliverable_type: Some("report".to_string()),
            file_url: None,
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("deliverable")
    .0;
    let did = deliverable.id.clone();

    // Approve
    projects::review_deliverable(
        State(state.clone()),
        Path((cid.clone(), pid, did.clone())),
        Json(projects::ReviewDeliverableRequest {
            reviewed_by: "user-1".to_string(),
            review_status: "approved".to_string(),
            review_notes: None,
        }),
    )
    .await
    .expect("approve");

    (cid, did, "user-1".to_string())
}

/// Create a second circle for collaboration tests.
async fn seed_second_circle(state: &ApiState) -> String {
    let circle = circles::create_circle(
        State(state.clone()),
        Json(circles::CreateCircleRequest {
            name: format!("Second-{}", nexcore_id::NexId::v4()),
            description: "Target circle".to_string(),
            mission: None,
            formation: None,
            tenant_id: None,
            created_by: "user-2".to_string(),
            visibility: None,
            join_policy: None,
            circle_type: None,
            therapeutic_areas: None,
            tags: None,
        }),
    )
    .await
    .expect("second circle")
    .0;
    circle.id
}

// ============================================================================
// Publish
// ============================================================================

#[tokio::test]
async fn test_publish_deliverable_valid() {
    let state = test_state();
    let (cid, did, uid) = seed_approved_deliverable(&state).await;

    let pub_result = publish_deliverable(
        State(state),
        Path(cid),
        Json(PublishRequest {
            deliverable_id: did,
            title: "New Signal Detected".to_string(),
            abstract_text: "We found a signal for DrugX".to_string(),
            visibility: None,
            published_by: uid,
        }),
    )
    .await
    .expect("publish")
    .0;
    assert_eq!(pub_result.title, "New Signal Detected");
    assert_eq!(pub_result.visibility, "community");
}

#[tokio::test]
async fn test_publish_deliverable_not_approved() {
    let state = test_state();
    // Create circle + project + deliverable (NOT approved)
    let circle = circles::create_circle(
        State(state.clone()),
        Json(circles::CreateCircleRequest {
            name: format!("T-{}", nexcore_id::NexId::v4()),
            description: "T".to_string(),
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
    .0;
    let project = projects::create_project(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(projects::CreateProjectRequest {
            name: "S".to_string(),
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
    let deliverable = projects::create_deliverable(
        State(state.clone()),
        Path((circle.id.clone(), project.id)),
        Json(projects::CreateDeliverableRequest {
            name: "Draft".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("deliverable")
    .0;

    let result = publish_deliverable(
        State(state),
        Path(circle.id),
        Json(PublishRequest {
            deliverable_id: deliverable.id,
            title: "Title".to_string(),
            abstract_text: "Abstract".to_string(),
            visibility: None,
            published_by: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "CONFLICT");
}

#[tokio::test]
async fn test_publish_deliverable_requires_lead() {
    let state = test_state();
    let (cid, did, _uid) = seed_approved_deliverable(&state).await;

    // Add member-user as basic Member
    circles::join_circle(
        State(state.clone()),
        Path(cid.clone()),
        Json(circles::JoinRequest {
            user_id: "member-user".to_string(),
        }),
    )
    .await
    .expect("join");

    let result = publish_deliverable(
        State(state),
        Path(cid),
        Json(PublishRequest {
            deliverable_id: did,
            title: "Title".to_string(),
            abstract_text: "Abstract".to_string(),
            visibility: None,
            published_by: "member-user".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_publish_increments_count() {
    let state = test_state();
    let (cid, did, uid) = seed_approved_deliverable(&state).await;

    publish_deliverable(
        State(state.clone()),
        Path(cid.clone()),
        Json(PublishRequest {
            deliverable_id: did,
            title: "Title".to_string(),
            abstract_text: "Abstract".to_string(),
            visibility: None,
            published_by: uid,
        }),
    )
    .await
    .expect("publish");

    let circle = circles::get_circle(State(state), Path(cid))
        .await
        .expect("get")
        .0;
    assert_eq!(circle.publication_count, 1);
}

// ============================================================================
// List Publications
// ============================================================================

#[tokio::test]
async fn test_list_publications_community_only() {
    let state = test_state();
    let (cid, did, uid) = seed_approved_deliverable(&state).await;

    publish_deliverable(
        State(state.clone()),
        Path(cid.clone()),
        Json(PublishRequest {
            deliverable_id: did,
            title: "Community Pub".to_string(),
            abstract_text: "Abstract".to_string(),
            visibility: Some("community".to_string()),
            published_by: uid,
        }),
    )
    .await
    .expect("publish");

    let pubs = list_publications(State(state)).await.expect("list").0;
    assert_eq!(pubs.len(), 1);
    assert_eq!(pubs[0].title, "Community Pub");
}

#[tokio::test]
async fn test_list_circle_publications() {
    let state = test_state();
    let (cid, did, uid) = seed_approved_deliverable(&state).await;

    publish_deliverable(
        State(state.clone()),
        Path(cid.clone()),
        Json(PublishRequest {
            deliverable_id: did,
            title: "Circle Pub".to_string(),
            abstract_text: "Abstract".to_string(),
            visibility: None,
            published_by: uid,
        }),
    )
    .await
    .expect("publish");

    let pubs = list_circle_publications(State(state), Path(cid))
        .await
        .expect("list")
        .0;
    assert_eq!(pubs.len(), 1);
}

// ============================================================================
// Collaboration
// ============================================================================

#[tokio::test]
async fn test_request_collaboration_valid() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;
    let target_cid = seed_second_circle(&state).await;

    let collab = request_collaboration(
        State(state),
        Path(cid),
        Json(CollaborateRequest {
            target_circle_id: target_cid,
            request_type: Some("peer_review".to_string()),
            message: "Would you review our signal assessment?".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("collab")
    .0;
    assert_eq!(collab.status, "pending");
    assert_eq!(collab.request_type, "peer_review");
}

#[tokio::test]
async fn test_request_collaboration_self_error() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;

    let result = request_collaboration(
        State(state),
        Path(cid.clone()),
        Json(CollaborateRequest {
            target_circle_id: cid,
            request_type: None,
            message: "Self collab".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_request_collaboration_target_not_found() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;

    let result = request_collaboration(
        State(state),
        Path(cid),
        Json(CollaborateRequest {
            target_circle_id: "nonexistent".to_string(),
            request_type: None,
            message: "Hello".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_request_collaboration_requires_lead() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;
    let target_cid = seed_second_circle(&state).await;

    // Add member-user as basic Member
    circles::join_circle(
        State(state.clone()),
        Path(cid.clone()),
        Json(circles::JoinRequest {
            user_id: "member-user".to_string(),
        }),
    )
    .await
    .expect("join");

    let result = request_collaboration(
        State(state),
        Path(cid),
        Json(CollaborateRequest {
            target_circle_id: target_cid,
            request_type: None,
            message: "Hello".to_string(),
            created_by: "member-user".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_list_collaborations_both_directions() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;
    let target_cid = seed_second_circle(&state).await;

    request_collaboration(
        State(state.clone()),
        Path(cid.clone()),
        Json(CollaborateRequest {
            target_circle_id: target_cid.clone(),
            request_type: None,
            message: "Hello".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("collab");

    // Both circles should see the collaboration
    let from_requesting = list_collaborations(State(state.clone()), Path(cid))
        .await
        .expect("list")
        .0;
    assert_eq!(from_requesting.len(), 1);

    let from_target = list_collaborations(State(state), Path(target_cid))
        .await
        .expect("list")
        .0;
    assert_eq!(from_target.len(), 1);
}

#[tokio::test]
async fn test_update_collaboration_status() {
    let state = test_state();
    let (cid, _did, _uid) = seed_approved_deliverable(&state).await;
    let target_cid = seed_second_circle(&state).await;

    let collab = request_collaboration(
        State(state.clone()),
        Path(cid),
        Json(CollaborateRequest {
            target_circle_id: target_cid,
            request_type: None,
            message: "Hello".to_string(),
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("collab")
    .0;

    let updated = update_collaboration(
        State(state),
        Path(collab.id),
        Json(UpdateCollaborationRequest {
            status: "accepted".to_string(),
        }),
    )
    .await
    .expect("update")
    .0;
    assert_eq!(updated.status, "accepted");
}

#[tokio::test]
async fn test_update_collaboration_not_found() {
    let state = test_state();
    let result = update_collaboration(
        State(state),
        Path("nonexistent".to_string()),
        Json(UpdateCollaborationRequest {
            status: "accepted".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}
