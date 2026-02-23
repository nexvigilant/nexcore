//! Configuration validation traits and utilities

use nexcore_error::{nexerror, Result};
use std::path::Path;

/// Validation trait for configuration types
pub trait Validate {
    /// Validate the configuration, returning errors if invalid
    fn validate(&self) -> Result<()>;

    /// Validate with additional context
    fn validate_with_context(&self, context: &str) -> Result<()> {
        self.validate().map_err(|e| nexerror!("{}: {}", context, e))
    }
}

impl Validate for crate::claude::ClaudeConfig {
    fn validate(&self) -> Result<()> {
        // Validate MCP server configurations
        for (name, server) in &self.mcp_servers {
            server.validate_with_context(&format!("MCP server '{}'", name))?;
        }

        // Validate project configurations
        for (path, project) in &self.projects {
            project.validate_with_context(&format!("Project '{}'", path.display()))?;
        }

        Ok(())
    }
}

impl Validate for crate::claude::ProjectConfig {
    fn validate(&self) -> Result<()> {
        // Validate project-specific MCP servers
        for (name, server) in &self.mcp_servers {
            server.validate_with_context(&format!("Project MCP server '{}'", name))?;
        }

        Ok(())
    }
}

impl Validate for crate::claude::McpServerConfig {
    fn validate(&self) -> Result<()> {
        match self {
            crate::claude::McpServerConfig::Stdio { command, args, .. } => {
                // Check if command exists (if it's an absolute path)
                let cmd_path = Path::new(command);
                if cmd_path.is_absolute() && !cmd_path.exists() {
                    return Err(nexerror!("MCP server command does not exist: {}", command));
                }

                // Validate no suspicious command patterns
                if command.contains("..") || command.contains("~") {
                    return Err(nexerror!(
                        "MCP server command contains suspicious path: {}",
                        command
                    ));
                }

                // Check for common executables in PATH
                let known_executables = ["nexcore-mcp", "claude-code-docs-mcp", "npx", "uv"];
                let is_known = known_executables.iter().any(|&exe| command.contains(exe));

                if !is_known && !cmd_path.is_absolute() {
                    eprintln!(
                        "Warning: MCP server command '{}' is not in PATH and not absolute",
                        command
                    );
                }

                // Validate args don't contain injection patterns
                for arg in args {
                    if arg.contains(";") || arg.contains("&&") || arg.contains("|") {
                        return Err(nexerror!(
                            "MCP server arg contains suspicious pattern: {}",
                            arg
                        ));
                    }
                }

                Ok(())
            }
        }
    }
}

impl Validate for crate::gemini::GeminiConfig {
    fn validate(&self) -> Result<()> {
        for hook in &self.hooks {
            hook.validate()?;
        }
        Ok(())
    }
}

impl Validate for crate::gemini::HookDefinition {
    fn validate(&self) -> Result<()> {
        // Validate hook command exists
        if !self.command.exists() {
            return Err(nexerror!(
                "Hook command does not exist: {}",
                self.command.display()
            ));
        }

        // Validate timeout is reasonable
        if self.timeout == 0 {
            return Err(nexerror!("Hook timeout must be greater than 0"));
        }

        if self.timeout > 300_000 {
            // 5 minutes
            eprintln!(
                "Warning: Hook '{}' has very long timeout: {}ms",
                self.name, self.timeout
            );
        }

        Ok(())
    }
}

impl Validate for crate::git::GitConfig {
    fn validate(&self) -> Result<()> {
        // Validate user configuration
        if self.user.name.is_empty() {
            return Err(nexerror!("Git user.name is required"));
        }

        if self.user.email.is_empty() {
            return Err(nexerror!("Git user.email is required"));
        }

        // Validate email format (basic check)
        if !self.user.email.contains('@') {
            return Err(nexerror!("Git user.email must be a valid email address"));
        }

        // Validate credential helpers
        for cred in &self.credentials {
            if !cred.helper_command.exists() {
                // Check if it starts with '!' (shell command)
                let cmd_str = cred.helper_command.to_string_lossy();
                if !cmd_str.starts_with('!') {
                    eprintln!(
                        "Warning: Git credential helper not found: {}",
                        cred.helper_command.display()
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::McpServerConfig;
    use std::collections::HashMap;

    #[test]
    fn test_validate_mcp_server_suspicious_path() {
        let config = McpServerConfig::Stdio {
            command: "../../../etc/passwd".to_string(),
            args: vec![],
            env: HashMap::new(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_mcp_server_injection() {
        let config = McpServerConfig::Stdio {
            command: "safe-command".to_string(),
            args: vec!["arg1; rm -rf /".to_string()],
            env: HashMap::new(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_git_config_missing_email() {
        let mut config = crate::git::GitConfig {
            init: crate::git::GitInit::default(),
            user: crate::git::GitUser {
                name: "Test User".to_string(),
                email: String::new(),
            },
            pull: crate::git::GitPull::default(),
            fetch: crate::git::GitFetch::default(),
            diff: crate::git::GitDiff::default(),
            core: crate::git::GitCore::default(),
            aliases: HashMap::new(),
            credentials: vec![],
        };

        assert!(config.validate().is_err());

        config.user.email = "invalid-email".to_string();
        assert!(config.validate().is_err());

        config.user.email = "valid@example.com".to_string();
        assert!(config.validate().is_ok());
    }
}
