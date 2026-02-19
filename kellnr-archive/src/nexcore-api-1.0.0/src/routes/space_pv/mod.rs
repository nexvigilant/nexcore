//! Space Pharmacovigilance API Routes
//!
//! The federalized gateway for pharmaceutical space clearance.
//! Consolidated from ~/projects/space-pv/nexvigilant-api/
//!
//! ## Routes
//!
//! - `/api/v1/space-pv/drugs` - Drug registry management
//! - `/api/v1/space-pv/clearances` - Space clearance applications
//! - `/api/v1/space-pv/organizations` - Organization management
//!
//! ## Authentication
//!
//! Uses API key authentication via `X-API-Key` header.
//! Organizations are identified by their registered API keys.

pub mod clearances;
pub mod drugs;
pub mod models;
pub mod organizations;

use axum::Router;

/// Build all Space PV API routes (stateless).
///
/// Mount under `/api/v1/space-pv`
#[allow(dead_code)]
pub fn router() -> Router {
    Router::new()
        .merge(drugs::router())
        .merge(clearances::router())
        .merge(organizations::router())
}
