//! MCP Efficacy Configuration Management
//!
//! Loads configuration from ~/.claude/mcp_efficacy_config.toml
//! Provides runtime configuration for feature flags, thresholds, and cleanup.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Feature flags for gradual rollout (Problem 3 fix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Master enable/disable switch
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rollout percentage (0-100) for canary deployment
    #[serde(default = "default_100")]
    pub rollout_percentage: u8,

    /// Session sampling rate (0.0-1.0)
    #[serde(default = "default_one")]
    pub sampling_rate: f64,
}

/// Threshold configuration for CTVP validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    /// Target CAR for Phase 2 validation
    #[serde(default = "default_car")]
    pub car_target: f64,

    /// Minimum sessions before CAR is valid
    #[serde(default = "default_min_sessions")]
    pub min_sessions: u32,

    /// Alert threshold for CAR drops
    #[serde(default = "default_car_alert")]
    pub car_alert_threshold: f64,

    /// Alert when suggestion rate drops below this fraction of baseline
    #[serde(default = "default_suggestion_alert")]
    pub suggestion_rate_alert: f64,
}

/// Cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Days to retain events
    #[serde(default = "default_retention")]
    pub retention_days: u32,

    /// Maximum events to keep (0 = unlimited)
    #[serde(default)]
    pub max_events: usize,

    /// Auto-cleanup on each write
    #[serde(default = "default_true")]
    pub auto_cleanup: bool,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Include per-tool breakdown
    #[serde(default = "default_true")]
    pub include_tool_breakdown: bool,

    /// Top N tools to show
    #[serde(default = "default_top_tools")]
    pub top_tools_count: usize,

    /// Trend analysis periods in hours
    #[serde(default = "default_trend_periods")]
    pub trend_periods: Vec<u32>,
}

/// Custom keyword mapping (Problem 4 fix)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordMapping {
    pub keywords: Vec<String>,
    pub tools: Vec<String>,
    pub category: String,
    #[serde(default)]
    pub use_case: String,
}

/// Complete MCP Efficacy Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEfficacyConfig {
    #[serde(default)]
    pub feature_flags: FeatureFlags,

    #[serde(default)]
    pub thresholds: Thresholds,

    #[serde(default)]
    pub cleanup: CleanupConfig,

    #[serde(default)]
    pub reporting: ReportingConfig,

    #[serde(default)]
    pub keywords: HashMap<String, KeywordMapping>,
}

// Default value functions
fn default_true() -> bool {
    true
}
fn default_100() -> u8 {
    100
}
fn default_one() -> f64 {
    1.0
}
fn default_car() -> f64 {
    0.80
}
fn default_min_sessions() -> u32 {
    10
}
fn default_car_alert() -> f64 {
    0.70
}
fn default_suggestion_alert() -> f64 {
    0.5
}
fn default_retention() -> u32 {
    30
}
fn default_top_tools() -> usize {
    10
}
fn default_trend_periods() -> Vec<u32> {
    vec![24, 168, 720]
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enabled: true,
            rollout_percentage: 100,
            sampling_rate: 1.0,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            car_target: 0.80,
            min_sessions: 10,
            car_alert_threshold: 0.70,
            suggestion_rate_alert: 0.5,
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            max_events: 10000,
            auto_cleanup: true,
        }
    }
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            include_tool_breakdown: true,
            top_tools_count: 10,
            trend_periods: vec![24, 168, 720],
        }
    }
}

impl Default for McpEfficacyConfig {
    fn default() -> Self {
        Self {
            feature_flags: FeatureFlags::default(),
            thresholds: Thresholds::default(),
            cleanup: CleanupConfig::default(),
            reporting: ReportingConfig::default(),
            keywords: HashMap::new(),
        }
    }
}

impl McpEfficacyConfig {
    /// Config file path
    pub fn config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("mcp_efficacy_config.toml")
    }

    /// Load config from file, returns default if not found
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| toml::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Check if tracking should be enabled for this session
    /// Implements gradual rollout via session_id hash
    pub fn should_track(&self, session_id: &str) -> bool {
        if !self.feature_flags.enabled {
            return false;
        }

        if self.feature_flags.rollout_percentage >= 100 {
            return true;
        }

        if self.feature_flags.rollout_percentage == 0 {
            return false;
        }

        // Deterministic rollout based on session_id hash
        // Hash to 0-99 range, compare to rollout percentage
        let hash = session_id
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_add(b as u64));
        (hash % 100) < self.feature_flags.rollout_percentage as u64
    }

    /// Check if this session should be sampled
    pub fn should_sample(&self) -> bool {
        if self.feature_flags.sampling_rate >= 1.0 {
            return true;
        }
        // Use random sampling
        rand_simple() < self.feature_flags.sampling_rate
    }

    /// Get custom keywords merged with defaults
    pub fn get_custom_keywords(&self) -> &HashMap<String, KeywordMapping> {
        &self.keywords
    }
}

/// Simple deterministic "random" based on timestamp
fn rand_simple() -> f64 {
    let now = crate::state::now();
    let frac = now.fract();
    frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = McpEfficacyConfig::default();
        assert!(cfg.feature_flags.enabled);
        assert_eq!(cfg.feature_flags.rollout_percentage, 100);
        assert_eq!(cfg.thresholds.car_target, 0.80);
    }

    #[test]
    fn test_should_track_enabled() {
        let cfg = McpEfficacyConfig::default();
        assert!(cfg.should_track("any-session"));
    }

    #[test]
    fn test_should_track_disabled() {
        let mut cfg = McpEfficacyConfig::default();
        cfg.feature_flags.enabled = false;
        assert!(!cfg.should_track("any-session"));
    }

    #[test]
    fn test_rollout_zero() {
        let mut cfg = McpEfficacyConfig::default();
        cfg.feature_flags.rollout_percentage = 0;
        assert!(!cfg.should_track("any-session"));
    }

    #[test]
    fn test_rollout_partial() {
        let mut cfg = McpEfficacyConfig::default();
        cfg.feature_flags.rollout_percentage = 50;
        // Should be deterministic for same session_id
        let result1 = cfg.should_track("session-123");
        let result2 = cfg.should_track("session-123");
        assert_eq!(result1, result2);
    }
}
