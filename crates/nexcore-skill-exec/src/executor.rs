//! Skill executor trait and composite implementation.

use crate::{ExecutionRequest, ExecutionResult, Result, ShellExecutor, SkillInfo};
use async_trait::async_trait;
use nexcore_fs::dirs;
use std::path::Path;
use std::path::PathBuf;

/// Trait for skill executors.
///
/// Each executor type (shell, binary, library) implements this trait.
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    /// Execute a skill with the given request.
    async fn execute(
        &self,
        skill: &SkillInfo,
        request: &ExecutionRequest,
    ) -> Result<ExecutionResult>;

    /// Check if this executor can handle the given skill.
    fn can_execute(&self, skill: &SkillInfo) -> bool;

    /// Get the executor name for logging.
    fn name(&self) -> &'static str;
}

/// Composite executor that delegates to appropriate sub-executors.
///
/// Tries executors in priority order: Binary > Shell > Documentation fallback.
pub struct CompositeExecutor {
    shell: ShellExecutor,
    primary_skills_dir: PathBuf,
    skill_roots: Vec<PathBuf>,
}

impl CompositeExecutor {
    fn default_skill_roots() -> Vec<PathBuf> {
        let mut roots = Vec::<PathBuf>::new();

        let mut push_unique = |p: PathBuf| {
            if !roots.contains(&p) {
                roots.push(p);
            }
        };

        if let Ok(path) = std::env::var("NEXCORE_SKILLS_DIR") {
            push_unique(PathBuf::from(path));
        }
        if let Ok(path) = std::env::var("CLAUDE_SKILLS_DIR") {
            push_unique(PathBuf::from(path));
        }
        if let Ok(codex_home) = std::env::var("CODEX_HOME") {
            push_unique(PathBuf::from(codex_home).join("skills"));
        }
        if let Some(home) = dirs::home_dir() {
            push_unique(home.join(".claude").join("skills"));
        }

        if let Ok(cwd) = std::env::current_dir() {
            push_unique(cwd.join(".claude").join("skills"));
            push_unique(cwd.join("nexcore").join(".claude").join("skills"));
            for ancestor in cwd.ancestors() {
                push_unique(ancestor.join(".claude").join("skills"));
            }
        }

        roots
    }

    /// Create a new composite executor.
    #[must_use]
    pub fn new() -> Self {
        let skill_roots = Self::default_skill_roots();
        let primary_skills_dir = skill_roots.first().cloned().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".claude").join("skills"))
                .unwrap_or_else(|| PathBuf::from("/home/matthew/.claude/skills"))
        });

        Self {
            shell: ShellExecutor::new(),
            primary_skills_dir,
            skill_roots,
        }
    }

    /// Create with a custom skills directory.
    #[must_use]
    pub fn with_skills_dir(skills_dir: impl Into<std::path::PathBuf>) -> Self {
        let path = skills_dir.into();
        Self {
            shell: ShellExecutor::new(),
            primary_skills_dir: path.clone(),
            skill_roots: vec![path],
        }
    }

    fn resolve_skill_path(&self, name: &str) -> Option<PathBuf> {
        self.skill_roots
            .iter()
            .map(|root| root.join(name))
            .find(|p| p.exists())
    }

    /// Discover skill info from filesystem.
    pub fn discover_skill(&self, name: &str) -> Result<SkillInfo> {
        let skill_path = self
            .resolve_skill_path(name)
            .unwrap_or_else(|| self.primary_skills_dir.join(name));

        if !skill_path.exists() {
            return Err(crate::ExecutionError::SkillNotFound {
                name: name.to_string(),
                path: skill_path,
            });
        }

        let mut methods = Vec::new();

        // Check for shell scripts
        let scripts_dir = skill_path.join("scripts");
        if scripts_dir.exists() {
            // Look for executable shell scripts
            if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && (path.extension().is_some_and(|e| e == "sh") || is_executable(&path))
                    {
                        methods.push(crate::models::ExecutionMethod::Shell(path));
                    }
                }
            }

            // Look for compiled binaries in target/release
            let release_dir = scripts_dir.join("target").join("release");
            if release_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&release_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && is_executable(&path) {
                            methods.push(crate::models::ExecutionMethod::Binary(path));
                        }
                    }
                }
            }

            // Check for Rust crate with binary
            for subdir in &["ctvp_lib", "lib"] {
                let crate_dir = scripts_dir.join(subdir);
                let cargo_toml = crate_dir.join("Cargo.toml");
                if cargo_toml.exists() {
                    let bin_dir = crate_dir.join("target").join("release");
                    if bin_dir.exists() {
                        if let Ok(entries) = std::fs::read_dir(&bin_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_file()
                                    && is_executable(&path)
                                    && !path.extension().is_some()
                                {
                                    methods.push(crate::models::ExecutionMethod::Binary(path));
                                }
                            }
                        }
                    }
                }
            }
        }

        // TODO: Parse SKILL.md for input/output schemas
        let skill_md = skill_path.join("SKILL.md");
        if methods.is_empty() && skill_md.exists() {
            // Documentation-only skills are still executable via fallback mode.
            methods.push(crate::models::ExecutionMethod::Library(skill_md.clone()));
        }
        let (input_schema, output_schema) = if skill_md.exists() {
            // For now, return None - will implement YAML frontmatter parsing
            (None, None)
        } else {
            (None, None)
        };

        Ok(SkillInfo {
            name: name.to_string(),
            path: skill_path,
            execution_methods: methods,
            input_schema,
            output_schema,
        })
    }

    /// Execute a skill by name.
    pub async fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResult> {
        let skill = self.discover_skill(&request.skill_name)?;

        // Find an executor that can handle this skill
        if self.shell.can_execute(&skill) {
            return self.shell.execute(&skill, request).await;
        }

        // No executor available
        Err(crate::ExecutionError::NoExecutorAvailable {
            name: request.skill_name.clone(),
            skill_type: "unknown".to_string(),
        })
    }
}

impl Default for CompositeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a path is executable (Unix).
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Check if a path is executable (non-Unix fallback).
#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.extension()
        .map(|e| e == "exe" || e == "bat" || e == "cmd")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_executor_creation() {
        let executor = CompositeExecutor::new();
        assert!(
            executor
                .primary_skills_dir
                .to_string_lossy()
                .contains(".claude/skills")
        );
    }
}
