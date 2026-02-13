// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Firewall — packet filtering rules for OS-level network security.
//!
//! Tier: T2-C (∂ Boundary + κ Comparison + σ Sequence)
//!
//! The firewall is a boundary guard (∂). Rules are compared (κ) in
//! sequence (σ) — first match wins. Integrates with Guardian's
//! security monitor for threat-reactive rule injection.

use crate::interface::IpAddr;
use serde::{Deserialize, Serialize};

/// Network packet disposition (Allow/Drop/Reject/Log).
///
/// Tier: T2-P (∂ — boundary decision)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PacketDisposition {
    /// Allow the packet.
    Allow,
    /// Drop the packet silently.
    #[default]
    Drop,
    /// Reject the packet (send ICMP unreachable).
    Reject,
    /// Log the packet and continue evaluating.
    Log,
}

/// Backward-compatible alias.
#[deprecated(note = "use PacketDisposition — F2 equivocation fix")]
pub type Action = PacketDisposition;

impl PacketDisposition {
    /// Human-readable label.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Drop => "DROP",
            Self::Reject => "REJECT",
            Self::Log => "LOG",
        }
    }
}

/// Network protocol to match.
///
/// Tier: T2-P (Σ Sum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// Any protocol.
    Any,
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
    /// ICMP.
    Icmp,
}

/// Traffic direction.
///
/// Tier: T2-P (σ Sequence — direction of flow)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrafficDirection {
    /// Incoming traffic.
    Inbound,
    /// Outgoing traffic.
    Outbound,
    /// Both directions.
    Both,
}

/// A port range specification.
///
/// Tier: T2-P (∂ Boundary — numeric range)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRange {
    /// Start port (inclusive).
    pub start: u16,
    /// End port (inclusive).
    pub end: u16,
}

impl PortRange {
    /// Create a single port.
    pub const fn single(port: u16) -> Self {
        Self {
            start: port,
            end: port,
        }
    }

    /// Create a port range.
    pub const fn range(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// All ports (0-65535).
    pub const fn all() -> Self {
        Self {
            start: 0,
            end: 65535,
        }
    }

    /// Whether a port falls within this range.
    pub const fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Whether this is a single port.
    pub const fn is_single(&self) -> bool {
        self.start == self.end
    }
}

/// A single firewall rule.
///
/// Tier: T2-C (∂ + κ + σ — boundary comparison in sequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Rule name (for logging/identification).
    pub name: String,
    /// Traffic direction.
    pub direction: TrafficDirection,
    /// Protocol to match.
    pub protocol: Protocol,
    /// Source address to match (None = any).
    pub source_addr: Option<IpAddr>,
    /// Destination address to match (None = any).
    pub dest_addr: Option<IpAddr>,
    /// Source port range (None = any).
    pub source_port: Option<PortRange>,
    /// Destination port range (None = any).
    pub dest_port: Option<PortRange>,
    /// Disposition when matched.
    pub action: PacketDisposition,
    /// Whether this rule is active.
    pub enabled: bool,
    /// Hit counter (how many packets matched).
    pub hit_count: u64,
}

impl FirewallRule {
    /// Create a new rule.
    pub fn new(
        name: impl Into<String>,
        direction: TrafficDirection,
        action: PacketDisposition,
    ) -> Self {
        Self {
            name: name.into(),
            direction,
            protocol: Protocol::Any,
            source_addr: None,
            dest_addr: None,
            source_port: None,
            dest_port: None,
            action,
            enabled: true,
            hit_count: 0,
        }
    }

    /// Builder: set protocol.
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Builder: set source address.
    pub fn from_addr(mut self, addr: IpAddr) -> Self {
        self.source_addr = Some(addr);
        self
    }

    /// Builder: set destination address.
    pub fn to_addr(mut self, addr: IpAddr) -> Self {
        self.dest_addr = Some(addr);
        self
    }

    /// Builder: set destination port.
    pub fn to_port(mut self, port: u16) -> Self {
        self.dest_port = Some(PortRange::single(port));
        self
    }

    /// Builder: set destination port range.
    pub fn to_port_range(mut self, start: u16, end: u16) -> Self {
        self.dest_port = Some(PortRange::range(start, end));
        self
    }

    /// Builder: set source port.
    pub fn from_port(mut self, port: u16) -> Self {
        self.source_port = Some(PortRange::single(port));
        self
    }

    /// Check if this rule matches a packet description.
    pub fn matches(
        &self,
        direction: TrafficDirection,
        protocol: Protocol,
        source: Option<&IpAddr>,
        dest: Option<&IpAddr>,
        source_port: Option<u16>,
        dest_port: Option<u16>,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        // Direction check
        if self.direction != TrafficDirection::Both && self.direction != direction {
            return false;
        }

        // Protocol check
        if self.protocol != Protocol::Any && self.protocol != protocol {
            return false;
        }

        // Source address check
        if let Some(rule_src) = &self.source_addr {
            match source {
                Some(pkt_src) if pkt_src != rule_src => return false,
                None => return false,
                _ => {}
            }
        }

        // Destination address check
        if let Some(rule_dst) = &self.dest_addr {
            match dest {
                Some(pkt_dst) if pkt_dst != rule_dst => return false,
                None => return false,
                _ => {}
            }
        }

        // Source port check
        if let Some(rule_port) = &self.source_port {
            match source_port {
                Some(p) if !rule_port.contains(p) => return false,
                None => return false,
                _ => {}
            }
        }

        // Destination port check
        if let Some(rule_port) = &self.dest_port {
            match dest_port {
                Some(p) if !rule_port.contains(p) => return false,
                None => return false,
                _ => {}
            }
        }

        true
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        let status = if self.enabled { "ON" } else { "OFF" };
        format!(
            "[{}] {} {:?} {:?} → {} (hits: {})",
            status,
            self.name,
            self.direction,
            self.protocol,
            self.action.label(),
            self.hit_count,
        )
    }
}

/// Firewall — ordered rule set with first-match semantics.
///
/// Tier: T2-C (∂ + κ + σ — boundary rules in sequential evaluation)
#[derive(Debug, Default)]
pub struct Firewall {
    /// Ordered rule set (first match wins).
    rules: Vec<FirewallRule>,
    /// Default disposition when no rule matches.
    default_action: PacketDisposition,
    /// Total packets evaluated.
    packets_evaluated: u64,
    /// Total packets blocked (dropped or rejected).
    packets_blocked: u64,
}

impl Firewall {
    /// Create a new firewall with default ALLOW policy.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_action: PacketDisposition::Allow,
            packets_evaluated: 0,
            packets_blocked: 0,
        }
    }

    /// Create with a specific default disposition.
    pub fn with_default_action(action: PacketDisposition) -> Self {
        Self {
            default_action: action,
            ..Self::new()
        }
    }

    /// Add a rule to the end of the chain.
    pub fn add_rule(&mut self, rule: FirewallRule) {
        self.rules.push(rule);
    }

    /// Insert a rule at a specific position.
    pub fn insert_rule(&mut self, index: usize, rule: FirewallRule) {
        let pos = index.min(self.rules.len());
        self.rules.insert(pos, rule);
    }

    /// Remove a rule by name.
    pub fn remove_rule(&mut self, name: &str) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.name != name);
        self.rules.len() < before
    }

    /// Evaluate a packet against the firewall rules.
    ///
    /// Returns the action to take. First matching rule wins.
    pub fn evaluate(
        &mut self,
        direction: TrafficDirection,
        protocol: Protocol,
        source: Option<&IpAddr>,
        dest: Option<&IpAddr>,
        source_port: Option<u16>,
        dest_port: Option<u16>,
    ) -> PacketDisposition {
        self.packets_evaluated += 1;

        for rule in &mut self.rules {
            if rule.matches(direction, protocol, source, dest, source_port, dest_port) {
                rule.hit_count += 1;
                let action = rule.action;
                if matches!(action, PacketDisposition::Drop | PacketDisposition::Reject) {
                    self.packets_blocked += 1;
                }
                return action;
            }
        }

        // No rule matched — use default
        if matches!(
            self.default_action,
            PacketDisposition::Drop | PacketDisposition::Reject
        ) {
            self.packets_blocked += 1;
        }
        self.default_action
    }

    /// Get all rules.
    pub fn rules(&self) -> &[FirewallRule] {
        &self.rules
    }

    /// Number of rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Total packets evaluated.
    pub fn packets_evaluated(&self) -> u64 {
        self.packets_evaluated
    }

    /// Total packets blocked.
    pub fn packets_blocked(&self) -> u64 {
        self.packets_blocked
    }

    /// Block rate as percentage.
    pub fn block_rate(&self) -> f64 {
        if self.packets_evaluated == 0 {
            return 0.0;
        }
        self.packets_blocked as f64 / self.packets_evaluated as f64 * 100.0
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!(
            "Firewall: {} rules, {} evaluated, {} blocked ({:.1}%), default={}",
            self.rules.len(),
            self.packets_evaluated,
            self.packets_blocked,
            self.block_rate(),
            self.default_action.label(),
        )
    }
}

// ── Common rule presets ──

/// Block all inbound traffic on a specific port.
pub fn block_inbound_port(name: impl Into<String>, port: u16) -> FirewallRule {
    FirewallRule::new(name, TrafficDirection::Inbound, PacketDisposition::Drop)
        .with_protocol(Protocol::Tcp)
        .to_port(port)
}

/// Allow inbound traffic on a specific port.
pub fn allow_inbound_port(name: impl Into<String>, port: u16) -> FirewallRule {
    FirewallRule::new(name, TrafficDirection::Inbound, PacketDisposition::Allow)
        .with_protocol(Protocol::Tcp)
        .to_port(port)
}

/// Block all traffic from a specific IP.
pub fn block_ip(name: impl Into<String>, addr: IpAddr) -> FirewallRule {
    FirewallRule::new(name, TrafficDirection::Both, PacketDisposition::Drop).from_addr(addr)
}

/// Allow loopback traffic.
pub fn allow_loopback() -> FirewallRule {
    FirewallRule::new(
        "allow_loopback",
        TrafficDirection::Both,
        PacketDisposition::Allow,
    )
    .from_addr(IpAddr::loopback_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_range_single() {
        let p = PortRange::single(443);
        assert!(p.contains(443));
        assert!(!p.contains(80));
        assert!(p.is_single());
    }

    #[test]
    fn port_range_range() {
        let p = PortRange::range(8000, 8999);
        assert!(p.contains(8080));
        assert!(p.contains(8000));
        assert!(p.contains(8999));
        assert!(!p.contains(7999));
        assert!(!p.contains(9000));
    }

    #[test]
    fn port_range_all() {
        let p = PortRange::all();
        assert!(p.contains(0));
        assert!(p.contains(80));
        assert!(p.contains(65535));
    }

    #[test]
    fn rule_matches_basic() {
        let rule = FirewallRule::new("test", TrafficDirection::Inbound, PacketDisposition::Drop)
            .with_protocol(Protocol::Tcp)
            .to_port(22);

        assert!(rule.matches(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(22),
        ));
    }

    #[test]
    fn rule_no_match_wrong_direction() {
        let rule = FirewallRule::new("test", TrafficDirection::Inbound, PacketDisposition::Drop)
            .to_port(22);

        assert!(!rule.matches(
            TrafficDirection::Outbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(22),
        ));
    }

    #[test]
    fn rule_both_direction_matches_all() {
        let rule =
            FirewallRule::new("test", TrafficDirection::Both, PacketDisposition::Drop).to_port(22);

        assert!(rule.matches(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(22),
        ));
        assert!(rule.matches(
            TrafficDirection::Outbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(22),
        ));
    }

    #[test]
    fn rule_disabled_never_matches() {
        let mut rule = FirewallRule::new("test", TrafficDirection::Both, PacketDisposition::Drop);
        rule.enabled = false;

        assert!(!rule.matches(
            TrafficDirection::Inbound,
            Protocol::Any,
            None,
            None,
            None,
            None,
        ));
    }

    #[test]
    fn rule_matches_source_addr() {
        let attacker = IpAddr::v4(10, 0, 0, 99);
        let rule = block_ip("block_attacker", attacker.clone());

        assert!(rule.matches(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            Some(&attacker),
            None,
            None,
            None,
        ));
        assert!(!rule.matches(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            Some(&IpAddr::v4(10, 0, 0, 1)),
            None,
            None,
            None,
        ));
    }

    #[test]
    fn firewall_first_match_wins() {
        let mut fw = Firewall::new();
        fw.add_rule(allow_inbound_port("allow_https", 443));
        fw.add_rule(FirewallRule::new(
            "block_all_inbound",
            TrafficDirection::Inbound,
            PacketDisposition::Drop,
        ));

        // Port 443 should be allowed (first rule matches)
        let action = fw.evaluate(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(443),
        );
        assert_eq!(action, PacketDisposition::Allow);

        // Port 80 should be dropped (second rule matches)
        let action = fw.evaluate(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(80),
        );
        assert_eq!(action, PacketDisposition::Drop);
    }

    #[test]
    fn firewall_default_action() {
        let mut fw = Firewall::with_default_action(PacketDisposition::Drop);
        // No rules, should use default
        let action = fw.evaluate(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(80),
        );
        assert_eq!(action, PacketDisposition::Drop);
    }

    #[test]
    fn firewall_hit_counter() {
        let mut fw = Firewall::new();
        fw.add_rule(block_inbound_port("block_ssh", 22));

        for _ in 0..5 {
            fw.evaluate(
                TrafficDirection::Inbound,
                Protocol::Tcp,
                None,
                None,
                None,
                Some(22),
            );
        }

        assert_eq!(fw.rules()[0].hit_count, 5);
        assert_eq!(fw.packets_evaluated(), 5);
        assert_eq!(fw.packets_blocked(), 5);
    }

    #[test]
    fn firewall_remove_rule() {
        let mut fw = Firewall::new();
        fw.add_rule(block_inbound_port("block_ssh", 22));
        fw.add_rule(allow_inbound_port("allow_http", 80));
        assert_eq!(fw.rule_count(), 2);

        assert!(fw.remove_rule("block_ssh"));
        assert_eq!(fw.rule_count(), 1);
        assert!(!fw.remove_rule("nonexistent"));
    }

    #[test]
    fn firewall_insert_rule() {
        let mut fw = Firewall::new();
        fw.add_rule(block_inbound_port("rule_b", 80));
        fw.insert_rule(0, allow_loopback());
        assert_eq!(fw.rules()[0].name, "allow_loopback");
    }

    #[test]
    fn firewall_block_rate() {
        let mut fw = Firewall::new();
        fw.add_rule(block_inbound_port("block_ssh", 22));

        // 1 blocked
        fw.evaluate(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(22),
        );
        // 1 allowed (no rule matches, default=allow)
        fw.evaluate(
            TrafficDirection::Inbound,
            Protocol::Tcp,
            None,
            None,
            None,
            Some(80),
        );

        let rate = fw.block_rate();
        assert!((rate - 50.0).abs() < 0.1);
    }

    #[test]
    fn firewall_summary() {
        let fw = Firewall::new();
        let s = fw.summary();
        assert!(s.contains("Firewall"));
        assert!(s.contains("0 rules"));
    }

    #[test]
    fn preset_block_inbound() {
        let rule = block_inbound_port("block_ssh", 22);
        assert_eq!(rule.action, PacketDisposition::Drop);
        assert_eq!(rule.direction, TrafficDirection::Inbound);
    }

    #[test]
    fn preset_allow_loopback() {
        let rule = allow_loopback();
        assert_eq!(rule.action, PacketDisposition::Allow);
        assert_eq!(rule.source_addr, Some(IpAddr::loopback_v4()));
    }
}
