//! Rule engine — signal-theory boundaries for network connections.
//!
//! Tier: T2-P (∂ Boundary + κ Comparison — rules define boundaries, matching compares)
//!
//! Each `FenceRule` is a `FixedBoundary` in signal theory terms: a deterministic
//! criterion that classifies a connection as signal (allowed) or noise (denied).

use std::net::IpAddr;
use std::path::PathBuf;

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use crate::connection::Protocol;
use crate::error::{FenceError, FenceResult};
use crate::process::{ConnectionEvent, Direction};

/// How a fence rule matches processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessMatch {
    /// Match by exact process name (/proc/<pid>/comm).
    ByName(String),
    /// Match by executable path (/proc/<pid>/exe).
    ByExe(PathBuf),
    /// Match by user ID.
    ByUid(u32),
    /// Match any process.
    Any,
}

impl ProcessMatch {
    /// Test whether this matcher accepts the given connection event.
    pub fn matches(&self, event: &ConnectionEvent) -> bool {
        match self {
            Self::Any => true,
            Self::ByUid(uid) => event.socket.uid == *uid,
            Self::ByName(name) => event.process.as_ref().map_or(false, |p| p.name == *name),
            Self::ByExe(exe) => event.process.as_ref().map_or(false, |p| p.exe == *exe),
        }
    }
}

/// How a fence rule matches network addresses/ports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMatch {
    /// Match a specific IP address.
    ByAddr(IpAddr),
    /// Match a CIDR network range.
    ByCidr(IpNetwork),
    /// Match a specific port.
    ByPort(u16),
    /// Match a port range (inclusive).
    ByPortRange(u16, u16),
    /// Match a specific protocol.
    ByProtocol(Protocol),
    /// Match any network parameters.
    Any,
}

impl NetworkMatch {
    /// Test whether this matcher accepts the given connection event.
    ///
    /// For address-based matches, checks both local and remote addresses.
    /// For port-based matches, checks both local and remote ports.
    pub fn matches(&self, event: &ConnectionEvent) -> bool {
        match self {
            Self::Any => true,
            Self::ByAddr(addr) => {
                event.socket.remote_addr == *addr || event.socket.local_addr == *addr
            }
            Self::ByCidr(network) => {
                network.contains(event.socket.remote_addr)
                    || network.contains(event.socket.local_addr)
            }
            Self::ByPort(port) => {
                event.socket.remote_port == *port || event.socket.local_port == *port
            }
            Self::ByPortRange(lo, hi) => {
                let rp = event.socket.remote_port;
                let lp = event.socket.local_port;
                (rp >= *lo && rp <= *hi) || (lp >= *lo && lp <= *hi)
            }
            Self::ByProtocol(proto) => event.socket.protocol == *proto,
        }
    }
}

/// The verdict a rule produces when matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FenceVerdict {
    /// Allow the connection through.
    Allow,
    /// Block the connection.
    Deny,
    /// Log for review but take no action.
    Alert,
}

impl FenceVerdict {
    /// Whether this verdict permits the connection.
    pub fn is_permissive(&self) -> bool {
        matches!(self, Self::Allow | Self::Alert)
    }

    /// Whether this verdict blocks the connection.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Deny)
    }
}

/// A single fence rule — a signal-theory boundary criterion.
///
/// Rules are evaluated in priority order (lower number = higher priority).
/// The first matching rule determines the verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FenceRule {
    /// Unique rule identifier.
    pub id: String,
    /// Process matching criterion.
    pub process_match: ProcessMatch,
    /// Network matching criterion.
    pub network_match: NetworkMatch,
    /// Direction criterion.
    pub direction: Direction,
    /// Verdict when this rule matches.
    pub verdict: FenceVerdict,
    /// Priority (lower = higher priority, evaluated first).
    pub priority: u32,
    /// Human-readable description.
    pub description: String,
}

impl FenceRule {
    /// Test whether this rule matches the given connection event.
    ///
    /// All criteria must match (AND logic — CompositeBoundary).
    pub fn matches(&self, event: &ConnectionEvent) -> bool {
        let dir_match = match self.direction {
            Direction::Both => true,
            dir => event.direction == dir,
        };
        dir_match && self.process_match.matches(event) && self.network_match.matches(event)
    }

    /// Validate this rule for internal consistency.
    pub fn validate(&self) -> FenceResult<()> {
        if self.id.is_empty() {
            return Err(FenceError::InvalidRule {
                rule_id: "(empty)".to_string(),
                reason: "rule ID must not be empty".to_string(),
            });
        }
        if let NetworkMatch::ByPortRange(lo, hi) = &self.network_match {
            if lo > hi {
                return Err(FenceError::InvalidRule {
                    rule_id: self.id.clone(),
                    reason: format!("port range {lo}-{hi} is inverted"),
                });
            }
        }
        Ok(())
    }
}

/// An ordered set of fence rules.
///
/// Rules are sorted by priority and evaluated sequentially.
/// In signal theory: this is a `CompositeBoundary` with ordered evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSet {
    rules: Vec<FenceRule>,
}

impl RuleSet {
    /// Create an empty rule set.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule and re-sort by priority.
    pub fn add(&mut self, rule: FenceRule) {
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
    }

    /// Evaluate the rule set against a connection event.
    ///
    /// Returns the first matching rule, or None if no rule matches.
    pub fn evaluate(&self, event: &ConnectionEvent) -> Option<&FenceRule> {
        self.rules.iter().find(|r| r.matches(event))
    }

    /// Number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the rule set is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Iterate over rules in priority order.
    pub fn iter(&self) -> impl Iterator<Item = &FenceRule> {
        self.rules.iter()
    }

    /// Validate all rules.
    pub fn validate(&self) -> FenceResult<()> {
        for rule in &self.rules {
            rule.validate()?;
        }
        Ok(())
    }

    /// Builder method: add a rule for a process+port combination.
    pub fn allow_process_port(
        &mut self,
        id: impl Into<String>,
        process_name: impl Into<String>,
        port: u16,
        direction: Direction,
        priority: u32,
    ) {
        self.add(FenceRule {
            id: id.into(),
            process_match: ProcessMatch::ByName(process_name.into()),
            network_match: NetworkMatch::ByPort(port),
            direction,
            verdict: FenceVerdict::Allow,
            priority,
            description: String::new(),
        });
    }

    /// Builder method: add a rule allowing a specific UID.
    pub fn allow_uid(&mut self, id: impl Into<String>, uid: u32, priority: u32) {
        self.add(FenceRule {
            id: id.into(),
            process_match: ProcessMatch::ByUid(uid),
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority,
            description: String::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use crate::connection::{SocketEntry, TcpState};
    use crate::process::ProcessInfo;

    fn make_event(
        process_name: &str,
        local_port: u16,
        remote_port: u16,
        remote_addr: IpAddr,
    ) -> ConnectionEvent {
        let socket = SocketEntry {
            local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
            local_port,
            remote_addr,
            remote_port,
            inode: 100,
            state: TcpState::Established,
            protocol: Protocol::Tcp,
            uid: 1000,
        };
        let proc = ProcessInfo {
            pid: 1234,
            name: process_name.to_string(),
            exe: PathBuf::from(format!("/usr/bin/{process_name}")),
            uid: 1000,
            cmdline: process_name.to_string(),
        };
        ConnectionEvent::new(socket, Some(proc))
    }

    #[test]
    fn test_process_match_any() {
        let event = make_event("nginx", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(ProcessMatch::Any.matches(&event));
    }

    #[test]
    fn test_process_match_by_name() {
        let event = make_event("nginx", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(ProcessMatch::ByName("nginx".to_string()).matches(&event));
        assert!(!ProcessMatch::ByName("apache".to_string()).matches(&event));
    }

    #[test]
    fn test_process_match_by_exe() {
        let event = make_event("nginx", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(ProcessMatch::ByExe(PathBuf::from("/usr/bin/nginx")).matches(&event));
        assert!(!ProcessMatch::ByExe(PathBuf::from("/usr/sbin/nginx")).matches(&event));
    }

    #[test]
    fn test_process_match_by_uid() {
        let event = make_event("nginx", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(ProcessMatch::ByUid(1000).matches(&event));
        assert!(!ProcessMatch::ByUid(33).matches(&event));
    }

    #[test]
    fn test_process_match_no_process() {
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
        assert!(ProcessMatch::Any.matches(&event));
        assert!(!ProcessMatch::ByName("nginx".to_string()).matches(&event));
    }

    #[test]
    fn test_network_match_any() {
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(NetworkMatch::Any.matches(&event));
    }

    #[test]
    fn test_network_match_by_addr() {
        let remote = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let event = make_event("curl", 54321, 443, remote);
        assert!(NetworkMatch::ByAddr(remote).matches(&event));
        assert!(!NetworkMatch::ByAddr(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))).matches(&event));
    }

    #[test]
    fn test_network_match_by_cidr() {
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 50)));
        let cidr: IpNetwork = "10.0.0.0/24".parse().ok().unwrap_or_else(|| {
            "127.0.0.0/8".parse().ok().unwrap_or_else(|| {
                IpNetwork::V4(
                    "0.0.0.0/0"
                        .parse()
                        .ok()
                        .unwrap_or_else(|| panic!("test infra: CIDR parse")),
                )
            })
        });
        assert!(NetworkMatch::ByCidr(cidr).matches(&event));
    }

    #[test]
    fn test_network_match_by_port() {
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(NetworkMatch::ByPort(443).matches(&event));
        assert!(!NetworkMatch::ByPort(80).matches(&event));
    }

    #[test]
    fn test_network_match_by_port_range() {
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(NetworkMatch::ByPortRange(400, 500).matches(&event));
        assert!(!NetworkMatch::ByPortRange(80, 100).matches(&event));
    }

    #[test]
    fn test_network_match_by_protocol() {
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(NetworkMatch::ByProtocol(Protocol::Tcp).matches(&event));
        assert!(!NetworkMatch::ByProtocol(Protocol::Udp).matches(&event));
    }

    #[test]
    fn test_fence_verdict_permissive() {
        assert!(FenceVerdict::Allow.is_permissive());
        assert!(FenceVerdict::Alert.is_permissive());
        assert!(!FenceVerdict::Deny.is_permissive());
    }

    #[test]
    fn test_fence_verdict_blocking() {
        assert!(FenceVerdict::Deny.is_blocking());
        assert!(!FenceVerdict::Allow.is_blocking());
        assert!(!FenceVerdict::Alert.is_blocking());
    }

    #[test]
    fn test_fence_rule_matches_all_criteria() {
        let rule = FenceRule {
            id: "r1".to_string(),
            process_match: ProcessMatch::ByName("nginx".to_string()),
            network_match: NetworkMatch::ByPort(80),
            direction: Direction::Ingress,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: "Allow nginx on port 80".to_string(),
        };
        let event = make_event("nginx", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(rule.matches(&event));
    }

    #[test]
    fn test_fence_rule_fails_on_process_mismatch() {
        let rule = FenceRule {
            id: "r1".to_string(),
            process_match: ProcessMatch::ByName("nginx".to_string()),
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: String::new(),
        };
        let event = make_event("apache", 80, 54321, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!rule.matches(&event));
    }

    #[test]
    fn test_fence_rule_direction_both_matches_any() {
        let rule = FenceRule {
            id: "r1".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: String::new(),
        };
        let event = make_event("curl", 54321, 443, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert!(rule.matches(&event));
    }

    #[test]
    fn test_fence_rule_validate_empty_id() {
        let rule = FenceRule {
            id: String::new(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: String::new(),
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_fence_rule_validate_inverted_port_range() {
        let rule = FenceRule {
            id: "bad".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::ByPortRange(500, 100),
            direction: Direction::Both,
            verdict: FenceVerdict::Deny,
            priority: 10,
            description: String::new(),
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_rule_set_priority_order() {
        let mut rules = RuleSet::new();
        rules.add(FenceRule {
            id: "low".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Deny,
            priority: 100,
            description: String::new(),
        });
        rules.add(FenceRule {
            id: "high".to_string(),
            process_match: ProcessMatch::Any,
            network_match: NetworkMatch::Any,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 1,
            description: String::new(),
        });
        let event = make_event("test", 80, 443, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        let matched = rules.evaluate(&event);
        assert!(matched.is_some());
        let matched = matched.unwrap_or_else(|| &rules.rules[0]);
        assert_eq!(matched.id, "high");
        assert_eq!(matched.verdict, FenceVerdict::Allow);
    }

    #[test]
    fn test_rule_set_empty() {
        let rules = RuleSet::new();
        assert!(rules.is_empty());
        assert_eq!(rules.len(), 0);
        let event = make_event("test", 80, 443, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(rules.evaluate(&event).is_none());
    }

    #[test]
    fn test_rule_set_builder_allow_process_port() {
        let mut rules = RuleSet::new();
        rules.allow_process_port("ssh", "sshd", 22, Direction::Ingress, 10);
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_rule_set_builder_allow_uid() {
        let mut rules = RuleSet::new();
        rules.allow_uid("root", 0, 5);
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_rule_set_validate() {
        let mut rules = RuleSet::new();
        rules.allow_process_port("ok", "nginx", 80, Direction::Both, 10);
        assert!(rules.validate().is_ok());
    }
}
