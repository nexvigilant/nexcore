//! Publications & Collaboration module — inter-circle research sharing.
//!
//! Enables circles to publish deliverables to the community and
//! request cross-circle collaboration (shared projects, peer review, etc.).

use crate::ApiState;
use crate::persistence::{
    CircleRole, CollabStatus, CollabType, CollaborationRequestRecord, DeliverableStatus,
    FeedEntryRecord, FeedEntryType, MemberStatus, PublicationRecord, PublicationVisibility,
};
use crate::routes::common::ApiError;
use axum::extract::{Json, Path, State};
use axum::routing::{get, patch, post};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// API Types
// ============================================================================

/// Published research for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Publication {
    pub id: String,
    pub source_circle_id: String,
    pub deliverable_id: String,
    pub title: String,
    pub abstract_text: String,
    pub visibility: String,
    pub published_at: DateTime,
    pub published_by: String,
}

/// Publish deliverable request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PublishRequest {
    pub deliverable_id: String,
    pub title: String,
    pub abstract_text: String,
    pub visibility: Option<String>,
    pub published_by: String,
}

/// Collaboration request for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Collaboration {
    pub id: String,
    pub requesting_circle_id: String,
    pub target_circle_id: String,
    pub request_type: String,
    pub message: String,
    pub status: String,
    pub created_by: String,
    pub created_at: DateTime,
}

/// Create collaboration request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CollaborateRequest {
    pub target_circle_id: String,
    pub request_type: Option<String>,
    pub message: String,
    pub created_by: String,
}

/// Update collaboration status (accept/decline)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateCollaborationRequest {
    pub status: String,
}

// ============================================================================
// Helpers
// ============================================================================

fn record_to_publication(r: PublicationRecord) -> Publication {
    Publication {
        id: r.id,
        source_circle_id: r.source_circle_id,
        deliverable_id: r.deliverable_id,
        title: r.title,
        abstract_text: r.abstract_text,
        visibility: enum_to_str(&r.visibility),
        published_at: r.published_at,
        published_by: r.published_by,
    }
}

fn record_to_collaboration(r: CollaborationRequestRecord) -> Collaboration {
    Collaboration {
        id: r.id,
        requesting_circle_id: r.requesting_circle_id,
        target_circle_id: r.target_circle_id,
        request_type: enum_to_str(&r.request_type),
        message: r.message,
        status: enum_to_str(&r.status),
        created_by: r.created_by,
        created_at: r.created_at,
    }
}

fn enum_to_str<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_string(val)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn parse_enum<T: serde::de::DeserializeOwned + Default>(s: &str) -> T {
    let quoted = format!("\"{s}\"");
    serde_json::from_str(&quoted).unwrap_or_default()
}

fn err(code: &str, msg: impl Into<String>) -> ApiError {
    ApiError::new(code, msg)
}

/// Best-effort feed entry save.
async fn save_feed_best_effort(state: &ApiState, entry: FeedEntryRecord) {
    if let Err(e) = state.persistence.save_feed_entry(&entry).await {
        tracing::warn!("Failed to save feed entry: {e}");
    }
}

/// Check if user has at least Lead role in a circle.
async fn check_lead(state: &ApiState, circle_id: &str, user_id: &str) -> Result<bool, ApiError> {
    let member = state
        .persistence
        .get_circle_member(circle_id, user_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    let Some(m) = member else {
        return Ok(false);
    };
    if m.status != MemberStatus::Active {
        return Ok(false);
    }

    Ok(matches!(m.role, CircleRole::Founder | CircleRole::Lead))
}

// ============================================================================
// Publication Endpoints
// ============================================================================

/// Publish a deliverable to the community (Lead+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/publish",
    request_body = PublishRequest,
    responses(
        (status = 200, description = "Deliverable published", body = Publication),
    ),
    tag = "publications"
)]
pub async fn publish_deliverable(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<Publication>, ApiError> {
    // Require Lead+
    if !check_lead(&state, &circle_id, &req.published_by).await? {
        return Err(err("FORBIDDEN", "Requires Lead role or higher to publish"));
    }

    // Verify deliverable exists and is approved
    let deliverable = state
        .persistence
        .get_deliverable(&req.deliverable_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Deliverable not found"))?;

    if deliverable.circle_id != circle_id {
        return Err(err("NOT_FOUND", "Deliverable not found in this circle"));
    }

    if deliverable.status != DeliverableStatus::Approved {
        return Err(err(
            "CONFLICT",
            "Only approved deliverables can be published",
        ));
    }

    let now = DateTime::now();
    let record = PublicationRecord {
        id: nexcore_id::NexId::v4().to_string(),
        source_circle_id: circle_id.clone(),
        deliverable_id: req.deliverable_id,
        title: req.title,
        abstract_text: req.abstract_text,
        visibility: req
            .visibility
            .map(|v| parse_enum(&v))
            .unwrap_or(PublicationVisibility::Community),
        published_at: now,
        published_by: req.published_by.clone(),
    };

    state
        .persistence
        .save_publication(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Update deliverable status to Published
    let mut updated_deliverable = deliverable;
    updated_deliverable.status = DeliverableStatus::Published;
    updated_deliverable.updated_at = now;
    if let Err(e) = state
        .persistence
        .save_deliverable(&updated_deliverable)
        .await
    {
        tracing::warn!("Failed to update deliverable status to Published: {e}");
    }

    // Increment publication count on circle
    if let Ok(Some(mut circle)) = state.persistence.get_circle(&circle_id).await {
        circle.publication_count += 1;
        circle.updated_at = now;
        if let Err(e) = state.persistence.save_circle(&circle).await {
            tracing::warn!("Failed to update circle publication count: {e}");
        }
    }

    // Feed entry
    save_feed_best_effort(
        &state,
        FeedEntryRecord {
            id: nexcore_id::NexId::v4().to_string(),
            circle_id,
            entry_type: FeedEntryType::ReviewCompleted,
            actor_user_id: req.published_by,
            content: format!("Published: {}", record.title),
            reference_id: Some(record.id.clone()),
            reference_type: Some("publication".to_string()),
            created_at: now,
        },
    )
    .await;

    Ok(Json(record_to_publication(record)))
}

/// Browse published research (community-visible)
#[utoipa::path(
    get,
    path = "/api/v1/publications",
    responses(
        (status = 200, description = "Published research", body = Vec<Publication>),
    ),
    tag = "publications"
)]
pub async fn list_publications(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Publication>>, ApiError> {
    let records = state
        .persistence
        .list_publications()
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    // Only return community-visible publications
    let pubs: Vec<Publication> = records
        .into_iter()
        .filter(|r| r.visibility == PublicationVisibility::Community)
        .map(record_to_publication)
        .collect();

    Ok(Json(pubs))
}

/// List publications from a specific circle
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}/publications",
    responses(
        (status = 200, description = "Circle publications", body = Vec<Publication>),
    ),
    tag = "publications"
)]
pub async fn list_circle_publications(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
) -> Result<Json<Vec<Publication>>, ApiError> {
    let records = state
        .persistence
        .list_circle_publications(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(
        records.into_iter().map(record_to_publication).collect(),
    ))
}

// ============================================================================
// Collaboration Endpoints
// ============================================================================

/// Request cross-circle collaboration (Lead+ only)
#[utoipa::path(
    post,
    path = "/api/v1/circles/{id}/collaborate",
    request_body = CollaborateRequest,
    responses(
        (status = 200, description = "Collaboration requested", body = Collaboration),
    ),
    tag = "publications"
)]
pub async fn request_collaboration(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<CollaborateRequest>,
) -> Result<Json<Collaboration>, ApiError> {
    // Require Lead+
    if !check_lead(&state, &circle_id, &req.created_by).await? {
        return Err(err("FORBIDDEN", "Requires Lead role or higher"));
    }

    // Verify target circle exists
    state
        .persistence
        .get_circle(&req.target_circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Target circle not found"))?;

    if circle_id == req.target_circle_id {
        return Err(err("VALIDATION_ERROR", "Cannot collaborate with yourself"));
    }

    let now = DateTime::now();
    let record = CollaborationRequestRecord {
        id: nexcore_id::NexId::v4().to_string(),
        requesting_circle_id: circle_id,
        target_circle_id: req.target_circle_id,
        request_type: req
            .request_type
            .map(|t| parse_enum(&t))
            .unwrap_or(CollabType::SharedProject),
        message: req.message,
        status: CollabStatus::Pending,
        created_by: req.created_by,
        created_at: now,
    };

    state
        .persistence
        .save_collaboration(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_collaboration(record)))
}

/// List collaboration requests for a circle (both sent and received)
#[utoipa::path(
    get,
    path = "/api/v1/circles/{id}/collaborations",
    responses(
        (status = 200, description = "Collaboration requests", body = Vec<Collaboration>),
    ),
    tag = "publications"
)]
pub async fn list_collaborations(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
) -> Result<Json<Vec<Collaboration>>, ApiError> {
    let records = state
        .persistence
        .list_collaborations_for_circle(&circle_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(
        records.into_iter().map(record_to_collaboration).collect(),
    ))
}

/// Update collaboration status (accept/decline/withdraw)
#[utoipa::path(
    patch,
    path = "/api/v1/collaborations/{id}",
    request_body = UpdateCollaborationRequest,
    responses(
        (status = 200, description = "Collaboration updated", body = Collaboration),
    ),
    tag = "publications"
)]
pub async fn update_collaboration(
    State(state): State<ApiState>,
    Path(collab_id): Path<String>,
    Json(req): Json<UpdateCollaborationRequest>,
) -> Result<Json<Collaboration>, ApiError> {
    let mut record = state
        .persistence
        .get_collaboration(&collab_id)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?
        .ok_or_else(|| err("NOT_FOUND", "Collaboration request not found"))?;

    record.status = parse_enum(&req.status);

    state
        .persistence
        .save_collaboration(&record)
        .await
        .map_err(|e| err("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(record_to_collaboration(record)))
}

// ============================================================================
// Routers
// ============================================================================

/// Circle-scoped publication/collaboration routes (nested under /circles/{id})
pub fn circle_router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/publish", post(publish_deliverable))
        .route("/publications", get(list_circle_publications))
        .route("/collaborate", post(request_collaboration))
        .route("/collaborations", get(list_collaborations))
}

/// Top-level routes
pub fn router() -> axum::Router<ApiState> {
    axum::Router::new().route("/", get(list_publications))
}

/// Collaboration management routes (top-level)
pub fn collab_router() -> axum::Router<ApiState> {
    axum::Router::new().route("/{id}", patch(update_collaboration))
}
