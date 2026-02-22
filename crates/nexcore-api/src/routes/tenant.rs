//! Tenant management API routes.
//!
//! CRUD operations for tenant lifecycle, team management, and tier inspection.
//! All mutating operations require Admin or Owner role.

use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vr_core::tenant::SubscriptionTier;

use crate::ApiState;
use crate::tenant::VerifiedTenant;
const BYTES_PER_GB: u64 = 1_073_741_824;

/// Create the tenant management router.
pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/me", get(get_current_tenant))
        .route("/limits", get(get_tenant_limits))
        .route("/tiers", get(list_tiers))
        .route("/provision", post(provision_tenant))
}

// ── Response types ──────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantInfoResponse {
    pub tenant_id: String,
    pub org_name: String,
    pub tier: String,
    pub data_classification: String,
    pub verified_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TenantLimitsResponse {
    pub max_concurrent_programs: u32,
    pub storage_quota_gb: u64,
    pub api_rate_limit_rpm: u32,
    pub max_team_members: u32,
    pub ml_compute_enabled: bool,
    pub marketplace_publish: bool,
    pub custom_compliance: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TierInfo {
    pub name: String,
    pub display_name: String,
    pub limits: TenantLimitsResponse,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProvisionRequest {
    pub org_name: String,
    pub tier: String,
    pub admin_email: String,
    pub admin_display_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProvisionResponse {
    pub tenant_id: String,
    pub org_name: String,
    pub tier: String,
    pub state: String,
    pub message: String,
}

// ── Handlers ──────────────────────────

/// Get the current tenant's info from their verified JWT context.
#[utoipa::path(
    get,
    path = "/api/v1/tenant/me",
    tag = "tenant",
    responses(
        (status = 200, description = "Current tenant info", body = TenantInfoResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_current_tenant(tenant: VerifiedTenant) -> Json<TenantInfoResponse> {
    let ctx = tenant.0;
    Json(TenantInfoResponse {
        tenant_id: ctx.tenant_id().to_string(),
        org_name: "Unknown Organization".to_string(),
        tier: tier_display_name(*ctx.tier()).to_string(),
        data_classification: "unknown".to_string(),
        verified_at: Utc::now().to_rfc3339(),
    })
}

/// Get the current tenant's resource limits.
#[utoipa::path(
    get,
    path = "/api/v1/tenant/limits",
    tag = "tenant",
    responses(
        (status = 200, description = "Tenant resource limits", body = TenantLimitsResponse),
        (status = 401, description = "Missing or invalid token"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_tenant_limits(tenant: VerifiedTenant) -> Json<TenantLimitsResponse> {
    Json(limits_to_response(tenant.0.tier()))
}

/// List all available subscription tiers with their limits.
#[utoipa::path(
    get,
    path = "/api/v1/tenant/tiers",
    tag = "tenant",
    responses(
        (status = 200, description = "All subscription tiers", body = Vec<TierInfo>),
    )
)]
pub async fn list_tiers() -> Json<Vec<TierInfo>> {
    let tiers = [
        SubscriptionTier::Academic,
        SubscriptionTier::Explorer,
        SubscriptionTier::Accelerator,
        SubscriptionTier::Enterprise,
        SubscriptionTier::Custom,
    ];

    let infos: Vec<TierInfo> = tiers
        .iter()
        .map(|t: &SubscriptionTier| TierInfo {
            name: format!("{t:?}").to_lowercase(),
            display_name: tier_display_name(*t).to_string(),
            limits: limits_to_response(t),
        })
        .collect();

    Json(infos)
}

/// Provision a new tenant (admin operation).
#[utoipa::path(
    post,
    path = "/api/v1/tenant/provision",
    tag = "tenant",
    request_body = ProvisionRequest,
    responses(
        (status = 201, description = "Tenant provisioned", body = ProvisionResponse),
        (status = 400, description = "Invalid request"),
    )
)]
pub async fn provision_tenant(
    Json(req): Json<ProvisionRequest>,
) -> (axum::http::StatusCode, Json<ProvisionResponse>) {
    // Parse tier
    let tier_str = req.tier.to_lowercase();
    let Some(tier) = parse_subscription_tier(&tier_str) else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(ProvisionResponse {
                tenant_id: String::new(),
                org_name: req.org_name,
                tier: req.tier,
                state: "error".to_string(),
                message: "Unknown subscription tier".to_string(),
            }),
        );
    };

    // Stub: returns synthetic provision until Supabase tenant schema lands.
    // Real flow: INSERT tenant row → CREATE schema → SET RLS policies → activate.

    (
        axum::http::StatusCode::CREATED,
        Json(ProvisionResponse {
            tenant_id: uuid::Uuid::new_v4().to_string(),
            org_name: req.org_name,
            tier: tier_display_name(tier).to_string(),
            state: "trial".to_string(),
            message: "Tenant provisioned. Activate after setup completes.".to_string(),
        }),
    )
}

fn limits_to_response(tier: &SubscriptionTier) -> TenantLimitsResponse {
    let max_concurrent_programs = tier.max_programs().unwrap_or(u32::MAX);
    let max_team_members = tier.max_users().unwrap_or(u32::MAX);
    let storage_quota_gb = tier.storage_bytes() / BYTES_PER_GB;

    TenantLimitsResponse {
        max_concurrent_programs,
        storage_quota_gb,
        api_rate_limit_rpm: api_rate_limit_for_tier(tier),
        max_team_members,
        ml_compute_enabled: tier.has_api_access()
            && tier.rank() >= SubscriptionTier::Accelerator.rank(),
        marketplace_publish: tier.rank() >= SubscriptionTier::Enterprise.rank(),
        custom_compliance: matches!(tier, SubscriptionTier::Custom),
    }
}

fn tier_display_name(tier: SubscriptionTier) -> &'static str {
    match tier {
        SubscriptionTier::Academic => "Academic",
        SubscriptionTier::Explorer => "Explorer",
        SubscriptionTier::Accelerator => "Accelerator",
        SubscriptionTier::Enterprise => "Enterprise",
        SubscriptionTier::Custom => "Custom",
    }
}

fn parse_subscription_tier(value: &str) -> Option<SubscriptionTier> {
    match value {
        "academic" => Some(SubscriptionTier::Academic),
        "explorer" => Some(SubscriptionTier::Explorer),
        "accelerator" => Some(SubscriptionTier::Accelerator),
        "enterprise" => Some(SubscriptionTier::Enterprise),
        "custom" => Some(SubscriptionTier::Custom),
        _ => None,
    }
}

fn api_rate_limit_for_tier(tier: &SubscriptionTier) -> u32 {
    match tier {
        SubscriptionTier::Academic => 120,
        SubscriptionTier::Explorer => 60,
        SubscriptionTier::Accelerator => 600,
        SubscriptionTier::Enterprise => 2_000,
        SubscriptionTier::Custom => 5_000,
    }
}
