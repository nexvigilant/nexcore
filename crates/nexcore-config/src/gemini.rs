//! Gemini hooks configuration types
//!
//! Type-safe representation of `.gemini.json` configuration.

use nexcore_error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Root Gemini configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub hooks: Vec<HookDefinition>,
}

impl GeminiConfig {
    /// Load configuration from JSON file
    pub fn from_file(path: &str) -> Result<Self> {
        use nexcore_error::Context;
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read Gemini config: {}", path))?;
        let config = serde_json::from_str(&content)
            .context(format!("Failed to parse Gemini config: {}", path))?;
        Ok(config)
    }
}

/// Hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// Hook identifier
    pub name: String,

    /// Hook type
    #[serde(rename = "type")]
    pub hook_type: HookType,

    /// Matcher pattern
    pub matcher: String,

    /// Command to execute
    pub command: PathBuf,

    /// Timeout in milliseconds
    pub timeout: u64,

    /// Human-readable description
    pub description: String,
}

/// Hook type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    /// Run a shell command
    RunCommand,

    /// Validation hook
    Validation,

    /// Post-processing hook
    PostProcess,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_definition_deserialization() {
        let json = r#"{
            "name": "git-safety",
            "type": "run_command",
            "matcher": "run_shell_command",
            "command": "/home/user/hooks/git_safety.py",
            "timeout": 2000,
            "description": "Prevents force pushes to protected branches."
        }"#;

        let hook: HookDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(hook.name, "git-safety");
        assert_eq!(hook.timeout, 2000);
        assert!(matches!(hook.hook_type, HookType::RunCommand));
    }
}
