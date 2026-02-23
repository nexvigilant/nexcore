//! # Cross-Boundary Validator
//!
//! Validates operations that cross classification boundaries.
//!
//! ## Primitive Grounding
//! - **Tier**: T2-P
//! - **Dominant**: ∂ Boundary (∂ + κ)

use crate::access::AccessMode;
use crate::level::ClassificationLevel;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Direction of a classification change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    /// Moving to a higher classification (more restrictive).
    Upgrade,
    /// Moving to a lower classification (less restrictive).
    Downgrade,
    /// Same level — no change.
    Neutral,
}

/// Result of a cross-boundary validation check.
///
/// ## Tier: T2-P
/// ## Dominant: ∂ Boundary
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Operation is allowed.
    Allowed,
    /// Operation is allowed but should be logged.
    AllowedWithAudit(String),
    /// Operation is allowed but user should be warned.
    Warned(String),
    /// Operation requires dual authorization.
    RequiresDualAuth(String),
    /// Operation is denied.
    Denied(String),
}

impl ValidationResult {
    /// Whether the operation is permitted (Allowed, AllowedWithAudit, or Warned).
    #[must_use]
    pub fn is_permitted(&self) -> bool {
        matches!(
            self,
            Self::Allowed | Self::AllowedWithAudit(_) | Self::Warned(_)
        )
    }

    /// Whether the operation is blocked (Denied).
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Denied(_))
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allowed => write!(f, "ALLOWED"),
            Self::AllowedWithAudit(msg) => write!(f, "ALLOWED (audit: {msg})"),
            Self::Warned(msg) => write!(f, "WARNED: {msg}"),
            Self::RequiresDualAuth(msg) => write!(f, "DUAL_AUTH_REQUIRED: {msg}"),
            Self::Denied(msg) => write!(f, "DENIED: {msg}"),
        }
    }
}

/// Validates cross-boundary classification operations.
///
/// ## Tier: T2-P
/// ## Dominant: ∂ Boundary
pub struct CrossBoundaryValidator;

impl CrossBoundaryValidator {
    /// Determine the direction of a level change.
    #[must_use]
    pub fn direction(from: ClassificationLevel, to: ClassificationLevel) -> ChangeDirection {
        match to.cmp(&from) {
            std::cmp::Ordering::Greater => ChangeDirection::Upgrade,
            std::cmp::Ordering::Less => ChangeDirection::Downgrade,
            std::cmp::Ordering::Equal => ChangeDirection::Neutral,
        }
    }

    /// Validate a classification change given the current access mode.
    #[must_use]
    pub fn validate_change(
        from: ClassificationLevel,
        to: ClassificationLevel,
        mode: AccessMode,
        downgrade_permitted: bool,
    ) -> ValidationResult {
        let direction = Self::direction(from, to);

        match direction {
            ChangeDirection::Neutral => ValidationResult::Allowed,
            ChangeDirection::Upgrade => {
                // Upgrades are always safer — but may need audit
                if mode.requires_access_log() {
                    ValidationResult::AllowedWithAudit(format!("upgrade from {from} to {to}"))
                } else {
                    ValidationResult::Allowed
                }
            }
            ChangeDirection::Downgrade => {
                Self::validate_downgrade(from, to, mode, downgrade_permitted)
            }
        }
    }

    /// Validate a downgrade specifically.
    #[must_use]
    fn validate_downgrade(
        from: ClassificationLevel,
        to: ClassificationLevel,
        mode: AccessMode,
        downgrade_permitted: bool,
    ) -> ValidationResult {
        // In Lockdown mode, no downgrades allowed
        if mode == AccessMode::Lockdown {
            return ValidationResult::Denied(format!(
                "downgrade from {from} to {to} blocked in Lockdown mode"
            ));
        }

        // In Enforced mode, dual-auth required for any downgrade
        if mode == AccessMode::Enforced {
            if !downgrade_permitted {
                return ValidationResult::RequiresDualAuth(format!(
                    "downgrade from {from} to {to} requires dual authorization"
                ));
            }
            return ValidationResult::AllowedWithAudit(format!(
                "permitted downgrade from {from} to {to}"
            ));
        }

        // In Guarded mode, warn on downgrade
        if mode == AccessMode::Guarded {
            return ValidationResult::Warned(format!("downgrading from {from} to {to}"));
        }

        // Aware or Unrestricted — allow with audit if aware
        if mode.requires_access_log() {
            ValidationResult::AllowedWithAudit(format!("downgrade from {from} to {to}"))
        } else {
            ValidationResult::Allowed
        }
    }

    /// Check if an access operation crosses a boundary.
    #[must_use]
    pub fn crosses_boundary(
        actor_level: ClassificationLevel,
        target_level: ClassificationLevel,
    ) -> bool {
        actor_level != target_level
    }

    /// Validate an access operation (reading classified data).
    #[must_use]
    pub fn validate_access(
        actor_level: ClassificationLevel,
        target_level: ClassificationLevel,
        mode: AccessMode,
    ) -> ValidationResult {
        // Same level — always allowed
        if actor_level == target_level {
            return ValidationResult::Allowed;
        }

        // Actor level >= target level — permitted (higher clearance can read lower)
        if actor_level >= target_level {
            if mode.requires_access_log() {
                return ValidationResult::AllowedWithAudit(format!(
                    "cross-level access: {actor_level} accessing {target_level}"
                ));
            }
            return ValidationResult::Allowed;
        }

        // Actor level < target level — accessing above clearance
        if mode.is_enforcement_active() {
            ValidationResult::Denied(format!(
                "insufficient clearance: {actor_level} cannot access {target_level} data"
            ))
        } else if mode == AccessMode::Guarded {
            ValidationResult::Warned(format!(
                "accessing above clearance: {actor_level} -> {target_level}"
            ))
        } else if mode.requires_access_log() {
            ValidationResult::AllowedWithAudit(format!(
                "above-clearance access: {actor_level} -> {target_level}"
            ))
        } else {
            ValidationResult::Allowed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_upgrade() {
        assert_eq!(
            CrossBoundaryValidator::direction(
                ClassificationLevel::Internal,
                ClassificationLevel::Secret,
            ),
            ChangeDirection::Upgrade,
        );
    }

    #[test]
    fn direction_downgrade() {
        assert_eq!(
            CrossBoundaryValidator::direction(
                ClassificationLevel::Secret,
                ClassificationLevel::Public,
            ),
            ChangeDirection::Downgrade,
        );
    }

    #[test]
    fn direction_neutral() {
        assert_eq!(
            CrossBoundaryValidator::direction(
                ClassificationLevel::Internal,
                ClassificationLevel::Internal,
            ),
            ChangeDirection::Neutral,
        );
    }

    #[test]
    fn same_level_always_allowed() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Secret,
            ClassificationLevel::Secret,
            AccessMode::Lockdown,
            false,
        );
        assert_eq!(result, ValidationResult::Allowed);
    }

    #[test]
    fn upgrade_allowed_with_audit() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Internal,
            ClassificationLevel::Secret,
            AccessMode::Aware,
            false,
        );
        assert!(result.is_permitted());
    }

    #[test]
    fn downgrade_blocked_in_lockdown() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Secret,
            ClassificationLevel::Public,
            AccessMode::Lockdown,
            false,
        );
        assert!(result.is_blocked());
    }

    #[test]
    fn downgrade_requires_dual_auth_in_enforced() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Secret,
            ClassificationLevel::Internal,
            AccessMode::Enforced,
            false,
        );
        assert!(matches!(result, ValidationResult::RequiresDualAuth(_)));
    }

    #[test]
    fn downgrade_permitted_in_enforced_with_flag() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Secret,
            ClassificationLevel::Internal,
            AccessMode::Enforced,
            true,
        );
        assert!(result.is_permitted());
    }

    #[test]
    fn downgrade_warned_in_guarded() {
        let result = CrossBoundaryValidator::validate_change(
            ClassificationLevel::Confidential,
            ClassificationLevel::Public,
            AccessMode::Guarded,
            false,
        );
        assert!(matches!(result, ValidationResult::Warned(_)));
    }

    #[test]
    fn crosses_boundary_different_levels() {
        assert!(CrossBoundaryValidator::crosses_boundary(
            ClassificationLevel::Internal,
            ClassificationLevel::Secret,
        ));
    }

    #[test]
    fn no_boundary_same_level() {
        assert!(!CrossBoundaryValidator::crosses_boundary(
            ClassificationLevel::Secret,
            ClassificationLevel::Secret,
        ));
    }
}
