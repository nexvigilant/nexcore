//! # Clearance Policy
//!
//! Per-level enforcement rules defining what is allowed/blocked.
//!
//! ## Primitive Grounding
//! - **Tier**: T2-P
//! - **Dominant**: ς State (policy is a state configuration)
//! - **Composition**: ς + ∂ + κ

use crate::access::AccessMode;
use crate::level::ClassificationLevel;
use serde::{Deserialize, Serialize};

/// Enforcement rules for a single classification level.
///
/// ## Tier: T2-P
/// ## Dominant: ς State
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearancePolicy {
    /// The Claude behavioral mode for this level.
    pub access_mode: AccessMode,
    /// Whether access should be audit-logged.
    pub audit: bool,
    /// Whether to warn before writing classified data.
    pub warn_on_write: bool,
    /// Whether external data sharing is blocked.
    pub block_external: bool,
    /// Whether dual authorization is required for sensitive ops.
    pub require_dual_auth: bool,
    /// Whether external tool calls (WebFetch, etc.) are blocked.
    pub block_external_tools: bool,
}

impl ClearancePolicy {
    /// Default policy for a given classification level.
    #[must_use]
    pub fn default_for(level: ClassificationLevel) -> Self {
        match level {
            ClassificationLevel::Public => Self {
                access_mode: AccessMode::Unrestricted,
                audit: false,
                warn_on_write: false,
                block_external: false,
                require_dual_auth: false,
                block_external_tools: false,
            },
            ClassificationLevel::Internal => Self {
                access_mode: AccessMode::Aware,
                audit: true,
                warn_on_write: false,
                block_external: false,
                require_dual_auth: false,
                block_external_tools: false,
            },
            ClassificationLevel::Confidential => Self {
                access_mode: AccessMode::Guarded,
                audit: true,
                warn_on_write: true,
                block_external: true,
                require_dual_auth: false,
                block_external_tools: false,
            },
            ClassificationLevel::Secret => Self {
                access_mode: AccessMode::Enforced,
                audit: true,
                warn_on_write: true,
                block_external: true,
                require_dual_auth: true,
                block_external_tools: false,
            },
            ClassificationLevel::TopSecret => Self {
                access_mode: AccessMode::Lockdown,
                audit: true,
                warn_on_write: true,
                block_external: true,
                require_dual_auth: true,
                block_external_tools: true,
            },
        }
    }

    /// Strict policy: everything enabled regardless of level.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            access_mode: AccessMode::Lockdown,
            audit: true,
            warn_on_write: true,
            block_external: true,
            require_dual_auth: true,
            block_external_tools: true,
        }
    }

    /// Whether this policy escalates enforcement compared to another.
    #[must_use]
    pub fn escalates_from(&self, other: &Self) -> bool {
        self.access_mode > other.access_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_policy_defaults() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Public);
        assert_eq!(p.access_mode, AccessMode::Unrestricted);
        assert!(!p.audit);
        assert!(!p.warn_on_write);
        assert!(!p.block_external);
        assert!(!p.require_dual_auth);
        assert!(!p.block_external_tools);
    }

    #[test]
    fn internal_policy_defaults() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Internal);
        assert_eq!(p.access_mode, AccessMode::Aware);
        assert!(p.audit);
        assert!(!p.warn_on_write);
    }

    #[test]
    fn confidential_policy_defaults() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Confidential);
        assert_eq!(p.access_mode, AccessMode::Guarded);
        assert!(p.audit);
        assert!(p.warn_on_write);
        assert!(p.block_external);
        assert!(!p.require_dual_auth);
    }

    #[test]
    fn secret_policy_defaults() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Secret);
        assert_eq!(p.access_mode, AccessMode::Enforced);
        assert!(p.require_dual_auth);
        assert!(!p.block_external_tools);
    }

    #[test]
    fn top_secret_policy_defaults() {
        let p = ClearancePolicy::default_for(ClassificationLevel::TopSecret);
        assert_eq!(p.access_mode, AccessMode::Lockdown);
        assert!(p.require_dual_auth);
        assert!(p.block_external_tools);
    }

    #[test]
    fn strict_policy() {
        let p = ClearancePolicy::strict();
        assert_eq!(p.access_mode, AccessMode::Lockdown);
        assert!(p.audit);
        assert!(p.warn_on_write);
        assert!(p.block_external);
        assert!(p.require_dual_auth);
        assert!(p.block_external_tools);
    }

    #[test]
    fn escalation_detection() {
        let public = ClearancePolicy::default_for(ClassificationLevel::Public);
        let secret = ClearancePolicy::default_for(ClassificationLevel::Secret);
        assert!(secret.escalates_from(&public));
        assert!(!public.escalates_from(&secret));
    }

    #[test]
    fn audit_required_at_internal() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Internal);
        assert!(p.audit);
    }

    #[test]
    fn serde_roundtrip() {
        let p = ClearancePolicy::default_for(ClassificationLevel::Confidential);
        let json = serde_json::to_string(&p).unwrap_or_default();
        let parsed: Result<ClearancePolicy, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        if let Ok(parsed) = parsed {
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn same_level_no_escalation() {
        let a = ClearancePolicy::default_for(ClassificationLevel::Secret);
        let b = ClearancePolicy::default_for(ClassificationLevel::Secret);
        assert!(!a.escalates_from(&b));
    }
}
