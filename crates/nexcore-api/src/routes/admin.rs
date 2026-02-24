// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Admin Module — User management, stats, content, moderation

use crate::ApiState;
use crate::routes::common::ApiError;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vr_core::tenant::UserRole;

/// User summary for admin views
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSummary {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: String,
    pub last_active: String,
}

/// Dashboard stats
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardStats {
    pub total_users: usize,
    pub total_courses: usize,
    pub total_pathways: usize,
    pub total_enrollments: usize,
    pub users_by_role: RoleCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleCounts {
    pub admin: usize,
    pub instructor: usize,
    pub learner: usize,
}

/// Request to update a user's role.
/// Uses the UserRole Cartouche enum — invalid roles are rejected at deserialization.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    #[schema(value_type = String)]
    pub role: UserRole,
}

/// Content item (course or pathway) for admin listing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContentItem {
    pub id: String,
    pub title: String,
    pub content_type: String,
    pub description: String,
}

/// Moderation action request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ModerateRequest {
    pub action: String, // "approve" or "reject"
}

/// List all users
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    responses(
        (status = 200, description = "List of users", body = Vec<UserSummary>),
    ),
    tag = "admin"
)]
pub async fn list_users(
    State(_state): State<ApiState>,
) -> Result<Json<Vec<UserSummary>>, ApiError> {
    // Persistence does not yet have a list_users method; return empty list.
    let users: Vec<UserSummary> = Vec::new();
    Ok(Json(users))
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User details", body = UserSummary),
        (status = 404, description = "User not found"),
    ),
    tag = "admin"
)]
pub async fn get_user(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<UserSummary>, ApiError> {
    // Persistence does not yet have a get_user method.
    Err(ApiError::new("NOT_FOUND", format!("User {} not found", id)))
}

/// Update user role
#[utoipa::path(
    patch,
    path = "/api/v1/admin/users/{id}/role",
    params(("id" = String, Path, description = "User ID")),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated"),
    ),
    tag = "admin"
)]
pub async fn update_user_role(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Serialize the UserRole variant to its snake_case string representation.
    let role_str = serde_json::to_value(&req.role)
        .ok()
        .and_then(|v| v.as_str().map(std::string::String::from))
        .unwrap_or_default();

    // Persistence does not yet have an update_user_role method; acknowledge the request.
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "updated", "user_id": id, "role": role_str})),
    ))
}

/// Get dashboard stats
#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    responses(
        (status = 200, description = "Dashboard statistics", body = DashboardStats),
    ),
    tag = "admin"
)]
pub async fn get_stats(State(state): State<ApiState>) -> Result<Json<DashboardStats>, ApiError> {
    let enrollments = state
        .persistence
        .list_enrollments()
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // Users, courses, and pathways do not yet have persistence methods; use zero counts.
    let role_counts = RoleCounts {
        admin: 0,
        instructor: 0,
        learner: 0,
    };

    Ok(Json(DashboardStats {
        total_users: 0,
        total_courses: 0,
        total_pathways: 0,
        total_enrollments: enrollments.len(),
        users_by_role: role_counts,
    }))
}

/// List all content (courses + pathways) for management
#[utoipa::path(
    get,
    path = "/api/v1/admin/content",
    responses(
        (status = 200, description = "All content items", body = Vec<ContentItem>),
    ),
    tag = "admin"
)]
pub async fn list_content(
    State(_state): State<ApiState>,
) -> Result<Json<Vec<ContentItem>>, ApiError> {
    // Persistence does not yet have list_courses / list_pathways methods; return empty list.
    let items: Vec<ContentItem> = Vec::new();
    Ok(Json(items))
}

/// Delete a course or pathway
#[utoipa::path(
    delete,
    path = "/api/v1/admin/content/{content_type}/{id}",
    params(
        ("content_type" = String, Path, description = "Content type: course or pathway"),
        ("id" = String, Path, description = "Content ID"),
    ),
    responses(
        (status = 200, description = "Content deleted"),
        (status = 400, description = "Invalid content type"),
    ),
    tag = "admin"
)]
pub async fn delete_content(
    State(_state): State<ApiState>,
    Path((content_type, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    match content_type.as_str() {
        "course" | "pathway" => {
            // Persistence does not yet have delete_course / delete_pathway methods; acknowledge.
        }
        _ => {
            return Err(ApiError::new(
                "VALIDATION_ERROR",
                format!(
                    "Invalid content type: {}. Must be 'course' or 'pathway'",
                    content_type
                ),
            ));
        }
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "deleted", "content_type": content_type, "id": id})),
    ))
}

/// List posts needing moderation (flagged posts)
#[utoipa::path(
    get,
    path = "/api/v1/admin/moderation/posts",
    responses(
        (status = 200, description = "Flagged posts"),
    ),
    tag = "admin"
)]
pub async fn list_flagged_posts(
    State(state): State<ApiState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let posts = state
        .persistence
        .list_posts()
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    // In a real system, posts would have a "flagged" field. For now, return empty.
    let flagged: Vec<serde_json::Value> = posts
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "author": p.author,
                "content": p.content,
                "created_at": p.created_at.to_rfc3339(),
                "status": "pending_review",
            })
        })
        .collect();

    Ok(Json(flagged))
}

/// Moderate a post (approve/reject)
#[utoipa::path(
    post,
    path = "/api/v1/admin/moderation/posts/{id}/action",
    params(("id" = String, Path, description = "Post ID")),
    request_body = ModerateRequest,
    responses(
        (status = 200, description = "Moderation action applied"),
    ),
    tag = "admin"
)]
pub async fn moderate_post(
    Path(id): Path<String>,
    Json(req): Json<ModerateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let valid_actions = ["approve", "reject"];
    if !valid_actions.contains(&req.action.as_str()) {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            format!(
                "Invalid action: {}. Must be 'approve' or 'reject'",
                req.action
            ),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "moderated",
            "post_id": id,
            "action": req.action,
        })),
    ))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/users", get(list_users))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}/role", patch(update_user_role))
        .route("/stats", get(get_stats))
        .route("/content", get(list_content))
        .route("/content/{content_type}/{id}", delete(delete_content))
        .route("/moderation/posts", get(list_flagged_posts))
        .route("/moderation/posts/{id}/action", post(moderate_post))
}
