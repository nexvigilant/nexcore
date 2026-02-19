//! Fence engine — the main scan→evaluate→enforce→audit loop.
//!
//! Tier: T3 (σ Sequence + ∂ Boundary + ς State + κ Comparison + π Persistence)
//!
//! The `FenceEngine` orchestrates the full signal detection pipeline:
//! 1. **Scan** — observe all connections (/proc/net/*)
//! 2. **Evaluate** — classify each connection against the policy (∂ boundary)
//! 3. **Enforce** — apply verdicts (allow/deny/alert)
//! 4. **Audit** — record decisions for forensic analysis

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::audit::AuditLog;
use crate::connection;
use crate::enforcer::{Enforcer, LogOnlyEnforcer};
use crate::error::FenceResult;
use crate::policy::{FenceDecision, FencePolicy};
use crate::process::{self, ConnectionEvent};
use crate::rule::FenceVerdict;

/// Statistics tracking — maps to signal theory's DecisionMatrix.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FenceStats {
    /// Total connections observed.
    pub total_observed: u64,
    /// Connections allowed.
    pub allowed: u64,
    /// Connections denied.
    pub denied: u64,
    /// Connections that triggered alerts.
    pub alerted: u64,
    /// Connections matched by a rule.
    pub rule_matched: u64,
    /// Connections where default verdict applied.
    pub default_applied: u64,
    /// Number of tick cycles completed.
    pub ticks: u64,
}

impl FenceStats {
    /// Record a decision in the stats.
    pub fn record(&mut self, decision: &FenceDecision) {
        self.total_observed += 1;
        match decision.verdict {
            FenceVerdict::Allow => self.allowed += 1,
            FenceVerdict::Deny => self.denied += 1,
            FenceVerdict::Alert => self.alerted += 1,
        }
        if decision.matched_rule_id.is_some() {
            self.rule_matched += 1;
        } else {
            self.default_applied += 1;
        }
    }

    /// Denial rate as a fraction [0.0, 1.0].
    pub fn denial_rate(&self) -> f64 {
        if self.total_observed == 0 {
            return 0.0;
        }
        self.denied as f64 / self.total_observed as f64
    }

    /// Rule match rate as a fraction [0.0, 1.0].
    pub fn rule_match_rate(&self) -> f64 {
        if self.total_observed == 0 {
            return 0.0;
        }
        self.rule_matched as f64 / self.total_observed as f64
    }
}

/// Result of a single engine tick (scan→evaluate→enforce→audit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenceTickResult {
    /// Number of connections observed in this tick.
    pub connections_observed: usize,
    /// Number of connections allowed.
    pub allowed: usize,
    /// Number of connections denied.
    pub denied: usize,
    /// Number of alerts generated.
    pub alerted: usize,
    /// Individual decisions (for detailed reporting).
    pub decisions: Vec<TickDecision>,
}

/// A single decision from a tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickDecision {
    /// Process name.
    pub process_name: String,
    /// Remote endpoint.
    pub remote: String,
    /// Verdict.
    pub verdict: FenceVerdict,
    /// Rule that matched.
    pub matched_rule: Option<String>,
}

/// A status report for the fence engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenceReport {
    /// Current stats.
    pub stats: FenceStats,
    /// Current mode.
    pub mode: String,
    /// Default verdict.
    pub default_verdict: String,
    /// Number of rules.
    pub rule_count: usize,
    /// Enforcer name.
    pub enforcer: String,
    /// Audit log size.
    pub audit_log_size: usize,
    /// Total audit entries ever recorded.
    pub total_audit_recorded: u64,
}

/// The fence engine — orchestrates scan→evaluate→enforce→audit.
pub struct FenceEngine {
    /// The policy defining what's allowed and what's denied.
    pub policy: FencePolicy,
    /// The enforcement backend.
    enforcer: Arc<dyn Enforcer>,
    /// Audit log of decisions.
    pub audit: AuditLog,
    /// Running statistics.
    pub stats: FenceStats,
}

impl FenceEngine {
    /// Create a new fence engine with the given policy and enforcer.
    pub fn new(policy: FencePolicy, enforcer: Arc<dyn Enforcer>) -> Self {
        Self {
            policy,
            enforcer,
            audit: AuditLog::default(),
            stats: FenceStats::default(),
        }
    }

    /// Create a new fence engine with default-deny policy and log-only enforcer.
    pub fn default_deny() -> Self {
        Self::new(FencePolicy::default_deny(), Arc::new(LogOnlyEnforcer))
    }

    /// Create a monitor-only engine (default-allow, log-only).
    pub fn monitor() -> Self {
        Self::new(FencePolicy::default_allow(), Arc::new(LogOnlyEnforcer))
    }

    /// Scan current connections from /proc.
    pub fn scan(&self) -> Vec<ConnectionEvent> {
        let sockets = connection::scan_all();
        process::resolve_all(&sockets)
    }

    /// Evaluate a single connection event against the policy.
    pub fn evaluate(&self, event: &ConnectionEvent) -> FenceDecision {
        self.policy.evaluate(event)
    }

    /// Evaluate a list of pre-built connection events (for testing).
    pub fn evaluate_events(&mut self, events: &[ConnectionEvent]) -> FenceTickResult {
        self.process_events(events)
    }

    /// Run one tick: scan → evaluate → enforce → audit.
    ///
    /// This is the main loop body. Call repeatedly on a timer.
    pub fn tick(&mut self) -> FenceResult<FenceTickResult> {
        self.stats.ticks += 1;
        let events = self.scan();
        Ok(self.process_events(&events))
    }

    /// Process a batch of events through evaluate → enforce → audit.
    fn process_events(&mut self, events: &[ConnectionEvent]) -> FenceTickResult {
        let mut allowed = 0usize;
        let mut denied = 0usize;
        let mut alerted = 0usize;
        let mut decisions = Vec::with_capacity(events.len());

        for event in events {
            let decision = self.policy.evaluate(event);

            // Enforce
            if self.policy.mode.enforces() {
                let _ = self.enforcer.enforce(event, decision.verdict);
            }

            // Audit
            if self.policy.should_audit(&decision) {
                self.audit.record_event(event, &decision);
            }

            // Stats
            self.stats.record(&decision);

            match decision.verdict {
                FenceVerdict::Allow => allowed += 1,
                FenceVerdict::Deny => denied += 1,
                FenceVerdict::Alert => alerted += 1,
            }

            decisions.push(TickDecision {
                process_name: event.process_name().to_string(),
                remote: format!("{}:{}", event.socket.remote_addr, event.socket.remote_port),
                verdict: decision.verdict,
                matched_rule: decision.matched_rule_id,
            });
        }

        FenceTickResult {
            connections_observed: events.len(),
            allowed,
            denied,
            alerted,
            decisions,
        }
    }

    /// Generate a status report.
    pub fn report(&self) -> FenceReport {
        FenceReport {
            stats: self.stats.clone(),
            mode: format!("{:?}", self.policy.mode),
            default_verdict: format!("{:?}", self.policy.default_verdict),
            rule_count: self.policy.rule_count(),
            enforcer: self.enforcer.name().to_string(),
            audit_log_size: self.audit.len(),
            total_audit_recorded: self.audit.total_recorded,
        }
    }

    /// Get the enforcer name.
    pub fn enforcer_name(&self) -> &str {
        self.enforcer.name()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;
    use crate::connection::{Protocol, SocketEntry, TcpState};
    use crate::enforcer::MockEnforcer;
    use crate::policy::FenceMode;
    use crate::process::{Direction, ProcessInfo};
    use crate::rule::{FenceRule, NetworkMatch, ProcessMatch};

    fn make_event(name: &str, remote_port: u16) -> ConnectionEvent {
        let socket = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port: 54321,
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
    fn test_fence_stats_record() {
        let mut stats = FenceStats::default();
        let decision = FenceDecision::default_verdict(FenceVerdict::Deny, true);
        stats.record(&decision);
        assert_eq!(stats.total_observed, 1);
        assert_eq!(stats.denied, 1);
        assert_eq!(stats.default_applied, 1);
    }

    #[test]
    fn test_fence_stats_denial_rate() {
        let mut stats = FenceStats::default();
        stats.total_observed = 10;
        stats.denied = 3;
        let rate = stats.denial_rate();
        assert!((rate - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fence_stats_denial_rate_zero() {
        let stats = FenceStats::default();
        assert!((stats.denial_rate()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fence_stats_rule_match_rate() {
        let mut stats = FenceStats::default();
        stats.total_observed = 10;
        stats.rule_matched = 7;
        let rate = stats.rule_match_rate();
        assert!((rate - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_engine_default_deny() {
        let engine = FenceEngine::default_deny();
        assert_eq!(engine.policy.default_verdict, FenceVerdict::Deny);
        assert_eq!(engine.enforcer_name(), "log-only");
    }

    #[test]
    fn test_engine_monitor() {
        let engine = FenceEngine::monitor();
        assert_eq!(engine.policy.default_verdict, FenceVerdict::Allow);
        assert_eq!(engine.policy.mode, FenceMode::Monitor);
    }

    #[test]
    fn test_engine_evaluate_deny_by_default() {
        let engine = FenceEngine::default_deny();
        let event = make_event("suspicious", 4444);
        let decision = engine.evaluate(&event);
        assert_eq!(decision.verdict, FenceVerdict::Deny);
    }

    #[test]
    fn test_engine_evaluate_with_allow_rule() {
        let mut policy = FencePolicy::default_deny();
        policy.add_rule(FenceRule {
            id: "allow-curl-443".to_string(),
            process_match: ProcessMatch::ByName("curl".to_string()),
            network_match: NetworkMatch::ByPort(443),
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: "Allow curl to HTTPS".to_string(),
        });
        let engine = FenceEngine::new(policy, Arc::new(LogOnlyEnforcer));

        let event = make_event("curl", 443);
        let decision = engine.evaluate(&event);
        assert_eq!(decision.verdict, FenceVerdict::Allow);

        let event2 = make_event("curl", 80);
        let decision2 = engine.evaluate(&event2);
        assert_eq!(decision2.verdict, FenceVerdict::Deny); // port 80 not allowed
    }

    #[test]
    fn test_engine_evaluate_events() {
        let mock = Arc::new(MockEnforcer::new());
        let mut engine = FenceEngine::new(FencePolicy::default_deny(), mock.clone());
        engine.policy.add_rule(FenceRule {
            id: "allow-nginx".to_string(),
            process_match: ProcessMatch::ByName("nginx".to_string()),
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: String::new(),
        });

        let events = vec![
            make_event("nginx", 443),
            make_event("malware", 4444),
            make_event("nginx", 80),
        ];

        let result = engine.evaluate_events(&events);
        assert_eq!(result.connections_observed, 3);
        assert_eq!(result.allowed, 2);
        assert_eq!(result.denied, 1);

        // MockEnforcer should have recorded 3 operations
        assert_eq!(mock.ops().len(), 3);
        assert_eq!(mock.count_verdict(FenceVerdict::Allow), 2);
        assert_eq!(mock.count_verdict(FenceVerdict::Deny), 1);
    }

    #[test]
    fn test_engine_audit_trail() {
        let mock = Arc::new(MockEnforcer::new());
        let mut policy = FencePolicy::default_deny();
        policy.audit_all = true;
        let mut engine = FenceEngine::new(policy, mock);

        let events = vec![make_event("test", 80)];
        let _ = engine.evaluate_events(&events);

        assert_eq!(engine.audit.len(), 1);
    }

    #[test]
    fn test_engine_stats_accumulate() {
        let mock = Arc::new(MockEnforcer::new());
        let mut engine = FenceEngine::new(FencePolicy::default_deny(), mock);

        let events = vec![make_event("a", 80), make_event("b", 443)];
        let _ = engine.evaluate_events(&events);

        assert_eq!(engine.stats.total_observed, 2);
        assert_eq!(engine.stats.denied, 2);
        assert_eq!(engine.stats.default_applied, 2);
    }

    #[test]
    fn test_engine_report() {
        let mock = Arc::new(MockEnforcer::new());
        let engine = FenceEngine::new(FencePolicy::default_deny(), mock);
        let report = engine.report();
        assert_eq!(report.mode, "Enforce");
        assert_eq!(report.default_verdict, "Deny");
        assert_eq!(report.enforcer, "mock");
        assert_eq!(report.rule_count, 0);
    }

    #[test]
    fn test_engine_report_serialization() {
        let engine = FenceEngine::default_deny();
        let report = engine.report();
        let json = serde_json::to_string(&report);
        assert!(json.is_ok());
    }

    #[test]
    fn test_tick_decision_serialization() {
        let td = TickDecision {
            process_name: "test".to_string(),
            remote: "10.0.0.1:443".to_string(),
            verdict: FenceVerdict::Allow,
            matched_rule: Some("r1".to_string()),
        };
        let json = serde_json::to_string(&td);
        assert!(json.is_ok());
    }

    #[test]
    fn test_monitor_mode_no_enforcement() {
        let mock = Arc::new(MockEnforcer::new());
        let mut engine = FenceEngine::new(FencePolicy::default_allow(), mock.clone());

        let events = vec![make_event("test", 80)];
        let _ = engine.evaluate_events(&events);

        // Monitor mode: enforcer NOT called
        assert_eq!(mock.ops().len(), 0);
    }
}
