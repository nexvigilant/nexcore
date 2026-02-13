//! # Configuration Management
//!
//! Environment-aware configuration loading with sensible defaults.

use serde::{Deserialize, Serialize};

/// Runtime environment for configuration selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment (local)
    #[default]
    Development,
    /// Staging environment (pre-production)
    Staging,
    /// Production environment
    Production,
    /// Testing environment (CI/CD)
    Testing,
}

impl std::str::FromStr for Environment {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "staging" | "stage" => Self::Staging,
            "testing" | "test" | "ci" => Self::Testing,
            _ => Self::Development,
        })
    }
}

impl Environment {
    /// Get the current environment from RUN_MODE or NEXCORE_ENV.
    #[must_use]
    pub fn current() -> Self {
        std::env::var("NEXCORE_ENV")
            .or_else(|_| std::env::var("RUN_MODE"))
            .map(|s| s.parse().unwrap_or_default())
            .unwrap_or_default()
    }

    /// Returns true if this is a production-like environment.
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production | Self::Staging)
    }
}

/// GCP-specific configuration settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GcpSettings {
    /// GCP project ID
    pub project_id: Option<String>,
    /// Default region (e.g., "us-central1")
    pub region: String,
    /// Path to service account credentials
    pub credentials_path: Option<String>,
    /// Default location for resources
    pub location: String,
}

impl GcpSettings {
    /// Create new GCP settings with defaults.
    pub fn new() -> Self {
        Self {
            project_id: std::env::var("GCP_PROJECT_ID").ok(),
            region: std::env::var("GCP_REGION").unwrap_or_else(|_| "us-central1".to_string()),
            credentials_path: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
            location: std::env::var("GCP_LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
        }
    }
}

/// Logging configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR)
    pub level: String,
    /// Log format (text, json)
    pub format: String,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string()),
            format: std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string()),
        }
    }
}

/// NexCore application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NexCoreConfig {
    /// Current environment
    pub environment: Environment,
    /// GCP settings
    pub gcp: GcpSettings,
    /// Logging settings
    pub logging: LoggingSettings,
    /// Skills directory path
    pub skills_dir: Option<String>,
    /// Knowledge directory path
    pub knowledge_dir: Option<String>,
}

impl NexCoreConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            environment: Environment::current(),
            gcp: GcpSettings::new(),
            logging: LoggingSettings::default(),
            skills_dir: std::env::var("NEXCORE_SKILLS_DIR").ok(),
            knowledge_dir: std::env::var("NEXCORE_KNOWLEDGE_DIR").ok(),
        }
    }

    /// Get the effective skills directory.
    pub fn skills_directory(&self) -> String {
        self.skills_dir.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".claude/skills").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/.claude/skills".to_string())
        })
    }

    /// Get the effective knowledge directory.
    pub fn knowledge_directory(&self) -> String {
        self.knowledge_dir.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".claude/knowledge").to_string_lossy().to_string())
                .unwrap_or_else(|| "~/.claude/knowledge".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        assert_eq!(
            "production".parse::<Environment>(),
            Ok(Environment::Production)
        );
        assert_eq!("PROD".parse::<Environment>(), Ok(Environment::Production));
        assert_eq!("staging".parse::<Environment>(), Ok(Environment::Staging));
        assert_eq!("test".parse::<Environment>(), Ok(Environment::Testing));
        assert_eq!(
            "unknown".parse::<Environment>(),
            Ok(Environment::Development)
        );
    }

    #[test]
    fn test_is_production() {
        assert!(Environment::Production.is_production());
        assert!(Environment::Staging.is_production());
        assert!(!Environment::Development.is_production());
        assert!(!Environment::Testing.is_production());
    }

    #[test]
    fn test_config_from_env() {
        let config = NexCoreConfig::from_env();
        assert!(!config.skills_directory().is_empty());
    }
}
