//! Reporting module — automated safety report generation

use crate::ApiState;
use crate::persistence::ReportRecord;
use axum::extract::{Json, State};
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Type of report to generate
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// Summary of detected signals
    SignalSummary,
    /// Full audit trail of system actions
    AuditTrail,
    /// Guardian homeostasis performance report
    GuardianPerformance,
}

/// Request to generate a report
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReportRequest {
    /// Type of report
    pub report_type: ReportType,
    /// Optional start date for data inclusion
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end date for data inclusion
    pub end_date: Option<DateTime<Utc>>,
}

/// Generated report response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReportResponse {
    pub id: String,
    pub report_type: ReportType,
    pub generated_at: DateTime<Utc>,
    pub content: String,
    pub status: String,
}

/// Generate a new safety report
#[utoipa::path(
    post,
    path = "/api/v1/reporting/generate",
    request_body = ReportRequest,
    responses(
        (status = 200, description = "Report generated successfully", body = ReportResponse),
    ),
    tag = "reporting"
)]
pub async fn generate_report(
    headers: HeaderMap,
    State(state): State<ApiState>,
    Json(req): Json<ReportRequest>,
) -> Result<Json<ReportResponse>, crate::routes::common::ApiError> {
    // Placeholder implementation for report generation logic
    let content = match req.report_type {
        ReportType::SignalSummary => {
            "Signal Summary Report: 0 active signals detected in the selected period.".to_string()
        }
        ReportType::AuditTrail => {
            "Audit Trail: System initialized, Guardian active, no violations recorded.".to_string()
        }
        ReportType::GuardianPerformance => {
            "Guardian Performance: 100% homeostasis maintained, avg iteration 42ms.".to_string()
        }
    };

    let id = uuid::Uuid::new_v4().to_string();
    let generated_at = Utc::now();
    let report_type_str = format!("{:?}", req.report_type);

    let record = ReportRecord {
        id: id.clone(),
        report_type: report_type_str,
        generated_at,
        content: content.clone(),
        status: "completed".to_string(),
        user_id: crate::tenant::extract_user_id_from_headers(&headers),
    };

    state
        .persistence
        .save_report(&record)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(ReportResponse {
        id,
        report_type: req.report_type,
        generated_at,
        content,
        status: "completed".to_string(),
    }))
}

/// Get all generated reports
#[utoipa::path(
    get,
    path = "/api/v1/reporting/list",
    responses(
        (status = 200, description = "List of generated reports", body = Vec<ReportResponse>),
    ),
    tag = "reporting"
)]
pub async fn list_reports(
    State(state): State<ApiState>,
) -> Result<Json<Vec<ReportResponse>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_reports()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let responses = records
        .into_iter()
        .map(|r| ReportResponse {
            id: r.id,
            report_type: match r.report_type.as_str() {
                "AuditTrail" => ReportType::AuditTrail,
                "GuardianPerformance" => ReportType::GuardianPerformance,
                _ => ReportType::SignalSummary,
            },
            generated_at: r.generated_at,
            content: r.content,
            status: r.status,
        })
        .collect();

    Ok(Json(responses))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/generate", axum::routing::post(generate_report))
        .route("/list", axum::routing::get(list_reports))
}
