//! Sentinel MCP tools — SSH brute-force protection queries.
//!
//! Exposes nexcore-sentinel functionality as MCP tools:
//! - `sentinel_status`: Get default config and engine stats
//! - `sentinel_check_ip`: Check if an IP is whitelisted
//! - `sentinel_parse_line`: Parse an auth log line into an AuthEvent
//! - `sentinel_config_defaults`: Show default configuration values

use crate::params::{SentinelCheckIpParams, SentinelConfigDefaultsParams, SentinelParseLineParams};
use nexcore_sentinel::config::SentinelConfig;
use nexcore_sentinel::parser::parse_line;
use nexcore_sentinel::whitelist::Whitelist;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// sentinel_status — show default config values + structure
// ---------------------------------------------------------------------------

/// Return default sentinel configuration and architecture overview.
pub fn sentinel_status() -> Result<CallToolResult, McpError> {
    let config = SentinelConfig::default();
    let result = json!({
        "config": {
            "max_retry": config.max_retry,
            "ban_time_secs": config.ban_time_secs,
            "find_time_secs": config.find_time_secs,
            "log_path": config.log_path.display().to_string(),
            "chain_name": config.chain_name,
            "block_type": config.block_type,
            "port": config.port,
            "whitelist": config.whitelist,
        },
        "architecture": "auth.log → [watcher] → [parser] → [tracker] → [firewall]",
        "primitives": {
            "T1_Sequence": "Auth log lines streamed via inotify",
            "T1_Mapping": "IP → failure timestamps (sliding window)",
            "T1_State": "Ban records, failure counts, persistence",
            "T1_Exists": "IP-in-banlist check, whitelist membership",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// sentinel_check_ip — test if an IP is in a whitelist
// ---------------------------------------------------------------------------

/// Check if an IP address would be whitelisted by sentinel.
pub fn sentinel_check_ip(params: SentinelCheckIpParams) -> Result<CallToolResult, McpError> {
    let ip: std::net::IpAddr = params
        .ip
        .parse()
        .map_err(|e| McpError::invalid_params(format!("Invalid IP: {e}"), None))?;

    // Use provided CIDRs or fall back to defaults
    let cidrs = if params.whitelist_cidrs.is_empty() {
        vec!["127.0.0.1/8".to_string(), "::1/128".to_string()]
    } else {
        params.whitelist_cidrs
    };

    let wl = Whitelist::new(&cidrs)
        .map_err(|e| McpError::invalid_params(format!("Invalid CIDR: {e}"), None))?;

    let is_whitelisted = wl.contains(ip);

    let result = json!({
        "ip": params.ip,
        "whitelisted": is_whitelisted,
        "checked_against": cidrs,
        "action": if is_whitelisted { "SKIP — never ban" } else { "TRACK — eligible for ban" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// sentinel_parse_line — parse an auth log line
// ---------------------------------------------------------------------------

/// Parse a syslog auth line into a structured AuthEvent.
pub fn sentinel_parse_line(params: SentinelParseLineParams) -> Result<CallToolResult, McpError> {
    let event = parse_line(&params.line)
        .map_err(|e| McpError::internal_error(format!("Parse error: {e}"), None))?;

    let result = match event {
        Some(evt) => json!({
            "matched": true,
            "event_type": format!("{}", evt).split(' ').next().unwrap_or("unknown"),
            "ip": evt.ip().to_string(),
            "user": evt.user(),
            "timestamp": evt.timestamp().to_rfc3339(),
            "display": format!("{evt}"),
        }),
        None => json!({
            "matched": false,
            "line": params.line,
            "reason": "Line does not match 'Failed password' or 'Invalid user' patterns",
        }),
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// sentinel_config_defaults — show config with descriptions
// ---------------------------------------------------------------------------

/// Return annotated default configuration with descriptions for each field.
pub fn sentinel_config_defaults(
    _params: SentinelConfigDefaultsParams,
) -> Result<CallToolResult, McpError> {
    let result = json!({
        "defaults": {
            "max_retry": {
                "value": 3,
                "description": "Number of failures before banning (matches fail2ban jail.local)",
            },
            "ban_time_secs": {
                "value": 86400,
                "description": "Ban duration in seconds (24 hours, matches fail2ban)",
            },
            "find_time_secs": {
                "value": 600,
                "description": "Sliding window for counting failures (10 minutes)",
            },
            "log_path": {
                "value": "/var/log/auth.log",
                "description": "Auth log file to watch",
            },
            "chain_name": {
                "value": "f2b-sentinel",
                "description": "iptables chain name (separate from fail2ban for safe migration)",
            },
            "block_type": {
                "value": "REJECT",
                "description": "iptables target: REJECT (sends RST) or DROP (silent)",
            },
            "port": {
                "value": 22,
                "description": "SSH port to protect",
            },
            "whitelist": {
                "value": ["127.0.0.1/8", "::1/128"],
                "description": "CIDR ranges that are never banned",
            },
        },
        "sample_toml": SentinelConfig::sample_toml(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
