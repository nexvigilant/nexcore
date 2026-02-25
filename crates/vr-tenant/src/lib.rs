//! # vr-tenant — PRPaaS Tenant Lifecycle Management
//!
//! Provisioning, tier enforcement, team management, and lifecycle
//! transitions for multi-tenant pharmaceutical research platform.
//!
//! ## Modules
//!
//! - [`tiers`] — Feature gating, usage limits, overage handling
//! - [`teams`] — User invitation, role management, RBAC validation
//! - [`provisioning`] — Self-service signup, resource allocation
//! - [`lifecycle`] — Suspend, reactivate, offboard, data retention

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod lifecycle;
pub mod provisioning;
pub mod teams;
pub mod tiers;

pub use lifecycle::{
    LifecycleTransition, OffboardingChecklist, RetentionPolicy, create_transition,
    validate_transition,
};
pub use provisioning::{
    ProvisioningPlan, ProvisioningStep, SignupRequest, StepStatus, build_tenant_record,
    create_provisioning_plan, generate_slug, is_trial_expired,
};
pub use teams::{
    InviteMemberRequest, MemberStatus, TeamMember, validate_invitation, validate_role_change,
};
pub use tiers::{
    Feature, LimitCheckResult, TenantUsage, check_all_limits, check_program_limit,
    check_storage_limit, check_user_limit, require_feature, storage_overage_gb,
};
