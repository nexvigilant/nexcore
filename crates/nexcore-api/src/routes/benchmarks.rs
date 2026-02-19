//! Benchmarks module — anonymized peer benchmarking
//!
//! PRPaaS: Cross-tenant anonymous comparison metrics.
//! Tenants see their percentile vs platform averages.
//! All data anonymized — no tenant identifiers exposed.

use crate::ApiState;
use axum::extract::{Json, Query};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Single benchmark data point
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkDataPoint {
    pub dimension: String,
    pub value: f64,
    pub percentile: f64,
    pub platform_median: f64,
    pub platform_p25: f64,
    pub platform_p75: f64,
    pub sample_size: u32,
    pub period: String,
}

/// Tenant benchmark report
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkReport {
    pub tenant_id: String,
    pub period: String,
    pub data_points: Vec<BenchmarkDataPoint>,
    pub overall_score: f64,
    pub overall_percentile: f64,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Platform-wide benchmark aggregates
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BenchmarkAggregates {
    pub period: String,
    pub total_tenants: u32,
    pub dimensions: Vec<DimensionAggregate>,
}

/// Aggregate stats for a single benchmark dimension
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DimensionAggregate {
    pub dimension: String,
    pub median: f64,
    pub p25: f64,
    pub p75: f64,
    pub sample_size: u32,
}

/// Query parameters for benchmark requests
#[derive(Debug, Deserialize, IntoParams)]
pub struct BenchmarkParams {
    /// Tenant ID
    pub tenant: Option<String>,
    /// Period (e.g. "2026-Q1")
    pub period: Option<String>,
}

/// Get benchmark report for a tenant
#[utoipa::path(
    get,
    path = "/api/v1/benchmarks",
    params(BenchmarkParams),
    responses(
        (status = 200, description = "Tenant benchmark report", body = BenchmarkReport),
    ),
    tag = "benchmarks"
)]
pub async fn get_benchmarks(
    Query(params): Query<BenchmarkParams>,
) -> Result<Json<BenchmarkReport>, crate::routes::common::ApiError> {
    let tenant_id = params.tenant.unwrap_or_else(|| "default".to_string());
    let period = params.period.unwrap_or_else(|| "2026-Q1".to_string());

    // Phase 1: Return synthetic benchmark data
    // Phase 2+: Compute from aggregated tenant metrics in BigQuery
    let report = BenchmarkReport {
        tenant_id,
        period: period.clone(),
        data_points: vec![
            BenchmarkDataPoint {
                dimension: "signal_detection_rate".to_string(),
                value: 0.85,
                percentile: 72.0,
                platform_median: 0.78,
                platform_p25: 0.65,
                platform_p75: 0.89,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "case_processing_time".to_string(),
                value: 4.2, // days
                percentile: 65.0,
                platform_median: 5.1,
                platform_p25: 3.8,
                platform_p75: 7.2,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "report_quality_score".to_string(),
                value: 0.91,
                percentile: 80.0,
                platform_median: 0.84,
                platform_p25: 0.76,
                platform_p75: 0.92,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "regulatory_compliance_rate".to_string(),
                value: 0.97,
                percentile: 88.0,
                platform_median: 0.93,
                platform_p25: 0.88,
                platform_p75: 0.97,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "team_competency_level".to_string(),
                value: 3.8,
                percentile: 70.0,
                platform_median: 3.5,
                platform_p25: 2.8,
                platform_p75: 4.1,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "knowledge_coverage".to_string(),
                value: 0.72,
                percentile: 62.0,
                platform_median: 0.68,
                platform_p25: 0.55,
                platform_p75: 0.81,
                sample_size: 47,
                period: period.clone(),
            },
            BenchmarkDataPoint {
                dimension: "community_engagement".to_string(),
                value: 0.45,
                percentile: 55.0,
                platform_median: 0.42,
                platform_p25: 0.25,
                platform_p75: 0.60,
                sample_size: 47,
                period: period.clone(),
            },
        ],
        overall_score: 78.5,
        overall_percentile: 72.0,
        insights: vec![
            "Your signal detection rate is above the platform median — strong performance.".to_string(),
            "Case processing time is faster than 65% of tenants.".to_string(),
            "Community engagement has room for growth — consider participating in more forum discussions.".to_string(),
        ],
        recommendations: vec![
            "Explore the Signal Detection pathway in Academy to further improve detection capabilities.".to_string(),
            "Join the Signal Detection circle to connect with peers.".to_string(),
            "Consider listing your PV expertise on the Expert Marketplace.".to_string(),
        ],
    };

    Ok(Json(report))
}

/// Get platform-wide benchmark aggregates
#[utoipa::path(
    get,
    path = "/api/v1/benchmarks/platform",
    params(BenchmarkParams),
    responses(
        (status = 200, description = "Platform benchmark aggregates", body = BenchmarkAggregates),
    ),
    tag = "benchmarks"
)]
pub async fn get_platform_aggregates(
    Query(params): Query<BenchmarkParams>,
) -> Result<Json<BenchmarkAggregates>, crate::routes::common::ApiError> {
    let period = params.period.unwrap_or_else(|| "2026-Q1".to_string());

    Ok(Json(BenchmarkAggregates {
        period,
        total_tenants: 47,
        dimensions: vec![
            DimensionAggregate {
                dimension: "signal_detection_rate".to_string(),
                median: 0.78,
                p25: 0.65,
                p75: 0.89,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "case_processing_time".to_string(),
                median: 5.1,
                p25: 3.8,
                p75: 7.2,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "report_quality_score".to_string(),
                median: 0.84,
                p25: 0.76,
                p75: 0.92,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "regulatory_compliance_rate".to_string(),
                median: 0.93,
                p25: 0.88,
                p75: 0.97,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "team_competency_level".to_string(),
                median: 3.5,
                p25: 2.8,
                p75: 4.1,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "knowledge_coverage".to_string(),
                median: 0.68,
                p25: 0.55,
                p75: 0.81,
                sample_size: 47,
            },
            DimensionAggregate {
                dimension: "community_engagement".to_string(),
                median: 0.42,
                p25: 0.25,
                p75: 0.60,
                sample_size: 47,
            },
        ],
    }))
}

pub fn router() -> axum::Router<ApiState> {
    axum::Router::new()
        .route("/", get(get_benchmarks))
        .route("/platform", get(get_platform_aggregates))
}
