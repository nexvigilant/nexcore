//! TOML DSL configuration types.
//!
//! Defines the structure for declarative hook rules in TOML format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration structure for hooks.toml.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TomlConfig {
    /// Metadata about the configuration.
    #[serde(default)]
    pub meta: TomlMeta,

    /// List of rules.
    #[serde(default)]
    pub rules: Vec<TomlRule>,
}

/// Metadata section.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TomlMeta {
    /// Configuration version.
    #[serde(default = "default_version")]
    pub version: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Author or team name.
    #[serde(default)]
    pub author: Option<String>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// A single rule definition.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TomlRule {
    /// Rule name (must be unique).
    pub name: String,

    /// Hook event type (PreToolUse, PostToolUse, etc.).
    pub event: String,

    /// Tool matcher pattern (e.g., "Bash", "Write|Edit", "mcp__*__*").
    #[serde(default)]
    pub matcher: Option<String>,

    /// Conditions that must match for the rule to trigger.
    #[serde(default, rename = "when")]
    pub condition: Option<TomlCondition>,

    /// Action to take when the rule matches.
    pub action: TomlAction,

    /// Rule priority (higher = checked first).
    #[serde(default)]
    pub priority: i32,

    /// Whether the rule is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Condition types for matching.
///
/// TOML structure supports nested format:
/// ```toml
/// [rules.when]
/// input.command.contains = "rm -rf"
/// ```
///
/// Or array format for any/all:
/// ```toml
/// [rules.when]
/// any = [
///     { "input.command.contains" = "rm -rf" },
///     { "input.command.contains" = "sudo rm" },
/// ]
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TomlCondition {
    /// All conditions must match (AND logic).
    All { all: Vec<TomlCondition> },

    /// Any condition must match (OR logic).
    Any { any: Vec<TomlCondition> },

    /// Negate a condition.
    Not { not: Box<TomlCondition> },

    /// Nested input structure (from TOML [rules.when] section).
    Input { input: InputCondition },

    /// Flat field condition (from inline tables in arrays).
    FlatField(HashMap<String, String>),
}

/// Input field condition structure.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InputCondition {
    /// input.command.X conditions
    Command { command: OperatorCondition },

    /// input.file_path.X conditions
    FilePath { file_path: OperatorCondition },

    /// Generic input condition
    Generic(OperatorCondition),
}

/// Operator conditions (contains, matches, ends_with, etc.)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OperatorCondition {
    /// Contains substring
    #[serde(default)]
    pub contains: Option<String>,

    /// Matches regex pattern
    #[serde(default)]
    pub matches: Option<String>,

    /// Ends with suffix
    #[serde(default)]
    pub ends_with: Option<String>,

    /// Starts with prefix
    #[serde(default)]
    pub starts_with: Option<String>,

    /// Equals exactly
    #[serde(default)]
    pub equals: Option<String>,
}

/// Actions to take when a rule matches.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TomlAction {
    /// Simple deny with message.
    Deny { deny: String },

    /// Simple allow (explicit).
    Allow { allow: bool },

    /// Ask user for confirmation.
    Ask { ask: String },

    /// Warn but allow.
    Warn { warn: String },

    /// Run an external command.
    Run { run: String },

    /// Log to a file.
    Log {
        log: LogAction,
    },

    /// Multiple actions.
    Multiple(Vec<TomlAction>),
}

/// Log action configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogAction {
    /// Path to log file.
    pub path: String,

    /// Log format (json, text).
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_format() -> String {
    "json".to_string()
}

impl TomlConfig {
    /// Load configuration from a TOML string.
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Load configuration from a file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::parse(&content).map_err(ConfigError::Parse)
    }

    /// Serialize to TOML string.
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

/// Errors that can occur when loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error reading file.
    Io(std::io::Error),
    /// TOML parsing error.
    Parse(toml::de::Error),
    /// Validation error.
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
[meta]
version = "1.0"
description = "Test config"

[[rules]]
name = "block-rm"
event = "PreToolUse"
matcher = "Bash"

[rules.when]
input.command.contains = "rm -rf"

[rules.action]
deny = "Dangerous command blocked"
"#;

        let config = TomlConfig::parse(toml).unwrap();
        assert_eq!(config.meta.version, "1.0");
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "block-rm");
    }

    #[test]
    fn test_parse_multiple_rules() {
        let toml = r#"
[[rules]]
name = "rule1"
event = "PreToolUse"
matcher = "Bash"
[rules.action]
deny = "Blocked"

[[rules]]
name = "rule2"
event = "PostToolUse"
matcher = "Write"
[rules.action]
warn = "Warning"
"#;

        let config = TomlConfig::parse(toml).unwrap();
        assert_eq!(config.rules.len(), 2);
    }

    #[test]
    fn test_parse_any_condition() {
        let toml = r#"
[[rules]]
name = "block-dangerous"
event = "PreToolUse"
matcher = "Bash"

[rules.when]
any = [
    { "input.command.contains" = "rm -rf" },
    { "input.command.contains" = "sudo rm" },
]

[rules.action]
deny = "Blocked"
"#;

        let config = TomlConfig::parse(toml).unwrap();
        assert!(config.rules[0].condition.is_some());
    }

    #[test]
    fn test_parse_log_action() {
        let toml = r#"
[[rules]]
name = "audit"
event = "PostToolUse"

[rules.action.log]
path = "/var/log/audit.jsonl"
format = "json"
"#;

        let config = TomlConfig::parse(toml).unwrap();
        match &config.rules[0].action {
            TomlAction::Log { log } => {
                assert_eq!(log.path, "/var/log/audit.jsonl");
            }
            _ => panic!("Expected Log action"),
        }
    }
}
