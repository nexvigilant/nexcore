//! # Clearance Error Types
//!
//! Sum type for all classification-related errors.
//!
//! ## Primitive Grounding
//! - **Tier**: T2-P
//! - **Dominant**: ∂ Boundary (error = boundary violation)
//! - **Composition**: ∂ + Σ (boundary + sum)

use std::fmt;

/// All errors arising from the clearance system.
///
/// ## Tier: T2-P
/// ## Dominant: ∂ Boundary + Σ Sum
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClearanceError {
    /// Attempted operation above the actor's clearance level.
    InsufficientClearance {
        /// The level required for the operation.
        required: String,
        /// The level the actor holds.
        held: String,
    },
    /// Attempted to downgrade a classification without permission.
    DowngradeBlocked {
        /// The current classification level.
        from: String,
        /// The requested lower level.
        to: String,
    },
    /// Dual-authorization was required but not provided.
    DualAuthRequired(String),
    /// Access to an external tool was blocked by classification policy.
    ExternalToolBlocked(String),
    /// Cross-boundary operation was denied.
    BoundaryViolation {
        /// Source classification context.
        source: String,
        /// Target classification context.
        target: String,
    },
    /// The classification level is not recognized.
    UnknownLevel(String),
    /// Audit trail integrity was compromised.
    AuditIntegrity(String),
    /// Policy configuration is invalid.
    InvalidPolicy(String),
    /// Tag target is not recognized.
    InvalidTarget(String),
}

impl fmt::Display for ClearanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientClearance { required, held } => {
                write!(
                    f,
                    "insufficient clearance: requires {required}, holds {held}"
                )
            }
            Self::DowngradeBlocked { from, to } => {
                write!(f, "downgrade blocked: cannot lower from {from} to {to}")
            }
            Self::DualAuthRequired(ctx) => {
                write!(f, "dual authorization required: {ctx}")
            }
            Self::ExternalToolBlocked(tool) => {
                write!(f, "external tool blocked by classification: {tool}")
            }
            Self::BoundaryViolation { source, target } => {
                write!(f, "boundary violation: {source} -> {target}")
            }
            Self::UnknownLevel(lvl) => {
                write!(f, "unknown classification level: {lvl}")
            }
            Self::AuditIntegrity(msg) => {
                write!(f, "audit integrity error: {msg}")
            }
            Self::InvalidPolicy(msg) => {
                write!(f, "invalid policy: {msg}")
            }
            Self::InvalidTarget(msg) => {
                write!(f, "invalid tag target: {msg}")
            }
        }
    }
}

impl std::error::Error for ClearanceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_insufficient_clearance() {
        let err = ClearanceError::InsufficientClearance {
            required: "Secret".into(),
            held: "Internal".into(),
        };
        assert!(err.to_string().contains("Secret"));
        assert!(err.to_string().contains("Internal"));
    }

    #[test]
    fn display_downgrade_blocked() {
        let err = ClearanceError::DowngradeBlocked {
            from: "Secret".into(),
            to: "Public".into(),
        };
        assert!(err.to_string().contains("downgrade"));
    }

    #[test]
    fn display_dual_auth() {
        let err = ClearanceError::DualAuthRequired("file access".into());
        assert!(err.to_string().contains("dual authorization"));
    }

    #[test]
    fn display_external_tool_blocked() {
        let err = ClearanceError::ExternalToolBlocked("WebFetch".into());
        assert!(err.to_string().contains("WebFetch"));
    }

    #[test]
    fn error_trait_impl() {
        let err = ClearanceError::AuditIntegrity("tampered".into());
        let _: &dyn std::error::Error = &err;
        assert!(err.to_string().contains("tampered"));
    }
}
