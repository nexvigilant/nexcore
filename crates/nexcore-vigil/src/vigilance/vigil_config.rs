//! # Vigilance Configuration
//!
//! TOML-based configuration for the vigilance daemon.
//!
//! ## Tier: T2-P (π + ∂)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Top-level vigilance configuration.
///
/// Tier: T2-P (π + ∂), dominant π
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilConfig {
    /// Path to the vigilance ledger WAL file
    #[serde(default = "default_wal_path")]
    pub wal_path: PathBuf,

    /// How often the watcher polls sources (ms)
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,

    /// Maximum events to buffer before backpressure
    #[serde(default = "default_max_buffer")]
    pub max_event_buffer: usize,

    /// Watch sources
    #[serde(default)]
    pub sources: Vec<SourceConfig>,

    /// Boundary specifications
    #[serde(default)]
    pub boundaries: Vec<BoundaryConfig>,

    /// Consequence chain configuration
    #[serde(default)]
    pub consequences: Vec<ConsequenceConfig>,

    /// Enable WAL flush on every ledger append
    #[serde(default = "default_sync_wal")]
    pub sync_wal: bool,
}

/// Configuration for a single boundary spec.
///
/// Tier: T2-P (∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryConfig {
    pub name: String,
    #[serde(default)]
    pub source_filter: Option<String>,
    #[serde(default)]
    pub kind_filter: Option<String>,
    pub threshold: ThresholdConfig,
    #[serde(default = "default_cooldown_ms")]
    pub cooldown_ms: u64,
}

/// Threshold configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ThresholdConfig {
    #[serde(rename = "count_exceeds")]
    CountExceeds { count: u64, window_ms: u64 },
    #[serde(rename = "severity_at_least")]
    SeverityAtLeast { level: String },
    #[serde(rename = "payload_match")]
    PayloadMatch { json_path: String, pattern: String },
    #[serde(rename = "always")]
    Always,
}

/// Configuration for a watch source.
///
/// Tier: T2-P (ν)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SourceConfig {
    /// Interval-based timer source
    #[serde(rename = "timer")]
    Timer {
        /// Source name
        name: String,
        /// Interval in milliseconds
        interval_ms: u64,
    },

    /// File system watcher source
    #[serde(rename = "filesystem")]
    FileSystem {
        /// Source name
        name: String,
        /// Paths to watch
        paths: Vec<String>,
    },
}

/// Configuration for a consequence in the escalation chain.
///
/// Tier: T2-P (∝)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConsequenceConfig {
    /// Log to tracing
    #[serde(rename = "log")]
    Log {
        /// Escalation level (observe, warn, alert, act, audit)
        #[serde(default = "default_level_observe")]
        level: String,
    },

    /// Shell command execution
    #[serde(rename = "shell")]
    Shell {
        /// Command to execute
        command: String,
        /// Timeout in milliseconds
        #[serde(default = "default_shell_timeout")]
        timeout_ms: u64,
    },

    /// Write notification JSON file
    #[serde(rename = "notify")]
    Notify {
        /// Directory to write notification files
        dir: String,
    },

    /// HTTP POST webhook
    #[serde(rename = "webhook")]
    Webhook {
        /// Webhook URL
        url: String,
        /// Timeout in milliseconds
        #[serde(default = "default_shell_timeout")]
        timeout_ms: u64,
    },
}

fn default_level_observe() -> String {
    "observe".to_string()
}

fn default_shell_timeout() -> u64 {
    5000
}

impl Default for VigilConfig {
    fn default() -> Self {
        Self {
            wal_path: default_wal_path(),
            poll_interval_ms: default_poll_interval_ms(),
            max_event_buffer: default_max_buffer(),
            sources: Vec::new(),
            boundaries: Vec::new(),
            consequences: Vec::new(),
            sync_wal: default_sync_wal(),
        }
    }
}

impl VigilConfig {
    /// Load from a TOML file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, crate::vigilance::error::VigilError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| crate::vigilance::error::VigilError::Config(e.to_string()))
    }

    /// Poll interval as Duration.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }

    /// Cooldown duration for a boundary.
    pub fn cooldown_duration(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
}

fn default_wal_path() -> PathBuf {
    PathBuf::from("/tmp/vigil-ledger.wal")
}

fn default_poll_interval_ms() -> u64 {
    1000
}

fn default_max_buffer() -> usize {
    10_000
}

fn default_cooldown_ms() -> u64 {
    5000
}

fn default_sync_wal() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = VigilConfig::default();
        assert_eq!(cfg.poll_interval_ms, 1000);
        assert_eq!(cfg.max_event_buffer, 10_000);
        assert!(cfg.boundaries.is_empty());
        assert!(cfg.sync_wal);
    }

    #[test]
    fn poll_interval_duration() {
        let cfg = VigilConfig::default();
        assert_eq!(cfg.poll_interval(), Duration::from_millis(1000));
    }

    #[test]
    fn config_toml_roundtrip() {
        let cfg = VigilConfig::default();
        let toml_str = toml::to_string(&cfg);
        assert!(toml_str.is_ok());
        let parsed: Result<VigilConfig, _> = toml::from_str(&toml_str.unwrap_or_default());
        assert!(parsed.is_ok());
    }

    #[test]
    fn threshold_config_deserialization() {
        let toml_str = r#"
            name = "test"
            cooldown_ms = 3000
            [threshold]
            type = "count_exceeds"
            count = 5
            window_ms = 60000
        "#;
        let cfg: Result<BoundaryConfig, _> = toml::from_str(toml_str);
        assert!(cfg.is_ok());
    }
}
