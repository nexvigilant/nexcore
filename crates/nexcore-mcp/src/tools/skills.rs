//! Skills tools: registry, validation, taxonomy, execution
//!
//! Skill discovery, Diamond compliance validation, O(1) taxonomy lookups,
//! and real skill execution via nexcore-skill-exec.

use std::path::Path;
use std::sync::Arc;

use crate::params::{
    SkillExecuteParams, SkillGetParams, SkillScanParams, SkillSchemaParams, SkillSearchByTagParams,
    SkillValidateParams, TaxonomyListParams, TaxonomyQueryParams,
};
use crate::tooling::{ScanLimitNotice, ScanLimits, read_limited_file};
use nexcore_vigilance::skills::{
    SkillRegistry, compute_intensive_categories, default_skills_cache_path, default_skills_path,
    list_taxonomy, query_taxonomy, validate_diamond,
};
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Scan directory for skills
pub fn scan(
    registry: &Arc<RwLock<SkillRegistry>>,
    params: SkillScanParams,
) -> Result<CallToolResult, McpError> {
    let path = Path::new(&params.directory);

    if !path.exists() {
        let json = json!({
            "error": format!("Directory does not exist: {}", params.directory),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let mut reg = registry.write();
    match reg.scan(path) {
        Ok(count) => {
            // Persist cache when scanning the default skills path.
            if default_skills_path().as_deref() == Some(path) {
                if let Some(cache_path) = default_skills_cache_path() {
                    let _ = reg.save_cache(&cache_path);
                }
            }
            let json = json!({
                "scanned": params.directory,
                "skills_found": count,
                "total_registered": reg.len(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "error": e,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// List all registered skills
pub fn list(registry: &Arc<RwLock<SkillRegistry>>) -> Result<CallToolResult, McpError> {
    // Auto-load cache or scan default skills path when registry is empty.
    if registry.read().len() == 0 {
        let mut reg = registry.write();
        if reg.len() == 0 {
            if let Some(cache_path) = default_skills_cache_path() {
                let _ = reg.load_cache(&cache_path);
            }
        }
        if reg.len() == 0 {
            if let Some(path) = default_skills_path() {
                let _ = reg.scan(&path);
                if let Some(cache_path) = default_skills_cache_path() {
                    let _ = reg.save_cache(&cache_path);
                }
            }
        }
    }
    let reg = registry.read();
    let skills: Vec<_> = reg
        .list()
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "intent": s.intent,
                "compliance": s.compliance,
                "smst_score": s.smst_score,
                "tags": s.tags,
            })
        })
        .collect();

    let json = json!({
        "skills": skills,
        "count": skills.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get skill by name (with fuzzy "did you mean?" suggestions)
pub fn get(
    registry: &Arc<RwLock<SkillRegistry>>,
    params: SkillGetParams,
) -> Result<CallToolResult, McpError> {
    let reg = registry.read();

    match reg.get(&params.name) {
        Some(skill) => {
            let json = json!({
                "name": skill.name,
                "path": skill.path.to_string_lossy(),
                "intent": skill.intent,
                "compliance": skill.compliance,
                "smst_score": skill.smst_score,
                "tags": skill.tags,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        None => {
            // Fuzzy-match against registered skill names for suggestions
            let skill_names: Vec<String> = reg.list().iter().map(|s| s.name.clone()).collect();
            let suggestions: Vec<_> = if skill_names.is_empty() {
                Vec::new()
            } else {
                nexcore_vigilance::foundation::fuzzy_search(&params.name, &skill_names, 3)
                    .into_iter()
                    .filter(|m| m.similarity >= 0.4)
                    .map(|m| {
                        json!({
                            "name": m.candidate,
                            "similarity": m.similarity,
                        })
                    })
                    .collect()
            };

            let json = if suggestions.is_empty() {
                json!({
                    "error": format!("Skill not found: {}", params.name),
                    "hint": "Use skill_scan to populate the registry first.",
                })
            } else {
                json!({
                    "error": format!("Skill not found: {}", params.name),
                    "did_you_mean": suggestions,
                    "hint": "Use skill_scan to populate the registry first if skills are missing.",
                })
            };
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Validate skill for Diamond compliance
pub fn validate(params: SkillValidateParams) -> Result<CallToolResult, McpError> {
    let path = Path::new(&params.path);

    match validate_diamond(path) {
        Ok(result) => {
            let json = json!({
                "path": params.path,
                "level": result.level.to_string(),
                "smst_score": result.smst_score,
                "issues": result.issues,
                "suggestions": result.suggestions,
                "is_diamond": result.level == nexcore_vigilance::skills::validation::ComplianceLevel::Diamond,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "error": e,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Search skills by tag
pub fn search_by_tag(
    registry: &Arc<RwLock<SkillRegistry>>,
    params: SkillSearchByTagParams,
) -> Result<CallToolResult, McpError> {
    let reg = registry.read();
    let matches: Vec<_> = reg
        .search_by_tag(&params.tag)
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "intent": s.intent,
                "tags": s.tags,
            })
        })
        .collect();

    let json = json!({
        "tag": params.tag,
        "matches": matches,
        "count": matches.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List nested skills for a compound/parent skill
///
/// Discovers sub-skills declared in a parent skill's `nested-skills` frontmatter.
/// Nested skills are resolved relative to the parent's directory.
pub fn list_nested(
    registry: &Arc<RwLock<SkillRegistry>>,
    params: crate::params::SkillListNestedParams,
) -> Result<CallToolResult, McpError> {
    let reg = registry.read();

    // First check if parent exists
    let parent = match reg.get(&params.parent) {
        Some(p) => p,
        None => {
            let json = json!({
                "error": format!("Parent skill not found: {}", params.parent),
                "hint": "Use skill_scan to populate the registry first.",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    // Get declared nested skills from frontmatter
    let declared = &parent.nested_skills;

    // Discover actual nested skills
    let nested = reg.list_nested(&params.parent);

    let nested_info: Vec<_> = nested
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "path": s.path.to_string_lossy(),
                "intent": s.intent,
                "compliance": s.compliance,
                "tags": s.tags,
            })
        })
        .collect();

    let json = json!({
        "parent": params.parent,
        "declared": declared,
        "nested_skills": nested_info,
        "count": nested.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Query taxonomy
pub fn taxonomy_query(params: TaxonomyQueryParams) -> Result<CallToolResult, McpError> {
    let result = query_taxonomy(&params.taxonomy_type, &params.key);

    let json = json!({
        "query_type": result.query_type,
        "key": result.key,
        "found": result.found,
        "data": result.data,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List taxonomy entries
pub fn taxonomy_list(params: TaxonomyListParams) -> Result<CallToolResult, McpError> {
    let result = list_taxonomy(&params.taxonomy_type);

    let json = json!({
        "taxonomy_type": result.taxonomy_type,
        "count": result.count,
        "entries": result.entries,
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get compute-intensive categories
pub fn categories_compute_intensive() -> Result<CallToolResult, McpError> {
    let categories: Vec<_> = compute_intensive_categories()
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "description": c.description,
                "examples": c.examples,
            })
        })
        .collect();

    let json = json!({
        "categories": categories,
        "count": categories.len(),
        "note": "These categories are candidates for NexCore/Rust delegation for 10-63x speedup.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Execute a skill by name with parameters
///
/// Uses nexcore-skill-exec for real skill execution (scripts, binaries).
pub fn execute(params: SkillExecuteParams) -> Result<CallToolResult, McpError> {
    use nexcore_skill_exec::{CompositeExecutor, ExecutionRequest};
    use std::time::Duration;

    // Create executor
    let executor = CompositeExecutor::new();

    // Discover skill first to validate it exists
    match executor.discover_skill(&params.name) {
        Ok(_skill_info) => {
            // Build execution request
            let request = ExecutionRequest::new(&params.name, params.parameters.clone())
                .with_timeout(Duration::from_secs(params.timeout_seconds));

            // Execute synchronously using tokio runtime
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { executor.execute(&request).await })
            });

            match result {
                Ok(exec_result) => {
                    let json = json!({
                        "skill_name": exec_result.skill_name,
                        "status": format!("{:?}", exec_result.status),
                        "output": exec_result.output,
                        "duration_ms": exec_result.duration_ms,
                        "exit_code": exec_result.exit_code,
                        "error": exec_result.error,
                    });
                    Ok(CallToolResult::success(vec![Content::text(
                        json.to_string(),
                    )]))
                }
                Err(e) => {
                    let json = json!({
                        "error": format!("{e}"),
                        "skill": params.name,
                    });
                    Ok(CallToolResult::success(vec![Content::text(
                        json.to_string(),
                    )]))
                }
            }
        }
        Err(e) => {
            let json = json!({
                "error": format!("{e}"),
                "hint": "Skill not found. Use skill_list to see available skills.",
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Get skill input/output schema and execution methods
pub fn schema(params: SkillSchemaParams) -> Result<CallToolResult, McpError> {
    use nexcore_skill_exec::CompositeExecutor;

    let executor = CompositeExecutor::new();
    match executor.discover_skill(&params.name) {
        Ok(skill_info) => {
            let methods: Vec<String> = skill_info
                .execution_methods
                .iter()
                .map(|m| format!("{m:?}"))
                .collect();
            let json = json!({
                "name": skill_info.name,
                "input_schema": skill_info.input_schema,
                "output_schema": skill_info.output_schema,
                "executable": !skill_info.execution_methods.is_empty(),
                "execution_methods": methods,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "error": format!("{e}"),
                "hint": "Skill not found. Use skill_list to see available skills.",
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

// ============================================================================
// Skill Compiler Tools
// ============================================================================

/// Compile multiple skills into a compound skill binary.
pub fn compile(params: crate::params::SkillCompileParams) -> Result<CallToolResult, McpError> {
    let toml_text = match nexcore_skill_compiler::spec_from_params(
        &params.skills,
        &params.strategy,
        &params.name,
    ) {
        Ok(t) => t,
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    match nexcore_skill_compiler::compile(&toml_text, params.build) {
        Ok(result) => {
            let json = json!({
                "name": result.name,
                "crate_dir": result.crate_dir.to_string_lossy(),
                "binary_path": result.binary_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                "skill_md": result.skill_md.to_string_lossy(),
                "diamond_compliant": result.diamond_compliant,
                "analysis": {
                    "can_compile": result.analysis.can_compile,
                    "warnings": result.analysis.warnings,
                    "blockers": result.analysis.blockers,
                },
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Check compatibility of skills for compilation (dry run).
pub fn compile_check(
    params: crate::params::SkillCompileCheckParams,
) -> Result<CallToolResult, McpError> {
    match nexcore_skill_compiler::check(&params.skills, &params.strategy) {
        Ok(report) => {
            let json = json!({
                "can_compile": report.can_compile,
                "warnings": report.warnings,
                "blockers": report.blockers,
                "skill_infos": report.skill_infos,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

// ============================================================================
// Vocabulary-Skill Mapping Tools
// ============================================================================

use crate::params::{PrimitiveSkillLookupParams, SkillChainLookupParams, VocabSkillLookupParams};

/// Load vocab skill map from default location
fn load_vocab_map() -> Option<(serde_json::Value, Option<ScanLimitNotice>)> {
    let home = std::env::var("HOME").ok()?;
    let path = format!("{}/.claude/implicit/vocab_skill_map.json", home);
    let limits = ScanLimits::from_env();
    let read_outcome = read_limited_file(std::path::Path::new(&path), limits).ok()?;
    let notice = read_outcome.notice;
    let map = serde_json::from_str(&read_outcome.content).ok()?;
    Some((map, notice))
}

/// Look up skills associated with a vocabulary shorthand
pub fn vocab_skill_lookup(params: VocabSkillLookupParams) -> Result<CallToolResult, McpError> {
    let (map, notice) = match load_vocab_map() {
        Some((m, notice)) => (m, notice),
        None => {
            let json = json!({
                "error": "vocab_skill_map.json not found",
                "hint": "Create ~/.claude/implicit/vocab_skill_map.json",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };
    let scan_notice = notice.and_then(|n| serde_json::to_value(vec![n]).ok());

    let vocab_to_skills = map.get("vocab_to_skills").and_then(|v| v.as_object());

    match vocab_to_skills.and_then(|m| m.get(&params.vocab)) {
        Some(mapping) => {
            let mut json = json!({
                "vocab": params.vocab,
                "primary": mapping.get("primary"),
                "secondary": mapping.get("secondary"),
                "hooks": mapping.get("hooks"),
                "primitives": mapping.get("primitives"),
            });
            if let Some(value) = scan_notice.clone() {
                json["scan_notice"] = value;
            }
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        None => {
            // Check trait_synonyms fallback (e.g. "apply" → Transform)
            if let Some(synonym_entry) = map
                .get("trait_synonyms")
                .and_then(|v| v.as_object())
                .and_then(|m| m.get(&params.vocab))
            {
                let canonical = synonym_entry
                    .get("canonical")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let skills = synonym_entry.get("skills").and_then(|v| v.as_array());
                let primary = skills.and_then(|a| a.first());
                let secondary: Vec<&serde_json::Value> = skills
                    .map(|a| a.iter().skip(1).collect())
                    .unwrap_or_default();
                let mut json = json!({
                    "vocab": params.vocab,
                    "synonym_of": canonical,
                    "grounding": synonym_entry.get("grounding"),
                    "domain": synonym_entry.get("domain"),
                    "primary": primary,
                    "secondary": secondary,
                    "resolved": format!("{} is a synonym of STEM trait {}", params.vocab, canonical),
                });
                if let Some(value) = scan_notice.clone() {
                    json["scan_notice"] = value;
                }
                return Ok(CallToolResult::success(vec![Content::text(
                    json.to_string(),
                )]));
            }

            let available: Vec<_> = vocab_to_skills
                .map(|m| m.keys().collect())
                .unwrap_or_default();
            let mut json = json!({
                "error": format!("Vocabulary '{}' not found", params.vocab),
                "available": available,
            });
            if let Some(value) = scan_notice {
                json["scan_notice"] = value;
            }
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Look up skills associated with a primitive
pub fn primitive_skill_lookup(
    params: PrimitiveSkillLookupParams,
) -> Result<CallToolResult, McpError> {
    let (map, notice) = match load_vocab_map() {
        Some((m, notice)) => (m, notice),
        None => {
            let json = json!({
                "error": "vocab_skill_map.json not found",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };
    let scan_notice = notice.and_then(|n| serde_json::to_value(vec![n]).ok());

    let primitive_to_skills = map.get("primitive_to_skills").and_then(|v| v.as_object());

    match primitive_to_skills.and_then(|m| m.get(&params.primitive)) {
        Some(skills) => {
            let mut json = json!({
                "primitive": params.primitive,
                "skills": skills,
            });
            if let Some(value) = scan_notice.clone() {
                json["scan_notice"] = value;
            }
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        None => {
            let available: Vec<_> = primitive_to_skills
                .map(|m| m.keys().collect())
                .unwrap_or_default();
            let mut json = json!({
                "error": format!("Primitive '{}' not found", params.primitive),
                "available": available,
            });
            if let Some(value) = scan_notice {
                json["scan_notice"] = value;
            }
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Look up skill chains by name or trigger phrase
pub fn skill_chain_lookup(params: SkillChainLookupParams) -> Result<CallToolResult, McpError> {
    let (map, notice) = match load_vocab_map() {
        Some((m, notice)) => (m, notice),
        None => {
            let json = json!({
                "error": "vocab_skill_map.json not found",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };
    let scan_notice = notice.and_then(|n| serde_json::to_value(vec![n]).ok());

    let skill_chains = map.get("skill_chains").and_then(|v| v.as_object());
    let query_lower = params.query.to_lowercase();

    // Search by name or trigger
    let matches: Vec<_> = skill_chains
        .map(|chains| {
            chains
                .iter()
                .filter(|(name, chain)| {
                    name.to_lowercase().contains(&query_lower)
                        || chain
                            .get("trigger")
                            .and_then(|t| t.as_str())
                            .map(|t| t.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                })
                .map(|(name, chain)| {
                    json!({
                        "name": name,
                        "trigger": chain.get("trigger"),
                        "chain": chain.get("chain"),
                        "subagent": chain.get("subagent"),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let mut json = json!({
        "query": params.query,
        "matches": matches,
        "count": matches.len(),
    });
    if let Some(value) = scan_notice {
        json["scan_notice"] = value;
    }
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List all vocabulary shorthands
pub fn vocab_list() -> Result<CallToolResult, McpError> {
    let (map, notice) = match load_vocab_map() {
        Some((m, notice)) => (m, notice),
        None => {
            let json = json!({
                "error": "vocab_skill_map.json not found",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };
    let scan_notice = notice.and_then(|n| serde_json::to_value(vec![n]).ok());

    let vocab_to_skills = map.get("vocab_to_skills").and_then(|v| v.as_object());
    let vocabs: Vec<_> = vocab_to_skills
        .map(|m| m.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();

    let mut json = json!({
        "vocabularies": vocabs,
        "count": vocabs.len(),
    });
    if let Some(value) = scan_notice {
        json["scan_notice"] = value;
    }
    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

// ============================================================================
// Skill Orchestration Analysis Tools
// ============================================================================

use crate::params::SkillOrchestrationAnalyzeParams;

/// Orchestrator detection patterns
const ORCHESTRATOR_PATTERNS: &[&str] = &[
    "spawn",
    "delegate",
    "phase",
    "pipeline",
    "orchestrat",
    "subagent",
    "chain",
    "compound",
    "meta-skill",
    "nested-skills",
    "upstream",
    "downstream",
];

/// Subagent type recommendations based on skill content
#[derive(Debug, Clone)]
struct SubagentRecommendation {
    agent_type: String,
    rationale: String,
    model_hint: String,
}

/// Analyze a single skill for orchestration patterns
fn analyze_skill_orchestration(skill_path: &Path) -> serde_json::Value {
    let skill_md_path = skill_path.join("SKILL.md");

    if !skill_md_path.exists() {
        return json!({
            "path": skill_path.display().to_string(),
            "error": "SKILL.md not found"
        });
    }

    let content = match std::fs::read_to_string(&skill_md_path) {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "path": skill_path.display().to_string(),
                "error": format!("Failed to read SKILL.md: {}", e)
            });
        }
    };

    // Parse frontmatter
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract current triggers
    let triggers = extract_triggers(&frontmatter);

    // Detect if this is an orchestrator
    let orchestrator_signals = detect_orchestrator_patterns(&content, &frontmatter);
    let is_orchestrator = !orchestrator_signals.is_empty();

    // Extract existing subagents field
    let existing_subagents = frontmatter
        .get("subagents")
        .or_else(|| frontmatter.get("nested-skills"))
        .or_else(|| frontmatter.get("dependencies"));

    // Generate subagent recommendations
    let recommendations = generate_subagent_recommendations(&content, &frontmatter, &body);

    // Suggest frontmatter additions
    let frontmatter_suggestions =
        generate_frontmatter_suggestions(&frontmatter, is_orchestrator, &recommendations);

    json!({
        "path": skill_path.display().to_string(),
        "name": frontmatter.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "triggers": triggers,
        "is_orchestrator": is_orchestrator,
        "orchestrator_signals": orchestrator_signals,
        "existing_subagents": existing_subagents,
        "recommended_subagents": recommendations.iter().map(|r| {
            json!({
                "type": r.agent_type,
                "rationale": r.rationale,
                "model_hint": r.model_hint,
            })
        }).collect::<Vec<_>>(),
        "frontmatter_suggestions": frontmatter_suggestions,
    })
}

/// Parse YAML frontmatter from SKILL.md content
fn parse_frontmatter(content: &str) -> (serde_json::Map<String, serde_json::Value>, String) {
    let empty_map = serde_json::Map::new();

    if !content.starts_with("---") {
        return (empty_map, content.to_string());
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return (empty_map, content.to_string());
    }

    let yaml_content = parts[1].trim();
    let body = parts[2].to_string();

    match serde_yaml::from_str::<serde_json::Value>(yaml_content) {
        Ok(serde_json::Value::Object(map)) => (map, body),
        _ => (empty_map, content.to_string()),
    }
}

/// Extract triggers from frontmatter
fn extract_triggers(frontmatter: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    frontmatter
        .get("triggers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Detect orchestrator patterns in skill content
fn detect_orchestrator_patterns(
    content: &str,
    frontmatter: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let content_lower = content.to_lowercase();
    let mut signals = Vec::new();

    // Check content for orchestrator patterns
    for pattern in ORCHESTRATOR_PATTERNS {
        if content_lower.contains(pattern) {
            signals.push(format!("content contains '{}'", pattern));
        }
    }

    // Check frontmatter fields
    if frontmatter.contains_key("nested-skills") {
        signals.push("has nested-skills field".to_string());
    }
    if frontmatter.contains_key("subagents") {
        signals.push("has subagents field".to_string());
    }
    if frontmatter.contains_key("pipeline") {
        signals.push("has pipeline field".to_string());
    }
    if frontmatter.contains_key("upstream") || frontmatter.contains_key("downstream") {
        signals.push("has upstream/downstream fields".to_string());
    }

    // Check tags for orchestrator indicators
    if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
        let tag_strings: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
        if tag_strings
            .iter()
            .any(|t| t.contains("orchestrator") || t.contains("compound") || t.contains("meta"))
        {
            signals.push("tags indicate orchestrator".to_string());
        }
    }

    // Check chain-position
    if let Some(pos) = frontmatter.get("chain-position").and_then(|v| v.as_str()) {
        if pos == "head" || pos == "middle" {
            signals.push(format!("chain-position is '{}'", pos));
        }
    }

    signals
}

/// Generate subagent recommendations based on skill analysis
fn generate_subagent_recommendations(
    content: &str,
    frontmatter: &serde_json::Map<String, serde_json::Value>,
    _body: &str,
) -> Vec<SubagentRecommendation> {
    let mut recommendations = Vec::new();
    let content_lower = content.to_lowercase();

    // Check for Rust development patterns
    if content_lower.contains("rust") || content_lower.contains("cargo") {
        recommendations.push(SubagentRecommendation {
            agent_type: "rust-anatomy-expert".to_string(),
            rationale: "Skill involves Rust development - delegate architecture decisions"
                .to_string(),
            model_hint: "sonnet".to_string(),
        });
    }

    // Check for primitive extraction needs
    if content_lower.contains("primitive")
        || content_lower.contains("decompos")
        || content_lower.contains("t1")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "primitive-extractor".to_string(),
            rationale: "Skill involves concept decomposition - delegate primitive analysis"
                .to_string(),
            model_hint: "sonnet".to_string(),
        });
    }

    // Check for strategy patterns
    if content_lower.contains("strateg")
        || content_lower.contains("playing to win")
        || content_lower.contains("capabilit")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "strat-dev".to_string(),
            rationale: "Skill involves strategic thinking - delegate capability mapping"
                .to_string(),
            model_hint: "sonnet".to_string(),
        });
    }

    // Check for validation patterns
    if content_lower.contains("validat")
        || content_lower.contains("ctvp")
        || content_lower.contains("test")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "ctvp-validator".to_string(),
            rationale: "Skill involves validation - delegate test quality assessment".to_string(),
            model_hint: "haiku".to_string(),
        });
    }

    // Check for code inspection patterns
    if content_lower.contains("inspect")
        || content_lower.contains("audit")
        || content_lower.contains("review")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "code-inspector".to_string(),
            rationale: "Skill involves code review - delegate inspection tasks".to_string(),
            model_hint: "haiku".to_string(),
        });
    }

    // Check for knowledge compilation
    if content_lower.contains("knowledge")
        || content_lower.contains("document")
        || content_lower.contains("compil")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "knowledge-compiler".to_string(),
            rationale: "Skill involves knowledge aggregation - delegate compilation".to_string(),
            model_hint: "sonnet".to_string(),
        });
    }

    // Check for epistemology / learning loops
    if content_lower.contains("epistemolog")
        || content_lower.contains("learning loop")
        || content_lower.contains("reflect")
    {
        recommendations.push(SubagentRecommendation {
            agent_type: "constructive-epistemology".to_string(),
            rationale: "Skill involves knowledge construction - delegate epistemological analysis"
                .to_string(),
            model_hint: "sonnet".to_string(),
        });
    }

    // If it's marked as an orchestrator but has no recommendations, suggest general patterns
    if recommendations.is_empty() {
        if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
            if tags.iter().any(|t| {
                t.as_str()
                    .map(|s| s.contains("orchestrator"))
                    .unwrap_or(false)
            }) {
                recommendations.push(SubagentRecommendation {
                    agent_type: "primitive-extractor".to_string(),
                    rationale: "Orchestrators benefit from primitive decomposition".to_string(),
                    model_hint: "sonnet".to_string(),
                });
            }
        }
    }

    recommendations
}

/// Generate frontmatter suggestions for a skill
fn generate_frontmatter_suggestions(
    frontmatter: &serde_json::Map<String, serde_json::Value>,
    is_orchestrator: bool,
    recommendations: &[SubagentRecommendation],
) -> Vec<serde_json::Value> {
    let mut suggestions = Vec::new();

    // Suggest adding orchestrator tag if detected but not present
    if is_orchestrator {
        let has_orchestrator_tag = frontmatter
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|tags| {
                tags.iter().any(|t| {
                    t.as_str()
                        .map(|s| s.contains("orchestrator"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !has_orchestrator_tag {
            suggestions.push(json!({
                "field": "tags",
                "action": "add",
                "value": "orchestrator",
                "reason": "Skill exhibits orchestrator patterns but lacks orchestrator tag"
            }));
        }
    }

    // Suggest adding subagents field if recommendations exist and field is missing
    if !recommendations.is_empty() && !frontmatter.contains_key("subagents") {
        let subagent_list: Vec<&str> = recommendations
            .iter()
            .map(|r| r.agent_type.as_str())
            .collect();
        suggestions.push(json!({
            "field": "subagents",
            "action": "add",
            "value": subagent_list,
            "reason": "Skill would benefit from explicit subagent declarations"
        }));
    }

    // Suggest chain-position if missing and is orchestrator
    if is_orchestrator && !frontmatter.contains_key("chain-position") {
        suggestions.push(json!({
            "field": "chain-position",
            "action": "add",
            "value": "head",
            "reason": "Orchestrators typically start chains"
        }));
    }

    // Suggest pipeline field if missing and has multiple phases
    if is_orchestrator && !frontmatter.contains_key("pipeline") {
        suggestions.push(json!({
            "field": "pipeline",
            "action": "add",
            "value": "<pipeline-name>",
            "reason": "Orchestrators benefit from explicit pipeline naming"
        }));
    }

    suggestions
}

/// Expand tilde in path (e.g., ~/foo -> /home/user/foo)
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

/// Analyze skills matching a path or glob pattern
pub fn orchestration_analyze(
    params: SkillOrchestrationAnalyzeParams,
) -> Result<CallToolResult, McpError> {
    let expanded_path = expand_tilde(&params.path_or_pattern);
    let path = Path::new(&expanded_path);

    let mut results = Vec::new();

    // Check if it's a glob pattern (contains *)
    if expanded_path.contains('*') {
        // Manual glob expansion for simple patterns like ~/.claude/skills/*
        // Split at the first * and scan the parent directory
        if let Some(star_pos) = expanded_path.find('*') {
            let parent_path = &expanded_path[..star_pos];
            let parent = Path::new(parent_path.trim_end_matches('/'));

            if parent.is_dir() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_dir() && entry_path.join("SKILL.md").exists() {
                            let analysis = analyze_skill_orchestration(&entry_path);
                            results.push(analysis);
                        }
                    }
                }
            } else {
                let json = json!({
                    "error": format!("Parent directory does not exist: {}", parent.display()),
                    "pattern": params.path_or_pattern,
                });
                return Ok(CallToolResult::success(vec![Content::text(
                    json.to_string(),
                )]));
            }
        }
    } else if path.is_dir() {
        // Single skill directory
        if path.join("SKILL.md").exists() {
            let analysis = analyze_skill_orchestration(path);
            results.push(analysis);
        } else {
            // Maybe it's a parent directory - scan subdirectories
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() && entry_path.join("SKILL.md").exists() {
                        let analysis = analyze_skill_orchestration(&entry_path);
                        results.push(analysis);
                    }
                }
            }
        }
    } else {
        let json = json!({
            "error": "Path does not exist or is not a directory",
            "path": params.path_or_pattern,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    // Calculate summary statistics
    let total_skills = results.len();
    let orchestrator_count = results
        .iter()
        .filter(|r| {
            r.get("is_orchestrator")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();

    let json = json!({
        "summary": {
            "total_skills_analyzed": total_skills,
            "orchestrators_detected": orchestrator_count,
            "non_orchestrators": total_skills - orchestrator_count,
        },
        "skills": results,
        "include_recommendations": params.include_recommendations,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string()),
    )]))
}
