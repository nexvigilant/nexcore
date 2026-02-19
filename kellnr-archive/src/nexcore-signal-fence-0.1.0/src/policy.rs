//! Fence policy — default-deny posture grounded in A2 (Noise Dominance).
//!
//! Tier: T2-C (∂ Boundary + ∅ Void + κ Comparison — composite boundary with defaults)
//!
//! The policy embodies the core thesis: "All traffic is noise until proven otherwise."
//! Default-deny means the FencePolicy starts with `FenceVerdict::Deny` and only
//! allows connections that match explicit allowlist rules.

use serde::{Deserialize, Serialize};

use crate::process::ConnectionEvent;
use crate::rule::{FenceRule, FenceVerdict, RuleSet};

/// Operating mode for the fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FenceMode {
    /// Observe and log only — no enforcement. Learning mode.
    Monitor,
    /// Enforce rules: allow matches pass, others get default verdict.
    Enforce,
    /// Maximum security: enforce + alert on every non-allow event.
    Lockdown,
}

impl FenceMode {
    /// Whether this mode actually blocks traffic.
    pub fn enforces(&self) -> bool {
        matches!(self, Self::Enforce | Self::Lockdown)
    }

    /// Whether this mode generates alerts for denied connections.
    pub fn alerts_on_deny(&self) -> bool {
        matches!(self, Self::Lockdown)
    }
}

/// A decision produced by evaluating a connection against the policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenceDecision {
    /// The final verdict.
    pub verdict: FenceVerdict,
    /// The rule that matched (None if default verdict applied).
    pub matched_rule_id: Option<String>,
    /// Reason for the decision.
    pub reason: String,
    /// Whether this decision was actually enforced.
    pub enforced: bool,
}

impl FenceDecision {
    /// Create a decision from a matching rule.
    pub fn from_rule(rule: &FenceRule, enforced: bool) -> Self {
        Self {
            verdict: rule.verdict,
            matched_rule_id: Some(rule.id.clone()),
            reason: if rule.description.is_empty() {
                format!("matched rule '{}'", rule.id)
            } else {
                rule.description.clone()
            },
            enforced,
        }
    }

    /// Create a default-verdict decision (no rule matched).
    pub fn default_verdict(verdict: FenceVerdict, enforced: bool) -> Self {
        Self {
            verdict,
            matched_rule_id: None,
            reason: format!("no rule matched, default verdict: {verdict:?}"),
            enforced,
        }
    }
}

/// The fence policy: operating mode + default verdict + allowlist rules.
///
/// Evaluates connections in priority order against the rule set.
/// If no rule matches, applies the default verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FencePolicy {
    /// Operating mode.
    pub mode: FenceMode,
    /// Default verdict when no rule matches (A2: Deny = noise dominance).
    pub default_verdict: FenceVerdict,
    /// Ordered rule set.
    pub rules: RuleSet,
    /// Whether to audit all decisions (not just denies).
    pub audit_all: bool,
}

impl FencePolicy {
    /// Create a default-deny policy (secure posture).
    ///
    /// Embodies A2 (Noise Dominance): all traffic is noise until an
    /// explicit rule classifies it as signal.
    pub fn default_deny() -> Self {
        Self {
            mode: FenceMode::Enforce,
            default_verdict: FenceVerdict::Deny,
            rules: RuleSet::new(),
            audit_all: false,
        }
    }

    /// Create a default-allow policy (permissive posture).
    ///
    /// For monitoring/learning phases — allow everything, log denials.
    pub fn default_allow() -> Self {
        Self {
            mode: FenceMode::Monitor,
            default_verdict: FenceVerdict::Allow,
            rules: RuleSet::new(),
            audit_all: true,
        }
    }

    /// Create a lockdown policy (maximum security).
    pub fn lockdown() -> Self {
        Self {
            mode: FenceMode::Lockdown,
            default_verdict: FenceVerdict::Deny,
            rules: RuleSet::new(),
            audit_all: true,
        }
    }

    /// Evaluate a connection event against this policy.
    pub fn evaluate(&self, event: &ConnectionEvent) -> FenceDecision {
        let enforced = self.mode.enforces();

        // Check rules in priority order
        if let Some(rule) = self.rules.evaluate(event) {
            return FenceDecision::from_rule(rule, enforced);
        }

        // No rule matched → default verdict
        FenceDecision::default_verdict(self.default_verdict, enforced)
    }

    /// Add a rule to the policy.
    pub fn add_rule(&mut self, rule: FenceRule) {
        self.rules.add(rule);
    }

    /// Number of rules in the policy.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Whether this policy should audit a given decision.
    pub fn should_audit(&self, decision: &FenceDecision) -> bool {
        self.audit_all || decision.verdict.is_blocking()
    }
}

impl Default for FencePolicy {
    fn default() -> Self {
        Self::default_deny()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;
    use crate::connection::{Protocol, SocketEntry, TcpState};
    use crate::process::{Direction, ProcessInfo};
    use crate::rule::{NetworkMatch, ProcessMatch};

    fn make_event(name: &str, local_port: u16, remote_port: u16) -> ConnectionEvent {
        let socket = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port,
            inode: 100,
            state: TcpState::Established,
            protocol: Protocol::Tcp,
            uid: 1000,
        };
        let proc_info = ProcessInfo {
            pid: 1234,
            name: name.to_string(),
            exe: PathBuf::from(format!("/usr/bin/{name}")),
            uid: 1000,
            cmdline: name.to_string(),
        };
        ConnectionEvent::new(socket, Some(proc_info))
    }

    #[test]
    fn test_fence_mode_enforces() {
        assert!(!FenceMode::Monitor.enforces());
        assert!(FenceMode::Enforce.enforces());
        assert!(FenceMode::Lockdown.enforces());
    }

    #[test]
    fn test_fence_mode_alerts() {
        assert!(!FenceMode::Monitor.alerts_on_deny());
        assert!(!FenceMode::Enforce.alerts_on_deny());
        assert!(FenceMode::Lockdown.alerts_on_deny());
    }

    #[test]
    fn test_default_deny_policy() {
        let policy = FencePolicy::default_deny();
        assert_eq!(policy.default_verdict, FenceVerdict::Deny);
        assert_eq!(policy.mode, FenceMode::Enforce);
        assert!(!policy.audit_all);
    }

    #[test]
    fn test_default_allow_policy() {
        let policy = FencePolicy::default_allow();
        assert_eq!(policy.default_verdict, FenceVerdict::Allow);
        assert_eq!(policy.mode, FenceMode::Monitor);
        assert!(policy.audit_all);
    }

    #[test]
    fn test_lockdown_policy() {
        let policy = FencePolicy::lockdown();
        assert_eq!(policy.default_verdict, FenceVerdict::Deny);
        assert_eq!(policy.mode, FenceMode::Lockdown);
        assert!(policy.audit_all);
    }

    #[test]
    fn test_default_deny_blocks_unknown() {
        let policy = FencePolicy::default_deny();
        let event = make_event("suspicious", 54321, 4444);
        let decision = policy.evaluate(&event);
        assert_eq!(decision.verdict, FenceVerdict::Deny);
        assert!(decision.matched_rule_id.is_none());
        assert!(decision.enforced);
    }

    #[test]
    fn test_allow_rule_passes_through() {
        let mut policy = FencePolicy::default_deny();
        policy.add_rule(FenceRule {
            id: "allow-nginx".to_string(),
            process_match: ProcessMatch::ByName("nginx".to_string()),
            network_match: NetworkMatch::ByPort(80),
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: "Allow nginx on 80".to_string(),
        });

        let event = make_event("nginx", 80, 54321);
        let decision = policy.evaluate(&event);
        assert_eq!(decision.verdict, FenceVerdict::Allow);
        assert_eq!(decision.matched_rule_id.as_deref(), Some("allow-nginx"));
    }

    #[test]
    fn test_monitor_mode_does_not_enforce() {
        let mut policy = FencePolicy::default_deny();
        policy.mode = FenceMode::Monitor;
        let event = make_event("malware", 54321, 4444);
        let decision = policy.evaluate(&event);
        assert_eq!(decision.verdict, FenceVerdict::Deny);
        assert!(!decision.enforced); // Monitor mode doesn't enforce
    }

    #[test]
    fn test_should_audit_deny() {
        let policy = FencePolicy::default_deny();
        let decision = FenceDecision::default_verdict(FenceVerdict::Deny, true);
        assert!(policy.should_audit(&decision));
    }

    #[test]
    fn test_should_audit_allow_when_audit_all() {
        let mut policy = FencePolicy::default_deny();
        policy.audit_all = true;
        let decision = FenceDecision::default_verdict(FenceVerdict::Allow, true);
        assert!(policy.should_audit(&decision));
    }

    #[test]
    fn test_should_not_audit_allow_normally() {
        let policy = FencePolicy::default_deny();
        let decision = FenceDecision::default_verdict(FenceVerdict::Allow, true);
        assert!(!policy.should_audit(&decision));
    }

    #[test]
    fn test_decision_from_rule() {
        let rule = FenceRule {
            id: "test-rule".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 1,
            description: "test description".to_string(),
        };
        let decision = FenceDecision::from_rule(&rule, true);
        assert_eq!(decision.verdict, FenceVerdict::Allow);
        assert_eq!(decision.matched_rule_id.as_deref(), Some("test-rule"));
        assert_eq!(decision.reason, "test description");
    }

    #[test]
    fn test_decision_from_rule_no_description() {
        let rule = FenceRule {
            id: "r1".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Deny,
            priority: 1,
            description: String::new(),
        };
        let decision = FenceDecision::from_rule(&rule, false);
        assert!(decision.reason.contains("r1"));
    }

    #[test]
    fn test_policy_serialization() {
        let policy = FencePolicy::default_deny();
        let json = serde_json::to_string(&policy);
        assert!(json.is_ok());
    }
}
