//! Circles module — topic-focused professional groups

use crate::ApiState;
use crate::persistence::{CircleRecord, MembershipRecord};
use axum::extract::{Json, Path, State};
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Professional Circle
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Circle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: u32,
    pub post_count: u32,
    pub created_at: DateTime<Utc>,
}

/// Join circle request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct JoinRequest {
    pub user_id: String,
}

/// List all professional circles
#[utoipa::path(
    get,
    path = "/api/v1/community/circles",
    responses(
        (status = 200, description = "List of circles", body = Vec<Circle>),
    ),
    tag = "community"
)]
pub async fn list_circles(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Circle>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_circles()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let mut circles: Vec<Circle> = records
        .into_iter()
        .map(|r| Circle {
            id: r.id,
            name: r.name,
            description: r.description,
            member_count: r.member_count,
            post_count: r.post_count,
            created_at: r.created_at,
        })
        .collect();

    // Seed data if empty
    if circles.is_empty() {
        circles = vec![
            Circle {
                id: "c1".to_string(),
                name: "Signal Detection".to_string(),
                description: "Disproportionality analysis, data mining, and signal evaluation"
                    .to_string(),
                member_count: 156,
                post_count: 43,
                created_at: Utc::now(),
            },
            Circle {
                id: "c2".to_string(),
                name: "Case Processing".to_string(),
                description: "ICSR handling, coding, narrative writing".to_string(),
                member_count: 234,
                post_count: 87,
                created_at: Utc::now(),
            },
        ];
    }

    Ok(Json(circles))
}

/// Join a circle
#[utoipa::path(
    post,
    path = "/api/v1/community/circles/{id}/join",
    request_body = JoinRequest,
    responses(
        (status = 200, description = "Joined successfully"),
    ),
    tag = "community"
)]
pub async fn join_circle(
    State(state): State<ApiState>,
    Path(circle_id): Path<String>,
    Json(req): Json<JoinRequest>,
) -> Result<Json<serde_json::Value>, crate::routes::common::ApiError> {
    let membership = MembershipRecord {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: req.user_id,
        circle_id,
        joined_at: Utc::now(),
    };

    state
        .persistence
        .save_membership(&membership)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", get(list_circles))
        .route("/{id}/join", post(join_circle))
}
