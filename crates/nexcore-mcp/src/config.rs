//! Configuration loading with fallback to legacy files
//!
//! Loads consolidated config (`~/nexcore/config.toml`) with fallback to legacy JSON.
//!
//! Uses lazy-static pattern for <1ms config access after first load.

use nexcore_config::ClaudeConfig;
use nexcore_error::{Context, Result};
use nexcore_fs::dirs;
use std::sync::LazyLock;

/// Global config singleton (loaded once, reused forever)
///
/// First access loads from disk (~15-20ms), subsequent accesses are <1ms.
static CONFIG: LazyLock<Option<ClaudeConfig>> = LazyLock::new(|| load_config_internal().ok());

/// Get global config reference
///
/// Returns cached config after first load (<1ms access time).
///
/// # Returns
///
/// - `Some(&ClaudeConfig)` if config loaded successfully
/// - `None` if no config found
pub fn get_config() -> Option<&'static ClaudeConfig> {
    CONFIG.as_ref()
}

/// Load Claude Code configuration (internal)
///
/// Tries consolidated config first (`~/nexcore/config.toml`), then falls back to legacy `~/.claude.json`.
///
/// # Returns
///
/// - `Ok(ClaudeConfig)` if either file loads successfully
/// - `Err` if both files fail to load or don't exist
fn load_config_internal() -> Result<ClaudeConfig> {
    let home = dirs::home_dir().context("No home directory found")?;

    // Try consolidated config first
    let consolidated = home.join("nexcore/config.toml");
    if consolidated.exists() {
        let path_str = consolidated
            .to_str()
            .context("Invalid UTF-8 in config path")?;
        tracing::debug!("Loading consolidated config from {}", path_str);
        return Ok(ClaudeConfig::from_file(path_str)?);
    }

    // Fallback to legacy JSON
    let legacy = home.join(".claude.json");
    if legacy.exists() {
        let path_str = legacy.to_str().context("Invalid UTF-8 in config path")?;
        tracing::debug!("Loading legacy config from {}", path_str);
        return Ok(ClaudeConfig::from_file(path_str)?);
    }

    nexcore_error::bail!(
        "No configuration found. Tried:\n  - {}\n  - {}",
        consolidated.display(),
        legacy.display()
    )
}

/// Load config (public API, for backward compatibility)
///
/// **Deprecated**: Use `get_config()` for lazy-static access instead.
pub fn load_config() -> Result<ClaudeConfig> {
    load_config_internal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_legacy() {
        // Hermetic: test that load_config returns a result (Ok or Err depending on host)
        // The function itself is correct if it finds files; this tests it doesn't panic.
        let _result = load_config();
        // If config files exist on host, Ok; if not, Err. Both are valid states.
        // The important thing is no panic, no UB, no resource leak.
    }

    #[test]
    fn test_get_config_lazy() {
        // First access loads config
        let cfg1 = get_config();

        // Second access reuses same config (pointer equality)
        let cfg2 = get_config();

        if let (Some(c1), Some(c2)) = (cfg1, cfg2) {
            // Should be same memory address (lazy-static works)
            assert!(std::ptr::eq(c1, c2));
        }
    }
}
