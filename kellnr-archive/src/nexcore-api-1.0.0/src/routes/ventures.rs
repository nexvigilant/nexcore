//! Ventures module — strategic partnerships and investments

use crate::ApiState;
use crate::persistence::InquiryRecord;
use axum::extract::{Json, Path, State};
use axum::routing::{get, patch};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Partnership inquiry
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PartnershipInquiry {
    pub id: String,
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

/// Request to create a new partnership inquiry
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PartnershipRequest {
    pub name: String,
    pub email: String,
    pub organization: String,
    pub interest: String,
    pub message: String,
}

/// Request to update inquiry status
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateInquiryStatusRequest {
    pub status: String,
}

/// List all partnership inquiries
#[utoipa::path(
    get,
    path = "/api/v1/ventures/inquiries",
    responses(
        (status = 200, description = "List of partnership inquiries", body = Vec<PartnershipInquiry>),
    ),
    tag = "ventures"
)]
pub async fn list_inquiries(
    State(state): State<ApiState>,
) -> Result<Json<Vec<PartnershipInquiry>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_inquiries()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let responses = records
        .into_iter()
        .map(|r| PartnershipInquiry {
            id: r.id,
            name: r.name,
            email: r.email,
            organization: r.organization,
            interest: r.interest,
            message: r.message,
            created_at: r.created_at,
            status: r.status,
        })
        .collect();

    Ok(Json(responses))
}

/// Submit a new partnership inquiry
#[utoipa::path(
    post,
    path = "/api/v1/ventures/inquiries",
    request_body = PartnershipRequest,
    responses(
        (status = 201, description = "Inquiry submitted successfully", body = PartnershipInquiry),
    ),
    tag = "ventures"
)]
pub async fn submit_inquiry(
    State(state): State<ApiState>,
    Json(req): Json<PartnershipRequest>,
) -> Result<Json<PartnershipInquiry>, crate::routes::common::ApiError> {
    let inquiry = PartnershipInquiry {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        email: req.email,
        organization: req.organization,
        interest: req.interest,
        message: req.message,
        created_at: Utc::now(),
        status: "received".to_string(),
    };

    let record = InquiryRecord {
        id: inquiry.id.clone(),
        name: inquiry.name.clone(),
        email: inquiry.email.clone(),
        organization: inquiry.organization.clone(),
        interest: inquiry.interest.clone(),
        message: inquiry.message.clone(),
        created_at: inquiry.created_at,
        status: inquiry.status.clone(),
    };

    state
        .persistence
        .save_inquiry(&record)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(inquiry))
}

/// Update inquiry status
#[utoipa::path(
    patch,
    path = "/api/v1/ventures/inquiries/{id}/status",
    request_body = UpdateInquiryStatusRequest,
    responses(
        (status = 200, description = "Status updated successfully"),
    ),
    tag = "ventures"
)]
pub async fn update_status(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateInquiryStatusRequest>,
) -> Result<Json<serde_json::Value>, crate::routes::common::ApiError> {
    state
        .persistence
        .update_inquiry_status(&id, &req.status)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/inquiries", get(list_inquiries).post(submit_inquiry))
        .route("/inquiries/{id}/status", patch(update_status))
}
