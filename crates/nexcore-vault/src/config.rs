//! Configuration for nexcore-vault.
//!
//! Loaded from TOML; every field has a serde default.

use nexcore_fs::dirs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VaultError};

/// Tier: T3 — Complete vault configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VaultConfig {
    /// Path to the encrypted vault file.
    pub vault_path: PathBuf,

    /// PBKDF2 iteration count (OWASP 2023: 600,000 for HMAC-SHA256).
    pub pbkdf2_iterations: u32,

    /// Whether to create a backup before writing.
    pub backup_on_write: bool,

    /// Unix file permissions for the vault file (octal).
    pub file_mode: u32,

    /// Unix directory permissions for the vault directory (octal).
    pub dir_mode: u32,

    /// Config file path (not serialized, set at load time).
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        let vault_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexcore")
            .join("vault");

        Self {
            vault_path: vault_dir.join("secrets.enc"),
            pbkdf2_iterations: 600_000,
            backup_on_write: true,
            file_mode: 0o600,
            dir_mode: 0o700,
            config_path: None,
        }
    }
}

impl VaultConfig {
    /// Load config from a TOML file. Falls back to defaults if file doesn't exist.
    ///
    /// # Errors
    /// Returns `VaultError::Io` if the file exists but cannot be read,
    /// or `VaultError::Toml` if the TOML is malformed.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::info!(
                "Config file not found at {}, using defaults",
                path.display()
            );
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path).map_err(|e| VaultError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut config: Self = toml::from_str(&content)?;
        config.config_path = Some(path.to_path_buf());
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values.
    ///
    /// # Errors
    /// Returns `VaultError::Config` if any value is out of range.
    pub fn validate(&self) -> Result<()> {
        if self.pbkdf2_iterations < 100_000 {
            return Err(VaultError::Config(
                "pbkdf2_iterations must be at least 100,000".into(),
            ));
        }
        if self.vault_path.as_os_str().is_empty() {
            return Err(VaultError::Config("vault_path must not be empty".into()));
        }
        Ok(())
    }

    /// Get the default config file path.
    #[must_use]
    pub fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexcore")
            .join("vault")
            .join("vault.toml")
    }

    /// Generate a sample TOML config string with defaults.
    #[must_use]
    pub fn sample_toml() -> String {
        let cfg = Self::default();
        toml::to_string_pretty(&cfg).unwrap_or_default()
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let cfg = VaultConfig::default();
        assert_eq!(cfg.pbkdf2_iterations, 600_000);
        assert!(cfg.backup_on_write);
        assert_eq!(cfg.file_mode, 0o600);
        assert_eq!(cfg.dir_mode, 0o700);
        assert!(cfg.vault_path.to_string_lossy().contains("secrets.enc"));
    }

    #[test]
    fn validate_rejects_low_iterations() {
        let mut cfg = VaultConfig::default();
        cfg.pbkdf2_iterations = 1000;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_accepts_defaults() {
        let cfg = VaultConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn sample_toml_is_parseable() {
        let toml_str = VaultConfig::sample_toml();
        let parsed: std::result::Result<VaultConfig, _> = toml::from_str(&toml_str);
        assert!(parsed.is_ok());
    }

    #[test]
    fn load_nonexistent_returns_defaults() {
        let path = Path::new("/tmp/nonexistent-vault-config-12345.toml");
        let cfg = VaultConfig::load(path);
        assert!(cfg.is_ok());
    }

    #[test]
    fn default_config_path_contains_vault() {
        let path = VaultConfig::default_config_path();
        assert!(path.to_string_lossy().contains("vault"));
    }
}
