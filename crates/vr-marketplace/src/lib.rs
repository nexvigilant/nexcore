//! # vr-marketplace — PRPaaS Marketplace Engine
//!
//! Manages the marketplace where tenants discover and order services from
//! third-party providers (CROs, ML model creators, experts, data providers).
//!
//! ## Modules
//!
//! - [`providers`] — Provider registration, review, and rating.
//! - [`catalog`] — Service catalog with pricing and specifications.
//! - [`ordering`] — Order lifecycle state machine and commission calculation.
//! - [`models`] — ML model marketplace with revenue sharing.
//! - [`scoring`] — CRO performance scoring and tier classification.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod catalog;
pub mod models;
pub mod ordering;
pub mod providers;
pub mod scoring;

// Re-export key types at crate root for ergonomic imports.
pub use catalog::{
    CatalogEntry, CatalogEntryId, CatalogEntryStatus, PricingModel, ServiceType,
    estimate_order_cost,
};
pub use models::{MarketplaceModel, ModelStatus, ModelType, calculate_model_revenue_share};
pub use ordering::{Order, OrderStatus, calculate_order_commission, validate_order_transition};
pub use providers::{Provider, ProviderStatus, ProviderType, update_rating};
pub use scoring::{CroPerformanceMetrics, calculate_composite_score, performance_tier};
