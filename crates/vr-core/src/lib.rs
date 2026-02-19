//! # vr-core — PRPaaS Core Domain Types
//!
//! Foundation crate for the Pharmaceutical Research Platform as a Service.
//! Provides type-safe IDs, tenant isolation primitives, money types,
//! and the permission model.
//!
//! ## Key Types
//!
//! - [`TenantContext`] — Extracted from every authenticated request.
//!   All repository methods require it. Cross-tenant access is a compile error.
//! - [`TenantScoped<T>`] — Wraps any value with its owning tenant_id.
//! - [`SubscriptionTier`] — Explorer/Accelerator/Enterprise/Academic/Custom.
//! - [`Money`] — Integer cents for all financial calculations.
//! - Type-safe IDs: [`TenantId`], [`UserId`], [`ProgramId`], etc.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod error;
pub mod ids;
pub mod money;
pub mod tenant;

// Re-export key types at crate root for ergonomic imports.
pub use error::{VrError, VrResult};
pub use ids::*;
pub use money::{Currency, Money};
pub use tenant::{
    Action, Permissions, Resource, SubscriptionTier, Tenant, TenantContext, TenantScoped,
    TenantStatus, UserRole,
};
