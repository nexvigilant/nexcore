//! Organization API endpoints.
//!
//! Consolidated from ~/projects/space-pv/nexvigilant-api/src/api/organizations.rs

use axum::{Json, Router, extract::Path, http::StatusCode, routing::get};
use serde_json::{Value, json};

use super::models::{CreateOrganization, Organization};

/// Organization routes - mount under `/api/v1/space-pv`
#[allow(dead_code)]
pub fn router() -> Router {
    Router::new()
        .route(
            "/organizations",
            get(list_organizations).post(create_organization),
        )
        .route("/organizations/{id}", get(get_organization))
}

/// List organizations.
#[allow(dead_code)]
async fn list_organizations() -> Result<Json<Vec<Organization>>, (StatusCode, Json<Value>)> {
    Ok(Json(vec![]))
}

/// Get an organization by ID.
#[allow(dead_code)]
async fn get_organization(
    Path(id): Path<String>,
) -> Result<Json<Organization>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Organization {} not found", id) })),
    ))
}

/// Register a new organization.
#[allow(dead_code)]
async fn create_organization(
    Json(req): Json<CreateOrganization>,
) -> Result<(StatusCode, Json<Organization>), (StatusCode, Json<Value>)> {
    let org = Organization {
        id: nexcore_id::NexId::v4().to_string(),
        name: req.name,
        org_type: req.org_type,
        country: req.country,
        registration_number: req.registration_number,
        created_at: nexcore_chrono::DateTime::now(),
        updated_at: nexcore_chrono::DateTime::now(),
    };
    Ok((StatusCode::CREATED, Json(org)))
}
