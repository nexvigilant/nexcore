//! Academy Forge MCP Tools
//!
//! Extract structured knowledge from NexCore Rust source into IR,
//! and validate generated academy content against schema + accuracy rules.

use std::path::PathBuf;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{ForgeCompileParams, ForgeExtractParams, ForgeScaffoldParams, ForgeValidateParams};

/// Extract a complete IR from a Rust crate.
///
/// Resolves `crate_name` to `~/nexcore/crates/{crate_name}/` and parses
/// Cargo.toml, source files, and optionally applies a domain plugin.
pub fn forge_extract(params: ForgeExtractParams) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let crate_path = home.join("nexcore").join("crates").join(&params.crate_name);

    if !crate_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Crate not found: {}. Path: {}",
            params.crate_name,
            crate_path.display()
        ))]));
    }

    match academy_forge::extract_crate(&crate_path, params.domain.as_deref()) {
        Ok(analysis) => {
            let json = serde_json::json!({
                "success": true,
                "crate": analysis.name,
                "version": analysis.version,
                "description": analysis.description,
                "modules": analysis.modules.len(),
                "public_types": analysis.public_types.len(),
                "public_enums": analysis.public_enums.len(),
                "constants": analysis.constants.len(),
                "traits": analysis.traits.len(),
                "has_domain": analysis.domain.is_some(),
                "analysis": analysis,
            });

            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Extraction failed: {e}"
        ))])),
    }
}

/// Validate academy content JSON against all rules (R1-R27).
///
/// If the content includes domain-specific data, accuracy rules (R9-R14)
/// use the vigilance domain IR for cross-referencing.
pub fn forge_validate(params: ForgeValidateParams) -> Result<CallToolResult, McpError> {
    // Check if domain field exists in content to determine if we should load domain IR
    let domain_analysis = if params
        .content
        .get("domain")
        .and_then(|d| d.as_str())
        .is_some()
    {
        // Load vigilance domain IR for accuracy checking
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
        let workspace_root = home.join("nexcore");
        academy_forge::domain::extract_domain("vigilance", &workspace_root).ok()
    } else {
        None
    };

    let report = academy_forge::validate(&params.content, domain_analysis.as_ref());

    let json = serde_json::json!({
        "success": true,
        "passed": report.passed,
        "total_findings": report.total_findings,
        "error_count": report.error_count,
        "warning_count": report.warning_count,
        "advisory_count": report.advisory_count,
        "findings": report.findings,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Generate a pathway authoring template from domain IR.
///
/// Extracts domain analysis from the specified crate, then generates a
/// complete StaticPathway JSON with pre-filled stages, quiz skeletons,
/// and TODO markers for narrative content.
pub fn forge_scaffold(params: ForgeScaffoldParams) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let crate_path = home.join("nexcore").join("crates").join(&params.crate_name);

    if !crate_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Crate not found: {}. Path: {}",
            params.crate_name,
            crate_path.display()
        ))]));
    }

    // Extract domain IR
    let analysis = match academy_forge::extract_crate(&crate_path, Some(&params.domain)) {
        Ok(a) => a,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Domain extraction failed: {e}"
            ))]));
        }
    };

    let domain = match analysis.domain.as_ref() {
        Some(d) => d,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "No domain analysis returned for domain '{}'",
                params.domain
            ))]));
        }
    };

    // Build scaffold params
    let scaffold_params = academy_forge::ScaffoldParams {
        pathway_id: params.pathway_id,
        title: params.title,
        domain: params.domain.clone(),
    };

    let scaffold = academy_forge::scaffold(domain, &scaffold_params);

    let json = serde_json::json!({
        "success": true,
        "domain": params.domain,
        "stages": scaffold.get("stages").map(|s| s.as_array().map(|a| a.len()).unwrap_or(0)).unwrap_or(0),
        "scaffold": scaffold,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Return the StaticPathway JSON Schema for academy content.
///
/// This schema describes the expected structure of academy pathway JSON
/// that `forge_validate` checks against.
pub fn forge_schema() -> Result<CallToolResult, McpError> {
    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "StaticPathway",
        "description": "Academy learning pathway content structure",
        "type": "object",
        "required": ["id", "title", "description", "stages"],
        "properties": {
            "id": {
                "type": "string",
                "pattern": "^[a-z]+-\\d{2}(-\\d{2})?(-[a-z-]+)?$",
                "description": "Pathway identifier (e.g., 'tov-01')"
            },
            "title": { "type": "string" },
            "description": { "type": "string" },
            "domain": { "type": "string", "description": "Domain name for accuracy checking (e.g., 'vigilance')" },
            "componentCount": { "type": "integer", "description": "Total number of lessons/activities" },
            "estimatedDuration": { "type": "string", "description": "Total estimated duration" },
            "stages": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "title", "description"],
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "bloomLevel": { "type": "string", "enum": ["Remember", "Understand", "Apply", "Analyze", "Evaluate", "Create"] },
                        "passingScore": { "type": "number", "minimum": 0, "maximum": 100 },
                        "estimatedDuration": { "type": "string" },
                        "activities": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["id", "title", "type"],
                                "properties": {
                                    "id": { "type": "string" },
                                    "title": { "type": "string" },
                                    "type": { "type": "string" },
                                    "estimatedDuration": { "type": "string" },
                                    "quiz": {
                                        "type": "object",
                                        "properties": {
                                            "questions": {
                                                "type": "array",
                                                "items": {
                                                    "type": "object",
                                                    "required": ["id", "type", "question", "correctAnswer"],
                                                    "properties": {
                                                        "id": { "type": "string" },
                                                        "type": { "type": "string", "enum": ["multiple-choice", "true-false", "multiple-select"] },
                                                        "question": { "type": "string" },
                                                        "options": { "type": "array", "items": { "type": "string" } },
                                                        "correctAnswer": { "type": ["integer", "boolean", "array"] },
                                                        "points": { "type": "integer" },
                                                        "explanation": { "type": "string" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        schema.to_string(),
    )]))
}

/// Compile pathway JSON into Studio-compatible TypeScript stage files.
///
/// Takes a forge-generated pathway JSON (from `forge_scaffold` + authoring)
/// and produces TypeScript source files matching the `CapabilityStage` type
/// from `@/types/academy`. Generates one stage file per stage, plus
/// `config.ts` and `index.ts`.
pub fn forge_compile(params: ForgeCompileParams) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let nexcore = home.join("nexcore");

    let input_path = if params.pathway_json.starts_with('/') {
        PathBuf::from(&params.pathway_json)
    } else {
        nexcore.join(&params.pathway_json)
    };

    let output_dir = if params.output_dir.starts_with('/') {
        PathBuf::from(&params.output_dir)
    } else {
        nexcore.join(&params.output_dir)
    };

    if !input_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Pathway JSON not found: {}",
            input_path.display()
        ))]));
    }

    let compile_params = academy_forge::CompileParams {
        input_path,
        output_dir,
        overwrite: params.overwrite,
    };

    match academy_forge::compile_pathway(&compile_params) {
        Ok(result) => {
            let files: Vec<String> = result
                .files_written
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            let json = serde_json::json!({
                "success": true,
                "stages_compiled": result.stages_compiled,
                "files_written": files,
                "file_count": files.len(),
                "warnings": result.warnings,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Compile failed: {e}"
        ))])),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    /// Helper: extract text from CallToolResult content[0]
    fn extract_text(result: &CallToolResult) -> String {
        result
            .content
            .first()
            .and_then(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .expect("expected text content in result")
    }

    /// Helper: parse text content as JSON
    fn extract_json(result: &CallToolResult) -> serde_json::Value {
        let text = extract_text(result);
        serde_json::from_str(&text).expect("expected valid JSON in result text")
    }

    #[test]
    fn test_forge_extract_success() {
        let params = ForgeExtractParams {
            crate_name: "nexcore-tov".to_string(),
            domain: Some("vigilance".to_string()),
        };
        let result = forge_extract(params).expect("forge_extract should not error");
        assert!(!result.is_error.unwrap_or(false), "should be success");

        let json = extract_json(&result);
        assert_eq!(json["success"], true);
        assert_eq!(json["crate"], "nexcore-tov");
        assert_eq!(json["has_domain"], true);
        assert!(json["modules"].as_u64().unwrap() > 0);

        // Verify nested analysis is serialized
        let analysis = &json["analysis"];
        assert!(analysis["domain"]["axioms"].as_array().unwrap().len() == 5);
    }

    #[test]
    fn test_forge_extract_missing_crate() {
        let params = ForgeExtractParams {
            crate_name: "nonexistent-crate-xyz".to_string(),
            domain: None,
        };
        let result = forge_extract(params).expect("should return Ok with error content");
        assert!(result.is_error.unwrap_or(false), "should be error result");
        let text = extract_text(&result);
        assert!(text.contains("Crate not found"));
    }

    #[test]
    fn test_forge_validate_passing() {
        let params = ForgeValidateParams {
            content: serde_json::json!({
                "id": "tov-01",
                "title": "Test Pathway",
                "description": "A test pathway",
                "stages": []
            }),
        };
        let result = forge_validate(params).expect("forge_validate should not error");
        let json = extract_json(&result);
        assert_eq!(json["success"], true);
        assert_eq!(json["passed"], true);
        assert_eq!(json["error_count"], 0);
    }

    #[test]
    fn test_forge_validate_failing() {
        let params = ForgeValidateParams {
            content: serde_json::json!({}),
        };
        let result = forge_validate(params).expect("forge_validate should not error");
        let json = extract_json(&result);
        assert_eq!(json["passed"], false);
        assert!(json["error_count"].as_u64().unwrap() >= 3);
        assert!(json["findings"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn test_forge_scaffold_success() {
        let params = ForgeScaffoldParams {
            crate_name: "nexcore-tov".to_string(),
            domain: "vigilance".to_string(),
            pathway_id: "tov-99".to_string(),
            title: "Test Scaffold".to_string(),
        };
        let result = forge_scaffold(params).expect("forge_scaffold should not error");
        assert!(!result.is_error.unwrap_or(false), "should be success");

        let json = extract_json(&result);
        assert_eq!(json["success"], true);
        assert_eq!(json["domain"], "vigilance");
        assert!(json["stages"].as_u64().unwrap() > 0, "should have stages");

        // Scaffold should be a valid pathway structure
        let scaffold = &json["scaffold"];
        assert_eq!(scaffold["id"], "tov-99");
        assert!(scaffold["stages"].as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_forge_scaffold_bad_crate() {
        let params = ForgeScaffoldParams {
            crate_name: "nonexistent-xyz".to_string(),
            domain: "vigilance".to_string(),
            pathway_id: "tov-99".to_string(),
            title: "Test".to_string(),
        };
        let result = forge_scaffold(params).expect("should return Ok with error content");
        assert!(result.is_error.unwrap_or(false), "should be error result");
        let text = extract_text(&result);
        assert!(text.contains("Crate not found"));
    }

    #[test]
    fn test_forge_schema_returns_valid_schema() {
        let result = forge_schema().expect("forge_schema should not error");
        let json = extract_json(&result);
        assert_eq!(json["title"], "StaticPathway");
        assert!(json["required"].as_array().unwrap().len() == 4);
        assert!(json["properties"]["stages"].is_object());
    }
}
