//! Claude Code configuration types
//!
//! Type-safe representation of `.claude.json` configuration.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root Claude Code configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaudeConfig {
    #[serde(default)]
    pub num_startups: u32,
    #[serde(default)]
    pub install_method: String,
    #[serde(default)]
    pub auto_updates: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub auto_compact_enabled: bool,
    #[serde(default)]
    pub has_seen_tasks_hint: bool,

    #[serde(rename = "userID", default)]
    pub user_id: String,
    #[serde(default)]
    pub first_start_time: String,

    /// Project-specific configurations
    #[serde(default)]
    pub projects: HashMap<PathBuf, ProjectConfig>,

    /// Global MCP server definitions
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,

    /// Skill usage statistics
    #[serde(default)]
    pub skill_usage: HashMap<String, SkillUsageStats>,

    /// OAuth account information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_account: Option<OAuthAccount>,

    /// Cached feature gates (dynamic)
    #[serde(default)]
    pub cached_statsig_gates: HashMap<String, serde_json::Value>,

    /// Cached growth book features (dynamic)
    #[serde(default)]
    pub cached_growth_book_features: HashMap<String, serde_json::Value>,

    /// All other fields (catch-all for forward compatibility)
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

impl ClaudeConfig {
    /// Load configuration from JSON file
    pub fn from_file(path: &str) -> Result<Self> {
        use anyhow::Context;
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read Claude config: {}", path))?;
        let config = serde_json::from_str(&content)
            .context(format!("Failed to parse Claude config: {}", path))?;
        Ok(config)
    }

    /// Load from default path (~/.claude.json)
    pub fn load_default() -> Result<Self> {
        use anyhow::Context;
        let home = std::env::var("HOME").context("HOME env not set")?;
        Self::from_file(&format!("{}/.claude.json", home))
    }

    /// Save configuration to JSON file with atomic write and backup
    ///
    /// Creates a `.bak` backup before overwriting to prevent data loss.
    /// Uses atomic write pattern: write to temp, then rename.
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        use anyhow::Context;
        use std::fs;
        use std::path::Path;

        let path = Path::new(path);

        // Create backup if file exists
        if path.exists() {
            let backup_path = path.with_extension("json.bak");
            fs::copy(path, &backup_path)
                .context(format!("Failed to backup: {}", path.display()))?;
        }

        // Serialize with pretty formatting
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize Claude config")?;

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, &content)
            .context(format!("Failed to write temp: {}", temp_path.display()))?;
        fs::rename(&temp_path, path).context(format!("Failed to rename: {}", path.display()))?;

        Ok(())
    }

    /// Save to default path (~/.claude.json)
    pub fn save_default(&self) -> Result<()> {
        use anyhow::Context;
        let home = std::env::var("HOME").context("HOME env not set")?;
        self.save_to_file(&format!("{}/.claude.json", home))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MCP Server Management API
    // ─────────────────────────────────────────────────────────────────────────

    /// Add or update a global MCP server configuration
    ///
    /// # Example
    /// ```ignore
    /// config.add_mcp_server("my-server", McpServerConfig::Stdio {
    ///     command: "my-mcp-server".into(),
    ///     args: vec!["--mode".into(), "stdio".into()],
    ///     env: HashMap::new(),
    /// });
    /// ```
    pub fn add_mcp_server(&mut self, name: impl Into<String>, config: McpServerConfig) {
        self.mcp_servers.insert(name.into(), config);
    }

    /// Remove a global MCP server by name
    ///
    /// Returns the removed config if it existed.
    pub fn remove_mcp_server(&mut self, name: &str) -> Option<McpServerConfig> {
        self.mcp_servers.remove(name)
    }

    /// Check if a global MCP server exists
    pub fn has_mcp_server(&self, name: &str) -> bool {
        self.mcp_servers.contains_key(name)
    }

    /// Get a global MCP server configuration
    pub fn get_mcp_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.mcp_servers.get(name)
    }

    /// List all global MCP server names
    pub fn list_mcp_servers(&self) -> Vec<&String> {
        self.mcp_servers.keys().collect()
    }

    /// Add or update a project-specific MCP server
    pub fn add_project_mcp_server(
        &mut self,
        project_path: impl Into<PathBuf>,
        server_name: impl Into<String>,
        config: McpServerConfig,
    ) {
        let path = project_path.into();
        if let Some(project) = self.projects.get_mut(&path) {
            project.mcp_servers.insert(server_name.into(), config);
        }
    }

    /// Remove a project-specific MCP server
    pub fn remove_project_mcp_server(
        &mut self,
        project_path: &PathBuf,
        server_name: &str,
    ) -> Option<McpServerConfig> {
        self.projects
            .get_mut(project_path)
            .and_then(|p| p.mcp_servers.remove(server_name))
    }
}

/// Per-project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    #[serde(default)]
    pub allowed_tools: Vec<String>,

    #[serde(default)]
    pub mcp_context_uris: Vec<String>,

    /// Project-specific MCP servers (overrides global)
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,

    #[serde(default)]
    pub enabled_mcpjson_servers: Vec<String>,

    #[serde(default)]
    pub disabled_mcpjson_servers: Vec<String>,

    #[serde(default)]
    pub disabled_mcp_servers: Vec<String>,

    pub has_trust_dialog_accepted: bool,

    #[serde(default)]
    pub project_onboarding_seen_count: u32,

    /// Performance statistics
    #[serde(flatten)]
    pub performance: PerformanceStats,

    /// Vulnerability detection cache
    pub react_vulnerability_cache: VulnerabilityCache,

    #[serde(default)]
    pub example_files: Vec<String>,
}

/// MCP Server configuration (tagged enum for different types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpServerConfig {
    /// Standard stdio-based MCP server
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

/// Feature flags and experimental features
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    #[serde(default)]
    pub mcp_tool_search: bool,

    #[serde(default)]
    pub streaming_tool_execution2: bool,

    #[serde(default)]
    pub session_memory: bool,

    #[serde(default)]
    pub plan_mode_interview_phase: bool,

    #[serde(default)]
    pub keybinding_customization_release: bool,

    #[serde(default)]
    pub file_write_optimization: bool,

    #[serde(default)]
    pub compact_cache_prefix: bool,

    // Cached feature gates from remote
    #[serde(flatten)]
    pub cached_gates: HashMap<String, serde_json::Value>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            mcp_tool_search: true,
            streaming_tool_execution2: true,
            session_memory: false,
            plan_mode_interview_phase: false,
            keybinding_customization_release: true,
            file_write_optimization: true,
            compact_cache_prefix: true,
            cached_gates: HashMap::new(),
        }
    }
}

/// Performance statistics for a project/session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceStats {
    #[serde(default)]
    pub last_cost: f64,

    #[serde(default)]
    pub last_api_duration: u64,

    #[serde(default)]
    pub last_tool_duration: u64,

    #[serde(default)]
    pub last_duration: u64,

    #[serde(default)]
    pub last_lines_added: u32,

    #[serde(default)]
    pub last_lines_removed: u32,

    #[serde(default)]
    pub last_total_input_tokens: u64,

    #[serde(default)]
    pub last_total_output_tokens: u64,

    #[serde(default)]
    pub last_total_cache_creation_input_tokens: u64,

    #[serde(default)]
    pub last_total_cache_read_input_tokens: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_session_id: Option<String>,

    #[serde(default)]
    pub last_model_usage: HashMap<String, ModelUsage>,
}

/// Model-specific usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub web_search_requests: u32,
    #[serde(rename = "costUSD")]
    pub cost_usd: f64,
}

/// React vulnerability detection cache
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VulnerabilityCache {
    #[serde(default)]
    pub detected: bool,
    pub package: Option<String>,
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub package_manager: Option<String>,
}

/// Skill usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillUsageStats {
    pub usage_count: u32,
    pub last_used_at: u64,
}

/// OAuth account information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAccount {
    pub account_uuid: String,
    pub email_address: String,
    pub organization_uuid: String,
    pub has_extra_usage_enabled: bool,
    pub display_name: String,
    pub organization_role: String,
    pub organization_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_stdio_deserialization() {
        let json = r#"{
            "type": "stdio",
            "command": "nexcore-mcp",
            "args": [],
            "env": {}
        }"#;

        let config: McpServerConfig = serde_json::from_str(json).unwrap();
        match config {
            McpServerConfig::Stdio { command, .. } => {
                assert_eq!(command, "nexcore-mcp");
            }
        }
    }

    #[test]
    fn test_mcp_server_management_api() {
        // Create minimal config for testing
        let mut config = ClaudeConfig {
            num_startups: 1,
            install_method: "test".into(),
            auto_updates: false,
            verbose: false,
            auto_compact_enabled: false,
            has_seen_tasks_hint: false,
            user_id: "test-user".into(),
            first_start_time: "2026-01-01".into(),
            projects: HashMap::new(),
            mcp_servers: HashMap::new(),
            skill_usage: HashMap::new(),
            oauth_account: None,
            cached_statsig_gates: HashMap::new(),
            cached_growth_book_features: HashMap::new(),
            extra_fields: HashMap::new(),
        };

        // Test: Initially no servers
        assert!(config.list_mcp_servers().is_empty());
        assert!(!config.has_mcp_server("test-server"));

        // Test: Add server
        config.add_mcp_server(
            "test-server",
            McpServerConfig::Stdio {
                command: "test-mcp".into(),
                args: vec!["--mode".into(), "stdio".into()],
                env: HashMap::new(),
            },
        );

        assert!(config.has_mcp_server("test-server"));
        assert_eq!(config.list_mcp_servers().len(), 1);

        // Test: Get server
        let server = config.get_mcp_server("test-server").unwrap();
        match server {
            McpServerConfig::Stdio { command, args, .. } => {
                assert_eq!(command, "test-mcp");
                assert_eq!(args.len(), 2);
            }
        }

        // Test: Remove server
        let removed = config.remove_mcp_server("test-server");
        assert!(removed.is_some());
        assert!(!config.has_mcp_server("test-server"));
        assert!(config.list_mcp_servers().is_empty());

        // Test: Remove non-existent returns None
        assert!(config.remove_mcp_server("ghost-server").is_none());
    }

    #[test]
    fn test_mcp_server_serialization_roundtrip() {
        let server = McpServerConfig::Stdio {
            command: "my-mcp".into(),
            args: vec!["arg1".into()],
            env: [("KEY".into(), "value".into())].into_iter().collect(),
        };

        // Serialize
        let json = serde_json::to_string(&server).unwrap();

        // Deserialize
        let restored: McpServerConfig = serde_json::from_str(&json).unwrap();

        match restored {
            McpServerConfig::Stdio { command, args, env } => {
                assert_eq!(command, "my-mcp");
                assert_eq!(args, vec!["arg1"]);
                assert_eq!(env.get("KEY"), Some(&"value".to_string()));
            }
        }
    }
}
