//! Terminal configuration — tier-based resource limits and sandbox policies.
//!
//! Maps `SubscriptionTier` to concrete terminal capabilities: CPU, memory,
//! session limits, network policy, and syscall profiles.

use serde::{Deserialize, Serialize};
use vr_core::tenant::SubscriptionTier;

/// Per-tenant sandbox configuration derived from subscription tier.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// CPU limit in millicores (1000 = 1 CPU core).
    pub cpu_limit_millicores: u32,
    /// Memory limit in megabytes.
    pub memory_limit_mb: u32,
    /// Disk limit in megabytes.
    pub disk_limit_mb: u32,
    /// Maximum concurrent terminal sessions per user.
    pub max_concurrent_sessions: u32,
    /// Seconds of inactivity before session is suspended.
    pub idle_timeout_secs: u64,
    /// Maximum session duration in seconds.
    pub max_session_duration_secs: u64,
    /// Network egress policy.
    pub network_egress: NetworkPolicy,
}

/// Network egress policy for sandboxed terminal sessions.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    /// No outbound network access (empty network namespace).
    None,
    /// Allowlisted domains only.
    Restricted,
    /// Unrestricted egress (Enterprise/Custom tiers).
    Full,
}

impl SandboxConfig {
    /// Derive sandbox configuration from the tenant's subscription tier.
    #[must_use]
    pub fn from_tier(tier: &SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Academic => Self {
                cpu_limit_millicores: 500,
                memory_limit_mb: 256,
                disk_limit_mb: 512,
                max_concurrent_sessions: 1,
                idle_timeout_secs: 600,
                max_session_duration_secs: 3600,
                network_egress: NetworkPolicy::None,
            },
            SubscriptionTier::Explorer => Self {
                cpu_limit_millicores: 500,
                memory_limit_mb: 256,
                disk_limit_mb: 512,
                max_concurrent_sessions: 1,
                idle_timeout_secs: 300,
                max_session_duration_secs: 1800,
                network_egress: NetworkPolicy::None,
            },
            SubscriptionTier::Accelerator => Self {
                cpu_limit_millicores: 1000,
                memory_limit_mb: 512,
                disk_limit_mb: 2048,
                max_concurrent_sessions: 2,
                idle_timeout_secs: 1800,
                max_session_duration_secs: 14400,
                network_egress: NetworkPolicy::Restricted,
            },
            SubscriptionTier::Enterprise | SubscriptionTier::Custom => Self {
                cpu_limit_millicores: 2000,
                memory_limit_mb: 2048,
                disk_limit_mb: 10240,
                max_concurrent_sessions: 5,
                idle_timeout_secs: 3600,
                max_session_duration_secs: 28800,
                network_egress: NetworkPolicy::Full,
            },
            _ => Self {
                cpu_limit_millicores: 500,
                memory_limit_mb: 256,
                disk_limit_mb: 512,
                max_concurrent_sessions: 1,
                idle_timeout_secs: 300,
                max_session_duration_secs: 1800,
                network_egress: NetworkPolicy::None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn academic_tier_has_restrictive_limits() {
        let config = SandboxConfig::from_tier(&SubscriptionTier::Academic);
        assert_eq!(config.max_concurrent_sessions, 1);
        assert_eq!(config.cpu_limit_millicores, 500);
        assert_eq!(config.network_egress, NetworkPolicy::None);
    }

    #[test]
    fn enterprise_tier_has_generous_limits() {
        let config = SandboxConfig::from_tier(&SubscriptionTier::Enterprise);
        assert_eq!(config.max_concurrent_sessions, 5);
        assert_eq!(config.cpu_limit_millicores, 2000);
        assert_eq!(config.network_egress, NetworkPolicy::Full);
    }

    #[test]
    fn custom_matches_enterprise() {
        let enterprise = SandboxConfig::from_tier(&SubscriptionTier::Enterprise);
        let custom = SandboxConfig::from_tier(&SubscriptionTier::Custom);
        assert_eq!(enterprise.cpu_limit_millicores, custom.cpu_limit_millicores);
        assert_eq!(enterprise.memory_limit_mb, custom.memory_limit_mb);
    }

    #[test]
    fn network_policy_serde_roundtrip() {
        let json = serde_json::to_string(&NetworkPolicy::Restricted);
        assert!(json.is_ok());
        let json = json.unwrap_or_default();
        assert_eq!(json, "\"restricted\"");
    }
}
