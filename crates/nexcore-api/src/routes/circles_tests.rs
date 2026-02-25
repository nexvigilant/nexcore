//! Tests for circles endpoints (15 handlers → 33 tests)

use super::circles::*;
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

fn create_req(name: &str, created_by: &str) -> CreateCircleRequest {
    CreateCircleRequest {
        name: name.to_string(),
        description: "Test description".to_string(),
        mission: None,
        formation: None,
        tenant_id: None,
        created_by: created_by.to_string(),
        visibility: None,
        join_policy: None,
        circle_type: None,
        therapeutic_areas: None,
        tags: None,
    }
}

async fn seed_circle(state: &ApiState, name: &str, user: &str) -> Circle {
    create_circle(State(state.clone()), Json(create_req(name, user)))
        .await
        .expect("seed_circle failed")
        .0
}

// ============================================================================
// Create Circle
// ============================================================================

#[tokio::test]
async fn test_create_circle_valid() {
    let state = test_state();
    let circle = seed_circle(&state, "PV Signal Hunters", "user-1").await;
    assert_eq!(circle.name, "PV Signal Hunters");
    assert_eq!(circle.slug, "pv-signal-hunters");
    assert_eq!(circle.created_by, "user-1");
    assert_eq!(circle.member_count, 1);
    assert_eq!(circle.project_count, 0);
}

#[tokio::test]
async fn test_create_circle_slug_generation() {
    let state = test_state();
    let circle = seed_circle(&state, "  Hello   World!!  ", "user-1").await;
    assert_eq!(circle.slug, "hello-world");
}

#[tokio::test]
async fn test_create_circle_auto_founder() {
    let state = test_state();
    let circle = seed_circle(&state, "Test Circle", "user-1").await;
    let members = list_members(State(state.clone()), Path(circle.id))
        .await
        .expect("list_members failed")
        .0;
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, "user-1");
    assert_eq!(members[0].role, "founder");
    assert_eq!(members[0].status, "active");
}

#[tokio::test]
async fn test_create_circle_empty_name() {
    let state = test_state();
    let mut req = create_req("x", "user-1");
    req.name = "  ".to_string();
    let result = create_circle(State(state), Json(req)).await;
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_create_circle_duplicate_slug() {
    let state = test_state();
    seed_circle(&state, "My Circle", "user-1").await;
    let result = create_circle(
        State(state.clone()),
        Json(create_req("My Circle", "user-2")),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "CONFLICT");
}

// ============================================================================
// List / Get / Update / Archive
// ============================================================================

#[tokio::test]
async fn test_list_circles_filters_private() {
    let state = test_state();
    seed_circle(&state, "Public Circle", "user-1").await;
    let mut req = create_req("Private Circle", "user-2");
    req.visibility = Some("private".to_string());
    create_circle(State(state.clone()), Json(req))
        .await
        .expect("create private circle");

    let circles = list_circles(State(state)).await.expect("list").0;
    assert_eq!(circles.len(), 1);
    assert_eq!(circles[0].name, "Public Circle");
}

#[tokio::test]
async fn test_list_circles_filters_archived() {
    let state = test_state();
    let c1 = seed_circle(&state, "Active Circle", "user-1").await;
    let c2 = seed_circle(&state, "Soon Archived", "user-2").await;
    archive_circle(State(state.clone()), Path(c2.id))
        .await
        .expect("archive");

    let circles = list_circles(State(state)).await.expect("list").0;
    assert_eq!(circles.len(), 1);
    assert_eq!(circles[0].id, c1.id);
}

#[tokio::test]
async fn test_get_circle_found() {
    let state = test_state();
    let circle = seed_circle(&state, "Test", "user-1").await;
    let result = get_circle(State(state), Path(circle.id.clone()))
        .await
        .expect("get")
        .0;
    assert_eq!(result.id, circle.id);
    assert_eq!(result.name, "Test");
}

#[tokio::test]
async fn test_get_circle_not_found() {
    let state = test_state();
    let result = get_circle(State(state), Path("nonexistent".to_string())).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_update_circle_partial() {
    let state = test_state();
    let circle = seed_circle(&state, "Original", "user-1").await;
    let update = UpdateCircleRequest {
        name: Some("Updated Name".to_string()),
        description: None,
        mission: Some("New mission".to_string()),
        visibility: None,
        join_policy: None,
        status: None,
        therapeutic_areas: None,
        tags: None,
    };
    let updated = update_circle(State(state), Path(circle.id), Json(update))
        .await
        .expect("update")
        .0;
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.mission, Some("New mission".to_string()));
    assert_eq!(updated.description, "Test description"); // unchanged
}

#[tokio::test]
async fn test_update_circle_not_found() {
    let state = test_state();
    let update = UpdateCircleRequest {
        name: Some("X".to_string()),
        description: None,
        mission: None,
        visibility: None,
        join_policy: None,
        status: None,
        therapeutic_areas: None,
        tags: None,
    };
    let result = update_circle(State(state), Path("nonexistent".to_string()), Json(update)).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_archive_circle() {
    let state = test_state();
    let circle = seed_circle(&state, "To Archive", "user-1").await;
    let resp = archive_circle(State(state.clone()), Path(circle.id.clone()))
        .await
        .expect("archive")
        .0;
    assert_eq!(resp["status"], "archived");

    let fetched = get_circle(State(state), Path(circle.id))
        .await
        .expect("get")
        .0;
    assert_eq!(fetched.status, "archived");
}

#[tokio::test]
async fn test_archive_circle_not_found() {
    let state = test_state();
    let result = archive_circle(State(state), Path("nonexistent".to_string())).await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

// ============================================================================
// My Circles / Org Circles / Discover
// ============================================================================

#[tokio::test]
async fn test_my_circles_active_only() {
    let state = test_state();
    let c1 = seed_circle(&state, "Circle A", "user-1").await;
    let _c2 = seed_circle(&state, "Circle B", "user-2").await;

    let my = my_circles(State(state), Path("user-1".to_string()))
        .await
        .expect("my_circles")
        .0;
    assert_eq!(my.len(), 1);
    assert_eq!(my[0].id, c1.id);
}

#[tokio::test]
async fn test_my_circles_empty_for_non_member() {
    let state = test_state();
    seed_circle(&state, "Circle A", "user-1").await;
    let my = my_circles(State(state), Path("user-999".to_string()))
        .await
        .expect("my_circles")
        .0;
    assert!(my.is_empty());
}

#[tokio::test]
async fn test_list_org_circles() {
    let state = test_state();
    let mut req = create_req("Org Circle", "user-1");
    req.formation = Some("org_backed".to_string());
    req.tenant_id = Some("tenant-1".to_string());
    create_circle(State(state.clone()), Json(req))
        .await
        .expect("create org circle");
    seed_circle(&state, "Ad Hoc Circle", "user-2").await;

    let org = list_org_circles(State(state), Path("tenant-1".to_string()))
        .await
        .expect("org_circles")
        .0;
    assert_eq!(org.len(), 1);
    assert_eq!(org[0].name, "Org Circle");
}

#[tokio::test]
async fn test_list_org_circles_filters_active() {
    let state = test_state();
    let mut req = create_req("Org Circle", "user-1");
    req.formation = Some("org_backed".to_string());
    req.tenant_id = Some("tenant-1".to_string());
    let circle = create_circle(State(state.clone()), Json(req))
        .await
        .expect("create")
        .0;
    archive_circle(State(state.clone()), Path(circle.id))
        .await
        .expect("archive");

    let org = list_org_circles(State(state), Path("tenant-1".to_string()))
        .await
        .expect("org")
        .0;
    assert!(org.is_empty());
}

#[tokio::test]
async fn test_discover_public_and_semi() {
    let state = test_state();
    seed_circle(&state, "Public Circle", "user-1").await;

    let mut semi_req = create_req("Semi Circle", "user-2");
    semi_req.visibility = Some("semi_private".to_string());
    create_circle(State(state.clone()), Json(semi_req))
        .await
        .expect("semi");

    let mut priv_req = create_req("Private Circle", "user-3");
    priv_req.visibility = Some("private".to_string());
    create_circle(State(state.clone()), Json(priv_req))
        .await
        .expect("priv");

    let discovered = discover_circles(State(state)).await.expect("discover").0;
    assert_eq!(discovered.len(), 2);
}

#[tokio::test]
async fn test_discover_excludes_archived() {
    let state = test_state();
    let circle = seed_circle(&state, "To Archive", "user-1").await;
    archive_circle(State(state.clone()), Path(circle.id))
        .await
        .expect("archive");

    let discovered = discover_circles(State(state)).await.expect("discover").0;
    assert!(discovered.is_empty());
}

// ============================================================================
// Membership
// ============================================================================

#[tokio::test]
async fn test_join_circle_open() {
    let state = test_state();
    let circle = seed_circle(&state, "Open Circle", "user-1").await;

    let resp = join_circle(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join")
    .0;
    assert_eq!(resp["status"], "joined");

    let updated = get_circle(State(state), Path(circle.id))
        .await
        .expect("get")
        .0;
    assert_eq!(updated.member_count, 2);
}

#[tokio::test]
async fn test_join_circle_approval() {
    let state = test_state();
    let mut req = create_req("Approval Circle", "user-1");
    req.join_policy = Some("request_approval".to_string());
    let circle = create_circle(State(state.clone()), Json(req))
        .await
        .expect("create")
        .0;

    let resp = join_circle(
        State(state),
        Path(circle.id),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join")
    .0;
    assert_eq!(resp["status"], "requested");
}

#[tokio::test]
async fn test_join_circle_invite_only() {
    let state = test_state();
    let mut req = create_req("Invite Only", "user-1");
    req.join_policy = Some("invite_only".to_string());
    let circle = create_circle(State(state.clone()), Json(req))
        .await
        .expect("create")
        .0;

    let result = join_circle(
        State(state),
        Path(circle.id),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "FORBIDDEN");
}

#[tokio::test]
async fn test_join_circle_already_member() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;

    let result = join_circle(
        State(state),
        Path(circle.id),
        Json(JoinRequest {
            user_id: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "CONFLICT");
}

#[tokio::test]
async fn test_join_circle_not_found() {
    let state = test_state();
    let result = join_circle(
        State(state),
        Path("nonexistent".to_string()),
        Json(JoinRequest {
            user_id: "user-1".to_string(),
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_invite_members() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;

    let resp = invite_members(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(InviteRequest {
            user_ids: vec!["user-2".to_string(), "user-3".to_string()],
            invited_by: "user-1".to_string(),
        }),
    )
    .await
    .expect("invite")
    .0;
    assert_eq!(resp["count"], 2);

    let members = list_members(State(state), Path(circle.id))
        .await
        .expect("list")
        .0;
    assert_eq!(members.len(), 3); // founder + 2 invited
}

#[tokio::test]
async fn test_update_member_role() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;
    join_circle(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join");

    let updated = update_member(
        State(state),
        Path((circle.id, "user-2".to_string())),
        Json(UpdateMemberRequest {
            role: Some("reviewer".to_string()),
            status: None,
        }),
    )
    .await
    .expect("update")
    .0;
    assert_eq!(updated.role, "reviewer");
}

#[tokio::test]
async fn test_update_member_not_found() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;
    let result = update_member(
        State(state),
        Path((circle.id, "nonexistent".to_string())),
        Json(UpdateMemberRequest {
            role: None,
            status: None,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "NOT_FOUND");
}

#[tokio::test]
async fn test_remove_member_decrements_count() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;
    join_circle(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join");

    remove_member(
        State(state.clone()),
        Path((circle.id.clone(), "user-2".to_string())),
    )
    .await
    .expect("remove");

    let updated = get_circle(State(state), Path(circle.id))
        .await
        .expect("get")
        .0;
    assert_eq!(updated.member_count, 1);
}

#[tokio::test]
async fn test_list_members() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;
    join_circle(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join");

    let members = list_members(State(state), Path(circle.id))
        .await
        .expect("list")
        .0;
    assert_eq!(members.len(), 2);
}

// ============================================================================
// Feed
// ============================================================================

#[tokio::test]
async fn test_get_feed_sorted() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;

    join_circle(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(JoinRequest {
            user_id: "user-2".to_string(),
        }),
    )
    .await
    .expect("join");

    post_to_feed(
        State(state.clone()),
        Path(circle.id.clone()),
        Json(CreateFeedEntryRequest {
            actor_user_id: "user-1".to_string(),
            entry_type: Some("discussion".to_string()),
            content: "Hello world".to_string(),
            reference_id: None,
            reference_type: None,
        }),
    )
    .await
    .expect("post");

    let feed = get_feed(State(state), Path(circle.id))
        .await
        .expect("feed")
        .0;
    assert!(feed.len() >= 2);
    // Sorted newest first
    assert!(feed[0].created_at >= feed[1].created_at);
}

#[tokio::test]
async fn test_post_to_feed() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;

    let entry = post_to_feed(
        State(state),
        Path(circle.id),
        Json(CreateFeedEntryRequest {
            actor_user_id: "user-1".to_string(),
            entry_type: Some("announcement".to_string()),
            content: "Important update".to_string(),
            reference_id: None,
            reference_type: None,
        }),
    )
    .await
    .expect("post")
    .0;
    assert_eq!(entry.content, "Important update");
    assert_eq!(entry.entry_type, "announcement");
}

#[tokio::test]
async fn test_post_to_feed_empty_content() {
    let state = test_state();
    let circle = seed_circle(&state, "Circle", "user-1").await;
    let result = post_to_feed(
        State(state),
        Path(circle.id),
        Json(CreateFeedEntryRequest {
            actor_user_id: "user-1".to_string(),
            entry_type: None,
            content: "  ".to_string(),
            reference_id: None,
            reference_type: None,
        }),
    )
    .await;
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}
