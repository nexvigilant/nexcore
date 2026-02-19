//! Space clearance API endpoints.
//!
//! Consolidated from ~/projects/space-pv/nexvigilant-api/src/api/clearances.rs

use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use serde_json::{Value, json};

use super::models::{
    ClearanceDecision, ClearanceStatus, CreateClearance, ListClearancesQuery, SpaceClearance,
    SubmitClearance,
};

/// Clearance routes - mount under `/api/v1/space-pv`
#[allow(dead_code)]
pub fn router() -> Router {
    Router::new()
        .route("/clearances", get(list_clearances).post(create_clearance))
        .route("/clearances/{id}", get(get_clearance))
        .route("/clearances/{id}/submit", post(submit_clearance))
        .route("/clearances/{id}/decide", post(decide_clearance))
}

/// List clearance applications.
#[allow(dead_code)]
async fn list_clearances(
    Query(query): Query<ListClearancesQuery>,
) -> Result<Json<Vec<SpaceClearance>>, (StatusCode, Json<Value>)> {
    let _limit = query.limit.unwrap_or(100);
    let _offset = query.offset.unwrap_or(0);
    let _status_filter = query.status;
    Ok(Json(vec![]))
}

/// Get a clearance by ID.
#[allow(dead_code)]
async fn get_clearance(
    Path(id): Path<String>,
) -> Result<Json<SpaceClearance>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Clearance {} not found", id) })),
    ))
}

/// Create a new clearance application.
#[allow(dead_code)]
async fn create_clearance(
    Json(req): Json<CreateClearance>,
) -> Result<(StatusCode, Json<SpaceClearance>), (StatusCode, Json<Value>)> {
    let clearance = SpaceClearance {
        id: nexcore_id::NexId::v4().to_string(),
        drug_id: req.drug_id,
        applicant_id: String::new(),
        status: ClearanceStatus::Draft,
        mission_type: req.mission_type,
        duration_days: req.duration_days,
        crew_size: req.crew_size,
        justification: req.justification,
        stability_data: None,
        pk_data: None,
        reviewer_notes: None,
        decision_date: None,
        expiry_date: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    Ok((StatusCode::CREATED, Json(clearance)))
}

/// Submit a clearance application for review.
#[allow(dead_code)]
async fn submit_clearance(
    Path(id): Path<String>,
    Json(_req): Json<SubmitClearance>,
) -> Result<Json<SpaceClearance>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Clearance {} not found", id) })),
    ))
}

/// Make a clearance decision (regulator only).
#[allow(dead_code)]
async fn decide_clearance(
    Path(id): Path<String>,
    Json(_decision): Json<ClearanceDecision>,
) -> Result<Json<SpaceClearance>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Clearance {} not found", id) })),
    ))
}
