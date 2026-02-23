//! CTVP Configuration Management

use nexcore_error::Error;
use nexcore_fs::dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parse error
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    /// TOML serialize error
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Whether CTVP tracking is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Rollout percentage (0-100) for canary
    #[serde(default = "default_rollout")]
    pub rollout_percentage: u8,
}

fn default_true() -> bool {
    true
}
fn default_rollout() -> u8 {
    100
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            enabled: true,
            rollout_percentage: 100,
        }
    }
}

/// Threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    /// Target CAR for Phase 2 validation
    #[serde(default = "default_car_target")]
    pub car_target: f64,
    /// Alert threshold for Phase 4 surveillance
    #[serde(default = "default_alert")]
    pub car_alert_threshold: f64,
    /// Minimum sessions before validation
    #[serde(default = "default_min_sessions")]
    pub min_sessions: u32,
}

fn default_car_target() -> f64 {
    super::DEFAULT_CAR_THRESHOLD
}
fn default_alert() -> f64 {
    super::DEFAULT_ALERT_THRESHOLD
}
fn default_min_sessions() -> u32 {
    super::DEFAULT_MIN_OBSERVATIONS
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            car_target: super::DEFAULT_CAR_THRESHOLD,
            car_alert_threshold: super::DEFAULT_ALERT_THRESHOLD,
            min_sessions: super::DEFAULT_MIN_OBSERVATIONS,
        }
    }
}

/// CTVP configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CtvpConfig {
    /// Feature flags
    #[serde(default)]
    pub feature_flags: FeatureFlags,
    /// Thresholds
    #[serde(default)]
    pub thresholds: Thresholds,
}

impl CtvpConfig {
    /// Loads config from default path
    ///
    /// # Returns
    /// Loaded config or default if file doesn't exist
    pub fn load() -> Self {
        Self::load_from(Self::default_path()).unwrap_or_default()
    }

    /// Loads config from specified path
    ///
    /// # Arguments
    /// * `path` - Path to config file
    ///
    /// # Returns
    /// Result with loaded config or error
    pub fn load_from(path: PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Saves config to default path
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to(Self::default_path())
    }

    /// Saves config to specified path
    ///
    /// # Arguments
    /// * `path` - Path to save config file
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn save_to(&self, path: PathBuf) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Returns default config path
    ///
    /// # Returns
    /// PathBuf to default config location
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("ctvp_config.toml")
    }

    /// Returns data directory
    ///
    /// # Returns
    /// PathBuf to data directory
    pub fn data_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("ctvp")
    }

    /// Returns events file path
    ///
    /// # Returns
    /// PathBuf to events JSONL file
    pub fn events_path() -> PathBuf {
        Self::data_dir().join("events.jsonl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CtvpConfig::default();
        assert!(config.feature_flags.enabled);
        assert!((config.thresholds.car_target - 0.80).abs() < 0.001);
    }

    #[test]
    fn test_serialization() {
        let config = CtvpConfig::default();
        let toml_str = toml::to_string(&config).expect("serialize");
        let parsed: CtvpConfig = toml::from_str(&toml_str).expect("parse");
        assert_eq!(config.feature_flags.enabled, parsed.feature_flags.enabled);
    }
}
