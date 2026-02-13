//! Hook registry types and parsing
//!
//! Type-safe representation of Claude Code hooks from hooks-catalog.json.
//!
//! ## Hook Tiers
//!
//! - **dev**: Fast daily development (11 hooks) - minimal blocking
//! - **review**: Code review (33 hooks) - quality checks with warnings
//! - **deploy**: Pre-deploy (76 hooks) - full validation, strict blocking
//!
//! ## Example
//!
//! ```no_run
//! use nexcore_config::hooks::{HookRegistry, HookTier};
//!
//! let registry = HookRegistry::from_file("hooks-catalog.json")?;
//! let dev_hooks = registry.filter_by_tier(HookTier::Dev);
//! assert!(dev_hooks.len() > 0);
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Hook registry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMeta {
    /// Catalog version
    pub version: String,
    /// Description of catalog
    pub description: String,
    /// Tier descriptions
    pub tiers: HashMap<String, String>,
}

/// Hook deployment tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookTier {
    /// Fast development mode - minimal hooks
    Dev,
    /// Code review mode - quality checks
    Review,
    /// Deploy mode - full validation
    Deploy,
}

/// Hook event type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    /// Session starts
    SessionStart,
    /// Session ends
    SessionEnd,
    /// User submits prompt
    UserPromptSubmit,
    /// Before Bash tool use
    #[serde(rename = "PreToolUse:Bash")]
    PreToolUseBash,
    /// Before Edit or Write tool use
    #[serde(rename = "PreToolUse:Edit|Write")]
    PreToolUseEditWrite,
    /// Before Task tool use
    #[serde(rename = "PreToolUse:Task")]
    PreToolUseTask,
    /// After any tool use
    PostToolUse,
    /// After tool use failure
    PostToolUseFailure,
    /// Before compacting context
    PreCompact,
    /// Permission requested
    PermissionRequest,
    /// Stop requested
    Stop,
    /// Setup phase
    Setup,
    /// Subagent starts
    SubagentStart,
}

/// Hook definition
#[derive(Debug, Clone, Serialize)]
pub struct HookDefinition {
    /// Hook name (e.g., "python_file_blocker")
    pub name: String,
    /// Event type
    pub event: HookEvent,
    /// Deployment tiers (dev/review/deploy)
    #[serde(rename = "tier")]
    pub tiers: Vec<HookTier>,
    /// Timeout in seconds
    pub timeout: u64,
    /// Description
    #[serde(rename = "desc")]
    pub description: String,
}

/// Complete hook registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistry {
    /// Metadata
    pub meta: HookMeta,
    /// Hooks organized by event type (internal representation)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    hooks: HashMap<String, HashMap<String, HookDefinitionRaw>>,
}

/// Raw hook definition (before event assignment)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookDefinitionRaw {
    #[serde(rename = "tier")]
    tiers: Vec<HookTier>,
    timeout: u64,
    #[serde(rename = "desc")]
    description: String,
}

impl HookRegistry {
    /// Load hook registry from file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to hooks-catalog.json
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or JSON is invalid
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read hook catalog: {}", path))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse hook catalog JSON: {}", path))
    }

    /// Get all hooks as flat list with event assigned
    pub fn all_hooks(&self) -> Vec<HookDefinition> {
        let mut result = Vec::new();

        for (event_str, hooks_map) in &self.hooks {
            let event = match event_str.as_str() {
                "SessionStart" => HookEvent::SessionStart,
                "SessionEnd" => HookEvent::SessionEnd,
                "UserPromptSubmit" => HookEvent::UserPromptSubmit,
                "PreToolUse:Bash" => HookEvent::PreToolUseBash,
                "PreToolUse:Edit|Write" => HookEvent::PreToolUseEditWrite,
                "PreToolUse:Task" => HookEvent::PreToolUseTask,
                "PostToolUse" => HookEvent::PostToolUse,
                "PostToolUseFailure" => HookEvent::PostToolUseFailure,
                "PreCompact" => HookEvent::PreCompact,
                "PermissionRequest" => HookEvent::PermissionRequest,
                "Stop" => HookEvent::Stop,
                "Setup" => HookEvent::Setup,
                "SubagentStart" => HookEvent::SubagentStart,
                _ => {
                    continue;
                } // Skip unknown events
            };

            for (name, def) in hooks_map {
                result.push(HookDefinition {
                    name: name.clone(),
                    event: event.clone(),
                    tiers: def.tiers.clone(),
                    timeout: def.timeout,
                    description: def.description.clone(),
                });
            }
        }

        result
    }

    /// Filter hooks by deployment tier
    ///
    /// Returns all hooks that include the specified tier in their tier list.
    ///
    /// # Arguments
    ///
    /// * `tier` - Deployment tier to filter by
    ///
    /// # Returns
    ///
    /// Vector of hooks that match the tier
    pub fn filter_by_tier(&self, tier: HookTier) -> Vec<HookDefinition> {
        self.all_hooks()
            .into_iter()
            .filter(|h| h.tiers.contains(&tier))
            .collect()
    }

    /// Get hooks for a specific event
    pub fn get_event_hooks(&self, event: &HookEvent) -> Vec<HookDefinition> {
        self.all_hooks()
            .into_iter()
            .filter(|h| &h.event == event)
            .collect()
    }

    /// Generate settings.json hooks array for a specific tier
    ///
    /// Creates the hooks array format expected by Claude Code settings.json.
    ///
    /// # Arguments
    ///
    /// * `tier` - Deployment tier to generate for
    ///
    /// # Returns
    ///
    /// JSON array of hook definitions compatible with settings.json
    pub fn generate_settings(&self, tier: HookTier) -> Result<serde_json::Value> {
        let hooks = self.filter_by_tier(tier);
        let mut settings_hooks = Vec::new();

        for hook in hooks {
            let event_str = match hook.event {
                HookEvent::SessionStart => "SessionStart",
                HookEvent::SessionEnd => "SessionEnd",
                HookEvent::UserPromptSubmit => "UserPromptSubmit",
                HookEvent::PreToolUseBash => "PreToolUse:Bash",
                HookEvent::PreToolUseEditWrite => "PreToolUse:Edit|Write",
                HookEvent::PreToolUseTask => "PreToolUse:Task",
                HookEvent::PostToolUse => "PostToolUse",
                HookEvent::PostToolUseFailure => "PostToolUseFailure",
                HookEvent::PreCompact => "PreCompact",
                HookEvent::PermissionRequest => "PermissionRequest",
                HookEvent::Stop => "Stop",
                HookEvent::Setup => "Setup",
                HookEvent::SubagentStart => "SubagentStart",
            };

            settings_hooks.push(serde_json::json!({
                "name": hook.name,
                "event": event_str,
                "command": format!("{}", hook.name), // Binary name same as hook name
                "timeout": hook.timeout * 1000, // Convert to milliseconds
            }));
        }

        Ok(serde_json::json!(settings_hooks))
    }

    /// Count hooks by tier
    pub fn count_by_tier(&self) -> HashMap<HookTier, usize> {
        let mut counts = HashMap::new();

        for hook in self.all_hooks() {
            for tier in &hook.tiers {
                *counts.entry(*tier).or_insert(0) += 1;
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_catalog_path() -> String {
        std::env::var("HOME")
            .map(|h| format!("{}/nexcore/crates/nexcore-hooks/hooks-catalog.json", h))
            .unwrap_or_else(|_| "../nexcore-hooks/hooks-catalog.json".to_string())
    }

    #[test]
    fn test_parse_hooks_catalog() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        // Should have 13 event types
        assert_eq!(registry.hooks.len(), 13);

        // Meta should be present
        assert_eq!(registry.meta.version, "1.0");
        assert!(registry.meta.tiers.contains_key("dev"));
        assert!(registry.meta.tiers.contains_key("review"));
        assert!(registry.meta.tiers.contains_key("deploy"));
    }

    #[test]
    fn test_filter_by_tier() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        let dev_hooks = registry.filter_by_tier(HookTier::Dev);
        let deploy_hooks = registry.filter_by_tier(HookTier::Deploy);

        // Dev should have fewer hooks than deploy
        assert!(dev_hooks.len() < deploy_hooks.len());
        assert!(deploy_hooks.len() > 50); // Should be ~76
    }

    #[test]
    fn test_all_hooks() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        let all = registry.all_hooks();

        // Should have ~93 hooks
        assert!(all.len() > 90);

        // All hooks should have names, events, timeouts
        // Note: Some hooks may have empty tiers (experimental/disabled hooks)
        for hook in &all {
            assert!(!hook.name.is_empty());
            assert!(hook.timeout > 0);
            assert!(!hook.description.is_empty());
            // tiers can be empty for experimental/disabled hooks
        }
    }

    #[test]
    fn test_get_event_hooks() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        let session_start = registry.get_event_hooks(&HookEvent::SessionStart);
        assert!(!session_start.is_empty());

        // All should be SessionStart events
        for hook in session_start {
            assert_eq!(hook.event, HookEvent::SessionStart);
        }
    }

    #[test]
    fn test_count_by_tier() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        let counts = registry.count_by_tier();

        assert!(counts.contains_key(&HookTier::Dev));
        assert!(counts.contains_key(&HookTier::Review));
        assert!(counts.contains_key(&HookTier::Deploy));

        // Deploy should have most hooks
        assert!(counts[&HookTier::Deploy] > counts[&HookTier::Dev]);
    }

    #[test]
    fn test_generate_settings() {
        let catalog_path = get_catalog_path();
        let registry = HookRegistry::from_file(&catalog_path).unwrap();

        let settings = registry.generate_settings(HookTier::Dev).unwrap();

        assert!(settings.is_array());
        let arr = settings.as_array().unwrap();
        assert!(!arr.is_empty());

        // Each entry should have required fields
        for entry in arr {
            assert!(entry.get("name").is_some());
            assert!(entry.get("event").is_some());
            assert!(entry.get("command").is_some());
            assert!(entry.get("timeout").is_some());
        }
    }
}
