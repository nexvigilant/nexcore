//! Circles module — the organizational primitive of the platform.
//!
//! A Circle is a working unit that produces real research output
//! across the pharmaceutical lifecycle. Not a chat room.

use crate::ApiState;
use crate::persistence::{
    CircleFormation, CircleMemberRecord, CircleRecord, CircleRole, CircleStatus, CircleType,
    CircleVisibility, FeedEntryRecord, FeedEntryType, JoinPolicy, MemberStatus,
};
use crate::routes::common::ApiError;
use axum::extract::{Json, Path, State};
use axum::routing::{delete, get, patch, post};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// API Types
// ============================================================================

/// Circle summary for list views
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub mission: Option<String>,
    pub formation: String,
    pub visibility: String,
    pub join_policy: String,
    pub circle_type: String,
    pub therapeutic_areas: Vec<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub member_count: u32,
    pub project_count: u32,
    pub publication_count: u32,
    pub created_by: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// Create circle request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCircleRequest {
    pub name: String,
    pub description: String,
    pub mission: Option<String>,
    pub formation: Option<String>,
    pub tenant_id: Option<String>,
    pub created_by: String,
    pub visibility: Option<String>,
    pub join_policy: Option<String>,
    pub circle_type: Option<String>,
    pub therapeutic_areas: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Update circle request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateCircleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub mission: Option<String>,
    pub visibility: Option<String>,
    pub join_policy: Option<String>,
    pub status: Option<String>,
    pub therapeutic_areas: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Circle member for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CircleMember {
    pub id: String,
    pub circle_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub joined_at: DateTime,
    pub invited_by: Option<String>,
}

/// Join/invite request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct JoinRequest {
    pub user_id: String,
}

/// Invite request (for leads to invite users)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct InviteRequest {
    pub user_ids: Vec<String>,
    pub invited_by: String,
}

/// Update member request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
    pub role: Option<String>,
    pub status: Option<String>,
}

/// Feed entry for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeedEntry {
    pub id: String,
    pub circle_id: String,
    pub entry_type: String,
    pub actor_user_id: String,
    pub content: String,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
    pub created_at: DateTime,
}

/// Post to feed request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateFeedEntryRequest {
    pub actor_user_id: String,
    pub entry_type: Option<String>,
    pub content: String,
    pub reference_id: Option<String>,
    pub reference_type: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn record_to_circle(r: CircleRecord) -> Circle {
    Circle {
        id: r.id,
        name: r.name,
        slug: r.slug,
        description: r.description,
        mission: r.mission,
        formation: serde_json::to_string(&r.formation)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        visibility: serde_json::to_string(&r.visibility)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        join_policy: serde_json::to_string(&r.join_policy)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        circle_type: serde_json::to_string(&r.circle_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        therapeutic_areas: r.therapeutic_areas,
        tags: r.tags,
        status: serde_json::to_string(&r.status)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        member_count: r.member_count,
        project_count: r.project_count,
        publication_count: r.publication_count,
        created_by: r.created_by,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn member_to_api(m: CircleMemberRecord) -> CircleMember {
    CircleMember {
        id: m.id,
        circle_id: m.circle_id,
        user_id: m.user_id,
        role: serde_json::to_string(&m.role)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        status: serde_json::to_string(&m.status)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        joined_at: m.joined_at,
        invited_by: m.invited_by,
    }
}

fn feed_to_api(e: FeedEntryRecord) -> FeedEntry {
    FeedEntry {
        id: e.id,
        circle_id: e.circle_id,
        entry_type: serde_json::to_string(&e.entry_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        actor_user_id: e.actor_user_id,
        content: e.content,
        reference_id: e.reference_id,
        reference_type: e.reference_type,
        created_at: e.created_at,
    }
}

fn parse_enum<T: serde::de::DeserializeOwned + Default>(s: &str) -> T {
    let quoted = format!("\"{}\"", s);
    serde_json::from_str(&quoted).unwrap_or_default()
}

fn err(code: &str, msg: impl Into<String>) -> ApiError {
    ApiError::new(code, msg)
}

/// Best-effort save of a feed entry (non-critical side effect).
async fn save_feed_best_effort(state: &ApiState, entry: FeedEntryRecord) {
    if let Err(e) = state.persistence.save_feed_entry(&entry).await {
        tracing::warn!("Failed to save feed entry: {e}");
    }
}

/// Best-effort update of circle counts (non-critical denormalized data).
async fn update_circle_count_best_effort(state: &ApiState, circle: &CircleRecord) {
    if let Err(e) = state.persistence.save_circle(circle).await {
        tracing::warn!("Failed to update circle counts: {e}");
    }
}

// ============================================================================
// Circle CRUD
// ============================================================================

/// Create a new circle
#[utoipa::path(
    post,
    path = "/api/v1/circles",
    request_body = CreateCircleRequest,
    responses(
        (status = 200, description = "Circle created", body = Circle),
    ),
    tag = "circles"
)]
pub async fn create_circle(
    State(state): State<ApiState>,
    Json(req): Json<CreateCircleRequest>,
) -> Result<Json<Circle>, ApiError> {
    if req.name.trim().is_empty() {
        return Err(err("VALIDATION_ERROR", "Circle name is required"));
    }

    let now = DateTime::now();
    let slug = slugify(&req.name);

    // Check slug uniqueness
    if let Some(_existing) = state
        .persistence
        .get_circle_by_slug(&slug)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
    {
        return Err(err("CONFLICT", "A circle with this name already exists"));
    }

    let record = CircleRecord {
        id: nexcore_id::NexId::v4().to_string(),
        name: req.name,
        slug,
        description: req.description,
        mission: req.mission,
        formation: req
            .formation
            .map(|f| parse_enum(&f))
            .unwrap_or(CircleFormation::AdHoc),
        tenant_id: req.tenant_id,
        created_by: req.created_by.clone(),
        visibility: req
            .visibility
            .map(|v| parse_enum(&v))
            .unwrap_or(CircleVisibility::Public),
        join_policy: req
            .join_policy
            .map(|j| parse_enum(&j))
            .unwrap_or(JoinPolicy::Open),
        circle_type: req
            .circle_type
            .map(|c| parse_enum(&c))
            .unwrap_or(CircleType::WorkingGroup),
        therapeutic_areas: req.therapeutic_areas.unwrap_or_default(),
        tags: req.tags.unwrap_or_default(),
        status: CircleStatus::Active,
        created_at: now,
        updated_at: now,
        member_count: 1, // Founder
        project_count: 0,
        publication_count: 0,
    };

    state
        .persistence
        .save_circle(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Auto-add founder as member
    let founder = CircleMemberRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id: record.id.clone(),
        user_id: req.created_by.clone(),
        role: CircleRole::Founder,
        status: MemberStatus::Active,
        joined_at: now,
        invited_by: None,
    };
    state
        .persistence
        .save_circle_member(&founder)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_circle(record)))
}

/// List circles (filtered by visibility for non-members)
#[utoipa::path(
    get,
    path = "/api/v1/circles",
    responses(
        (status = 200, description = "List of circles", body = Vec<Circle>),
    ),
    tag = "circles"
)]
pub async fn list_circles(State(state): State<ApiState>) -> Result<Json<Vec<Circle>>, ApiError> {
    let records = state
        .persistence
        .list_circles()
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Only return public + semi-private circles (private are invisible to non-members)
    let circles: Vec<Circle> = records
        .into_iter()
        .filter(|r| r.visibility != CircleVisibility::Private && r.status == CircleStatus::Active)
        .map(record_to_circle)
        .collect();

    Ok(Json(circles))
}

/// Get circle by ID
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}",
    responses(
        (status = 200, description = "Circle detail", body = Circle),
    ),
    tag = "circles"
)]
pub async fn get_circle(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<Circle>, ApiError> {
    let record = state
        .persistence
        .get_circle(&id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Circle not found"))?;

    Ok(Json(record_to_circle(record)))
}

/// Update a circle (Lead+ only)
#[utoipa::path(
    patch,
    path = "/api/v1/circles/{id}",
    request_body = UpdateCircleRequest,
    responses(
        (status = 200, description = "Circle updated", body = Circle),
    ),
    tag = "circles"
)]
pub async fn update_circle(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCircleRequest>,
) -> Result<Json<Circle>, ApiError> {
    let mut record = state
        .persistence
        .get_circle(&id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Circle not found"))?;

    if let Some(name) = req.name {
        record.name = name;
    }
    if let Some(description) = req.description {
        record.description = description;
    }
    if let Some(mission) = req.mission {
        record.mission = Some(mission);
    }
    if let Some(visibility) = req.visibility {
        record.visibility = parse_enum(&visibility);
    }
    if let Some(join_policy) = req.join_policy {
        record.join_policy = parse_enum(&join_policy);
    }
    if let Some(status) = req.status {
        record.status = parse_enum(&status);
    }
    if let Some(areas) = req.therapeutic_areas {
        record.therapeutic_areas = areas;
    }
    if let Some(tags) = req.tags {
        record.tags = tags;
    }
    record.updated_at = DateTime::now();

    state
        .persistence
        .save_circle(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_circle(record)))
}

/// Archive a circle (Founder only)
#[utoipa::path(
    delete,
    path = "/api/v1/circles/{id}",
    responses(
        (status = 200, description = "Circle archived"),
    ),
    tag = "circles"
)]
pub async fn archive_circle(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut record = state
        .persistence
        .get_circle(&id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Circle not found"))?;

    record.status = CircleStatus::Archived;
    record.updated_at = DateTime::now();

    state
        .persistence
        .save_circle(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

/// Get user's circles
#[utoipa::path(
    get,
    path = "/api/v1/circles/my/{user_id}",
    responses(
        (status = 200, description = "User's circles", body = Vec<Circle>),
    ),
    tag = "circles"
)]
pub async fn my_circles(
    State(state): State<ApiState>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Circle>>, ApiError> {
    let all_circles = state
        .persistence
        .list_circles()
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let mut user_circles = Vec::new();
    for circle in all_circles {
        let member = state
            .persistence
            .get_circle_member(&circle.id, &user_id)
            .await
            .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;
        if let Some(m) = member {
            if m.status == MemberStatus::Active {
                user_circles.push(record_to_circle(circle));
            }
        }
    }

    Ok(Json(user_circles))
}

/// List org-backed circles for a specific tenant
#[utoipa::path(
    get,
    path = "/api/v1/circles/org/{tenant_id}",
    responses(
        (status = 200, description = "Organization circles", body = Vec<Circle>),
    ),
    tag = "circles"
)]
pub async fn list_org_circles(
    State(state): State<ApiState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<Circle>>, ApiError> {
    let records = state
        .persistence
        .list_circles_by_tenant(&tenant_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let circles: Vec<Circle> = records
        .into_iter()
        .filter(|r| r.status == CircleStatus::Active)
        .map(record_to_circle)
        .collect();

    Ok(Json(circles))
}

/// Discovery feed (public + semi-private circles)
#[utoipa::path(
    get,
    path = "/api/v1/circles/discover",
    responses(
        (status = 200, description = "Discoverable circles", body = Vec<Circle>),
    ),
    tag = "circles"
)]
pub async fn discover_circles(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Circle>>, ApiError> {
    let records = state
        .persistence
        .list_circles()
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let circles: Vec<Circle> = records
        .into_iter()
        .filter(|r| r.status == CircleStatus::Active)
        .filter(|r| {
            r.visibility == CircleVisibility::Public
                || r.visibility == CircleVisibility::SemiPrivate
        })
        .map(record_to_circle)
        .collect();

    Ok(Json(circles))
}

// ============================================================================
// Membership
// ============================================================================

/// Join a circle (open) or request to join (approval)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/join",
    request_body = JoinRequest,
    responses(
        (status = 200, description = "Joined or request submitted"),
    ),
    tag = "circles"
)]
pub async fn join_circle(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<JoinRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let circle = state
        .persistence
        .get_circle(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Circle not found"))?;

    // Check if already a member
    if let Some(existing) = state
        .persistence
        .get_circle_member(&circle_id, &req.user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
    {
        if existing.status == MemberStatus::Active {
            return Err(err("CONFLICT", "Already a member"));
        }
    }

    let now = DateTime::now();
    let (member_status, response_status) = match circle.join_policy {
        JoinPolicy::Open => (MemberStatus::Active, "joined"),
        JoinPolicy::RequestApproval => (MemberStatus::Requested, "requested"),
        JoinPolicy::InviteOnly => {
            return Err(err("FORBIDDEN", "This circle is invite-only"));
        }
    };

    let member = CircleMemberRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id: circle_id.clone(),
        user_id: req.user_id.clone(),
        role: CircleRole::Member,
        status: member_status.clone(),
        joined_at: now,
        invited_by: None,
    };

    state
        .persistence
        .save_circle_member(&member)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Update member count if joined directly
    if member_status == MemberStatus::Active {
        let mut updated_circle = circle;
        updated_circle.member_count += 1;
        updated_circle.updated_at = now;
        update_circle_count_best_effort(&state, &updated_circle).await;

        // Auto-feed entry
        let entry = FeedEntryRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id: circle_id.clone(),
            entry_type: FeedEntryType::MemberJoined,
            actor_user_id: req.user_id,
            content: String::new(),
            reference_id: None,
            reference_type: None,
            created_at: now,
        };
        save_feed_best_effort(&state, entry).await;
    }

    Ok(Json(serde_json::json!({ "status": response_status })))
}

/// Invite users to a circle (Lead+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/invite",
    request_body = InviteRequest,
    responses(
        (status = 200, description = "Invitations sent"),
    ),
    tag = "circles"
)]
pub async fn invite_members(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<InviteRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = DateTime::now();
    let mut invited_count = 0u32;

    for user_id in &req.user_ids {
        let member = CircleMemberRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id: circle_id.clone(),
            user_id: user_id.clone(),
            role: CircleRole::Member,
            status: MemberStatus::Invited,
            joined_at: now,
            invited_by: Some(req.invited_by.clone()),
        };
        state
            .persistence
            .save_circle_member(&member)
            .await
            .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;
        invited_count += 1;
    }

    Ok(Json(
        serde_json::json!({ "status": "invited", "count": invited_count }),
    ))
}

/// Update a member's role or status
#[utoipa::path(
    patch,
    path = "/api/v1/circles/{cid}/members/{uid}",
    request_body = UpdateMemberRequest,
    responses(
        (status = 200, description = "Member updated", body = CircleMember),
    ),
    tag = "circles"
)]
pub async fn update_member(
    State(state): State<ApiState>,
    Path((circle_id, user_id)): Path<(String, String)>,
    Json(req): Json<UpdateMemberRequest>,
) -> Result<Json<CircleMember>, ApiError> {
    let mut member = state
        .persistence
        .get_circle_member(&circle_id, &user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Member not found"))?;

    let was_active = member.status == MemberStatus::Active;

    if let Some(role) = req.role {
        member.role = parse_enum(&role);
    }
    if let Some(status) = req.status {
        member.status = parse_enum(&status);
    }

    state
        .persistence
        .update_circle_member(&member)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Update member count if status changed to/from active
    let is_active = member.status == MemberStatus::Active;
    if was_active != is_active {
        if let Ok(Some(mut circle)) = state.persistence.get_circle(&circle_id).await {
            if is_active {
                circle.member_count += 1;
            } else if circle.member_count > 0 {
                circle.member_count -= 1;
            }
            circle.updated_at = DateTime::now();
            update_circle_count_best_effort(&state, &circle).await;
        }
    }

    Ok(Json(member_to_api(member)))
}

/// Leave or remove a member
#[utoipa::path(
    delete,
    path = "/api/v1/circles/{cid}/members/{uid}",
    responses(
        (status = 200, description = "Member removed"),
    ),
    tag = "circles"
)]
pub async fn remove_member(
    State(state): State<ApiState>,
    Path((circle_id, user_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .persistence
        .delete_circle_member(&circle_id, &user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Decrement member count
    if let Ok(Some(mut circle)) = state.persistence.get_circle(&circle_id).await {
        if circle.member_count > 0 {
            circle.member_count -= 1;
        }
        circle.updated_at = DateTime::now();
        update_circle_count_best_effort(&state, &circle).await;
    }

    // Feed entry
    let entry = FeedEntryRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id,
        entry_type: FeedEntryType::MemberLeft,
        actor_user_id: user_id,
        content: String::new(),
        reference_id: None,
        reference_type: None,
        created_at: DateTime::now(),
    };
    save_feed_best_effort(&state, entry).await;

    Ok(Json(serde_json::json!({ "status": "removed" })))
}

/// List circle members
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}/members",
    responses(
        (status = 200, description = "Circle members", body = Vec<CircleMember>),
    ),
    tag = "circles"
)]
pub async fn list_members(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
) -> Result<Json<Vec<CircleMember>>, ApiError> {
    let members = state
        .persistence
        .list_circle_members(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(members.into_iter().map(member_to_api).collect()))
}

// ============================================================================
// Feed
// ============================================================================

/// Get circle activity feed
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}/feed",
    responses(
        (status = 200, description = "Circle activity feed", body = Vec<FeedEntry>),
    ),
    tag = "circles"
)]
pub async fn get_feed(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
) -> Result<Json<Vec<FeedEntry>>, ApiError> {
    let entries = state
        .persistence
        .list_feed_entries(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let mut feed: Vec<FeedEntry> = entries.into_iter().map(feed_to_api).collect();
    feed.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(feed))
}

/// Post to circle feed (discussion/update/announcement)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/feed",
    request_body = CreateFeedEntryRequest,
    responses(
        (status = 200, description = "Entry posted", body = FeedEntry),
    ),
    tag = "circles"
)]
pub async fn post_to_feed(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<CreateFeedEntryRequest>,
) -> Result<Json<FeedEntry>, ApiError> {
    if req.content.trim().is_empty() {
        return Err(err("VALIDATION_ERROR", "Content is required"));
    }

    let entry = FeedEntryRecord {
        id: nexcore_id::NexId::v4().to_string(),
        circle_id,
        entry_type: req
            .entry_type
            .map(|t| parse_enum(&t))
            .unwrap_or(FeedEntryType::Discussion),
        actor_user_id: req.actor_user_id,
        content: req.content,
        reference_id: req.reference_id,
        reference_type: req.reference_type,
        created_at: DateTime::now(),
    };

    state
        .persistence
        .save_feed_entry(&entry)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(feed_to_api(entry)))
}

// ============================================================================
// Router
// ============================================================================

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        // Circle CRUD
        .route("/", get(list_circles).post(create_circle))
        .route("/discover", get(discover_circles))
        .route("/org/{tenant_id}", get(list_org_circles))
        .route("/my/{user_id}", get(my_circles))
        .route(
            "/{id}",
            get(get_circle).patch(update_circle).delete(archive_circle),
        )
        // Membership
        .route("/{id}/join", post(join_circle))
        .route("/{id}/invite", post(invite_members))
        .route("/{id}/members", get(list_members))
        .route(
            "/{cid}/members/{uid}",
            patch(update_member).delete(remove_member),
        )
        // Feed
        .route("/{id}/feed", get(get_feed).post(post_to_feed))
        // Projects & Deliverables (nested)
        .nest("/{id}/projects", super::projects::router())
        // Project Tools (MCP integration, nested under projects)
        .nest(
            "/{cid}/projects/{pid}/tools",
            super::project_tools::router(),
        )
        // Publications & Collaboration (nested under circle)
        .nest("/{id}", super::publications::circle_router())
}
