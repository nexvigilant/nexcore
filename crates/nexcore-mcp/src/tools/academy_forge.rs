//! Academy Forge MCP Tools
//!
//! Extract structured knowledge from NexCore Rust source into IR,
//! and validate generated academy content against schema + accuracy rules.

use std::path::PathBuf;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    ForgeCompileParams, ForgeExtractParams, ForgeGuidanceScaffoldParams, ForgeScaffoldParams,
    ForgeValidateParams,
};

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

/// Scaffold a pathway from an FDA guidance document.
///
/// Looks up the guidance document by slug/partial title, extracts metadata
/// (title, topics, centers, status), and generates a complete StaticPathway
/// JSON scaffold with stages mapped to document sections.
///
/// ## Pipeline
///
/// ```text
/// forge_scaffold_from_guidance(slug, pathway_id, title, sections?)
///     → Look up guidance via nexcore-fda-guidance
///     → Generate scaffold via academy-forge guidance module
///     → Return StaticPathway JSON template
/// ```
pub fn forge_scaffold_from_guidance(
    params: ForgeGuidanceScaffoldParams,
) -> Result<CallToolResult, McpError> {
    // Look up the guidance document in the embedded index
    let results =
        match nexcore_fda_guidance::index::search(&params.guidance_id, None, None, None, 1) {
            Ok(docs) => docs,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "FDA guidance search failed: {e}"
                ))]));
            }
        };

    let doc = match results.first() {
        Some(d) => d,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "No FDA guidance document found for: '{}'. Try a different slug or search term.",
                params.guidance_id
            ))]));
        }
    };

    // Convert FDA guidance doc to academy-forge GuidanceInput
    let guidance_input = academy_forge::GuidanceInput {
        slug: doc.slug.clone(),
        title: doc.title.clone(),
        topics: doc.topics.clone(),
        centers: doc.centers.clone(),
        status: doc.status.clone(),
        document_type: doc.document_type.clone(),
    };

    // Build scaffold params
    let scaffold_params = academy_forge::GuidanceScaffoldParams {
        pathway_id: params.pathway_id,
        title: params.title,
        domain: params.domain,
        sections: params.sections,
    };

    // Generate the scaffold
    let scaffold = academy_forge::guidance_scaffold(&guidance_input, &scaffold_params);

    let stages_count = scaffold
        .get("stages")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let json = serde_json::json!({
        "success": true,
        "guidance_slug": doc.slug,
        "guidance_title": doc.title,
        "guidance_status": doc.status,
        "guidance_centers": doc.centers,
        "pathway_id": scaffold.get("id"),
        "stages": stages_count,
        "component_count": scaffold.get("componentCount"),
        "scaffold": scaffold,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Atomize a pathway JSON into Atomic Learning Objects (ALOs).
///
/// Decomposes each stage into Hook + Concept + Activity + Reflection ALOs
/// with intra-pathway dependency edges. Returns the full `AtomizedPathway`
/// structure with per-type counts, duration analysis, and edge breakdown.
pub fn forge_atomize(params: crate::params::ForgeAtomizeParams) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let nexcore = home.join("nexcore");

    let input_path = if params.pathway_json.starts_with('/') {
        PathBuf::from(&params.pathway_json)
    } else {
        nexcore.join(&params.pathway_json)
    };

    if !input_path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Pathway JSON not found: {}",
            input_path.display()
        ))]));
    }

    let raw = match std::fs::read_to_string(&input_path) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to read {}: {e}",
                input_path.display()
            ))]));
        }
    };

    let pathway_json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Invalid JSON in {}: {e}",
                input_path.display()
            ))]));
        }
    };

    match academy_forge::atomize(&pathway_json) {
        Ok(atomized) => {
            let alo_count = atomized.alos.len();
            let edge_count = atomized.edges.len();

            // Type breakdown
            let hooks = atomized.alos.iter().filter(|a| a.alo_type == academy_forge::ir::AloType::Hook).count();
            let concepts = atomized.alos.iter().filter(|a| a.alo_type == academy_forge::ir::AloType::Concept).count();
            let activities = atomized.alos.iter().filter(|a| a.alo_type == academy_forge::ir::AloType::Activity).count();
            let reflections = atomized.alos.iter().filter(|a| a.alo_type == academy_forge::ir::AloType::Reflection).count();
            let total_duration: u16 = atomized.alos.iter().map(|a| a.estimated_duration).sum();

            // Validation
            let report = academy_forge::validate::alo::validate_alo_report(&atomized);

            let json = serde_json::json!({
                "success": true,
                "pathway_id": atomized.id,
                "title": atomized.title,
                "alo_count": alo_count,
                "edge_count": edge_count,
                "total_duration_min": total_duration,
                "avg_duration_min": if alo_count > 0 { total_duration as f32 / alo_count as f32 } else { 0.0 },
                "type_breakdown": {
                    "hooks": hooks,
                    "concepts": concepts,
                    "activities": activities,
                    "reflections": reflections,
                },
                "validation": {
                    "passed": report.passed,
                    "errors": report.error_count,
                    "warnings": report.warning_count,
                    "findings": report.findings,
                },
                "atomized_pathway": atomized,
            });

            if report.passed {
                Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
            } else {
                Ok(CallToolResult::error(vec![Content::text(json.to_string())]))
            }
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Atomize failed: {e}"
        ))])),
    }
}

/// Build a cross-pathway Learning Graph from multiple atomized pathways.
///
/// Merges ALOs and edges, detects cross-pathway overlaps, and builds a DAG.
/// Returns graph metadata (node count, edge count, components, diameter),
/// overlap clusters, and the full graph structure.
pub fn forge_graph(params: crate::params::ForgeGraphParams) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let nexcore = home.join("nexcore");

    let mut pathways = Vec::new();

    for file_path in &params.pathway_files {
        let input_path = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            nexcore.join(file_path)
        };

        if !input_path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Pathway JSON not found: {}",
                input_path.display()
            ))]));
        }

        let raw = match std::fs::read_to_string(&input_path) {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to read {}: {e}",
                    input_path.display()
                ))]));
            }
        };

        let pathway_json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid JSON in {}: {e}",
                    input_path.display()
                ))]));
            }
        };

        match academy_forge::atomize(&pathway_json) {
            Ok(atomized) => pathways.push(atomized),
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Atomize failed for {}: {e}",
                    input_path.display()
                ))]));
            }
        }
    }

    match academy_forge::build_graph(&pathways, params.include_fuzzy, params.similarity_threshold) {
        Ok(graph) => {
            let json = serde_json::json!({
                "success": true,
                "metadata": graph.metadata,
                "pathways": graph.pathways,
                "overlap_clusters": graph.overlap_clusters,
                "graph": graph,
            });

            Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Graph build failed: {e}"
        ))])),
    }
}

/// Find the shortest learning path to a target ALO or KSB.
///
/// Given a set of completed ALO IDs, computes the minimum remaining path
/// to reach a specific ALO or any ALO tagged with a specific KSB reference.
pub fn forge_shortest_path(
    params: crate::params::ForgeShortestPathParams,
) -> Result<CallToolResult, McpError> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    let nexcore = home.join("nexcore");

    // Build graph from pathway files
    let mut pathways = Vec::new();
    for file_path in &params.pathway_files {
        let input_path = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            nexcore.join(file_path)
        };

        let raw = match std::fs::read_to_string(&input_path) {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to read {}: {e}",
                    input_path.display()
                ))]));
            }
        };

        let pathway_json: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid JSON: {e}"
                ))]));
            }
        };

        match academy_forge::atomize(&pathway_json) {
            Ok(atomized) => pathways.push(atomized),
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Atomize failed: {e}"
                ))]));
            }
        }
    }

    let graph = match academy_forge::build_graph(&pathways, false, 0.6) {
        Ok(g) => g,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Graph build failed: {e}"
            ))]));
        }
    };

    let completed: std::collections::HashSet<String> = params.completed.into_iter().collect();

    // Dispatch to appropriate query
    if let Some(target_alo_id) = &params.target_alo_id {
        match academy_forge::graph::queries::shortest_path_to_alo(&graph, target_alo_id, &completed)
        {
            Some(result) => {
                let json = serde_json::json!({
                    "success": true,
                    "query": "shortest_path_to_alo",
                    "target": target_alo_id,
                    "result": result,
                });
                Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "No path found to ALO '{target_alo_id}'. It may not exist or may already be completed."
            ))])),
        }
    } else if let Some(target_ksb) = &params.target_ksb {
        match academy_forge::graph::queries::shortest_path_to_ksb(&graph, target_ksb, &completed) {
            Some(result) => {
                let json = serde_json::json!({
                    "success": true,
                    "query": "shortest_path_to_ksb",
                    "target": target_ksb,
                    "result": result,
                });
                Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "No path found to KSB '{target_ksb}'. No ALOs reference this KSB or all are completed."
            ))])),
        }
    } else {
        // Neither target specified — return capability surface
        let surface = academy_forge::graph::queries::capability_surface(&graph, &completed);
        let json = serde_json::json!({
            "success": true,
            "query": "capability_surface",
            "completed_count": completed.len(),
            "result": surface,
        });
        Ok(CallToolResult::success(vec![Content::text(json.to_string())]))
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
