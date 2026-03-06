//! # NexVigilant Core — Config
//!
//! Type-safe configuration consolidation for the nexcore ecosystem.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
//!
//! This crate consolidates scattered JSON, TOML, and INI configuration files
//! into unified Rust types with compile-time validation and serde support.
//!
//! ## Modules
//!
//! - `claude`: Claude Code configuration (`.claude.json`)
//! - `gemini`: Gemini hooks configuration (`.gemini.json`)
//! - `git`: Git configuration (`.gitconfig`)
//! - `hooks`: Hook registry (parses `hooks-catalog.json`)
//! - `validation`: Validation traits and utilities

pub mod asr;
pub mod claude;
pub mod gemini;
pub mod git;
pub mod grounding;
pub mod hooks;
pub mod validation;
pub mod vocab;

pub use asr::{
    AsrConfig, DoDChecklist, DoDItem, FlywheelConfig, FlywheelStage, Model, RoutingConfig,
};
pub use claude::{ClaudeConfig, McpServerConfig, ProjectConfig};
pub use gemini::{GeminiConfig, HookDefinition, HookType};
pub use git::{GitConfig, GitUser};
pub use hooks::{HookEvent, HookRegistry, HookTier};
pub use validation::Validate;
pub use vocab::{SkillChain, SkillMapping, VocabSkillMap};

use nexcore_error::Result;
use std::path::Path;

/// Load all configurations from standard paths
pub fn load_all_configs() -> Result<AllConfigs> {
    use nexcore_error::Context;
    let home = std::env::var("HOME").context("HOME env not set")?;

    let claude = ClaudeConfig::from_file(&format!("{}/.claude.json", home))?;
    let gemini = GeminiConfig::from_file(&format!("{}/.gemini.json", home))?;
    let git = GitConfig::from_file(&format!("{}/.gitconfig", home))?;
    let hooks = HookRegistry::from_file(&format!(
        "{}/nexcore/crates/nexcore-hooks/hooks-catalog.json",
        home
    ))?;

    Ok(AllConfigs {
        claude,
        gemini,
        git,
        hooks,
    })
}

/// Consolidated configurations
#[derive(Debug)]
pub struct AllConfigs {
    pub claude: ClaudeConfig,
    pub gemini: GeminiConfig,
    pub git: GitConfig,
    pub hooks: HookRegistry,
}

impl AllConfigs {
    /// Write consolidated config to TOML file
    pub fn write_toml<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use nexcore_error::Context;
        // Note: Hooks registry is loaded from hooks-catalog.json (not included in consolidated config)
        // This keeps the hook definitions in their source-of-truth location
        let claude_toml =
            toml::to_string_pretty(&self.claude).context("Failed to serialize claude config")?;
        let gemini_toml =
            toml::to_string_pretty(&self.gemini).context("Failed to serialize gemini config")?;
        let git_toml =
            toml::to_string_pretty(&self.git).context("Failed to serialize git config")?;

        let toml_content = format!(
            "# NexCore Consolidated Configuration\n\
             # Generated from scattered JSON/INI configs\n\
             # \n\
             # Hook registry loaded from: ~/nexcore/crates/nexcore-hooks/hooks-catalog.json\n\
             # Total hooks: {} (Dev: {}, Review: {}, Deploy: {})\n\n\
             [claude]\n{}\n\n\
             [gemini]\n{}\n\n\
             [git]\n{}\n",
            self.hooks.all_hooks().len(),
            self.hooks.filter_by_tier(crate::HookTier::Dev).len(),
            self.hooks.filter_by_tier(crate::HookTier::Review).len(),
            self.hooks.filter_by_tier(crate::HookTier::Deploy).len(),
            claude_toml,
            gemini_toml,
            git_toml
        );

        let path = path.as_ref();
        std::fs::write(path, toml_content)
            .context(format!("Failed to write config: {}", path.display()))?;
        Ok(())
    }
}
