//! # Documentation Generation MCP Tools
//!
//! Tools for autonomous CLAUDE.md generation by mining codebase primitives.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;

use crate::params::DocsGenerateClaudeMdParams;
use nexcore_vigilance::docs::{ClaudeMdGenerator, GeneratorConfig};

/// Generate CLAUDE.md by mining codebase primitives
///
/// Returns `CallToolResult` with generated markdown content and discovery metadata.
pub fn docs_generate_claude_md(
    params: DocsGenerateClaudeMdParams,
) -> Result<CallToolResult, McpError> {
    let root = params
        .path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let config = GeneratorConfig {
        root: root.clone(),
        include_architecture: params.include_architecture.unwrap_or(true),
        include_commands: params.include_commands.unwrap_or(true),
        include_directories: params.include_directories.unwrap_or(true),
        custom_sections: Vec::new(),
    };

    let mut generator = ClaudeMdGenerator::new(config);

    match generator.mine_sources() {
        Ok(()) => {
            let content = generator.generate();

            // Count discovered items (approximate from content)
            let modules_found = content.matches("| `").count().saturating_sub(2);
            let members_found = if content.contains("## Key Directories") {
                content
                    .split("## Key Directories")
                    .nth(1)
                    .map(|s| s.matches("| `").count())
                    .unwrap_or(0)
            } else {
                0
            };

            let result = json!({
                "content": content,
                "modules_found": modules_found,
                "members_found": members_found,
                "success": true,
            });

            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "content": "",
                "modules_found": 0,
                "members_found": 0,
                "success": false,
                "error": e.to_string(),
            });

            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_claude_md_default() {
        let params = DocsGenerateClaudeMdParams {
            path: Some(".".to_string()),
            include_architecture: Some(true),
            include_commands: Some(true),
            include_directories: Some(true),
        };

        let result = docs_generate_claude_md(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_claude_md_nonexistent() {
        let params = DocsGenerateClaudeMdParams {
            path: Some("/nonexistent/path/12345".to_string()),
            include_architecture: None,
            include_commands: None,
            include_directories: None,
        };

        let result = docs_generate_claude_md(params);
        // Returns Ok with error in JSON, not Err
        assert!(result.is_ok());
    }
}
