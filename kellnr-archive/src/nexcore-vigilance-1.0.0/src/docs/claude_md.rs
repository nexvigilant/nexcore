//! # CLAUDE.md Auto-Generator
//!
//! Mines codebase primitives to generate CLAUDE.md files automatically.
//!
//! ## Sources Mined (T2-C Composites)
//!
//! | Source | Primitive | Extraction |
//! |--------|-----------|------------|
//! | Cargo.toml | manifest | name, description, members, deps |
//! | src/lib.rs | public_api | pub mod declarations |
//! | README.md | doc_comment | existing documentation |
//! | tests/ | test_file | test patterns |
//!
//! ## Transformations Applied
//!
//! - **extraction**: Select relevant fields from sources
//! - **aggregation**: Combine related information
//! - **normalization**: Standardize formatting
//! - **inference**: Derive purpose from naming patterns

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Generator configuration (T1: state)
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Root directory to analyze
    pub root: PathBuf,
    /// Whether to include architecture section
    pub include_architecture: bool,
    /// Whether to include command reference
    pub include_commands: bool,
    /// Whether to include key directories
    pub include_directories: bool,
    /// Custom sections to add
    pub custom_sections: Vec<Section>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            include_architecture: true,
            include_commands: true,
            include_directories: true,
            custom_sections: Vec::new(),
        }
    }
}

/// A documentation section (T2-P: section)
#[derive(Debug, Clone)]
pub struct Section {
    /// Section heading
    pub title: String,
    /// Heading level (1-6)
    pub level: u8,
    /// Section content
    pub content: SectionContent,
}

/// Section content types (T2-C composites)
#[derive(Debug, Clone)]
pub enum SectionContent {
    /// Plain text paragraph
    Text(String),
    /// Command reference table
    CommandTable(Vec<CommandEntry>),
    /// Directory listing table
    DirectoryTable(Vec<DirectoryEntry>),
    /// Module architecture table
    ModuleTable(Vec<ModuleEntry>),
    /// Code block
    CodeBlock { language: String, code: String },
    /// Nested sections
    Subsections(Vec<Section>),
}

/// Command entry for quick reference (T3: command_reference)
#[derive(Debug, Clone)]
pub struct CommandEntry {
    /// Command or need
    pub need: String,
    /// Command to run
    pub command: String,
}

/// Directory entry (T3: key_directories)
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// Path
    pub path: String,
    /// Purpose description
    pub purpose: String,
}

/// Module entry for architecture (T2-C: public_api)
#[derive(Debug, Clone)]
pub struct ModuleEntry {
    /// Module name
    pub name: String,
    /// Module purpose
    pub purpose: String,
}

/// Extracted manifest data (T2-C: manifest)
#[derive(Debug, Clone, Default)]
pub struct ManifestData {
    /// Package name
    pub name: String,
    /// Package description
    pub description: String,
    /// Workspace members
    pub members: Vec<String>,
    /// Key dependencies
    pub dependencies: Vec<String>,
    /// Rust edition
    pub edition: String,
}

/// CLAUDE.md Generator
pub struct ClaudeMdGenerator {
    config: GeneratorConfig,
    manifest: Option<ManifestData>,
    modules: Vec<ModuleEntry>,
    readme_content: Option<String>,
}

impl ClaudeMdGenerator {
    /// Create new generator with config
    #[must_use]
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            config,
            manifest: None,
            modules: Vec::new(),
            readme_content: None,
        }
    }

    /// Mine all sources (T2-P: extraction pipeline)
    pub fn mine_sources(&mut self) -> Result<(), GeneratorError> {
        self.extract_manifest()?;
        self.extract_modules()?;
        self.extract_readme()?;
        Ok(())
    }

    /// Extract manifest data from Cargo.toml
    fn extract_manifest(&mut self) -> Result<(), GeneratorError> {
        let cargo_path = self.config.root.join("Cargo.toml");
        if !cargo_path.exists() {
            return Err(GeneratorError::SourceNotFound("Cargo.toml".to_string()));
        }

        let content = std::fs::read_to_string(&cargo_path)
            .map_err(|e| GeneratorError::ReadError(e.to_string()))?;

        let mut data = ManifestData::default();

        // Extract package info (simple parsing)
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = ") {
                data.name = extract_quoted_value(line);
            } else if line.starts_with("description = ") {
                data.description = extract_quoted_value(line);
            } else if line.starts_with("edition = ") {
                data.edition = extract_quoted_value(line);
            }
        }

        // Extract workspace members
        if let Some(members_start) = content.find("members = [") {
            let members_section = &content[members_start..];
            if let Some(end) = members_section.find(']') {
                let members_str = &members_section[11..end];
                for line in members_str.lines() {
                    let line = line.trim().trim_matches(',').trim_matches('"');
                    if !line.is_empty() && !line.starts_with('#') {
                        data.members.push(line.to_string());
                    }
                }
            }
        }

        self.manifest = Some(data);
        Ok(())
    }

    /// Extract module declarations from lib.rs
    fn extract_modules(&mut self) -> Result<(), GeneratorError> {
        let lib_paths = [
            self.config.root.join("src/lib.rs"),
            self.config.root.join("lib.rs"),
        ];

        let lib_path = lib_paths.iter().find(|p| p.exists());

        let Some(lib_path) = lib_path else {
            return Ok(()); // No lib.rs is fine
        };

        let content = std::fs::read_to_string(lib_path)
            .map_err(|e| GeneratorError::ReadError(e.to_string()))?;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("pub mod ") {
                let module_name = line
                    .strip_prefix("pub mod ")
                    .and_then(|s| s.strip_suffix(';'))
                    .unwrap_or("")
                    .to_string();

                if !module_name.is_empty() {
                    let purpose = infer_module_purpose(&module_name);
                    self.modules.push(ModuleEntry {
                        name: module_name,
                        purpose,
                    });
                }
            }
        }

        Ok(())
    }

    /// Extract README content
    fn extract_readme(&mut self) -> Result<(), GeneratorError> {
        let readme_path = self.config.root.join("README.md");
        if readme_path.exists() {
            self.readme_content = std::fs::read_to_string(&readme_path).ok();
        }
        Ok(())
    }

    /// Generate CLAUDE.md content (T1: mapping pipeline)
    #[must_use]
    pub fn generate(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# CLAUDE.md\n\n");
        output.push_str(
            "This file provides guidance to Claude Code (claude.ai/code) \
             when working with code in this repository.\n\n",
        );

        // Overview from manifest
        if let Some(ref manifest) = self.manifest {
            if !manifest.description.is_empty() {
                output.push_str("## Overview\n\n");
                output.push_str(&manifest.description);
                output.push_str("\n\n");
            }
        }

        // Quick Reference
        if self.config.include_commands {
            output.push_str(&self.generate_quick_reference());
        }

        // Architecture
        if self.config.include_architecture && !self.modules.is_empty() {
            output.push_str(&self.generate_architecture());
        }

        // Key Directories
        if self.config.include_directories {
            if let Some(ref manifest) = self.manifest {
                if !manifest.members.is_empty() {
                    output.push_str(&self.generate_directories(manifest));
                }
            }
        }

        // Custom sections
        for section in &self.config.custom_sections {
            output.push_str(&render_section(section));
        }

        output
    }

    /// Generate quick reference section
    fn generate_quick_reference(&self) -> String {
        let mut commands = vec![
            CommandEntry {
                need: "Run all tests".to_string(),
                command: "cargo test --workspace".to_string(),
            },
            CommandEntry {
                need: "Build release".to_string(),
                command: "cargo build --release".to_string(),
            },
            CommandEntry {
                need: "Lint".to_string(),
                command: "cargo clippy --workspace -- -D warnings".to_string(),
            },
        ];

        // Infer additional commands from manifest
        if let Some(ref manifest) = self.manifest {
            if manifest.members.iter().any(|m| m.contains("api")) {
                commands.push(CommandEntry {
                    need: "Start API".to_string(),
                    command: "cargo run -p *-api".to_string(),
                });
            }
        }

        let mut output = String::from("## Quick Reference\n\n");
        output.push_str("| Need | Command |\n");
        output.push_str("|------|--------|\n");
        for cmd in commands {
            output.push_str(&format!("| {} | `{}` |\n", cmd.need, cmd.command));
        }
        output.push('\n');
        output
    }

    /// Generate architecture section
    fn generate_architecture(&self) -> String {
        let mut output = String::from("## Architecture\n\n");
        output.push_str("| Module | Purpose |\n");
        output.push_str("|--------|--------|\n");
        for module in &self.modules {
            output.push_str(&format!("| `{}` | {} |\n", module.name, module.purpose));
        }
        output.push('\n');
        output
    }

    /// Generate key directories section
    fn generate_directories(&self, manifest: &ManifestData) -> String {
        let mut output = String::from("## Key Directories\n\n");
        output.push_str("| Path | Purpose |\n");
        output.push_str("|------|--------|\n");

        for member in &manifest.members {
            let purpose = infer_directory_purpose(member);
            output.push_str(&format!("| `{}` | {} |\n", member, purpose));
        }
        output.push('\n');
        output
    }

    /// Write generated content to file
    pub fn write_to(&self, path: &Path) -> Result<(), GeneratorError> {
        let content = self.generate();
        std::fs::write(path, content).map_err(|e| GeneratorError::WriteError(e.to_string()))
    }
}

/// Generator errors
#[derive(Debug, Clone)]
pub enum GeneratorError {
    /// Source file not found
    SourceNotFound(String),
    /// Error reading file
    ReadError(String),
    /// Error writing file
    WriteError(String),
}

impl std::fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceNotFound(s) => write!(f, "Source not found: {}", s),
            Self::ReadError(s) => write!(f, "Read error: {}", s),
            Self::WriteError(s) => write!(f, "Write error: {}", s),
        }
    }
}

impl std::error::Error for GeneratorError {}

// === Helper Functions (T2-P: normalization, inference) ===

/// Extract quoted value from TOML line
fn extract_quoted_value(line: &str) -> String {
    line.split('=')
        .nth(1)
        .map(|s| s.trim().trim_matches('"').to_string())
        .unwrap_or_default()
}

/// Infer module purpose from name (T1: inference)
fn infer_module_purpose(name: &str) -> String {
    let patterns: HashMap<&str, &str> = [
        ("api", "REST API endpoints"),
        ("cli", "Command-line interface"),
        ("config", "Configuration management"),
        ("db", "Database operations"),
        ("error", "Error types and handling"),
        ("model", "Data models"),
        ("service", "Business logic services"),
        ("util", "Utility functions"),
        ("test", "Test utilities"),
        ("auth", "Authentication/authorization"),
        ("handler", "Request handlers"),
        ("middleware", "Middleware components"),
        ("router", "Route definitions"),
        ("schema", "Schema definitions"),
        ("types", "Type definitions"),
        ("lib", "Core library"),
        ("pv", "Pharmacovigilance"),
        ("signal", "Signal detection"),
        ("guardian", "Guardian/safety systems"),
        ("brain", "Working memory"),
        ("skills", "Skill management"),
        ("hooks", "Hook enforcement"),
        ("mcp", "MCP server tools"),
        ("docs", "Documentation generation"),
        ("hud", "HUD governance"),
    ]
    .into_iter()
    .collect();

    for (pattern, purpose) in patterns {
        if name.contains(pattern) {
            return purpose.to_string();
        }
    }

    format!("{} module", name)
}

/// Infer directory purpose from path (T1: inference)
fn infer_directory_purpose(path: &str) -> String {
    let name = path.split('/').last().unwrap_or(path);
    infer_module_purpose(name)
}

/// Render a section to markdown
fn render_section(section: &Section) -> String {
    let mut output = String::new();
    let heading = "#".repeat(section.level as usize);
    output.push_str(&format!("{} {}\n\n", heading, section.title));

    match &section.content {
        SectionContent::Text(text) => {
            output.push_str(text);
            output.push_str("\n\n");
        }
        SectionContent::CodeBlock { language, code } => {
            output.push_str(&format!("```{}\n{}\n```\n\n", language, code));
        }
        SectionContent::CommandTable(entries) => {
            output.push_str("| Need | Command |\n");
            output.push_str("|------|--------|\n");
            for entry in entries {
                output.push_str(&format!("| {} | `{}` |\n", entry.need, entry.command));
            }
            output.push('\n');
        }
        SectionContent::DirectoryTable(entries) => {
            output.push_str("| Path | Purpose |\n");
            output.push_str("|------|--------|\n");
            for entry in entries {
                output.push_str(&format!("| `{}` | {} |\n", entry.path, entry.purpose));
            }
            output.push('\n');
        }
        SectionContent::ModuleTable(entries) => {
            output.push_str("| Module | Purpose |\n");
            output.push_str("|--------|--------|\n");
            for entry in entries {
                output.push_str(&format!("| `{}` | {} |\n", entry.name, entry.purpose));
            }
            output.push('\n');
        }
        SectionContent::Subsections(subs) => {
            for sub in subs {
                output.push_str(&render_section(sub));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_module_purpose() {
        assert_eq!(infer_module_purpose("api"), "REST API endpoints");
        assert_eq!(infer_module_purpose("guardian"), "Guardian/safety systems");
        assert_eq!(infer_module_purpose("unknown"), "unknown module");
    }

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(extract_quoted_value("name = \"my-crate\""), "my-crate");
        assert_eq!(extract_quoted_value("edition = \"2024\""), "2024");
    }

    #[test]
    fn test_generator_default_config() {
        let config = GeneratorConfig::default();
        assert!(config.include_architecture);
        assert!(config.include_commands);
        assert!(config.include_directories);
    }

    #[test]
    fn test_generate_empty() {
        let config = GeneratorConfig {
            include_architecture: false,
            include_commands: false,
            include_directories: false,
            ..Default::default()
        };
        let generator = ClaudeMdGenerator::new(config);
        let output = generator.generate();
        assert!(output.contains("# CLAUDE.md"));
        assert!(output.contains("claude.ai/code"));
    }

    #[test]
    fn test_render_section_text() {
        let section = Section {
            title: "Test".to_string(),
            level: 2,
            content: SectionContent::Text("Hello world".to_string()),
        };
        let output = render_section(&section);
        assert!(output.contains("## Test"));
        assert!(output.contains("Hello world"));
    }

    #[test]
    fn test_render_section_code() {
        let section = Section {
            title: "Code".to_string(),
            level: 3,
            content: SectionContent::CodeBlock {
                language: "rust".to_string(),
                code: "fn main() {}".to_string(),
            },
        };
        let output = render_section(&section);
        assert!(output.contains("### Code"));
        assert!(output.contains("```rust"));
        assert!(output.contains("fn main()"));
    }
}
