//! Enforcement backends — applies fence verdicts to the system.
//!
//! Tier: T2-P (∂ Boundary + ∝ Irreversibility — enforcement creates real boundaries)
//!
//! The Enforcer trait is modeled after `nexcore-sentinel`'s FirewallBackend
//! but specialized for process-aware UID-based rules via iptables `-m owner`.

use serde::{Deserialize, Serialize};

use crate::error::FenceResult;
use crate::process::ConnectionEvent;
use crate::rule::FenceVerdict;

/// An enforcement operation recorded by `MockEnforcer`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnforcerOp {
    /// Process name.
    pub process_name: String,
    /// PID.
    pub pid: u32,
    /// Remote address:port.
    pub remote: String,
    /// The verdict applied.
    pub verdict: FenceVerdict,
}

/// Trait for enforcement backends.
///
/// Implementations apply fence verdicts to the system. The simplest
/// implementation logs; the strongest creates iptables rules.
pub trait Enforcer: Send + Sync {
    /// Apply a verdict to a connection event.
    fn enforce(&self, event: &ConnectionEvent, verdict: FenceVerdict) -> FenceResult<()>;

    /// Return a display name for this enforcer.
    fn name(&self) -> &str;
}

/// Log-only enforcer — records decisions to tracing, takes no action.
///
/// Suitable for Monitor mode and initial deployment.
#[derive(Debug, Default)]
pub struct LogOnlyEnforcer;

impl Enforcer for LogOnlyEnforcer {
    fn enforce(&self, event: &ConnectionEvent, verdict: FenceVerdict) -> FenceResult<()> {
        tracing::info!(
            verdict = ?verdict,
            process = event.process_name(),
            local_port = event.socket.local_port,
            remote_addr = %event.socket.remote_addr,
            remote_port = event.socket.remote_port,
            "fence decision (log-only)"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "log-only"
    }
}

/// Iptables enforcer using `-m owner --uid-owner` for process-level blocking.
///
/// Requires root privileges. Creates rules in a dedicated chain.
#[derive(Debug)]
pub struct IptablesEnforcer {
    /// Chain name for fence rules (e.g., "SIGNAL-FENCE").
    pub chain: String,
    /// Whether to use ip6tables in addition to iptables.
    pub ipv6: bool,
}

impl IptablesEnforcer {
    /// Create a new iptables enforcer with the given chain name.
    pub fn new(chain: impl Into<String>) -> Self {
        Self {
            chain: chain.into(),
            ipv6: true,
        }
    }

    /// Build the iptables command for a deny verdict.
    fn build_deny_command(&self, event: &ConnectionEvent) -> Vec<String> {
        let mut args = vec![
            "-A".to_string(),
            self.chain.clone(),
            "-p".to_string(),
            "tcp".to_string(),
        ];

        // Add UID match if process is known
        if let Some(ref proc) = event.process {
            args.extend([
                "-m".to_string(),
                "owner".to_string(),
                "--uid-owner".to_string(),
                proc.uid.to_string(),
            ]);
        }

        // Add destination
        args.extend([
            "-d".to_string(),
            event.socket.remote_addr.to_string(),
            "--dport".to_string(),
            event.socket.remote_port.to_string(),
            "-j".to_string(),
            "DROP".to_string(),
        ]);

        args
    }
}

impl Enforcer for IptablesEnforcer {
    fn enforce(&self, event: &ConnectionEvent, verdict: FenceVerdict) -> FenceResult<()> {
        match verdict {
            FenceVerdict::Allow => {
                tracing::debug!(
                    process = event.process_name(),
                    "allowing connection (no iptables action)"
                );
                Ok(())
            }
            FenceVerdict::Alert => {
                tracing::warn!(
                    process = event.process_name(),
                    remote = %event.socket.remote_addr,
                    "ALERT: suspicious connection detected"
                );
                Ok(())
            }
            FenceVerdict::Deny => {
                let args = self.build_deny_command(event);
                tracing::info!(
                    chain = &self.chain,
                    args = ?args,
                    "would execute: iptables {}",
                    args.join(" ")
                );
                // In production: std::process::Command::new("iptables").args(&args).output()
                // For safety, we log the command but don't execute without explicit opt-in
                Ok(())
            }
        }
    }

    fn name(&self) -> &str {
        "iptables"
    }
}

/// Mock enforcer for testing — records all operations.
#[derive(Debug, Default)]
pub struct MockEnforcer {
    /// Recorded operations (behind a mutex for shared access).
    pub operations: std::sync::Mutex<Vec<EnforcerOp>>,
}

impl MockEnforcer {
    /// Create a new mock enforcer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get recorded operations.
    pub fn ops(&self) -> Vec<EnforcerOp> {
        self.operations
            .lock()
            .ok()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Count operations by verdict.
    pub fn count_verdict(&self, verdict: FenceVerdict) -> usize {
        self.ops().iter().filter(|op| op.verdict == verdict).count()
    }
}

impl Enforcer for MockEnforcer {
    fn enforce(&self, event: &ConnectionEvent, verdict: FenceVerdict) -> FenceResult<()> {
        let op = EnforcerOp {
            process_name: event.process_name().to_string(),
            pid: event.process.as_ref().map_or(0, |p| p.pid),
            remote: format!("{}:{}", event.socket.remote_addr, event.socket.remote_port),
            verdict,
        };
        if let Ok(mut ops) = self.operations.lock() {
            ops.push(op);
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use super::*;
    use crate::connection::{Protocol, SocketEntry, TcpState};
    use crate::process::ProcessInfo;

    fn make_event(name: &str) -> ConnectionEvent {
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
        let proc = ProcessInfo {
            pid: 1234,
            name: name.to_string(),
            exe: PathBuf::from(format!("/usr/bin/{name}")),
            uid: 1000,
            cmdline: name.to_string(),
        };
        ConnectionEvent::new(socket, Some(proc))
    }

    #[test]
    fn test_mock_enforcer_records_ops() {
        let enforcer = MockEnforcer::new();
        let event = make_event("nginx");
        let result = enforcer.enforce(&event, FenceVerdict::Allow);
        assert!(result.is_ok());
        let ops = enforcer.ops();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].process_name, "nginx");
        assert_eq!(ops[0].verdict, FenceVerdict::Allow);
    }

    #[test]
    fn test_mock_enforcer_count_verdict() {
        let enforcer = MockEnforcer::new();
        let event = make_event("test");
        let _ = enforcer.enforce(&event, FenceVerdict::Allow);
        let _ = enforcer.enforce(&event, FenceVerdict::Deny);
        let _ = enforcer.enforce(&event, FenceVerdict::Deny);
        assert_eq!(enforcer.count_verdict(FenceVerdict::Allow), 1);
        assert_eq!(enforcer.count_verdict(FenceVerdict::Deny), 2);
    }

    #[test]
    fn test_mock_enforcer_name() {
        let enforcer = MockEnforcer::new();
        assert_eq!(enforcer.name(), "mock");
    }

    #[test]
    fn test_log_only_enforcer() {
        let enforcer = LogOnlyEnforcer;
        let event = make_event("curl");
        let result = enforcer.enforce(&event, FenceVerdict::Deny);
        assert!(result.is_ok());
        assert_eq!(enforcer.name(), "log-only");
    }

    #[test]
    fn test_iptables_enforcer_name() {
        let enforcer = IptablesEnforcer::new("SIGNAL-FENCE");
        assert_eq!(enforcer.name(), "iptables");
        assert_eq!(enforcer.chain, "SIGNAL-FENCE");
    }

    #[test]
    fn test_iptables_build_deny_command() {
        let enforcer = IptablesEnforcer::new("SIGNAL-FENCE");
        let event = make_event("malware");
        let cmd = enforcer.build_deny_command(&event);
        assert!(cmd.contains(&"-A".to_string()));
        assert!(cmd.contains(&"SIGNAL-FENCE".to_string()));
        assert!(cmd.contains(&"--uid-owner".to_string()));
        assert!(cmd.contains(&"1000".to_string()));
        assert!(cmd.contains(&"DROP".to_string()));
    }

    #[test]
    fn test_iptables_enforcer_allow_does_nothing() {
        let enforcer = IptablesEnforcer::new("SIGNAL-FENCE");
        let event = make_event("nginx");
        let result = enforcer.enforce(&event, FenceVerdict::Allow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_iptables_enforcer_deny() {
        let enforcer = IptablesEnforcer::new("SIGNAL-FENCE");
        let event = make_event("suspicious");
        let result = enforcer.enforce(&event, FenceVerdict::Deny);
        assert!(result.is_ok()); // Logs but doesn't execute
    }

    #[test]
    fn test_iptables_enforcer_alert() {
        let enforcer = IptablesEnforcer::new("SIGNAL-FENCE");
        let event = make_event("unknown");
        let result = enforcer.enforce(&event, FenceVerdict::Alert);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enforcer_op_serialization() {
        let op = EnforcerOp {
            process_name: "test".to_string(),
            pid: 100,
            remote: "10.0.0.1:443".to_string(),
            verdict: FenceVerdict::Deny,
        };
        let json = serde_json::to_string(&op);
        assert!(json.is_ok());
    }
}
