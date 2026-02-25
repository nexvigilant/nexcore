//! Decision audit trail — records who, what, when, why for every fence decision.
//!
//! Tier: T2-C (π Persistence + σ Sequence + → Causality — persisted causal sequence)
//!
//! In signal theory terms, the audit log is the "experiment record" —
//! every observation, its classification, and the rationale.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::policy::FenceDecision;
use crate::process::ConnectionEvent;
use crate::rule::FenceVerdict;

/// A single audit entry recording a fence decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When the decision was made.
    pub timestamp: DateTime,
    /// Process name (or "<unknown>").
    pub process_name: String,
    /// Process PID (0 if unknown).
    pub pid: u32,
    /// Local address:port as string.
    pub local: String,
    /// Remote address:port as string.
    pub remote: String,
    /// Protocol.
    pub protocol: String,
    /// Verdict.
    pub verdict: FenceVerdict,
    /// Rule that matched (if any).
    pub matched_rule: Option<String>,
    /// Human-readable reason.
    pub reason: String,
    /// Whether enforcement was applied.
    pub enforced: bool,
}

impl AuditEntry {
    /// Create an audit entry from a connection event and its decision.
    pub fn from_event(event: &ConnectionEvent, decision: &FenceDecision) -> Self {
        let (proc_name, pid) = event
            .process
            .as_ref()
            .map(|p| (p.name.clone(), p.pid))
            .unwrap_or_else(|| ("<unknown>".to_string(), 0));

        Self {
            timestamp: DateTime::now(),
            process_name: proc_name,
            pid,
            local: format!("{}:{}", event.socket.local_addr, event.socket.local_port),
            remote: format!("{}:{}", event.socket.remote_addr, event.socket.remote_port),
            protocol: format!("{:?}", event.socket.protocol),
            verdict: decision.verdict,
            matched_rule: decision.matched_rule_id.clone(),
            reason: decision.reason.clone(),
            enforced: decision.enforced,
        }
    }

    /// One-line summary for log output.
    pub fn summary(&self) -> String {
        format!(
            "[{:?}] {} (pid:{}) {} → {} [{}] rule={} enforced={}",
            self.verdict,
            self.process_name,
            self.pid,
            self.local,
            self.remote,
            self.protocol,
            self.matched_rule.as_deref().unwrap_or("default"),
            self.enforced,
        )
    }
}

/// Audit log — bounded circular buffer of fence decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    max_entries: usize,
    /// Total entries ever recorded (including evicted).
    pub total_recorded: u64,
}

impl AuditLog {
    /// Create an audit log with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries.min(10_000)),
            max_entries,
            total_recorded: 0,
        }
    }

    /// Record an audit entry.
    pub fn record(&mut self, entry: AuditEntry) {
        self.total_recorded += 1;
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    /// Record from a connection event and decision.
    pub fn record_event(&mut self, event: &ConnectionEvent, decision: &FenceDecision) {
        self.record(AuditEntry::from_event(event, decision));
    }

    /// Get all entries.
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Number of entries currently held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the most recent N entries.
    pub fn recent(&self, n: usize) -> &[AuditEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// Count entries by verdict.
    pub fn count_by_verdict(&self, verdict: FenceVerdict) -> usize {
        self.entries.iter().filter(|e| e.verdict == verdict).count()
    }

    /// Get all denied entries.
    pub fn denied(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.verdict == FenceVerdict::Deny)
            .collect()
    }

    /// Clear the log.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;
    use crate::connection::{Protocol, SocketEntry, TcpState};
    use crate::policy::FenceDecision;
    use crate::process::{ConnectionEvent, ProcessInfo};

    fn make_event_and_decision(
        name: &str,
        verdict: FenceVerdict,
    ) -> (ConnectionEvent, FenceDecision) {
        let socket = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port: 80,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port: 54321,
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
        let event = ConnectionEvent::new(socket, Some(proc_info));
        let decision = FenceDecision::default_verdict(verdict, true);
        (event, decision)
    }

    #[test]
    fn test_audit_entry_from_event() {
        let (event, decision) = make_event_and_decision("nginx", FenceVerdict::Allow);
        let entry = AuditEntry::from_event(&event, &decision);
        assert_eq!(entry.process_name, "nginx");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.verdict, FenceVerdict::Allow);
        assert!(entry.local.contains("80"));
    }

    #[test]
    fn test_audit_entry_summary() {
        let (event, decision) = make_event_and_decision("curl", FenceVerdict::Deny);
        let entry = AuditEntry::from_event(&event, &decision);
        let summary = entry.summary();
        assert!(summary.contains("Deny"));
        assert!(summary.contains("curl"));
        assert!(summary.contains("1234"));
    }

    #[test]
    fn test_audit_log_record() {
        let mut log = AuditLog::new(100);
        let (event, decision) = make_event_and_decision("test", FenceVerdict::Deny);
        log.record_event(&event, &decision);
        assert_eq!(log.len(), 1);
        assert_eq!(log.total_recorded, 1);
    }

    #[test]
    fn test_audit_log_capacity_eviction() {
        let mut log = AuditLog::new(3);
        for i in 0..5 {
            let (event, decision) =
                make_event_and_decision(&format!("proc{i}"), FenceVerdict::Deny);
            log.record_event(&event, &decision);
        }
        assert_eq!(log.len(), 3);
        assert_eq!(log.total_recorded, 5);
        // Oldest entries evicted
        assert_eq!(log.entries()[0].process_name, "proc2");
    }

    #[test]
    fn test_audit_log_recent() {
        let mut log = AuditLog::new(100);
        for i in 0..10 {
            let (event, decision) =
                make_event_and_decision(&format!("proc{i}"), FenceVerdict::Allow);
            log.record_event(&event, &decision);
        }
        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].process_name, "proc7");
    }

    #[test]
    fn test_audit_log_count_by_verdict() {
        let mut log = AuditLog::new(100);
        let (e1, d1) = make_event_and_decision("allow1", FenceVerdict::Allow);
        let (e2, d2) = make_event_and_decision("deny1", FenceVerdict::Deny);
        let (e3, d3) = make_event_and_decision("deny2", FenceVerdict::Deny);
        log.record_event(&e1, &d1);
        log.record_event(&e2, &d2);
        log.record_event(&e3, &d3);
        assert_eq!(log.count_by_verdict(FenceVerdict::Allow), 1);
        assert_eq!(log.count_by_verdict(FenceVerdict::Deny), 2);
    }

    #[test]
    fn test_audit_log_denied() {
        let mut log = AuditLog::new(100);
        let (e1, d1) = make_event_and_decision("ok", FenceVerdict::Allow);
        let (e2, d2) = make_event_and_decision("bad", FenceVerdict::Deny);
        log.record_event(&e1, &d1);
        log.record_event(&e2, &d2);
        let denied = log.denied();
        assert_eq!(denied.len(), 1);
        assert_eq!(denied[0].process_name, "bad");
    }

    #[test]
    fn test_audit_log_clear() {
        let mut log = AuditLog::new(100);
        let (event, decision) = make_event_and_decision("test", FenceVerdict::Deny);
        log.record_event(&event, &decision);
        assert!(!log.is_empty());
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.total_recorded, 1); // total not reset
    }

    #[test]
    fn test_audit_log_default() {
        let log = AuditLog::default();
        assert!(log.is_empty());
        assert_eq!(log.total_recorded, 0);
    }

    #[test]
    fn test_audit_entry_unknown_process() {
        let socket = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port: 80,
            remote_addr: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            remote_port: 54321,
            inode: 100,
            state: TcpState::Established,
            protocol: Protocol::Tcp,
            uid: 0,
        };
        let event = ConnectionEvent::new(socket, None);
        let decision = FenceDecision::default_verdict(FenceVerdict::Deny, true);
        let entry = AuditEntry::from_event(&event, &decision);
        assert_eq!(entry.process_name, "<unknown>");
        assert_eq!(entry.pid, 0);
    }
}
