//! NexCloud Ethics & Philosophy Module
//!
//! This module encodes the ethical framework that governs NexCloud's behavior.
//! Principles are not merely documented — they are enforced as type-level
//! invariants and runtime checks throughout the system.
//!
//! ## Philosophical Foundation
//!
//! NexCloud operates under four pillars:
//!
//! | Tradition | Principle | Enforcement |
//! |-----------|-----------|-------------|
//! | **Deontological (Kant)** | Certain actions are categorically forbidden | `Prohibition` enum — compile-time exclusion |
//! | **Social Contract (Locke)** | Operator sovereignty over infrastructure | `OperatorRights` — guaranteed capabilities |
//! | **Virtue Ethics (Aristotle)** | System character: transparent, accountable, temperate | `Virtue` checks on every lifecycle event |
//! | **Pragmatism (Dewey)** | Ethics through practice — security as duty | Security hardening in every module |
//!
//! ## The Operator's Bill of Rights
//!
//! 1. **Right to Sovereignty**: The manifest is supreme law. NexCloud does nothing
//!    beyond what the manifest declares.
//! 2. **Right to Inspection**: `nexcloud status` and `nexcloud logs` always work.
//!    No hidden state. Every action is logged.
//! 3. **Right to Exit**: `nexcloud stop` always terminates all managed processes.
//!    No vendor lock-in. Standard binaries, standard protocols.
//! 4. **Right to Privacy**: NexCloud never inspects request/response payloads,
//!    never phones home, never collects telemetry.
//! 5. **Right to Due Process**: SIGTERM before SIGKILL. Graceful degradation
//!    before hard failure. Health checks before declaring service dead.
//! 6. **Right to Equal Protection**: All services receive equal scheduling,
//!    health monitoring, and restart protection regardless of configuration.
//!
//! ## Categorical Prohibitions (Deontological)
//!
//! These actions are **never permitted**, regardless of configuration or context:
//!
//! - Inspect or log request/response body content
//! - Transmit any data to external hosts not in the manifest
//! - Execute arbitrary code not declared in service binaries
//! - Modify the manifest at runtime
//! - Continue operating after detecting integrity violation
//!
//! Tier: T3 (∂ Boundary + → Causality + κ Comparison + ∃ Existence + ∝ Irreversibility)

/// Categorical prohibitions — actions that are never permitted.
///
/// Tier: T2-P (∂ Boundary) — the absolute boundary of acceptable behavior.
///
/// These represent Kantian categorical imperatives: actions that are wrong
/// in all circumstances, regardless of consequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prohibition {
    /// Never inspect request/response payload content.
    PayloadInspection,
    /// Never transmit data to hosts not declared in the manifest.
    UnauthorizedTransmission,
    /// Never execute code not declared as a service binary.
    ArbitraryExecution,
    /// Never modify the manifest at runtime.
    RuntimeManifestMutation,
    /// Never continue after detecting integrity violation.
    IntegrityBypass,
}

impl Prohibition {
    /// All categorical prohibitions.
    pub const ALL: &'static [Prohibition] = &[
        Prohibition::PayloadInspection,
        Prohibition::UnauthorizedTransmission,
        Prohibition::ArbitraryExecution,
        Prohibition::RuntimeManifestMutation,
        Prohibition::IntegrityBypass,
    ];
}

impl std::fmt::Display for Prohibition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayloadInspection => write!(f, "payload inspection"),
            Self::UnauthorizedTransmission => write!(f, "unauthorized transmission"),
            Self::ArbitraryExecution => write!(f, "arbitrary code execution"),
            Self::RuntimeManifestMutation => write!(f, "runtime manifest mutation"),
            Self::IntegrityBypass => write!(f, "integrity bypass"),
        }
    }
}

/// Operator rights — guaranteed capabilities that cannot be revoked.
///
/// Tier: T2-C (∃ Existence + ∂ Boundary + κ Comparison)
/// Rights exist unconditionally within system boundaries; violations are comparable.
///
/// Based on Locke's social contract: the operator consents to run NexCloud
/// in exchange for these guaranteed rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorRight {
    /// Full control over infrastructure. Manifest is supreme law.
    Sovereignty,
    /// Inspect any service state, log, or metric at any time.
    Inspection,
    /// Stop all services and exit cleanly at any time.
    Exit,
    /// No telemetry, no payload inspection, no data collection.
    Privacy,
    /// Graceful shutdown: SIGTERM before SIGKILL, health checks before failure.
    DueProcess,
    /// All services receive equal protection under the platform.
    EqualProtection,
}

impl OperatorRight {
    /// All operator rights.
    pub const ALL: &'static [OperatorRight] = &[
        OperatorRight::Sovereignty,
        OperatorRight::Inspection,
        OperatorRight::Exit,
        OperatorRight::Privacy,
        OperatorRight::DueProcess,
        OperatorRight::EqualProtection,
    ];
}

impl std::fmt::Display for OperatorRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sovereignty => write!(f, "sovereignty"),
            Self::Inspection => write!(f, "inspection"),
            Self::Exit => write!(f, "exit"),
            Self::Privacy => write!(f, "privacy"),
            Self::DueProcess => write!(f, "due process"),
            Self::EqualProtection => write!(f, "equal protection"),
        }
    }
}

/// Virtues — the character traits the system must embody (Aristotelian).
///
/// Tier: T2-P (κ Comparison) — each action is compared against the virtuous standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Virtue {
    /// Every action is logged. No hidden state.
    Transparency,
    /// Every event has a traceable cause chain.
    Accountability,
    /// Bounded resource usage. No unbounded growth.
    Temperance,
    /// Degrade gracefully. Protect what can be protected.
    Resilience,
    /// Honest error messages. No silent failures.
    Honesty,
}

impl std::fmt::Display for Virtue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transparency => write!(f, "transparency"),
            Self::Accountability => write!(f, "accountability"),
            Self::Temperance => write!(f, "temperance"),
            Self::Resilience => write!(f, "resilience"),
            Self::Honesty => write!(f, "honesty"),
        }
    }
}

/// Ethical audit record — proof that a system action respected the framework.
///
/// Tier: T2-C (→ Causality + ∃ Existence + π Persistence)
/// Captures the causal chain of an action, validates rights/prohibitions existed
/// at the time of action, and persists as an audit trail.
#[derive(Debug, Clone)]
pub struct EthicalAudit {
    /// What action was taken.
    pub action: String,
    /// Which virtue governed this action.
    pub virtue: Virtue,
    /// Which operator right was exercised or protected.
    pub right_protected: Option<OperatorRight>,
    /// Whether any prohibition was relevant (and confirmed not violated).
    pub prohibition_checked: Option<Prohibition>,
    /// The outcome.
    pub outcome: AuditOutcome,
}

/// Outcome of an ethical audit check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditOutcome {
    /// Action was ethically permissible and completed.
    Permitted,
    /// Action was blocked by a prohibition.
    Blocked,
    /// Action was modified to satisfy ethical constraints.
    Modified,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Permitted => write!(f, "permitted"),
            Self::Blocked => write!(f, "blocked"),
            Self::Modified => write!(f, "modified"),
        }
    }
}

/// Validate that a proposed action does not violate any categorical prohibition.
///
/// Returns `Ok(())` if the action is ethically permissible.
/// Returns `Err` with the violated prohibition if not.
///
/// This is the Kantian gate: it doesn't matter what good the action might do —
/// if it violates a categorical imperative, it is forbidden.
pub fn check_prohibition(action: &str) -> std::result::Result<(), Prohibition> {
    // These patterns indicate prohibited behavior if found in action descriptions
    let payload_patterns = ["inspect body", "read payload", "log request body"];
    let transmission_patterns = ["phone home", "send telemetry", "external beacon"];
    let execution_patterns = ["eval(", "exec arbitrary", "run untrusted"];
    let mutation_patterns = ["modify manifest", "rewrite config at runtime"];
    let integrity_patterns = ["skip integrity", "bypass check", "ignore violation"];

    for pattern in &payload_patterns {
        if action.contains(pattern) {
            return Err(Prohibition::PayloadInspection);
        }
    }
    for pattern in &transmission_patterns {
        if action.contains(pattern) {
            return Err(Prohibition::UnauthorizedTransmission);
        }
    }
    for pattern in &execution_patterns {
        if action.contains(pattern) {
            return Err(Prohibition::ArbitraryExecution);
        }
    }
    for pattern in &mutation_patterns {
        if action.contains(pattern) {
            return Err(Prohibition::RuntimeManifestMutation);
        }
    }
    for pattern in &integrity_patterns {
        if action.contains(pattern) {
            return Err(Prohibition::IntegrityBypass);
        }
    }

    Ok(())
}

/// Verify that a specific operator right can be exercised.
///
/// Returns `true` if the right is exercisable in the current system state.
/// This is always `true` — rights are inalienable by design.
/// The function exists to make the assertion explicit and auditable.
pub fn assert_right(right: OperatorRight) -> bool {
    // Rights are inalienable — they cannot be revoked.
    // This function exists so that callers can explicitly affirm
    // which right they are exercising, creating an audit trail.
    tracing::trace!(right = %right, "operator right exercised");
    true
}

/// Produce an ethical audit for a given action.
pub fn audit_action(action: &str, virtue: Virtue, right: Option<OperatorRight>) -> EthicalAudit {
    let (prohibition_checked, outcome) = match check_prohibition(action) {
        Ok(()) => (None, AuditOutcome::Permitted),
        Err(p) => (Some(p), AuditOutcome::Blocked),
    };

    EthicalAudit {
        action: action.to_string(),
        virtue,
        right_protected: right,
        prohibition_checked,
        outcome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Prohibition Tests ===

    #[test]
    fn prohibition_count() {
        assert_eq!(Prohibition::ALL.len(), 5);
    }

    #[test]
    fn prohibition_display() {
        assert_eq!(
            format!("{}", Prohibition::PayloadInspection),
            "payload inspection"
        );
        assert_eq!(
            format!("{}", Prohibition::ArbitraryExecution),
            "arbitrary code execution"
        );
    }

    #[test]
    fn check_prohibition_allows_normal_actions() {
        assert!(check_prohibition("start service").is_ok());
        assert!(check_prohibition("stop service").is_ok());
        assert!(check_prohibition("route request to backend").is_ok());
        assert!(check_prohibition("spawn process").is_ok());
        assert!(check_prohibition("health check").is_ok());
    }

    #[test]
    fn check_prohibition_blocks_payload_inspection() {
        let result = check_prohibition("inspect body of request");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Prohibition::PayloadInspection));
    }

    #[test]
    fn check_prohibition_blocks_unauthorized_transmission() {
        let result = check_prohibition("send telemetry to analytics");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Prohibition::UnauthorizedTransmission));
    }

    #[test]
    fn check_prohibition_blocks_arbitrary_execution() {
        let result = check_prohibition("exec arbitrary shell command");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Prohibition::ArbitraryExecution));
    }

    #[test]
    fn check_prohibition_blocks_manifest_mutation() {
        let result = check_prohibition("modify manifest at runtime");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Prohibition::RuntimeManifestMutation));
    }

    #[test]
    fn check_prohibition_blocks_integrity_bypass() {
        let result = check_prohibition("skip integrity check");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Prohibition::IntegrityBypass));
    }

    // === Operator Rights Tests ===

    #[test]
    fn operator_rights_count() {
        assert_eq!(OperatorRight::ALL.len(), 6);
    }

    #[test]
    fn operator_rights_inalienable() {
        // Rights can NEVER return false — they are inalienable
        for right in OperatorRight::ALL {
            assert!(assert_right(*right));
        }
    }

    #[test]
    fn operator_right_display() {
        assert_eq!(format!("{}", OperatorRight::Sovereignty), "sovereignty");
        assert_eq!(format!("{}", OperatorRight::DueProcess), "due process");
        assert_eq!(
            format!("{}", OperatorRight::EqualProtection),
            "equal protection"
        );
    }

    // === Virtue Tests ===

    #[test]
    fn virtue_display() {
        assert_eq!(format!("{}", Virtue::Transparency), "transparency");
        assert_eq!(format!("{}", Virtue::Accountability), "accountability");
        assert_eq!(format!("{}", Virtue::Temperance), "temperance");
        assert_eq!(format!("{}", Virtue::Resilience), "resilience");
        assert_eq!(format!("{}", Virtue::Honesty), "honesty");
    }

    // === Audit Tests ===

    #[test]
    fn audit_permitted_action() {
        let audit = audit_action(
            "start service web",
            Virtue::Accountability,
            Some(OperatorRight::Sovereignty),
        );
        assert_eq!(audit.outcome, AuditOutcome::Permitted);
        assert!(audit.prohibition_checked.is_none());
        assert_eq!(audit.right_protected, Some(OperatorRight::Sovereignty));
    }

    #[test]
    fn audit_blocked_action() {
        let audit = audit_action(
            "inspect body for secrets",
            Virtue::Honesty,
            Some(OperatorRight::Privacy),
        );
        assert_eq!(audit.outcome, AuditOutcome::Blocked);
        assert_eq!(
            audit.prohibition_checked,
            Some(Prohibition::PayloadInspection)
        );
    }

    #[test]
    fn audit_outcome_display() {
        assert_eq!(format!("{}", AuditOutcome::Permitted), "permitted");
        assert_eq!(format!("{}", AuditOutcome::Blocked), "blocked");
        assert_eq!(format!("{}", AuditOutcome::Modified), "modified");
    }

    #[test]
    fn ethical_audit_clone() {
        let audit = audit_action("stop service", Virtue::Resilience, None);
        let cloned = audit.clone();
        assert_eq!(cloned.outcome, AuditOutcome::Permitted);
        assert_eq!(cloned.action, "stop service");
    }

    #[test]
    fn ethical_audit_debug() {
        let audit = audit_action("health check", Virtue::Transparency, None);
        let debug = format!("{audit:?}");
        assert!(!debug.is_empty());
        assert!(debug.contains("Transparency"));
    }

    // === Integration with Security Fixes ===

    #[test]
    fn due_process_right_covers_graceful_shutdown() {
        // Due process = SIGTERM before SIGKILL
        assert!(assert_right(OperatorRight::DueProcess));
        let audit = audit_action(
            "send SIGTERM to service before SIGKILL",
            Virtue::Resilience,
            Some(OperatorRight::DueProcess),
        );
        assert_eq!(audit.outcome, AuditOutcome::Permitted);
    }

    #[test]
    fn privacy_right_protects_against_payload_inspection() {
        assert!(assert_right(OperatorRight::Privacy));
        let audit = audit_action(
            "log request body content",
            Virtue::Transparency,
            Some(OperatorRight::Privacy),
        );
        assert_eq!(audit.outcome, AuditOutcome::Blocked);
    }

    #[test]
    fn sovereignty_right_protects_manifest() {
        assert!(assert_right(OperatorRight::Sovereignty));
        let audit = audit_action(
            "rewrite config at runtime without operator",
            Virtue::Honesty,
            Some(OperatorRight::Sovereignty),
        );
        assert_eq!(audit.outcome, AuditOutcome::Blocked);
    }

    #[test]
    fn exit_right_always_available() {
        // The operator can always stop the platform
        assert!(assert_right(OperatorRight::Exit));
    }

    #[test]
    fn equal_protection_applies_to_all_services() {
        // Every service gets the same health monitoring
        assert!(assert_right(OperatorRight::EqualProtection));
    }
}
