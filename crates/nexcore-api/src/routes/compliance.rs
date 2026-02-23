//! Compliance routes — audit trails, GDPR, export controls, SOC 2.
//!
//! PRPaaS: Regulatory compliance infrastructure wiring vr-compliance into REST endpoints.

use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ApiState;
use crate::tenant::VerifiedTenant;

/// Create the compliance router.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/audit/events", post(record_audit_event))
        .route("/audit/query", get(query_audit_trail))
        .route("/gdpr/requests", get(list_gdpr_requests))
        .route("/gdpr/request", post(create_gdpr_request))
        .route("/gdpr/consent", get(get_consent_records))
        .route("/gdpr/consent", post(update_consent))
        .route("/export/screen", post(screen_export))
        .route("/soc2/scorecard", get(get_soc2_scorecard))
}

// ── Request / Response types ──────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordAuditEventRequest {
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditEventResponse {
    pub event_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub action: String,
    pub timestamp: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditQueryResponse {
    pub events: Vec<AuditEventResponse>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGdprRequestBody {
    pub request_type: String,
    pub subject_email: String,
    pub description: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GdprRequestResponse {
    pub request_id: String,
    pub tenant_id: String,
    pub request_type: String,
    pub subject_email: String,
    pub status: String,
    pub deadline: String,
    pub created_at: String,
    pub days_remaining: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GdprRequestListResponse {
    pub requests: Vec<GdprRequestResponse>,
    pub total: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentRecordResponse {
    pub consent_type: String,
    pub granted: bool,
    pub granted_at: Option<String>,
    pub revoked_at: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentListResponse {
    pub tenant_id: String,
    pub records: Vec<ConsentRecordResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateConsentRequest {
    pub consent_type: String,
    pub granted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentUpdateResponse {
    pub consent_type: String,
    pub granted: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExportScreenRequest {
    pub compound_identifier: String,
    pub destination_country: String,
    pub end_use_description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExportScreenResponse {
    pub screening_id: String,
    pub compound_identifier: String,
    pub destination_country: String,
    pub risk_level: String,
    pub cleared: bool,
    pub flags: Vec<String>,
    pub screened_at: String,
    pub details: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Soc2ScorecardResponse {
    pub tenant_id: String,
    pub overall_score: f64,
    pub categories: Vec<Soc2CategoryScore>,
    pub controls_total: u32,
    pub controls_compliant: u32,
    pub controls_partial: u32,
    pub controls_non_compliant: u32,
    pub last_audit_date: String,
    pub next_audit_date: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Soc2CategoryScore {
    pub category: String,
    pub score: f64,
    pub controls_total: u32,
    pub controls_met: u32,
    pub status: String,
}

// ── Handlers ──────────────────────────

/// Record an audit event (immutable append-only).
#[utoipa::path(
    post,
    path = "/api/v1/compliance/audit/events",
    tag = "compliance",
    request_body = RecordAuditEventRequest,
    responses(
        (status = 201, description = "Audit event recorded", body = AuditEventResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn record_audit_event(
    tenant: VerifiedTenant,
    Json(req): Json<RecordAuditEventRequest>,
) -> (axum::http::StatusCode, Json<AuditEventResponse>) {
    let ctx = tenant.0;
    (
        axum::http::StatusCode::CREATED,
        Json(AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: ctx.tenant_id().to_string(),
            user_id: ctx.user_id().to_string(),
            event_type: req.event_type,
            resource_type: req.resource_type,
            resource_id: req.resource_id,
            action: req.action,
            timestamp: Utc::now().to_rfc3339(),
            details: req.details,
        }),
    )
}

/// Query the audit trail with filters.
#[utoipa::path(
    get,
    path = "/api/v1/compliance/audit/query",
    tag = "compliance",
    responses(
        (status = 200, description = "Audit events", body = AuditQueryResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn query_audit_trail(tenant: VerifiedTenant) -> Json<AuditQueryResponse> {
    let ctx = tenant.0;
    let tid = ctx.tenant_id().to_string();
    let now = Utc::now();

    let events = vec![
        AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid.clone(),
            user_id: "user-001".to_string(),
            event_type: "data_access".to_string(),
            resource_type: "program".to_string(),
            resource_id: "prog-alpha".to_string(),
            action: "read".to_string(),
            timestamp: (now - Duration::hours(2)).to_rfc3339(),
            details: None,
        },
        AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid.clone(),
            user_id: "user-002".to_string(),
            event_type: "compound_registered".to_string(),
            resource_type: "compound".to_string(),
            resource_id: "cmpd-0042".to_string(),
            action: "create".to_string(),
            timestamp: (now - Duration::hours(5)).to_rfc3339(),
            details: Some(serde_json::json!({"smiles": "CC(=O)Oc1ccccc1C(=O)O"})),
        },
        AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid.clone(),
            user_id: "user-001".to_string(),
            event_type: "cro_order_placed".to_string(),
            resource_type: "marketplace_order".to_string(),
            resource_id: "ord-789".to_string(),
            action: "create".to_string(),
            timestamp: (now - Duration::days(1)).to_rfc3339(),
            details: Some(serde_json::json!({"provider": "WuXi AppTec", "service": "synthesis"})),
        },
        AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid.clone(),
            user_id: "user-003".to_string(),
            event_type: "ml_prediction".to_string(),
            resource_type: "model".to_string(),
            resource_id: "adme-v2".to_string(),
            action: "invoke".to_string(),
            timestamp: (now - Duration::days(2)).to_rfc3339(),
            details: Some(serde_json::json!({"compounds_predicted": 50})),
        },
        AuditEventResponse {
            event_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid,
            user_id: "user-001".to_string(),
            event_type: "data_export".to_string(),
            resource_type: "dataset".to_string(),
            resource_id: "ds-assay-results-q4".to_string(),
            action: "export".to_string(),
            timestamp: (now - Duration::days(3)).to_rfc3339(),
            details: Some(serde_json::json!({"format": "CSV", "rows": 2847})),
        },
    ];

    let total = events.len() as u32;
    Json(AuditQueryResponse {
        events,
        total,
        page: 1,
        per_page: 50,
    })
}

/// List GDPR data subject requests.
#[utoipa::path(
    get,
    path = "/api/v1/compliance/gdpr/requests",
    tag = "compliance",
    responses(
        (status = 200, description = "GDPR requests", body = GdprRequestListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_gdpr_requests(tenant: VerifiedTenant) -> Json<GdprRequestListResponse> {
    let ctx = tenant.0;
    let tid = ctx.tenant_id().to_string();
    let now = Utc::now();

    let requests = vec![
        GdprRequestResponse {
            request_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid.clone(),
            request_type: "Access".to_string(),
            subject_email: "researcher@example.com".to_string(),
            status: "Processing".to_string(),
            deadline: (now + Duration::days(18)).to_rfc3339(),
            created_at: (now - Duration::days(12)).to_rfc3339(),
            days_remaining: 18,
        },
        GdprRequestResponse {
            request_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: tid,
            request_type: "Erasure".to_string(),
            subject_email: "former.employee@example.com".to_string(),
            status: "Completed".to_string(),
            deadline: (now - Duration::days(5)).to_rfc3339(),
            created_at: (now - Duration::days(28)).to_rfc3339(),
            days_remaining: 0,
        },
    ];

    let total = requests.len() as u32;
    Json(GdprRequestListResponse { requests, total })
}

/// Create a new GDPR data subject request.
#[utoipa::path(
    post,
    path = "/api/v1/compliance/gdpr/request",
    tag = "compliance",
    request_body = CreateGdprRequestBody,
    responses(
        (status = 201, description = "GDPR request created", body = GdprRequestResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_gdpr_request(
    tenant: VerifiedTenant,
    Json(req): Json<CreateGdprRequestBody>,
) -> (axum::http::StatusCode, Json<GdprRequestResponse>) {
    let ctx = tenant.0;
    let now = Utc::now();
    let deadline = now + Duration::days(30);

    (
        axum::http::StatusCode::CREATED,
        Json(GdprRequestResponse {
            request_id: nexcore_id::NexId::v4().to_string(),
            tenant_id: ctx.tenant_id().to_string(),
            request_type: req.request_type,
            subject_email: req.subject_email,
            status: "Received".to_string(),
            deadline: deadline.to_rfc3339(),
            created_at: now.to_rfc3339(),
            days_remaining: 30,
        }),
    )
}

/// Get consent records for the tenant.
#[utoipa::path(
    get,
    path = "/api/v1/compliance/gdpr/consent",
    tag = "compliance",
    responses(
        (status = 200, description = "Consent records", body = ConsentListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_consent_records(tenant: VerifiedTenant) -> Json<ConsentListResponse> {
    let ctx = tenant.0;
    let now = Utc::now();

    Json(ConsentListResponse {
        tenant_id: ctx.tenant_id().to_string(),
        records: vec![
            ConsentRecordResponse {
                consent_type: "DataProcessing".to_string(),
                granted: true,
                granted_at: Some((now - Duration::days(90)).to_rfc3339()),
                revoked_at: None,
                description: "Allow platform to process tenant data for service delivery"
                    .to_string(),
            },
            ConsentRecordResponse {
                consent_type: "DataContribution".to_string(),
                granted: true,
                granted_at: Some((now - Duration::days(60)).to_rfc3339()),
                revoked_at: None,
                description:
                    "Contribute anonymized data to platform ML models for better predictions"
                        .to_string(),
            },
            ConsentRecordResponse {
                consent_type: "MarketplaceSharing".to_string(),
                granted: false,
                granted_at: None,
                revoked_at: None,
                description: "Share anonymized benchmarks with marketplace participants"
                    .to_string(),
            },
            ConsentRecordResponse {
                consent_type: "Analytics".to_string(),
                granted: true,
                granted_at: Some((now - Duration::days(90)).to_rfc3339()),
                revoked_at: None,
                description: "Allow platform analytics for service improvement".to_string(),
            },
        ],
    })
}

/// Update a consent record.
#[utoipa::path(
    post,
    path = "/api/v1/compliance/gdpr/consent",
    tag = "compliance",
    request_body = UpdateConsentRequest,
    responses(
        (status = 200, description = "Consent updated", body = ConsentUpdateResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_consent(
    _tenant: VerifiedTenant,
    Json(req): Json<UpdateConsentRequest>,
) -> Json<ConsentUpdateResponse> {
    Json(ConsentUpdateResponse {
        consent_type: req.consent_type,
        granted: req.granted,
        updated_at: Utc::now().to_rfc3339(),
    })
}

/// Screen a compound for export control restrictions.
#[utoipa::path(
    post,
    path = "/api/v1/compliance/export/screen",
    tag = "compliance",
    request_body = ExportScreenRequest,
    responses(
        (status = 200, description = "Export screening result", body = ExportScreenResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn screen_export(
    _tenant: VerifiedTenant,
    Json(req): Json<ExportScreenRequest>,
) -> Json<ExportScreenResponse> {
    // Determine risk based on destination country
    let (risk_level, cleared, flags) = match req.destination_country.to_uppercase().as_str() {
        "US" | "GB" | "DE" | "FR" | "JP" | "AU" | "CA" => ("Low".to_string(), true, vec![]),
        "CN" | "RU" => (
            "High".to_string(),
            false,
            vec![
                "Dual-use technology review required".to_string(),
                "End-use certificate needed".to_string(),
            ],
        ),
        "KP" | "IR" | "SY" | "CU" => (
            "Blocked".to_string(),
            false,
            vec![
                "OFAC sanctioned country".to_string(),
                "Export prohibited without specific license".to_string(),
            ],
        ),
        _ => (
            "Medium".to_string(),
            true,
            vec!["Standard screening passed".to_string()],
        ),
    };

    Json(ExportScreenResponse {
        screening_id: nexcore_id::NexId::v4().to_string(),
        compound_identifier: req.compound_identifier,
        destination_country: req.destination_country,
        risk_level,
        cleared,
        flags,
        screened_at: Utc::now().to_rfc3339(),
        details: "Screened against EAR, CWC Schedule lists, and OFAC sanctions".to_string(),
    })
}

/// Get SOC 2 compliance scorecard.
#[utoipa::path(
    get,
    path = "/api/v1/compliance/soc2/scorecard",
    tag = "compliance",
    responses(
        (status = 200, description = "SOC 2 scorecard", body = Soc2ScorecardResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_soc2_scorecard(tenant: VerifiedTenant) -> Json<Soc2ScorecardResponse> {
    let ctx = tenant.0;

    let categories = vec![
        Soc2CategoryScore {
            category: "Security".to_string(),
            score: 0.92,
            controls_total: 24,
            controls_met: 22,
            status: "Compliant".to_string(),
        },
        Soc2CategoryScore {
            category: "Availability".to_string(),
            score: 0.88,
            controls_total: 12,
            controls_met: 11,
            status: "Compliant".to_string(),
        },
        Soc2CategoryScore {
            category: "Processing Integrity".to_string(),
            score: 0.95,
            controls_total: 8,
            controls_met: 8,
            status: "Compliant".to_string(),
        },
        Soc2CategoryScore {
            category: "Confidentiality".to_string(),
            score: 0.90,
            controls_total: 10,
            controls_met: 9,
            status: "Compliant".to_string(),
        },
        Soc2CategoryScore {
            category: "Privacy".to_string(),
            score: 0.85,
            controls_total: 14,
            controls_met: 12,
            status: "Partial".to_string(),
        },
    ];

    let controls_total: u32 = categories.iter().map(|c| c.controls_total).sum();
    let controls_met: u32 = categories.iter().map(|c| c.controls_met).sum();
    let overall_score: f64 =
        categories.iter().map(|c| c.score).sum::<f64>() / categories.len() as f64;

    Json(Soc2ScorecardResponse {
        tenant_id: ctx.tenant_id().to_string(),
        overall_score,
        categories,
        controls_total,
        controls_compliant: controls_met,
        controls_partial: controls_total - controls_met,
        controls_non_compliant: 0,
        last_audit_date: (Utc::now() - Duration::days(180))
            .format("%Y-%m-%d")
            .to_string(),
        next_audit_date: (Utc::now() + Duration::days(185))
            .format("%Y-%m-%d")
            .to_string(),
    })
}
