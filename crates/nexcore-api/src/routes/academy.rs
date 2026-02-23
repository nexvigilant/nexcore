// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Academy Module — Competency-Based Learning & Certification

use crate::ApiState;
use crate::persistence::{EnrollmentRecord, KsbDomainRecord};
use axum::extract::{Json, State};
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Course information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Course {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub tier: String,
    pub level: u8,
}

/// Enrollment record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Enrollment {
    pub id: String,
    pub user_id: String,
    pub course_id: String,
    pub progress: f64,
    pub enrolled_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Request to enroll in a course
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct EnrollRequest {
    pub course_id: String,
    pub user_id: String,
}

/// KSB Domain summary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KsbDomainSummary {
    pub code: String,
    pub name: String,
    pub ksb_count: u32,
    pub dominant_primitive: String,
    pub cognitive_primitive: String,
    pub transfer_confidence: f64,
    pub pvos_layer: Option<String>,
    pub example_ksbs: Vec<String>,
}

/// List all available courses
#[utoipa::path(
    get,
    path = "/api/v1/academy/courses",
    responses(
        (status = 200, description = "List of courses", body = Vec<Course>),
    ),
    tag = "academy"
)]
pub async fn list_courses() -> Json<Vec<Course>> {
    let courses = vec![
        Course {
            id: "d08-001".to_string(),
            code: "D08-SIG".to_string(),
            title: "Signal Detection Fundamentals".to_string(),
            description: "Introduction to disproportionality analysis and PRR/ROR metrics."
                .to_string(),
            tier: "T2-P".to_string(),
            level: 1,
        },
        Course {
            id: "d01-002".to_string(),
            code: "D01-TOV".to_string(),
            title: "Theory of Vigilance".to_string(),
            description: "Deep dive into the 8 harm types and safety axioms.".to_string(),
            tier: "T1".to_string(),
            level: 3,
        },
    ];

    Json(courses)
}

/// Get enrollment for current user
#[utoipa::path(
    get,
    path = "/api/v1/academy/enrollments",
    responses(
        (status = 200, description = "User enrollments", body = Vec<Enrollment>),
    ),
    tag = "academy"
)]
pub async fn list_enrollments(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Enrollment>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_enrollments()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let responses = records
        .into_iter()
        .map(|r| Enrollment {
            id: r.id,
            user_id: r.user_id,
            course_id: r.course_id,
            progress: r.progress,
            enrolled_at: r.enrolled_at,
            completed_at: r.completed_at,
        })
        .collect();

    Ok(Json(responses))
}

/// Enroll in a course
#[utoipa::path(
    post,
    path = "/api/v1/academy/enroll",
    request_body = EnrollRequest,
    responses(
        (status = 201, description = "Enrolled successfully", body = Enrollment),
    ),
    tag = "academy"
)]
pub async fn enroll(
    State(state): State<ApiState>,
    Json(req): Json<EnrollRequest>,
) -> Result<Json<Enrollment>, crate::routes::common::ApiError> {
    let enrollment = Enrollment {
        id: nexcore_id::NexId::v4().to_string(),
        user_id: req.user_id,
        course_id: req.course_id,
        progress: 0.0,
        enrolled_at: Utc::now(),
        completed_at: None,
    };

    let record = EnrollmentRecord {
        id: enrollment.id.clone(),
        user_id: enrollment.user_id.clone(),
        course_id: enrollment.course_id.clone(),
        progress: enrollment.progress,
        enrolled_at: enrollment.enrolled_at,
        completed_at: enrollment.completed_at,
    };

    state
        .persistence
        .save_enrollment(&record)
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    Ok(Json(enrollment))
}

/// List all KSB domains
#[utoipa::path(
    get,
    path = "/api/v1/academy/ksb/domains",
    responses(
        (status = 200, description = "List of KSB domains", body = Vec<KsbDomainSummary>),
    ),
    tag = "academy"
)]
pub async fn list_ksb_domains(
    State(state): State<ApiState>,
) -> Result<Json<Vec<KsbDomainSummary>>, crate::routes::common::ApiError> {
    let records = state
        .persistence
        .list_ksb_domains()
        .await
        .map_err(|e| crate::routes::common::ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let mut domains: Vec<KsbDomainSummary> = records
        .into_iter()
        .map(|r| KsbDomainSummary {
            code: r.code,
            name: r.name,
            ksb_count: r.ksb_count,
            dominant_primitive: r.dominant_primitive,
            cognitive_primitive: r.cognitive_primitive,
            transfer_confidence: r.transfer_confidence,
            pvos_layer: r.pvos_layer,
            example_ksbs: r.example_ksbs,
        })
        .collect();

    // Seed if empty
    if domains.is_empty() {
        domains = vec![
            KsbDomainSummary {
                code: "D01".to_string(),
                name: "Theory of Vigilance".to_string(),
                ksb_count: 84,
                dominant_primitive: "Recursion".to_string(),
                cognitive_primitive: "Mapping".to_string(),
                transfer_confidence: 0.95,
                pvos_layer: Some("AVC".to_string()),
                example_ksbs: vec!["T1-TOV-001: Irreducibility Axiom".to_string()],
            },
            KsbDomainSummary {
                code: "D08".to_string(),
                name: "Signal Detection".to_string(),
                ksb_count: 156,
                dominant_primitive: "Comparison".to_string(),
                cognitive_primitive: "Sequence".to_string(),
                transfer_confidence: 0.88,
                pvos_layer: Some("PVSD".to_string()),
                example_ksbs: vec!["S1-SIG-042: PRR Calculation".to_string()],
            },
        ];
    }

    Ok(Json(domains))
}

/// Learning pathway node
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PathwayNode {
    pub id: String,
    pub title: String,
    pub level: String,
}

/// Learning pathway
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LearningPathway {
    pub id: String,
    pub title: String,
    pub description: String,
    pub nodes: Vec<PathwayNode>,
}

/// List all learning pathways
#[utoipa::path(
    get,
    path = "/api/v1/academy/pathways",
    responses(
        (status = 200, description = "List of learning pathways", body = Vec<LearningPathway>),
    ),
    tag = "academy"
)]
pub async fn list_pathways() -> Json<Vec<LearningPathway>> {
    let pathways = vec![LearningPathway {
        id: "path-001".to_string(),
        title: "Signal Detection Core".to_string(),
        description: "Master the sequence of signal detection operations.".to_string(),
        nodes: vec![PathwayNode {
            id: "epa-8".to_string(),
            title: "Run PRR/ROR analysis".to_string(),
            level: "Advanced".to_string(),
        }],
    }];
    Json(pathways)
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/courses", get(list_courses))
        .route("/enrollments", get(list_enrollments))
        .route("/enroll", post(enroll))
        .route("/ksb/domains", get(list_ksb_domains))
        .route("/pathways", get(list_pathways))
}
