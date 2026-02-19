//! Health check endpoints

use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

/// Health check response
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Uptime in seconds (placeholder)
    pub uptime_seconds: u64,
}

/// Readiness check response
#[derive(Serialize, ToSchema)]
pub struct ReadyResponse {
    /// Ready status
    pub ready: bool,
    /// Component statuses
    pub components: ComponentStatus,
}

/// Component status
#[derive(Serialize, ToSchema)]
pub struct ComponentStatus {
    /// Vigilance kernel status
    pub vigilance: bool,
    /// Brain system status
    pub brain: bool,
    /// Skills registry status
    pub skills: bool,
}

/// Health check router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/", get(health))
        .route("/ready", get(ready))
}

/// Basic health check
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0,
    })
}

/// Readiness check - verifies all components are operational
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service readiness status", body = ReadyResponse)
    )
)]
pub async fn ready() -> Json<ReadyResponse> {
    // Check components
    let vigilance = true; // Vigilance is a library, always available
    let brain_path = shellexpand::tilde("~/.claude/brain").to_string();
    let skills_path = shellexpand::tilde("~/.claude/skills").to_string();
    let brain = std::path::Path::new(&brain_path).exists();
    let skills = std::path::Path::new(&skills_path).exists();

    Json(ReadyResponse {
        ready: vigilance && brain && skills,
        components: ComponentStatus {
            vigilance,
            brain,
            skills,
        },
    })
}
