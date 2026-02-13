//! # Ghost Error Types
//!
//! All fallible operations in the ghost crate return `GhostError`.
//!
//! ## Tier: T2-C (∂ Boundary + Σ Sum)
//! Errors are boundary violations (∂) composed as a sum type (Σ).

use std::fmt;

/// Errors produced by the ghost privacy subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GhostError {
    /// HMAC key is missing or invalid length.
    InvalidKey(String),
    /// Attempted operation not permitted in the current mode.
    ModeViolation(String),
    /// Attempted reversal without proper authorization.
    ReversalDenied(String),
    /// Data category not found in config.
    UnknownCategory(String),
    /// k-anonymity threshold not met.
    KAnonymityViolation {
        /// Required minimum group size.
        required: u32,
        /// Actual group size observed.
        actual: u32,
    },
    /// Lifecycle transition not permitted.
    InvalidTransition(String),
    /// Audit trail integrity failure.
    AuditIntegrity(String),
    /// PII detected where it should not exist.
    PiiLeak(String),
}

impl fmt::Display for GhostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(msg) => write!(f, "ghost: invalid key — {msg}"),
            Self::ModeViolation(msg) => write!(f, "ghost: mode violation — {msg}"),
            Self::ReversalDenied(msg) => write!(f, "ghost: reversal denied — {msg}"),
            Self::UnknownCategory(cat) => write!(f, "ghost: unknown category — {cat}"),
            Self::KAnonymityViolation { required, actual } => {
                write!(
                    f,
                    "ghost: k-anonymity violation (need {required}, got {actual})"
                )
            }
            Self::InvalidTransition(msg) => write!(f, "ghost: invalid transition — {msg}"),
            Self::AuditIntegrity(msg) => write!(f, "ghost: audit integrity — {msg}"),
            Self::PiiLeak(msg) => write!(f, "ghost: PII leak detected — {msg}"),
        }
    }
}

impl std::error::Error for GhostError {}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, GhostError>;

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats_correctly() {
        let e = GhostError::KAnonymityViolation {
            required: 5,
            actual: 2,
        };
        let msg = format!("{e}");
        assert!(msg.contains("need 5"));
        assert!(msg.contains("got 2"));
    }

    #[test]
    fn error_display_mode_violation() {
        let e = GhostError::ModeViolation("reversal not allowed in Maximum mode".into());
        let msg = format!("{e}");
        assert!(msg.contains("mode violation"));
        assert!(msg.contains("Maximum"));
    }

    #[test]
    fn error_implements_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(GhostError::PiiLeak("email in output".into()));
        assert!(e.to_string().contains("PII leak"));
    }
}
