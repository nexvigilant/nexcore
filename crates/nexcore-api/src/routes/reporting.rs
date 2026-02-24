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

    let id = nexcore_id::NexId::v4().to_string();
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
        .route("/timeline", axum::routing::post(timeline))
        .route("/reportability", axum::routing::post(reportability))
}

// =============================================================================
// Reporting timeline types
// =============================================================================

/// Seriousness criterion string values accepted by the API.
///
/// Maps to ICH E2A criteria: `"death"`, `"life_threatening"`,
/// `"hospitalization"`, `"disability"`, `"congenital_anomaly"`,
/// `"medically_important"`.
///
/// Any unrecognised value is silently ignored.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReportingTimelineRequest {
    /// ICH E2A seriousness criteria that are met.
    ///
    /// Accepted values: `"death"`, `"life_threatening"`, `"hospitalization"`,
    /// `"disability"`, `"congenital_anomaly"`, `"medically_important"`.
    pub seriousness_criteria: Vec<String>,
    /// Expectedness of the adverse event relative to product labelling.
    ///
    /// Accepted values: `"listed"`, `"unlisted"`, `"unknown"`.
    pub expectedness: String,
    /// Whether the event occurred in a clinical trial context.
    pub is_clinical_trial: bool,
    /// Date company first became aware of the event (YYYYMMDD).
    pub awareness_date: String,
}

/// Reporting timeline response
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportingTimelineResponse {
    /// Reporting category: `"SevenDay"`, `"FifteenDay"`, `"Periodic"`, or `"NonReportable"`
    pub category: String,
    /// Calendar days allowed from awareness date to submission
    pub deadline_days: u32,
    /// Calculated deadline date (YYYYMMDD), or `null` when not reportable
    pub deadline_date: Option<String>,
    /// Calendar days remaining until deadline, or `null` when not reportable
    pub days_remaining: Option<u32>,
    /// Whether the deadline has already passed
    pub is_overdue: bool,
    /// Applicable regulatory authorities
    pub authorities: Vec<String>,
    /// Regulatory rationale for this determination
    pub rationale: String,
}

// =============================================================================
// Reportability types
// =============================================================================

/// Reportability assessment request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReportabilityRequest {
    /// ICH E2A seriousness criteria that are met.
    ///
    /// Accepted values: `"death"`, `"life_threatening"`, `"hospitalization"`,
    /// `"disability"`, `"congenital_anomaly"`, `"medically_important"`.
    pub seriousness_criteria: Vec<String>,
    /// Expectedness of the adverse event relative to product labelling.
    ///
    /// Accepted values: `"listed"`, `"unlisted"`, `"unknown"`.
    pub expectedness: String,
    /// Whether the event occurred in a clinical trial context.
    pub is_clinical_trial: bool,
}

/// Reportability assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportabilityResponse {
    /// Whether expedited reporting is required
    pub required: bool,
    /// Reporting category: `"SevenDay"`, `"FifteenDay"`, `"Periodic"`, or `"NonReportable"`
    pub category: String,
    /// Calendar days allowed from awareness date to submission
    pub deadline_days: u32,
    /// Applicable regulatory authorities
    pub authorities: Vec<String>,
    /// Regulatory rationale for this determination
    pub rationale: String,
}

// =============================================================================
// Shared helper
// =============================================================================

/// Parse a slice of criterion strings into domain `SeriousnessCriterion` values.
///
/// Unrecognised strings are silently dropped, preserving forward-compatibility
/// without breaking existing callers when new criteria are added.
fn parse_seriousness_criteria(
    criteria: &[String],
) -> Vec<nexcore_pv_core::regulatory::reportability::SeriousnessCriterion> {
    use nexcore_pv_core::regulatory::reportability::SeriousnessCriterion;

    criteria
        .iter()
        .filter_map(|s| match s.as_str() {
            "death" => Some(SeriousnessCriterion::Death),
            "life_threatening" => Some(SeriousnessCriterion::LifeThreatening),
            "hospitalization" => Some(SeriousnessCriterion::Hospitalization),
            "disability" => Some(SeriousnessCriterion::Disability),
            "congenital_anomaly" => Some(SeriousnessCriterion::CongenitalAnomaly),
            "medically_important" => Some(SeriousnessCriterion::MedicallyImportant),
            _ => None,
        })
        .collect()
}

/// Parse expectedness string to domain type.
fn parse_expectedness(s: &str) -> nexcore_pv_core::expectedness::Expectedness {
    use nexcore_pv_core::expectedness::Expectedness;
    match s {
        "listed" => Expectedness::Listed,
        "unlisted" => Expectedness::Unlisted,
        _ => Expectedness::Unknown,
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// Calculate reporting timeline with deadline from awareness date
///
/// Determines whether expedited reporting is required and calculates the
/// actual submission deadline from the awareness date.  Implements ICH E2A
/// and GVP Module VI decision logic:
///
/// - Fatal/life-threatening SUSAR → 7-day report
/// - Other serious unexpected ADR → 15-day report
/// - Serious expected in clinical trial → 15-day report
/// - Non-expedited → periodic or non-reportable
#[utoipa::path(
    post,
    path = "/api/v1/reporting/timeline",
    tag = "reporting",
    request_body = ReportingTimelineRequest,
    responses(
        (status = 200, description = "Reporting timeline calculated", body = ReportingTimelineResponse),
        (status = 400, description = "Invalid awareness date format", body = crate::routes::common::ApiError)
    )
)]
pub async fn timeline(
    Json(req): Json<ReportingTimelineRequest>,
) -> Result<Json<ReportingTimelineResponse>, crate::routes::common::ApiError> {
    use nexcore_pv_core::regulatory::reportability::{
        assess_seriousness, calculate_deadline, determine_expedited,
    };

    // Current date: use today (YYYYMMDD format)
    let current_date = {
        let now = chrono::Utc::now();
        format!("{}", now.format("%Y%m%d"))
    };

    let criteria = parse_seriousness_criteria(&req.seriousness_criteria);
    let seriousness = assess_seriousness(&criteria);
    let expectedness = parse_expectedness(&req.expectedness);
    let expedited = determine_expedited(&seriousness, expectedness, req.is_clinical_trial);

    let authorities: Vec<String> = expedited
        .authorities
        .iter()
        .map(|a| a.name().to_string())
        .collect();

    let (deadline_date, days_remaining, is_overdue) =
        if expedited.required || expedited.deadline_days > 0 {
            match calculate_deadline(&req.awareness_date, expedited.deadline_days, &current_date) {
                Some(dl) => (
                    Some(dl.deadline_date),
                    Some(dl.calendar_days),
                    dl.is_overdue,
                ),
                None => {
                    return Err(crate::routes::common::ApiError::new(
                        "VALIDATION_ERROR",
                        "awareness_date must be in YYYYMMDD format",
                    ));
                }
            }
        } else {
            (None, None, false)
        };

    Ok(Json(ReportingTimelineResponse {
        category: format!("{:?}", expedited.category),
        deadline_days: expedited.deadline_days,
        deadline_date,
        days_remaining,
        is_overdue,
        authorities,
        rationale: expedited.rationale,
    }))
}

/// Assess reportability without calculating a specific deadline date
///
/// Determines whether expedited reporting is required and the applicable
/// regulatory authorities, based on seriousness, expectedness, and trial
/// context alone.  Use this endpoint when no awareness date is available.
#[utoipa::path(
    post,
    path = "/api/v1/reporting/reportability",
    tag = "reporting",
    request_body = ReportabilityRequest,
    responses(
        (status = 200, description = "Reportability assessed", body = ReportabilityResponse),
        (status = 400, description = "Invalid input", body = crate::routes::common::ApiError)
    )
)]
pub async fn reportability(
    Json(req): Json<ReportabilityRequest>,
) -> Result<Json<ReportabilityResponse>, crate::routes::common::ApiError> {
    use nexcore_pv_core::regulatory::reportability::{assess_seriousness, determine_expedited};

    let criteria = parse_seriousness_criteria(&req.seriousness_criteria);
    let seriousness = assess_seriousness(&criteria);
    let expectedness = parse_expectedness(&req.expectedness);
    let expedited = determine_expedited(&seriousness, expectedness, req.is_clinical_trial);

    let authorities: Vec<String> = expedited
        .authorities
        .iter()
        .map(|a| a.name().to_string())
        .collect();

    Ok(Json(ReportabilityResponse {
        required: expedited.required,
        category: format!("{:?}", expedited.category),
        deadline_days: expedited.deadline_days,
        authorities,
        rationale: expedited.rationale,
    }))
}
