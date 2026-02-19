// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! OS-level security monitor inspired by Guardian homeostasis.
//!
//! ## Architecture
//!
//! This is the kernel-level security subsystem. The full Guardian
//! homeostasis engine runs as a managed service — this module provides
//! the thin, sync, zero-async security posture the OS core needs.
//!
//! ## Primitive Grounding
//!
//! - ∂ Boundary: Threat detection and security perimeters
//! - ς State: Security level state machine (Green→Yellow→Orange→Red)
//! - κ Comparison: Threat severity assessment
//! - → Causality: Threat → response chain

use crate::service::ServiceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Security level — kernel-level DEFCON equivalent.
///
/// Tier: T2-P (ς State — security posture)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Normal operation. No active threats.
    Green = 0,
    /// Elevated awareness. Minor anomalies detected.
    Yellow = 1,
    /// Active threat. Services may be quarantined.
    Orange = 2,
    /// Critical threat. Emergency lockdown.
    Red = 3,
}

// ═══════════════════════════════════════════════════════════
// GUARDIAN-INSPIRED THREAT PATTERNS (PAMPs/DAMPs)
// ═══════════════════════════════════════════════════════════

/// Pathogen-Associated Molecular Pattern (PAMP) — external threat signatures.
///
/// Tier: T2-C (∂ Boundary + κ Comparison — threat boundary detection)
///
/// Modeled after the biological innate immune system: PAMPs are patterns
/// recognized as foreign/hostile without prior exposure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pamp {
    /// Rapid authentication failures from the same source.
    RapidLoginFailure {
        /// Number of failures in the detection window.
        count: u32,
        /// Source identifier (IP, process ID, etc.).
        source: String,
    },
    /// Port scanning or network probing activity.
    PortScan {
        /// Number of ports probed.
        ports_probed: u32,
        /// Source identifier.
        source: String,
    },
    /// Unauthorized access attempt to a protected resource.
    UnauthorizedAccess {
        /// The resource that was targeted.
        resource: String,
        /// Who attempted access.
        actor: String,
    },
    /// Malicious payload detected (e.g., injection attempt).
    MaliciousPayload {
        /// Type of payload.
        payload_type: String,
        /// Where it was detected.
        location: String,
    },
    /// Privilege escalation attempt.
    PrivilegeEscalation {
        /// Process or actor attempting escalation.
        actor: String,
        /// Target privilege level.
        target_level: String,
    },
}

/// Damage-Associated Molecular Pattern (DAMP) — internal damage signals.
///
/// Tier: T2-C (∂ Boundary + ν Frequency — internal damage detection)
///
/// DAMPs are released when the system itself is damaged, analogous to
/// cellular distress signals in biology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Damp {
    /// Memory usage exceeds safe threshold.
    MemoryPressure {
        /// Current memory usage percentage (0-100).
        usage_pct: u8,
        /// Threshold that was exceeded.
        threshold_pct: u8,
    },
    /// CPU usage sustained above safe threshold.
    CpuExhaustion {
        /// Current CPU usage percentage (0-100).
        usage_pct: u8,
        /// Duration in ticks above threshold.
        sustained_ticks: u64,
    },
    /// A managed service has crashed.
    ServiceCrash {
        /// ID of the crashed service.
        service_id: ServiceId,
        /// Name of the crashed service.
        service_name: String,
        /// Number of crashes in the current window.
        crash_count: u32,
    },
    /// Disk usage exceeds safe threshold.
    DiskFull {
        /// Current disk usage percentage (0-100).
        usage_pct: u8,
        /// Mount point or partition.
        mount: String,
    },
    /// System configuration was modified unexpectedly.
    ConfigTamper {
        /// Configuration file or key that changed.
        config_key: String,
        /// Description of the change.
        description: String,
    },
}

/// A threat event combining pattern type with metadata.
///
/// Tier: T2-C (∂ + → + ς — boundary violation with causal response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatPattern {
    /// External threat (pathogen-like).
    External(Pamp),
    /// Internal damage (self-inflicted).
    Internal(Damp),
}

impl ThreatPattern {
    /// Classify the severity of this threat pattern.
    pub fn classify_severity(&self) -> ThreatSeverity {
        match self {
            Self::External(pamp) => match pamp {
                Pamp::RapidLoginFailure { count, .. } => {
                    if *count >= 10 {
                        ThreatSeverity::High
                    } else if *count >= 5 {
                        ThreatSeverity::Medium
                    } else {
                        ThreatSeverity::Low
                    }
                }
                Pamp::PortScan { ports_probed, .. } => {
                    if *ports_probed >= 100 {
                        ThreatSeverity::High
                    } else {
                        ThreatSeverity::Medium
                    }
                }
                Pamp::UnauthorizedAccess { .. } => ThreatSeverity::High,
                Pamp::MaliciousPayload { .. } | Pamp::PrivilegeEscalation { .. } => {
                    ThreatSeverity::Critical
                }
            },
            Self::Internal(damp) => match damp {
                Damp::MemoryPressure { usage_pct, .. } => {
                    if *usage_pct >= 95 {
                        ThreatSeverity::High
                    } else {
                        ThreatSeverity::Medium
                    }
                }
                Damp::CpuExhaustion {
                    sustained_ticks, ..
                } => {
                    if *sustained_ticks >= 100 {
                        ThreatSeverity::High
                    } else {
                        ThreatSeverity::Medium
                    }
                }
                Damp::ServiceCrash { crash_count, .. } => {
                    if *crash_count >= 3 {
                        ThreatSeverity::High
                    } else {
                        ThreatSeverity::Medium
                    }
                }
                Damp::DiskFull { usage_pct, .. } => {
                    if *usage_pct >= 99 {
                        ThreatSeverity::Critical
                    } else {
                        ThreatSeverity::High
                    }
                }
                Damp::ConfigTamper { .. } => ThreatSeverity::High,
            },
        }
    }

    /// Get a description suitable for the threat record.
    pub fn describe(&self) -> String {
        match self {
            Self::External(pamp) => match pamp {
                Pamp::RapidLoginFailure { count, source } => {
                    format!("PAMP: {count} login failures from {source}")
                }
                Pamp::PortScan {
                    ports_probed,
                    source,
                } => {
                    format!("PAMP: {ports_probed} ports scanned from {source}")
                }
                Pamp::UnauthorizedAccess { resource, actor } => {
                    format!("PAMP: Unauthorized access to {resource} by {actor}")
                }
                Pamp::MaliciousPayload {
                    payload_type,
                    location,
                } => {
                    format!("PAMP: Malicious {payload_type} at {location}")
                }
                Pamp::PrivilegeEscalation {
                    actor,
                    target_level,
                } => {
                    format!("PAMP: {actor} attempting escalation to {target_level}")
                }
            },
            Self::Internal(damp) => match damp {
                Damp::MemoryPressure {
                    usage_pct,
                    threshold_pct,
                } => {
                    format!("DAMP: Memory at {usage_pct}% (threshold {threshold_pct}%)")
                }
                Damp::CpuExhaustion {
                    usage_pct,
                    sustained_ticks,
                } => {
                    format!("DAMP: CPU at {usage_pct}% for {sustained_ticks} ticks")
                }
                Damp::ServiceCrash {
                    service_name,
                    crash_count,
                    ..
                } => {
                    format!("DAMP: Service {service_name} crashed ({crash_count}x)")
                }
                Damp::DiskFull { usage_pct, mount } => {
                    format!("DAMP: Disk {mount} at {usage_pct}%")
                }
                Damp::ConfigTamper {
                    config_key,
                    description,
                } => {
                    format!("DAMP: Config tamper on {config_key}: {description}")
                }
            },
        }
    }

    /// Extract the source service ID if attributable.
    pub fn source_service(&self) -> Option<ServiceId> {
        match self {
            Self::Internal(Damp::ServiceCrash { service_id, .. }) => Some(*service_id),
            _ => None,
        }
    }
}

/// Security response actions — what the OS should do.
///
/// Tier: T2-P (→ Causality — response to threat)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityResponse {
    /// Log and monitor. No intervention.
    Monitor,
    /// Quarantine the source service.
    QuarantineService,
    /// Suspend all non-critical services.
    SuspendNonCritical,
    /// Full system lockdown (shell lock + service freeze).
    Lockdown,
    /// Emit notification to the user.
    NotifyUser,
}

impl SecurityLevel {
    /// Whether this level requires active intervention.
    pub fn requires_action(self) -> bool {
        matches!(self, Self::Orange | Self::Red)
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Green => write!(f, "GREEN"),
            Self::Yellow => write!(f, "YELLOW"),
            Self::Orange => write!(f, "ORANGE"),
            Self::Red => write!(f, "RED"),
        }
    }
}

/// Threat severity classification.
///
/// Tier: T2-P (κ Comparison — severity ranking)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatSeverity {
    /// Informational — logged but no action.
    Info = 0,
    /// Low — minor anomaly, monitor.
    Low = 1,
    /// Medium — potential issue, investigate.
    Medium = 2,
    /// High — active threat, intervene.
    High = 3,
    /// Critical — system integrity at risk.
    Critical = 4,
}

/// A recorded threat event.
///
/// Tier: T2-C (∂ + ς + → — boundary violation with causal chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatRecord {
    /// When the threat was detected.
    pub timestamp: DateTime<Utc>,
    /// Threat severity.
    pub severity: ThreatSeverity,
    /// Source service (if attributable).
    pub source_service: Option<ServiceId>,
    /// Human-readable description.
    pub description: String,
    /// Whether this threat has been resolved.
    pub resolved: bool,
}

/// OS-level security monitor — Guardian-inspired homeostasis engine.
///
/// Tier: T3 (∂ + ς + κ + → — full security posture tracking)
///
/// Implements a sync, lightweight version of the Guardian homeostasis loop:
/// - **SENSE**: PAMPs (external threats) and DAMPs (internal damage)
/// - **DECIDE**: Threat severity → security level → response selection
/// - **ACT**: Quarantine, suspend, lockdown, notify
///
/// The full async Guardian engine runs as a managed service;
/// this is the kernel-level sync bridge.
pub struct SecurityMonitor {
    /// Active and historical threat records.
    threats: Vec<ThreatRecord>,
    /// Current security level.
    level: SecurityLevel,
    /// Services currently quarantined.
    quarantined: Vec<ServiceId>,
    /// Maximum threats before auto-escalation.
    escalation_threshold: usize,
    /// Pending security responses (consumed by kernel tick loop).
    pending_responses: Vec<SecurityResponse>,
    /// Total PAMP events recorded.
    pamp_count: u64,
    /// Total DAMP events recorded.
    damp_count: u64,
}

impl SecurityMonitor {
    /// Create a new security monitor.
    pub fn new() -> Self {
        Self {
            threats: Vec::new(),
            level: SecurityLevel::Green,
            quarantined: Vec::new(),
            escalation_threshold: 5,
            pending_responses: Vec::new(),
            pamp_count: 0,
            damp_count: 0,
        }
    }

    /// Record a new threat.
    ///
    /// Automatically recalculates the security level.
    pub fn record_threat(
        &mut self,
        severity: ThreatSeverity,
        description: impl Into<String>,
        source_service: Option<ServiceId>,
    ) {
        self.threats.push(ThreatRecord {
            timestamp: Utc::now(),
            severity,
            source_service,
            description: description.into(),
            resolved: false,
        });
        self.recalculate_level();
    }

    /// Mark a threat as resolved by index.
    pub fn resolve_threat(&mut self, index: usize) {
        if let Some(threat) = self.threats.get_mut(index) {
            threat.resolved = true;
        }
        self.recalculate_level();
    }

    /// Get the current security level.
    pub fn level(&self) -> SecurityLevel {
        self.level
    }

    /// Whether the system is in a critical state.
    pub fn is_critical(&self) -> bool {
        self.level == SecurityLevel::Red
    }

    /// Get all active (unresolved) threats.
    pub fn active_threats(&self) -> Vec<&ThreatRecord> {
        self.threats.iter().filter(|t| !t.resolved).collect()
    }

    /// Get threats attributed to a specific service.
    pub fn threats_for_service(&self, service_id: ServiceId) -> Vec<&ThreatRecord> {
        self.threats
            .iter()
            .filter(|t| t.source_service == Some(service_id) && !t.resolved)
            .collect()
    }

    /// Whether a service should be quarantined.
    ///
    /// A service is quarantined if it has 3+ active threats
    /// or any Critical-severity threat.
    pub fn should_quarantine(&self, service_id: ServiceId) -> bool {
        let service_threats = self.threats_for_service(service_id);
        let has_critical = service_threats
            .iter()
            .any(|t| t.severity == ThreatSeverity::Critical);
        has_critical || service_threats.len() >= 3
    }

    /// Quarantine a service (mark it as isolated).
    pub fn quarantine_service(&mut self, service_id: ServiceId) {
        if !self.quarantined.contains(&service_id) {
            self.quarantined.push(service_id);
        }
    }

    /// Release a service from quarantine.
    pub fn release_service(&mut self, service_id: ServiceId) {
        self.quarantined.retain(|&id| id != service_id);
    }

    /// Check if a service is quarantined.
    pub fn is_quarantined(&self, service_id: ServiceId) -> bool {
        self.quarantined.contains(&service_id)
    }

    /// Get count of quarantined services.
    pub fn quarantined_count(&self) -> usize {
        self.quarantined.len()
    }

    /// Total threats recorded (active + resolved).
    pub fn total_threats(&self) -> usize {
        self.threats.len()
    }

    // ═══════════════════════════════════════════════════════════
    // GUARDIAN-INSPIRED HOMEOSTASIS METHODS
    // ═══════════════════════════════════════════════════════════

    /// Record a threat from a PAMP or DAMP pattern.
    ///
    /// Automatically classifies severity, records the threat, and
    /// determines the appropriate security response.
    pub fn record_pattern(&mut self, pattern: &ThreatPattern) {
        let severity = pattern.classify_severity();
        let description = pattern.describe();
        let source = pattern.source_service();

        // Count by type
        match pattern {
            ThreatPattern::External(_) => self.pamp_count += 1,
            ThreatPattern::Internal(_) => self.damp_count += 1,
        }

        // Record the threat
        self.record_threat(severity, description, source);

        // Determine and queue response
        let response = self.assess_response();
        if response != SecurityResponse::Monitor {
            self.pending_responses.push(response);
        }
    }

    /// Assess what response the current security state demands.
    ///
    /// Implements the DECIDE phase of the homeostasis loop:
    /// SecurityLevel → SecurityResponse mapping.
    pub fn assess_response(&self) -> SecurityResponse {
        match self.level {
            SecurityLevel::Green => SecurityResponse::Monitor,
            SecurityLevel::Yellow => SecurityResponse::NotifyUser,
            SecurityLevel::Orange => {
                if self.quarantined.is_empty() {
                    SecurityResponse::QuarantineService
                } else {
                    SecurityResponse::SuspendNonCritical
                }
            }
            SecurityLevel::Red => SecurityResponse::Lockdown,
        }
    }

    /// Drain pending security responses (consumed by kernel tick loop).
    pub fn drain_responses(&mut self) -> Vec<SecurityResponse> {
        std::mem::take(&mut self.pending_responses)
    }

    /// Whether there are pending security responses.
    pub fn has_pending_responses(&self) -> bool {
        !self.pending_responses.is_empty()
    }

    /// Total PAMP events recorded.
    pub fn pamp_count(&self) -> u64 {
        self.pamp_count
    }

    /// Total DAMP events recorded.
    pub fn damp_count(&self) -> u64 {
        self.damp_count
    }

    /// Whether the current security level blocks app installation.
    ///
    /// Apps cannot be installed when security level is Orange or Red.
    pub fn blocks_app_install(&self) -> bool {
        self.level >= SecurityLevel::Orange
    }

    /// Whether the current security level blocks non-critical services.
    ///
    /// Non-critical services are suspended at Red.
    pub fn blocks_non_critical(&self) -> bool {
        self.level == SecurityLevel::Red
    }

    /// Recalculate the security level based on active threats.
    fn recalculate_level(&mut self) {
        let active = self.active_threats();

        // Any critical threat → Red
        if active
            .iter()
            .any(|t| t.severity == ThreatSeverity::Critical)
        {
            self.level = SecurityLevel::Red;
            return;
        }

        // Any high threat or many active threats → Orange
        let high_count = active
            .iter()
            .filter(|t| t.severity == ThreatSeverity::High)
            .count();
        if high_count > 0 || active.len() >= self.escalation_threshold {
            self.level = SecurityLevel::Orange;
            return;
        }

        // Any medium threats → Yellow
        let medium_count = active
            .iter()
            .filter(|t| t.severity == ThreatSeverity::Medium)
            .count();
        if medium_count > 0 {
            self.level = SecurityLevel::Yellow;
            return;
        }

        // Otherwise green
        self.level = SecurityLevel::Green;
    }
}

impl Default for SecurityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_green() {
        let monitor = SecurityMonitor::new();
        assert_eq!(monitor.level(), SecurityLevel::Green);
        assert!(!monitor.is_critical());
        assert_eq!(monitor.total_threats(), 0);
    }

    #[test]
    fn threat_escalation_medium() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_threat(ThreatSeverity::Medium, "Unusual activity", None);
        assert_eq!(monitor.level(), SecurityLevel::Yellow);
    }

    #[test]
    fn threat_escalation_high() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_threat(ThreatSeverity::High, "Brute force detected", None);
        assert_eq!(monitor.level(), SecurityLevel::Orange);
    }

    #[test]
    fn threat_escalation_critical() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_threat(ThreatSeverity::Critical, "Root compromise", None);
        assert_eq!(monitor.level(), SecurityLevel::Red);
        assert!(monitor.is_critical());
    }

    #[test]
    fn threat_resolution_deescalates() {
        let mut monitor = SecurityMonitor::new();
        monitor.record_threat(ThreatSeverity::High, "Attack detected", None);
        assert_eq!(monitor.level(), SecurityLevel::Orange);

        monitor.resolve_threat(0);
        assert_eq!(monitor.level(), SecurityLevel::Green);
    }

    #[test]
    fn service_quarantine() {
        use crate::service::ServiceId;
        let mut monitor = SecurityMonitor::new();
        let svc_id = ServiceId::new(42);

        // Record 3 threats for the same service
        for i in 0..3 {
            monitor.record_threat(ThreatSeverity::Low, format!("Anomaly {i}"), Some(svc_id));
        }

        assert!(monitor.should_quarantine(svc_id));

        monitor.quarantine_service(svc_id);
        assert!(monitor.is_quarantined(svc_id));
        assert_eq!(monitor.quarantined_count(), 1);

        monitor.release_service(svc_id);
        assert!(!monitor.is_quarantined(svc_id));
    }

    #[test]
    fn critical_threat_auto_quarantine() {
        use crate::service::ServiceId;
        let mut monitor = SecurityMonitor::new();
        let svc_id = ServiceId::new(7);

        monitor.record_threat(ThreatSeverity::Critical, "Exploit", Some(svc_id));
        assert!(monitor.should_quarantine(svc_id));
    }

    #[test]
    fn threshold_escalation() {
        let mut monitor = SecurityMonitor::new();
        // 5 low threats should escalate to Orange
        for i in 0..5 {
            monitor.record_threat(ThreatSeverity::Low, format!("Low threat {i}"), None);
        }
        assert_eq!(monitor.level(), SecurityLevel::Orange);
    }

    #[test]
    fn security_level_display() {
        assert_eq!(format!("{}", SecurityLevel::Green), "GREEN");
        assert_eq!(format!("{}", SecurityLevel::Red), "RED");
    }

    #[test]
    fn security_level_ordering() {
        assert!(SecurityLevel::Green < SecurityLevel::Yellow);
        assert!(SecurityLevel::Yellow < SecurityLevel::Orange);
        assert!(SecurityLevel::Orange < SecurityLevel::Red);
    }

    #[test]
    fn requires_action() {
        assert!(!SecurityLevel::Green.requires_action());
        assert!(!SecurityLevel::Yellow.requires_action());
        assert!(SecurityLevel::Orange.requires_action());
        assert!(SecurityLevel::Red.requires_action());
    }

    // ═══════════════════════════════════════════════════════════
    // PAMP/DAMP PATTERN TESTS
    // ═══════════════════════════════════════════════════════════

    #[test]
    fn pamp_login_failure_severity() {
        let low = Pamp::RapidLoginFailure {
            count: 3,
            source: "x".into(),
        };
        let pattern = ThreatPattern::External(low);
        assert_eq!(pattern.classify_severity(), ThreatSeverity::Low);

        let medium = Pamp::RapidLoginFailure {
            count: 5,
            source: "x".into(),
        };
        let pattern = ThreatPattern::External(medium);
        assert_eq!(pattern.classify_severity(), ThreatSeverity::Medium);

        let high = Pamp::RapidLoginFailure {
            count: 10,
            source: "x".into(),
        };
        let pattern = ThreatPattern::External(high);
        assert_eq!(pattern.classify_severity(), ThreatSeverity::High);
    }

    #[test]
    fn pamp_malicious_payload_critical() {
        let pattern = ThreatPattern::External(Pamp::MaliciousPayload {
            payload_type: "XSS".into(),
            location: "input".into(),
        });
        assert_eq!(pattern.classify_severity(), ThreatSeverity::Critical);
    }

    #[test]
    fn pamp_privilege_escalation_critical() {
        let pattern = ThreatPattern::External(Pamp::PrivilegeEscalation {
            actor: "proc".into(),
            target_level: "root".into(),
        });
        assert_eq!(pattern.classify_severity(), ThreatSeverity::Critical);
    }

    #[test]
    fn damp_memory_pressure_severity() {
        let medium = ThreatPattern::Internal(Damp::MemoryPressure {
            usage_pct: 85,
            threshold_pct: 80,
        });
        assert_eq!(medium.classify_severity(), ThreatSeverity::Medium);

        let high = ThreatPattern::Internal(Damp::MemoryPressure {
            usage_pct: 96,
            threshold_pct: 80,
        });
        assert_eq!(high.classify_severity(), ThreatSeverity::High);
    }

    #[test]
    fn damp_disk_full_severity() {
        let high = ThreatPattern::Internal(Damp::DiskFull {
            usage_pct: 95,
            mount: "/".into(),
        });
        assert_eq!(high.classify_severity(), ThreatSeverity::High);

        let critical = ThreatPattern::Internal(Damp::DiskFull {
            usage_pct: 99,
            mount: "/".into(),
        });
        assert_eq!(critical.classify_severity(), ThreatSeverity::Critical);
    }

    #[test]
    fn damp_service_crash_severity() {
        let svc_id = ServiceId::new(1);
        let medium = ThreatPattern::Internal(Damp::ServiceCrash {
            service_id: svc_id,
            service_name: "test".into(),
            crash_count: 1,
        });
        assert_eq!(medium.classify_severity(), ThreatSeverity::Medium);

        let high = ThreatPattern::Internal(Damp::ServiceCrash {
            service_id: svc_id,
            service_name: "test".into(),
            crash_count: 3,
        });
        assert_eq!(high.classify_severity(), ThreatSeverity::High);
    }

    #[test]
    fn pattern_describe() {
        let pattern = ThreatPattern::External(Pamp::PortScan {
            ports_probed: 50,
            source: "10.0.0.1".into(),
        });
        let desc = pattern.describe();
        assert!(desc.contains("PAMP"));
        assert!(desc.contains("50"));
        assert!(desc.contains("10.0.0.1"));
    }

    #[test]
    fn pattern_source_service() {
        let svc_id = ServiceId::new(42);
        let internal = ThreatPattern::Internal(Damp::ServiceCrash {
            service_id: svc_id,
            service_name: "test".into(),
            crash_count: 1,
        });
        assert_eq!(internal.source_service(), Some(svc_id));

        let external = ThreatPattern::External(Pamp::PortScan {
            ports_probed: 10,
            source: "x".into(),
        });
        assert_eq!(external.source_service(), None);
    }

    #[test]
    fn record_pattern_escalates() {
        let mut monitor = SecurityMonitor::new();
        let pattern = ThreatPattern::External(Pamp::UnauthorizedAccess {
            resource: "/etc/passwd".into(),
            actor: "attacker".into(),
        });

        monitor.record_pattern(&pattern);
        assert_eq!(monitor.level(), SecurityLevel::Orange);
        assert_eq!(monitor.pamp_count(), 1);
        assert!(monitor.has_pending_responses());
    }

    #[test]
    fn record_pattern_damp_counts() {
        let mut monitor = SecurityMonitor::new();
        let pattern = ThreatPattern::Internal(Damp::ConfigTamper {
            config_key: "/etc/nexcore.toml".into(),
            description: "modified".into(),
        });

        monitor.record_pattern(&pattern);
        assert_eq!(monitor.damp_count(), 1);
    }

    #[test]
    fn assess_response_by_level() {
        let mut monitor = SecurityMonitor::new();
        assert_eq!(monitor.assess_response(), SecurityResponse::Monitor);

        monitor.record_threat(ThreatSeverity::Medium, "test", None);
        assert_eq!(monitor.assess_response(), SecurityResponse::NotifyUser);

        monitor.record_threat(ThreatSeverity::High, "test", None);
        assert_eq!(
            monitor.assess_response(),
            SecurityResponse::QuarantineService
        );

        monitor.record_threat(ThreatSeverity::Critical, "test", None);
        assert_eq!(monitor.assess_response(), SecurityResponse::Lockdown);
    }

    #[test]
    fn drain_responses() {
        let mut monitor = SecurityMonitor::new();
        let pattern = ThreatPattern::External(Pamp::MaliciousPayload {
            payload_type: "RCE".into(),
            location: "api".into(),
        });

        monitor.record_pattern(&pattern);
        assert!(monitor.has_pending_responses());

        let responses = monitor.drain_responses();
        assert!(!responses.is_empty());
        assert!(!monitor.has_pending_responses());
    }

    #[test]
    fn blocks_app_install() {
        let mut monitor = SecurityMonitor::new();
        assert!(!monitor.blocks_app_install());

        monitor.record_threat(ThreatSeverity::High, "threat", None);
        assert!(monitor.blocks_app_install()); // Orange

        let mut monitor2 = SecurityMonitor::new();
        monitor2.record_threat(ThreatSeverity::Critical, "threat", None);
        assert!(monitor2.blocks_app_install()); // Red
    }

    #[test]
    fn blocks_non_critical() {
        let mut monitor = SecurityMonitor::new();
        assert!(!monitor.blocks_non_critical());

        monitor.record_threat(ThreatSeverity::High, "threat", None);
        assert!(!monitor.blocks_non_critical()); // Orange

        let mut monitor2 = SecurityMonitor::new();
        monitor2.record_threat(ThreatSeverity::Critical, "threat", None);
        assert!(monitor2.blocks_non_critical()); // Red
    }
}
