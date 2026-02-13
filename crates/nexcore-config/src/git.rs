//! Git configuration types
//!
//! Type-safe representation of `.gitconfig` (INI format).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub init: GitInit,
    pub user: GitUser,
    pub pull: GitPull,
    pub fetch: GitFetch,
    pub diff: GitDiff,
    pub core: GitCore,

    /// Git aliases (e.g., st = status)
    #[serde(default)]
    pub aliases: HashMap<String, String>,

    /// Credential helpers
    #[serde(default)]
    pub credentials: Vec<CredentialHelper>,
}

impl GitConfig {
    /// Load configuration from INI file
    pub fn from_file(path: &str) -> Result<Self> {
        use anyhow::Context;
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read Git config: {}", path))?;
        Self::parse_ini(&content)
    }

    /// Parse INI-style .gitconfig
    fn parse_ini(content: &str) -> Result<Self> {
        let mut init = GitInit::default();
        let mut user = GitUser::default();
        let mut pull = GitPull::default();
        let mut fetch = GitFetch::default();
        let mut diff = GitDiff::default();
        let mut core = GitCore::default();
        let mut aliases = HashMap::new();
        let mut credentials = Vec::new();

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Section header: [section] or [section "subsection"]
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].to_string();
                continue;
            }

            // Key-value pair
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match current_section.as_str() {
                    "init" => {
                        if key == "defaultBranch" {
                            init.default_branch = value.to_string();
                        }
                    }
                    "user" => {
                        if key == "name" {
                            user.name = value.to_string();
                        } else if key == "email" {
                            user.email = value.to_string();
                        }
                    }
                    "pull" => {
                        if key == "rebase" {
                            pull.rebase = value == "true";
                        }
                    }
                    "fetch" => {
                        if key == "prune" {
                            fetch.prune = value == "true";
                        }
                    }
                    "diff" => {
                        if key == "colorMoved" {
                            diff.color_moved = Some(value.to_string());
                        }
                    }
                    "core" => {
                        if key == "autocrlf" {
                            core.autocrlf = Some(value.to_string());
                        } else if key == "editor" {
                            core.editor = Some(value.to_string());
                        }
                    }
                    "alias" => {
                        aliases.insert(key.to_string(), value.to_string());
                    }
                    s if s.starts_with("credential") => {
                        if key == "helper" && !value.is_empty() {
                            let url_pattern = s
                                .strip_prefix("credential ")
                                .and_then(|s| s.strip_prefix('"'))
                                .and_then(|s| s.strip_suffix('"'))
                                .unwrap_or("")
                                .to_string();

                            credentials.push(CredentialHelper {
                                url_pattern,
                                helper_command: PathBuf::from(value),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            init,
            user,
            pull,
            fetch,
            diff,
            core,
            aliases,
            credentials,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInit {
    #[serde(default = "default_branch")]
    pub default_branch: String,
}

impl Default for GitInit {
    fn default() -> Self {
        Self {
            default_branch: "main".to_string(),
        }
    }
}

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitUser {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitPull {
    #[serde(default)]
    pub rebase: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitFetch {
    #[serde(default)]
    pub prune: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitDiff {
    pub color_moved: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitCore {
    pub autocrlf: Option<String>,
    pub editor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialHelper {
    /// URL pattern (e.g., "https://github.com")
    pub url_pattern: String,

    /// Helper command (e.g., "!/usr/bin/gh auth git-credential")
    pub helper_command: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_parsing() {
        let config = r#"
[init]
    defaultBranch = main
[user]
    name = TestUser
    email = test@example.com
[pull]
    rebase = true
[alias]
    st = status
    co = checkout
        "#;

        let parsed = GitConfig::parse_ini(config).unwrap();
        assert_eq!(parsed.init.default_branch, "main");
        assert_eq!(parsed.user.name, "TestUser");
        assert_eq!(parsed.user.email, "test@example.com");
        assert!(parsed.pull.rebase);
        assert_eq!(parsed.aliases.get("st"), Some(&"status".to_string()));
    }
}
