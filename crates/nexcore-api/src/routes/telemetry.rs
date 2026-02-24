// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Telemetry Module — Event ingestion and analytics

use crate::ApiState;
use crate::persistence::TelemetryEventRecord;
use crate::routes::common::ApiError;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Request to ingest a telemetry event
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct IngestEventRequest {
    pub event_type: String,
    pub user_id: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Telemetry event response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TelemetryEvent {
    pub id: String,
    pub event_type: String,
    pub user_id: String,
    pub metadata: serde_json::Value,
    pub timestamp: String,
}

impl From<TelemetryEventRecord> for TelemetryEvent {
    fn from(r: TelemetryEventRecord) -> Self {
        Self {
            id: r.id,
            event_type: r.event_type,
            user_id: r.user_id,
            metadata: r.metadata,
            timestamp: r.timestamp,
        }
    }
}

/// Telemetry summary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TelemetrySummary {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub active_users: usize,
    pub popular_pages: Vec<PageCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageCount {
    pub page: String,
    pub count: usize,
}

/// Ingest a telemetry event
#[utoipa::path(
    post,
    path = "/api/v1/telemetry/events",
    request_body = IngestEventRequest,
    responses(
        (status = 201, description = "Event recorded", body = TelemetryEvent),
    ),
    tag = "telemetry"
)]
pub async fn ingest_event(
    State(state): State<ApiState>,
    Json(req): Json<IngestEventRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let record = TelemetryEventRecord {
        id: nexcore_id::NexId::v4().to_string(),
        event_type: req.event_type,
        user_id: req.user_id,
        metadata: req.metadata,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    state
        .persistence
        .save_telemetry_event(&record)
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let event = TelemetryEvent::from(record);
    Ok((StatusCode::CREATED, Json(event)))
}

/// List telemetry events
#[utoipa::path(
    get,
    path = "/api/v1/telemetry/events",
    responses(
        (status = 200, description = "List of telemetry events", body = Vec<TelemetryEvent>),
    ),
    tag = "telemetry"
)]
pub async fn list_events(
    State(state): State<ApiState>,
) -> Result<Json<Vec<TelemetryEvent>>, ApiError> {
    let records = state
        .persistence
        .list_telemetry_events()
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let events: Vec<TelemetryEvent> = records.into_iter().map(TelemetryEvent::from).collect();
    Ok(Json(events))
}

/// Get telemetry summary
#[utoipa::path(
    get,
    path = "/api/v1/telemetry/summary",
    responses(
        (status = 200, description = "Aggregated telemetry stats", body = TelemetrySummary),
    ),
    tag = "telemetry"
)]
pub async fn get_summary(
    State(state): State<ApiState>,
) -> Result<Json<TelemetrySummary>, ApiError> {
    let events = state
        .persistence
        .list_telemetry_events()
        .await
        .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let total_events = events.len();

    let mut events_by_type: HashMap<String, usize> = HashMap::new();
    let mut unique_users = std::collections::HashSet::new();
    let mut page_counts: HashMap<String, usize> = HashMap::new();

    for event in &events {
        *events_by_type.entry(event.event_type.clone()).or_default() += 1;
        unique_users.insert(event.user_id.clone());

        if event.event_type == "page_view" {
            if let Some(page) = event.metadata.get("page").and_then(|p| p.as_str()) {
                *page_counts.entry(page.to_string()).or_default() += 1;
            }
        }
    }

    let mut popular_pages: Vec<PageCount> = page_counts
        .into_iter()
        .map(|(page, count)| PageCount { page, count })
        .collect();
    popular_pages.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(Json(TelemetrySummary {
        total_events,
        events_by_type,
        active_users: unique_users.len(),
        popular_pages,
    }))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/events", post(ingest_event).get(list_events))
        .route("/summary", get(get_summary))
}
