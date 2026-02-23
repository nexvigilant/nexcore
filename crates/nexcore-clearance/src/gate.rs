//! # Clearance Gate
//!
//! Enforcement engine that evaluates operations against classification policies.
//!
//! ## Primitive Grounding
//! - **GateResult**: T2-P, Dominant: ∂ Boundary (∂ + Σ)
//! - **ClearanceGate**: T3, Dominant: ∂ Boundary (∂ + κ + ς + σ + → + N)

use crate::access::AccessMode;
use crate::audit::{AuditAction, ClearanceAudit, ClearanceEntry};
use crate::config::ClearanceConfig;
use crate::level::ClassificationLevel;
use crate::tag::TagTarget;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of a gate evaluation.
///
/// ## Tier: T2-P
/// ## Dominant: ∂ Boundary
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateResult {
    /// Operation is allowed without restrictions.
    Allowed,
    /// Operation is allowed but was logged.
    Logged(String),
    /// Operation is allowed but user was warned.
    Warned(String),
    /// Operation is denied.
    Denied(String),
    /// Operation requires dual authorization to proceed.
    DualAuthRequired(String),
    /// Operation was escalated to a higher authority.
    Escalated(String),
}

impl GateResult {
    /// Whether the operation can proceed.
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Allowed | Self::Logged(_) | Self::Warned(_))
    }

    /// Whether the operation is blocked.
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Denied(_))
    }

    /// Hook exit code: 0=pass, 1=warn, 2=block.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Allowed | Self::Logged(_) => 0,
            Self::Warned(_) => 1,
            Self::Denied(_) | Self::DualAuthRequired(_) | Self::Escalated(_) => 2,
        }
    }
}

impl fmt::Display for GateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allowed => write!(f, "ALLOWED"),
            Self::Logged(msg) => write!(f, "LOGGED: {msg}"),
            Self::Warned(msg) => write!(f, "WARNED: {msg}"),
            Self::Denied(msg) => write!(f, "DENIED: {msg}"),
            Self::DualAuthRequired(msg) => write!(f, "DUAL_AUTH: {msg}"),
            Self::Escalated(msg) => write!(f, "ESCALATED: {msg}"),
        }
    }
}

/// The clearance enforcement engine.
///
/// Evaluates operations against the configured classification policies
/// and records decisions in an audit trail.
///
/// ## Tier: T3
/// ## Dominant: ∂ Boundary
pub struct ClearanceGate {
    config: ClearanceConfig,
    audit: ClearanceAudit,
}

impl ClearanceGate {
    /// Create a new gate with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ClearanceConfig::with_defaults(),
            audit: ClearanceAudit::new(),
        }
    }

    /// Create a gate with custom configuration.
    #[must_use]
    pub fn with_config(config: ClearanceConfig) -> Self {
        Self {
            config,
            audit: ClearanceAudit::new(),
        }
    }

    /// Get a reference to the audit trail.
    #[must_use]
    pub fn audit(&self) -> &ClearanceAudit {
        &self.audit
    }

    /// Get a reference to the configuration.
    #[must_use]
    pub fn config(&self) -> &ClearanceConfig {
        &self.config
    }

    /// Evaluate a read/access operation.
    pub fn evaluate_access(
        &mut self,
        target: &TagTarget,
        level: ClassificationLevel,
        actor: &str,
    ) -> GateResult {
        let policy = self.config.policy_for(level);
        let mode = self.config.effective_mode(level);

        let result = match mode {
            AccessMode::Unrestricted => GateResult::Allowed,
            AccessMode::Aware => GateResult::Logged(format!("access to {target} [{level}]")),
            AccessMode::Guarded => {
                if level.is_restricted() {
                    GateResult::Warned(format!("accessing restricted {target} [{level}]"))
                } else {
                    GateResult::Logged(format!("access to {target} [{level}]"))
                }
            }
            AccessMode::Enforced | AccessMode::Lockdown => {
                GateResult::Logged(format!("enforced access to {target} [{level}]"))
            }
        };

        // Record in audit trail if policy requires
        if policy.audit {
            self.audit.append(ClearanceEntry::new(
                target.clone(),
                AuditAction::Access,
                level,
                actor,
                result.to_string(),
            ));
        }

        result
    }

    /// Evaluate a write operation.
    pub fn evaluate_write(
        &mut self,
        target: &TagTarget,
        level: ClassificationLevel,
        actor: &str,
    ) -> GateResult {
        let policy = self.config.policy_for(level);
        let mode = self.config.effective_mode(level);

        let result = match mode {
            AccessMode::Unrestricted => GateResult::Allowed,
            AccessMode::Aware => GateResult::Logged(format!("write to {target} [{level}]")),
            AccessMode::Guarded => {
                if policy.warn_on_write {
                    GateResult::Warned(format!("writing to classified {target} [{level}]"))
                } else {
                    GateResult::Logged(format!("write to {target} [{level}]"))
                }
            }
            AccessMode::Enforced => {
                if policy.require_dual_auth {
                    GateResult::DualAuthRequired(format!(
                        "write to {target} [{level}] requires dual auth"
                    ))
                } else {
                    GateResult::Warned(format!("enforced write to {target} [{level}]"))
                }
            }
            AccessMode::Lockdown => {
                GateResult::Denied(format!("write to {target} [{level}] blocked in lockdown"))
            }
        };

        // Record in audit trail
        if policy.audit {
            let action = if result.is_block() {
                AuditAction::Denied
            } else {
                AuditAction::Write
            };
            self.audit.append(ClearanceEntry::new(
                target.clone(),
                action,
                level,
                actor,
                result.to_string(),
            ));
        }

        result
    }

    /// Evaluate an external tool call.
    pub fn evaluate_external_call(
        &mut self,
        target: &TagTarget,
        level: ClassificationLevel,
        tool_name: &str,
        actor: &str,
    ) -> GateResult {
        let policy = self.config.policy_for(level);

        let result = if policy.block_external_tools {
            GateResult::Denied(format!(
                "external tool '{tool_name}' blocked for {target} [{level}]"
            ))
        } else if policy.block_external {
            GateResult::Warned(format!(
                "external tool '{tool_name}' on classified {target} [{level}]"
            ))
        } else {
            GateResult::Allowed
        };

        if policy.audit {
            let action = if result.is_block() {
                AuditAction::Denied
            } else {
                AuditAction::ExternalCall
            };
            self.audit.append(ClearanceEntry::new(
                target.clone(),
                action,
                level,
                actor,
                format!("tool={tool_name}: {result}"),
            ));
        }

        result
    }
}

impl Default for ClearanceGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_target(name: &str) -> TagTarget {
        TagTarget::File(name.into())
    }

    #[test]
    fn public_access_allowed() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_access(
            &file_target("readme.md"),
            ClassificationLevel::Public,
            "user",
        );
        assert!(result.is_pass());
        assert_eq!(result, GateResult::Allowed);
    }

    #[test]
    fn internal_access_logged() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_access(
            &file_target("internal.rs"),
            ClassificationLevel::Internal,
            "user",
        );
        assert!(result.is_pass());
        assert!(matches!(result, GateResult::Logged(_)));
    }

    #[test]
    fn confidential_access_warned() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_access(
            &file_target("algo.rs"),
            ClassificationLevel::Confidential,
            "user",
        );
        assert!(result.is_pass());
        assert!(matches!(result, GateResult::Warned(_)));
    }

    #[test]
    fn secret_write_requires_dual_auth() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_write(
            &file_target("trade_secret.rs"),
            ClassificationLevel::Secret,
            "user",
        );
        assert!(matches!(result, GateResult::DualAuthRequired(_)));
    }

    #[test]
    fn top_secret_write_denied() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_write(
            &file_target("key.pem"),
            ClassificationLevel::TopSecret,
            "user",
        );
        assert!(result.is_block());
    }

    #[test]
    fn top_secret_external_tool_denied() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_external_call(
            &file_target("phi.json"),
            ClassificationLevel::TopSecret,
            "WebFetch",
            "user",
        );
        assert!(result.is_block());
    }

    #[test]
    fn public_external_tool_allowed() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_external_call(
            &file_target("open_data.json"),
            ClassificationLevel::Public,
            "WebFetch",
            "user",
        );
        assert!(result.is_pass());
        assert_eq!(result, GateResult::Allowed);
    }

    #[test]
    fn confidential_external_warned() {
        let mut gate = ClearanceGate::new();
        let result = gate.evaluate_external_call(
            &file_target("report.rs"),
            ClassificationLevel::Confidential,
            "WebSearch",
            "user",
        );
        assert!(matches!(result, GateResult::Warned(_)));
    }

    #[test]
    fn audit_trail_recorded_on_access() {
        let mut gate = ClearanceGate::new();
        let _ = gate.evaluate_access(
            &file_target("internal.rs"),
            ClassificationLevel::Internal,
            "user",
        );
        assert_eq!(gate.audit().len(), 1);
    }

    #[test]
    fn audit_trail_not_recorded_for_public() {
        let mut gate = ClearanceGate::new();
        let _ = gate.evaluate_access(
            &file_target("readme.md"),
            ClassificationLevel::Public,
            "user",
        );
        assert_eq!(gate.audit().len(), 0);
    }

    #[test]
    fn exit_code_pass() {
        assert_eq!(GateResult::Allowed.exit_code(), 0);
        assert_eq!(GateResult::Logged("test".into()).exit_code(), 0);
    }

    #[test]
    fn exit_code_warn() {
        assert_eq!(GateResult::Warned("test".into()).exit_code(), 1);
    }

    #[test]
    fn exit_code_block() {
        assert_eq!(GateResult::Denied("test".into()).exit_code(), 2);
        assert_eq!(GateResult::DualAuthRequired("test".into()).exit_code(), 2);
    }

    #[test]
    fn gate_result_display() {
        assert_eq!(GateResult::Allowed.to_string(), "ALLOWED");
        let denied = GateResult::Denied("blocked".into());
        assert!(denied.to_string().contains("DENIED"));
    }
}
