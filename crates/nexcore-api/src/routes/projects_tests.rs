//! Tests for projects & deliverables endpoints (9 handlers → 26 tests)

use super::circles;
use super::projects::*;
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

/// Create a circle + founder (Founder role = Researcher+).
/// Returns (circle_id, founder_user_id).
async fn seed_circle(state: &ApiState) -> (String, String) {
    let circle = circles::create_circle(
        State(state.clone()),
        Json(circles::CreateCircleRequest {
            name: format!("Test-{}", nexcore_id::NexId::v4()),
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
    .expect("seed_circle")
    .0;
    (circle.id, "user-1".to_string())
}

/// Add a Member-role user to the circle (Member < Researcher).
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

fn project_req(user: &str) -> CreateProjectRequest {
    CreateProjectRequest {
        name: "Signal Detection Study".to_string(),
        description: "Investigating drug X adverse events".to_string(),
        project_type: Some("signal_detection".to_string()),
        therapeutic_area: Some("Oncology".to_string()),
        drug_names: Some(vec!["DrugX".to_string()]),
        indications: None,
        data_sources: None,
        target_completion: None,
        lead_user_id: user.to_string(),
        created_by: user.to_string(),
    }
}

async fn seed_project(state: &ApiState, circle_id: &str, user: &str) -> Project {
    create_project(
        State(state.clone()),
        Path(circle_id.to_string()),
        Json(project_req(user)),
    )
    .await
    .expect("seed_project")
    .0
}

// ============================================================================
// Create Project
// ============================================================================

#[tokio::test]
async fn test_create_project_valid() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    assert_eq!(project.name, "Signal Detection Study");
    assert_eq!(project.stage, "initiate");
    assert_eq!(project.status, "active");
    assert_eq!(project.circle_id, cid);
}

#[tokio::test]
async fn test_create_project_empty_name() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let mut req = project_req(&uid);
    req.name = "  ".to_string();
    let result = create_project(State(state), Path(cid), Json(req)).await;
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_create_project_circle_not_found() {
    let state = test_state();
    let req = project_req("user-1");
    let result = create_project(State(state), Path("nonexistent".to_string()), Json(req)).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_create_project_requires_researcher() {
    let state = test_state();
    let (cid, _uid) = seed_circle(&state).await;
    add_member_user(&state, &cid).await;

    let req = project_req("member-user");
    let result = create_project(State(state), Path(cid), Json(req)).await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_create_project_increments_count() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    seed_project(&state, &cid, &uid).await;

    let circle = circles::get_circle(State(state.clone()), Path(cid.clone()))
        .await
        .expect("get")
        .0;
    assert_eq!(circle.project_count, 1);
}

// ============================================================================
// List / Get / Update
// ============================================================================

#[tokio::test]
async fn test_list_projects_by_circle() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    seed_project(&state, &cid, &uid).await;

    let projects = list_projects(State(state), Path(cid))
        .await
        .expect("list")
        .0;
    assert_eq!(projects.len(), 1);
}

#[tokio::test]
async fn test_get_project_found() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let result = get_project(State(state), Path((cid, project.id.clone())))
        .await
        .expect("get")
        .0;
    assert_eq!(result.id, project.id);
}

#[tokio::test]
async fn test_get_project_not_found() {
    let state = test_state();
    let (cid, _uid) = seed_circle(&state).await;
    let result = get_project(State(state), Path((cid, "nonexistent".to_string()))).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_get_project_wrong_circle() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let result = get_project(State(state), Path(("wrong-circle".to_string(), project.id))).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_update_project_partial() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let updated = update_project(
        State(state),
        Path((cid, project.id)),
        Json(UpdateProjectRequest {
            name: Some("Updated Name".to_string()),
            description: None,
            status: None,
            therapeutic_area: None,
            drug_names: None,
            indications: None,
            data_sources: None,
            target_completion: None,
            lead_user_id: None,
        }),
    )
    .await
    .expect("update")
    .0;
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, "Investigating drug X adverse events"); // unchanged
}

#[tokio::test]
async fn test_update_project_not_found() {
    let state = test_state();
    let result = update_project(
        State(state),
        Path(("c".to_string(), "nonexistent".to_string())),
        Json(UpdateProjectRequest {
            name: Some("X".to_string()),
            description: None,
            status: None,
            therapeutic_area: None,
            drug_names: None,
            indications: None,
            data_sources: None,
            target_completion: None,
            lead_user_id: None,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

// ============================================================================
// Advance Stage
// ============================================================================

#[tokio::test]
async fn test_advance_stage_initiate_to_design() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    assert_eq!(project.stage, "initiate");

    let advanced = advance_stage(
        State(state),
        Path((cid, project.id)),
        Json(AdvanceStageRequest { advanced_by: uid }),
    )
    .await
    .expect("advance")
    .0;
    assert_eq!(advanced.stage, "design");
}

#[tokio::test]
async fn test_advance_stage_to_closed() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    // Advance through all stages: initiate→design→execute→analyze→report→review→publish→closed
    let stages = [
        "design", "execute", "analyze", "report", "review", "publish", "closed",
    ];
    let mut pid = project.id;
    for expected in stages {
        let advanced = advance_stage(
            State(state.clone()),
            Path((cid.clone(), pid.clone())),
            Json(AdvanceStageRequest {
                advanced_by: uid.clone(),
            }),
        )
        .await
        .expect("advance")
        .0;
        assert_eq!(advanced.stage, expected);
        pid = advanced.id;
    }

    // Should be Completed status when Closed
    let final_project = get_project(State(state), Path((cid, pid)))
        .await
        .expect("get")
        .0;
    assert_eq!(final_project.status, "completed");
}

#[tokio::test]
async fn test_advance_stage_closed_blocks() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    // Advance to closed
    let mut pid = project.id;
    for _ in 0..7 {
        let a = advance_stage(
            State(state.clone()),
            Path((cid.clone(), pid.clone())),
            Json(AdvanceStageRequest {
                advanced_by: uid.clone(),
            }),
        )
        .await
        .expect("advance")
        .0;
        pid = a.id;
    }

    // Try to advance past closed
    let result = advance_stage(
        State(state),
        Path((cid, pid)),
        Json(AdvanceStageRequest { advanced_by: uid }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "CONFLICT");
}

#[tokio::test]
async fn test_advance_stage_requires_reviewer() {
    let state = test_state();
    let (cid, _uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, "user-1").await;
    add_member_user(&state, &cid).await;

    let result = advance_stage(
        State(state),
        Path((cid, project.id)),
        Json(AdvanceStageRequest {
            advanced_by: "member-user".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_advance_stage_wrong_circle() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let result = advance_stage(
        State(state),
        Path(("wrong-circle".to_string(), project.id)),
        Json(AdvanceStageRequest { advanced_by: uid }),
    )
    .await;
    // User not member of wrong-circle → FORBIDDEN (role check before project lookup)
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

// ============================================================================
// Deliverables
// ============================================================================

#[tokio::test]
async fn test_create_deliverable_valid() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let deliverable = create_deliverable(
        State(state),
        Path((cid, project.id)),
        Json(CreateDeliverableRequest {
            name: "Safety Report".to_string(),
            deliverable_type: Some("report".to_string()),
            file_url: None,
            created_by: uid,
        }),
    )
    .await
    .expect("create")
    .0;
    assert_eq!(deliverable.name, "Safety Report");
    assert_eq!(deliverable.status, "draft");
    assert_eq!(deliverable.review_status, "pending");
    assert_eq!(deliverable.version, 1);
}

#[tokio::test]
async fn test_create_deliverable_empty_name() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;

    let result = create_deliverable(
        State(state),
        Path((cid, project.id)),
        Json(CreateDeliverableRequest {
            name: "  ".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_create_deliverable_project_not_found() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;

    let result = create_deliverable(
        State(state),
        Path((cid, "nonexistent".to_string())),
        Json(CreateDeliverableRequest {
            name: "Report".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_list_deliverables_by_project() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Report 1".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid.clone(),
        }),
    )
    .await
    .expect("create 1");

    let deliverables = list_deliverables(State(state), Path((cid, project.id)))
        .await
        .expect("list")
        .0;
    assert_eq!(deliverables.len(), 1);
}

#[tokio::test]
async fn test_update_deliverable_partial() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    let deliverable = create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Draft".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid,
        }),
    )
    .await
    .expect("create")
    .0;

    let updated = update_deliverable(
        State(state),
        Path((cid, project.id, deliverable.id)),
        Json(UpdateDeliverableRequest {
            name: Some("Final Report".to_string()),
            status: Some("in_review".to_string()),
            file_url: None,
            content_hash: None,
        }),
    )
    .await
    .expect("update")
    .0;
    assert_eq!(updated.name, "Final Report");
    assert_eq!(updated.status, "in_review");
}

#[tokio::test]
async fn test_update_deliverable_not_found() {
    let state = test_state();
    let result = update_deliverable(
        State(state),
        Path(("c".to_string(), "p".to_string(), "nonexistent".to_string())),
        Json(UpdateDeliverableRequest {
            name: Some("X".to_string()),
            status: None,
            file_url: None,
            content_hash: None,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

// ============================================================================
// Review Deliverable
// ============================================================================

#[tokio::test]
async fn test_review_deliverable_approved() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    let deliverable = create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Report".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid.clone(),
        }),
    )
    .await
    .expect("create")
    .0;

    let reviewed = review_deliverable(
        State(state),
        Path((cid, project.id, deliverable.id)),
        Json(ReviewDeliverableRequest {
            reviewed_by: uid,
            review_status: "approved".to_string(),
            review_notes: Some("Looks good".to_string()),
        }),
    )
    .await
    .expect("review")
    .0;
    assert_eq!(reviewed.review_status, "approved");
    assert_eq!(reviewed.status, "approved");
}

#[tokio::test]
async fn test_review_deliverable_rejected() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    let deliverable = create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Report".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid.clone(),
        }),
    )
    .await
    .expect("create")
    .0;

    let reviewed = review_deliverable(
        State(state),
        Path((cid, project.id, deliverable.id)),
        Json(ReviewDeliverableRequest {
            reviewed_by: uid,
            review_status: "rejected".to_string(),
            review_notes: Some("Needs work".to_string()),
        }),
    )
    .await
    .expect("review")
    .0;
    assert_eq!(reviewed.review_status, "rejected");
    // Rejected doesn't change deliverable status (only Approved and RevisionRequested do)
    assert_eq!(reviewed.status, "draft");
}

#[tokio::test]
async fn test_review_deliverable_revision_requested() {
    let state = test_state();
    let (cid, uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, &uid).await;
    let deliverable = create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Report".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: uid.clone(),
        }),
    )
    .await
    .expect("create")
    .0;

    let reviewed = review_deliverable(
        State(state),
        Path((cid, project.id, deliverable.id)),
        Json(ReviewDeliverableRequest {
            reviewed_by: uid,
            review_status: "revision_requested".to_string(),
            review_notes: Some("Please revise section 3".to_string()),
        }),
    )
    .await
    .expect("review")
    .0;
    assert_eq!(reviewed.review_status, "revision_requested");
    assert_eq!(reviewed.status, "draft"); // reset to draft
    assert_eq!(reviewed.version, 2); // version incremented
}

#[tokio::test]
async fn test_review_deliverable_requires_reviewer() {
    let state = test_state();
    let (cid, _uid) = seed_circle(&state).await;
    let project = seed_project(&state, &cid, "user-1").await;
    add_member_user(&state, &cid).await;

    let deliverable = create_deliverable(
        State(state.clone()),
        Path((cid.clone(), project.id.clone())),
        Json(CreateDeliverableRequest {
            name: "Report".to_string(),
            deliverable_type: None,
            file_url: None,
            created_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("create")
    .0;

    let result = review_deliverable(
        State(state),
        Path((cid, project.id, deliverable.id)),
        Json(ReviewDeliverableRequest {
            reviewed_by: "member-user".to_string(),
            review_status: "approved".to_string(),
            review_notes: None,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}
