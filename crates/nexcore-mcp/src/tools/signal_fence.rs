//! Signal Fence tools — process-level network signal container
//!
//! Wraps `nexcore-signal-fence` crate for MCP access.
//! Dominant primitive: ∂ (Boundary). Core thesis: "All traffic is noise until proven otherwise."

use crate::params::SignalFenceEvaluateParams;
use nexcore_signal_fence::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Get the current status of a default-deny fence engine.
///
/// Returns mode, default verdict, stats, and rule count.
pub fn fence_status() -> Result<CallToolResult, McpError> {
    let engine = FenceEngine::default_deny();
    let report = engine.report();

    let result = json!({
        "crate": "nexcore-signal-fence",
        "thesis": "All traffic is noise until proven otherwise (A2 Noise Dominance)",
        "dominant_primitive": "∂ (Boundary)",
        "engine": {
            "mode": report.mode,
            "default_verdict": report.default_verdict,
            "enforcer": report.enforcer,
            "rule_count": report.rule_count,
            "audit_log_size": report.audit_log_size,
        },
        "stats": {
            "total_observed": report.stats.total_observed,
            "allowed": report.stats.allowed,
            "denied": report.stats.denied,
            "alerted": report.stats.alerted,
            "ticks": report.stats.ticks,
            "denial_rate": report.stats.denial_rate(),
            "rule_match_rate": report.stats.rule_match_rate(),
        },
        "signal_theory_mapping": {
            "ObservationSpace": "Active connections",
            "Baseline": "Default-deny (all traffic = noise)",
            "FixedBoundary": "Allowlist rules",
            "DetectionOutcome": "Allow/Deny/Alert",
            "A2_NoiseDominance": "Most connections unauthorized",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Scan current connections from /proc/net/tcp{,6} and resolve to processes.
///
/// Returns all active socket entries with process attribution.
pub fn fence_scan() -> Result<CallToolResult, McpError> {
    let engine = FenceEngine::default_deny();
    let events = engine.scan();

    let connections: Vec<_> = events
        .iter()
        .map(|e| {
            json!({
                "process": e.process_name(),
                "pid": e.process.as_ref().map(|p| p.pid),
                "local": format!("{}:{}", e.socket.local_addr, e.socket.local_port),
                "remote": format!("{}:{}", e.socket.remote_addr, e.socket.remote_port),
                "protocol": format!("{:?}", e.socket.protocol),
                "state": format!("{:?}", e.socket.state),
                "direction": format!("{:?}", e.direction),
                "attributed": e.is_attributed(),
            })
        })
        .collect();

    let result = json!({
        "total_connections": connections.len(),
        "attributed": events.iter().filter(|e| e.is_attributed()).count(),
        "unattributed": events.iter().filter(|e| !e.is_attributed()).count(),
        "connections": connections,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Evaluate a process+port combination against a default-deny policy.
///
/// Optionally add allow rules before evaluation.
pub fn fence_evaluate(params: SignalFenceEvaluateParams) -> Result<CallToolResult, McpError> {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::PathBuf;

    use nexcore_signal_fence::connection::{Protocol, SocketEntry, TcpState};

    let mut engine = FenceEngine::default_deny();

    // Add any allow rules from params
    for rule_def in &params.allow_rules {
        let process_match = match rule_def.process.as_deref() {
            Some(name) => ProcessMatch::ByName(name.to_string()),
            None => ProcessMatch::Any,
        };
        let network_match = match rule_def.port {
            Some(port) => NetworkMatch::ByPort(port),
            None => NetworkMatch::Any,
        };

        engine.policy.add_rule(FenceRule {
            id: format!("mcp-rule-{}", rule_def.process.as_deref().unwrap_or("any")),
            process_match,
            network_match,
            direction: Direction::Both,
            verdict: FenceVerdict::Allow,
            priority: 10,
            description: format!(
                "MCP-defined allow rule: process={}, port={}",
                rule_def.process.as_deref().unwrap_or("any"),
                rule_def
                    .port
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "any".to_string())
            ),
        });
    }

    // Build synthetic connection event for evaluation
    let remote_addr = params
        .remote_addr
        .as_deref()
        .and_then(|s| s.parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));

    let socket = SocketEntry {
        local_addr: IpAddr::V4(Ipv4Addr::LOCALHOST),
        local_port: params.local_port.unwrap_or(54321),
        remote_addr,
        remote_port: params.port,
        inode: 0,
        state: TcpState::Established,
        protocol: Protocol::Tcp,
        uid: 1000,
    };

    let proc_info = ProcessInfo {
        pid: 0,
        name: params.process.clone(),
        exe: PathBuf::from(format!("/usr/bin/{}", params.process)),
        uid: 1000,
        cmdline: params.process.clone(),
    };

    let event = ConnectionEvent::new(socket, Some(proc_info));
    let decision = engine.evaluate(&event);

    let result = json!({
        "process": params.process,
        "port": params.port,
        "remote_addr": remote_addr.to_string(),
        "verdict": format!("{:?}", decision.verdict),
        "matched_rule": decision.matched_rule_id,
        "reason": decision.reason,
        "enforced": decision.enforced,
        "direction": format!("{:?}", event.direction),
        "policy": {
            "mode": format!("{:?}", engine.policy.mode),
            "default_verdict": format!("{:?}", engine.policy.default_verdict),
            "rule_count": engine.policy.rule_count(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
