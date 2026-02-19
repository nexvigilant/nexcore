//! Marketplace module — expert network, engagement facilitation
//!
//! PRPaaS: Expert marketplace connects community members as consultants.
//! Commission: 15% of engagement fees.

use crate::ApiState;
use axum::extract::{Json, Query, State};
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Expert profile in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Expert {
    pub id: String,
    pub display_name: String,
    pub title: String,
    pub expertise_categories: Vec<String>,
    pub top_skills: Vec<String>,
    pub years_experience: u32,
    pub availability: String,
    pub rating: f64,
    pub review_count: u32,
    pub verified: bool,
    pub match_score: f64,
    pub match_reasons: Vec<String>,
}

/// Expert search response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExpertSearchResponse {
    pub experts: Vec<Expert>,
    pub total: u32,
    pub query: String,
}

/// Expert search query parameters
#[derive(Debug, Deserialize, IntoParams)]
pub struct ExpertSearchParams {
    /// Search query
    pub q: Option<String>,
    /// Filter by expertise categories (comma-separated)
    pub categories: Option<String>,
    /// Tenant ID for recommendations
    pub tenant: Option<String>,
}

/// Engagement request body
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateEngagementRequest {
    pub expert_id: String,
    pub engagement_type: String,
    pub title: String,
    pub description: String,
    pub estimated_hours: Option<f64>,
    pub budget_cents: Option<u64>,
}

/// Engagement response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EngagementResponse {
    pub id: String,
    pub expert_id: String,
    pub status: String,
    pub platform_commission_percent: f64,
    pub created_at: DateTime<Utc>,
}

/// Search experts in the marketplace
#[utoipa::path(
    get,
    path = "/api/v1/marketplace/experts",
    params(ExpertSearchParams),
    responses(
        (status = 200, description = "Expert search results", body = ExpertSearchResponse),
    ),
    tag = "marketplace"
)]
pub async fn search_experts(
    Query(params): Query<ExpertSearchParams>,
) -> Result<Json<ExpertSearchResponse>, crate::routes::common::ApiError> {
    let query = params.q.unwrap_or_default();
    let categories: Vec<String> = params
        .categories
        .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Phase 1: Return seed experts matching query
    let mut experts = seed_experts();

    // Filter by query
    if !query.is_empty() {
        let q = query.to_lowercase();
        experts.retain(|e| {
            e.display_name.to_lowercase().contains(&q)
                || e.title.to_lowercase().contains(&q)
                || e.expertise_categories
                    .iter()
                    .any(|c| c.to_lowercase().contains(&q))
                || e.top_skills.iter().any(|s| s.to_lowercase().contains(&q))
        });
    }

    // Filter by categories
    if !categories.is_empty() {
        experts.retain(|e| {
            e.expertise_categories
                .iter()
                .any(|c| categories.iter().any(|cat| c.eq_ignore_ascii_case(cat)))
        });
    }

    let total = experts.len() as u32;
    Ok(Json(ExpertSearchResponse {
        experts,
        total,
        query,
    }))
}

/// Get recommended experts for a tenant
#[utoipa::path(
    get,
    path = "/api/v1/marketplace/experts/recommend",
    params(ExpertSearchParams),
    responses(
        (status = 200, description = "Recommended experts", body = ExpertSearchResponse),
    ),
    tag = "marketplace"
)]
pub async fn recommend_experts(
    Query(params): Query<ExpertSearchParams>,
) -> Result<Json<ExpertSearchResponse>, crate::routes::common::ApiError> {
    let tenant = params.tenant.unwrap_or_else(|| "default".to_string());

    // Phase 1: Return top-rated seed experts as recommendations
    let mut experts = seed_experts();
    experts.sort_by(|a, b| {
        b.rating
            .partial_cmp(&a.rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    experts.truncate(5);

    // Add match reasons for recommendations
    for expert in &mut experts {
        expert.match_reasons = vec![
            "Top-rated in your therapeutic area".to_string(),
            "Available for consulting".to_string(),
        ];
        expert.match_score = expert.rating / 5.0;
    }

    let total = experts.len() as u32;
    Ok(Json(ExpertSearchResponse {
        experts,
        total,
        query: format!("recommendations for tenant {tenant}"),
    }))
}

/// Create an engagement with an expert
#[utoipa::path(
    post,
    path = "/api/v1/marketplace/engagements",
    request_body = CreateEngagementRequest,
    responses(
        (status = 201, description = "Engagement created", body = EngagementResponse),
    ),
    tag = "marketplace"
)]
pub async fn create_engagement(
    Json(req): Json<CreateEngagementRequest>,
) -> Result<Json<EngagementResponse>, crate::routes::common::ApiError> {
    Ok(Json(EngagementResponse {
        id: uuid::Uuid::new_v4().to_string(),
        expert_id: req.expert_id,
        status: "requested".to_string(),
        platform_commission_percent: 15.0,
        created_at: Utc::now(),
    }))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/experts", get(search_experts))
        .route("/experts/recommend", get(recommend_experts))
        .route("/engagements", axum::routing::post(create_engagement))
}

// ── Seed data ──────────────────────

fn seed_experts() -> Vec<Expert> {
    vec![
        Expert {
            id: "exp-001".to_string(),
            display_name: "Dr. Maria Santos".to_string(),
            title: "Senior PV Scientist & Signal Detection Lead".to_string(),
            expertise_categories: vec![
                "pharmacovigilance".to_string(),
                "signal_detection".to_string(),
            ],
            top_skills: vec![
                "PRR/ROR analysis".to_string(),
                "FAERS mining".to_string(),
                "PSUR authoring".to_string(),
            ],
            years_experience: 15,
            availability: "available".to_string(),
            rating: 4.9,
            review_count: 23,
            verified: true,
            match_score: 0.0,
            match_reasons: vec![],
        },
        Expert {
            id: "exp-002".to_string(),
            display_name: "Dr. James Okafor".to_string(),
            title: "Regulatory Affairs Director".to_string(),
            expertise_categories: vec!["regulatory_affairs".to_string(), "drug_safety".to_string()],
            top_skills: vec![
                "FDA submissions".to_string(),
                "ICH compliance".to_string(),
                "REMS design".to_string(),
            ],
            years_experience: 20,
            availability: "limited".to_string(),
            rating: 4.8,
            review_count: 31,
            verified: true,
            match_score: 0.0,
            match_reasons: vec![],
        },
        Expert {
            id: "exp-003".to_string(),
            display_name: "Dr. Lin Wei".to_string(),
            title: "Medicinal Chemistry Consultant".to_string(),
            expertise_categories: vec!["medicinal_chemistry".to_string(), "toxicology".to_string()],
            top_skills: vec![
                "SAR analysis".to_string(),
                "ADME optimization".to_string(),
                "Lead optimization".to_string(),
            ],
            years_experience: 12,
            availability: "available".to_string(),
            rating: 4.7,
            review_count: 18,
            verified: true,
            match_score: 0.0,
            match_reasons: vec![],
        },
        Expert {
            id: "exp-004".to_string(),
            display_name: "Dr. Priya Sharma".to_string(),
            title: "Biostatistician & Data Scientist".to_string(),
            expertise_categories: vec!["biostatistics".to_string(), "signal_detection".to_string()],
            top_skills: vec![
                "Bayesian analysis".to_string(),
                "EBGM computation".to_string(),
                "Clinical trial design".to_string(),
            ],
            years_experience: 10,
            availability: "available".to_string(),
            rating: 4.6,
            review_count: 14,
            verified: true,
            match_score: 0.0,
            match_reasons: vec![],
        },
        Expert {
            id: "exp-005".to_string(),
            display_name: "Michael Thompson, JD".to_string(),
            title: "Pharma IP & Patent Strategy".to_string(),
            expertise_categories: vec!["patent_strategy".to_string(), "bd_licensing".to_string()],
            top_skills: vec![
                "Freedom-to-operate analysis".to_string(),
                "Patent landscaping".to_string(),
                "Deal structuring".to_string(),
            ],
            years_experience: 18,
            availability: "limited".to_string(),
            rating: 4.5,
            review_count: 9,
            verified: true,
            match_score: 0.0,
            match_reasons: vec![],
        },
    ]
}
