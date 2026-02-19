//! Drug registry API endpoints.
//!
//! Consolidated from ~/projects/space-pv/nexvigilant-api/src/api/drugs.rs

use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
};
use serde_json::{Value, json};

use super::models::{CreateDrug, Drug, ListDrugsQuery, UpdateDrug};

/// Drug routes - mount under `/api/v1/space-pv`
#[allow(dead_code)]
pub fn router() -> Router {
    Router::new()
        .route("/drugs", get(list_drugs).post(create_drug))
        .route(
            "/drugs/{id}",
            get(get_drug).patch(update_drug).delete(delete_drug),
        )
}

/// List drugs in the registry.
#[allow(dead_code)]
async fn list_drugs(
    Query(query): Query<ListDrugsQuery>,
) -> Result<Json<Vec<Drug>>, (StatusCode, Json<Value>)> {
    // TODO: Implement database integration using query filters
    let _limit = query.limit.unwrap_or(100);
    let _offset = query.offset.unwrap_or(0);
    Ok(Json(vec![]))
}

/// Get a specific drug by ID.
#[allow(dead_code)]
async fn get_drug(Path(id): Path<String>) -> Result<Json<Drug>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Drug {} not found", id) })),
    ))
}

/// Register a new drug.
#[allow(dead_code)]
async fn create_drug(
    Json(req): Json<CreateDrug>,
) -> Result<(StatusCode, Json<Drug>), (StatusCode, Json<Value>)> {
    let drug = Drug {
        id: nexcore_id::NexId::v4().to_string(),
        name: req.name,
        generic_name: req.generic_name,
        ndc_code: req.ndc_code,
        manufacturer_id: req.manufacturer_id,
        therapeutic_class: req.therapeutic_class,
        route_of_administration: req.route_of_administration,
        dosage_form: req.dosage_form,
        strength: req.strength,
        description: req.description,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    Ok((StatusCode::CREATED, Json(drug)))
}

/// Update drug information.
#[allow(dead_code)]
async fn update_drug(
    Path(id): Path<String>,
    Json(_req): Json<UpdateDrug>,
) -> Result<Json<Drug>, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Drug {} not found", id) })),
    ))
}

/// Delete a drug.
#[allow(dead_code)]
async fn delete_drug(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    Err((
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("Drug {} not found", id) })),
    ))
}
