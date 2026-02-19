//! Error types for the VR platform.

use crate::ids::TenantId;
use crate::tenant::{Action, Resource, SubscriptionTier};

/// Platform-wide error type.
#[derive(Debug, thiserror::Error)]
pub enum VrError {
    #[error("tenant {tenant_id} not found")]
    TenantNotFound { tenant_id: TenantId },

    #[error("tenant {tenant_id} is not accessible (status: {status})")]
    TenantInaccessible { tenant_id: TenantId, status: String },

    #[error("permission denied: {action:?} on {resource:?} requires higher role")]
    PermissionDenied { action: Action, resource: Resource },

    #[error("tier {current:?} does not include this feature (requires {required:?})")]
    TierInsufficient {
        current: SubscriptionTier,
        required: SubscriptionTier,
    },

    #[error("limit exceeded: {resource} — used {used}/{limit}")]
    LimitExceeded {
        resource: String,
        used: u64,
        limit: u64,
    },

    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("internal error: {message}")]
    Internal { message: String },
}

/// Convenience alias.
pub type VrResult<T> = Result<T, VrError>;
