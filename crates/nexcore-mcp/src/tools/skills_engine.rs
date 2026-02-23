//! Skills Engine MCP tools — advanced skill analysis and quality metrics.
//!
//! Extends the existing skills toolset with deeper analytics:
//! - `skill_quality_index`: Extended SQI with registry grounding
//! - `skill_maturity`: Practice + Consistency + Transfer maturity assessment
//! - `skill_ksb_verify`: KSB compliance verification
//! - `skill_ecosystem_score`: Ecosystem health with taxonomy context
//! - `skill_dependency_graph`: Upstream/downstream graph traversal
//! - `skill_gap_analysis`: Gap identification against target compliance
//! - `skill_evolution_track`: Temporal evolution metrics

use crate::params::{
    SkillDependencyGraphParams, SkillEcosystemScoreParams, SkillEvolutionTrackParams,
    SkillGapAnalysisParams, SkillKsbVerifyParams, SkillMaturityParams, SkillQualityIndexParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Value, json};
use std::path::Path;

/// Expand `~/` to the user's home directory
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}{}", &path[1..]);
        }
    }
    path.to_string()
}

/// Compute Skill Quality Index with extended per-dimension analysis
pub fn skill_quality_index(params: SkillQualityIndexParams) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.path);
    let path = Path::new(&expanded);
    let skill_md = path.join("SKILL.md");

    let content = std::fs::read_to_string(&skill_md).map_err(|e| {
        McpError::invalid_params(format!("Cannot read {}: {e}", skill_md.display()), None)
    })?;

    let result = nexcore_skills_engine::sqi::compute_sqi(&content)
        .map_err(|e| McpError::internal_error(format!("SQI computation failed: {e:?}"), None))?;

    let sensitivity = nexcore_skills_engine::sqi::sensitivity_analysis(&result, 0.1);

    let response = json!({
        "skill_path": params.path,
        "score": result.sqi,
        "grade": format!("{:?}", result.grade),
        "limiting_dimension": format!("{:?}", result.limiting_dimension),
        "dimensions": result.dimensions.iter().map(|d| json!({
            "dimension": format!("{:?}", d.dimension),
            "score": d.score,
            "weight": d.weight,
            "weighted": d.weighted,
            "rationale": d.rationale,
        })).collect::<Vec<_>>(),
        "recommendations": result.recommendations,
        "sensitivity": sensitivity.iter().map(|(dim, delta)| json!({
            "dimension": format!("{dim:?}"),
            "sensitivity_delta": delta,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Compute skill maturity from Practice, Consistency, and Transfer primitives
pub fn skill_maturity(params: SkillMaturityParams) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.path);
    let path = Path::new(&expanded);
    let skill_md = path.join("SKILL.md");

    let content = std::fs::read_to_string(&skill_md).map_err(|e| {
        McpError::invalid_params(format!("Cannot read {}: {e}", skill_md.display()), None)
    })?;

    // Compute SQI as baseline maturity signal
    let sqi = nexcore_skills_engine::sqi::compute_sqi(&content)
        .map_err(|e| McpError::internal_error(format!("SQI computation failed: {e:?}"), None))?;

    // SMST validation for structural maturity
    let smst = nexcore_skills_engine::smst_v2::extract_smst_v2(&content);

    // Derive maturity tier from composite signals
    let maturity_score = sqi.sqi * 0.6 + if smst.is_diamond { 0.4 } else { 0.0 };
    let maturity_tier = if maturity_score >= 0.85 {
        "Expert"
    } else if maturity_score >= 0.70 {
        "Proficient"
    } else if maturity_score >= 0.50 {
        "Competent"
    } else if maturity_score >= 0.30 {
        "Advanced Beginner"
    } else {
        "Novice"
    };

    let response = json!({
        "skill_path": params.path,
        "maturity_score": (maturity_score * 100.0).round() / 100.0,
        "maturity_tier": maturity_tier,
        "sqi_score": sqi.sqi,
        "sqi_grade": format!("{:?}", sqi.grade),
        "smst_passed": smst.is_diamond,
        "smst_total_score": smst.total_score,
        "smst_compliance_level": smst.compliance_level,
        "limiting_factor": format!("{:?}", sqi.limiting_dimension),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Verify KSB (Knowledge, Skills, Behaviours) compliance
pub fn skill_ksb_verify(params: SkillKsbVerifyParams) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.path);
    let path = Path::new(&expanded);

    let validation = nexcore_skills_engine::ksb_verify::verify_ksb(path)
        .map_err(|e| McpError::internal_error(format!("KSB verification failed: {e:?}"), None))?;

    let response = json!({
        "skill_name": validation.skill_name,
        "passed": validation.passed,
        "passed_count": validation.passed_count,
        "total_count": validation.total_count,
        "compliance_level": format!("{:?}", validation.compliance_level),
        "checks": validation.checks.iter().map(|c| json!({
            "name": c.name,
            "passed": c.passed,
            "message": c.message,
            "match_ratio": c.match_ratio,
        })).collect::<Vec<_>>(),
        "suggestions": validation.suggestions,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Compute ecosystem-level score with taxonomy and distribution analysis
pub fn skill_ecosystem_score(
    params: SkillEcosystemScoreParams,
) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.directory);
    let dir = Path::new(&expanded);

    // Scan skills directory
    let mut registry = nexcore_skills_engine::registry::SkillRegistry::default();
    let count = registry
        .scan(dir)
        .map_err(|e| McpError::internal_error(format!("Skill scan failed: {e}"), None))?;

    let skills = registry.list();
    let tool_counts: Vec<usize> = skills.iter().map(|s| s.tags.len().max(1)).collect();

    // Score individual skills
    let mut sqi_results = Vec::new();
    for skill in &skills {
        if let Ok(content) = std::fs::read_to_string(&skill.path) {
            if let Ok(result) = nexcore_skills_engine::sqi::compute_sqi(&content) {
                sqi_results.push(result);
            }
        }
    }

    let ecosystem = nexcore_skills_engine::sqi::compute_ecosystem_sqi(&sqi_results, &tool_counts);

    let response = json!({
        "directory": params.directory,
        "total_skills": count,
        "scored_skills": sqi_results.len(),
        "mean_sqi_unweighted": ecosystem.mean_sqi_unweighted,
        "mean_sqi_weighted": ecosystem.mean_sqi_weighted,
        "distribution_entropy": ecosystem.distribution_entropy,
        "concentration_risk": ecosystem.concentration_risk,
        "ecosystem_grade": format!("{:?}", ecosystem.grade),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Traverse skill dependency graph (upstream/downstream/see_also)
pub fn skill_dependency_graph(
    params: SkillDependencyGraphParams,
) -> Result<CallToolResult, McpError> {
    let skills_dir = expand_tilde("~/.claude/skills");
    let dir = Path::new(&skills_dir);

    let mut registry = nexcore_skills_engine::registry::SkillRegistry::default();
    registry
        .scan(dir)
        .map_err(|e| McpError::internal_error(format!("Skill scan failed: {e}"), None))?;

    let root = registry.get(&params.root_skill).ok_or_else(|| {
        McpError::invalid_params(
            format!("Skill '{}' not found in registry", params.root_skill),
            None,
        )
    })?;

    // Build graph from registry metadata
    let mut nodes = vec![json!({
        "name": root.name,
        "depth": 0,
        "upstream": root.upstream,
        "downstream": root.downstream,
        "see_also": root.see_also,
    })];

    // BFS traversal up to max_depth
    let mut visited = std::collections::HashSet::new();
    visited.insert(root.name.clone());
    let mut frontier: Vec<(String, usize)> = root
        .upstream
        .iter()
        .chain(root.downstream.iter())
        .chain(root.see_also.iter())
        .map(|s| (s.clone(), 1))
        .collect();

    while let Some((name, depth)) = frontier.pop() {
        if depth > params.max_depth || visited.contains(&name) {
            continue;
        }
        visited.insert(name.clone());

        if let Some(skill) = registry.get(&name) {
            nodes.push(json!({
                "name": skill.name,
                "depth": depth,
                "upstream": skill.upstream,
                "downstream": skill.downstream,
                "see_also": skill.see_also,
            }));

            if depth < params.max_depth {
                for next in skill
                    .upstream
                    .iter()
                    .chain(skill.downstream.iter())
                    .chain(skill.see_also.iter())
                {
                    frontier.push((next.clone(), depth + 1));
                }
            }
        }
    }

    let response = json!({
        "root": params.root_skill,
        "max_depth": params.max_depth,
        "total_nodes": nodes.len(),
        "graph": nodes,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Identify skill gaps against a target compliance level
pub fn skill_gap_analysis(params: SkillGapAnalysisParams) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.path);
    let path = Path::new(&expanded);

    // KSB verification for structural gaps
    let ksb = nexcore_skills_engine::ksb_verify::verify_ksb(path)
        .map_err(|e| McpError::internal_error(format!("KSB verification failed: {e:?}"), None))?;

    // SQI for quality gaps
    let skill_md = path.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md).map_err(|e| {
        McpError::invalid_params(format!("Cannot read {}: {e}", skill_md.display()), None)
    })?;

    let sqi = nexcore_skills_engine::sqi::compute_sqi(&content)
        .map_err(|e| McpError::internal_error(format!("SQI computation failed: {e:?}"), None))?;

    // SMST structural assessment
    let smst = nexcore_skills_engine::smst_v2::extract_smst_v2(&content);

    // Collect all gaps
    let mut gaps: Vec<Value> = Vec::new();

    // KSB gaps
    for check in &ksb.checks {
        if !check.passed {
            gaps.push(json!({
                "category": "ksb",
                "check": check.name,
                "message": check.message,
                "severity": if check.match_ratio < 0.3 { "high" } else { "medium" },
            }));
        }
    }

    // SQI dimension gaps
    let target_score = match params.target_compliance.to_lowercase().as_str() {
        "diamond" => 0.90,
        "platinum" => 0.80,
        "gold" => 0.65,
        "silver" => 0.50,
        _ => 0.40,
    };

    for dim in &sqi.dimensions {
        if dim.score < target_score {
            gaps.push(json!({
                "category": "quality",
                "dimension": format!("{:?}", dim.dimension),
                "current": dim.score,
                "target": target_score,
                "gap": target_score - dim.score,
                "severity": if dim.score < target_score * 0.5 { "high" } else { "medium" },
            }));
        }
    }

    // SMST gaps
    if !smst.is_diamond {
        gaps.push(json!({
            "category": "structure",
            "check": "smst_v2",
            "total_score": smst.total_score,
            "compliance_level": smst.compliance_level,
            "severity": "medium",
        }));
    }

    let response = json!({
        "skill_path": params.path,
        "target_compliance": params.target_compliance,
        "current_grade": format!("{:?}", sqi.grade),
        "current_score": sqi.sqi,
        "target_score": target_score,
        "total_gaps": gaps.len(),
        "gaps": gaps,
        "suggestions": ksb.suggestions,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Track skill evolution over time using filesystem timestamps and build history
pub fn skill_evolution_track(
    params: SkillEvolutionTrackParams,
) -> Result<CallToolResult, McpError> {
    let skills_dir = expand_tilde("~/.claude/skills");
    let dir = Path::new(&skills_dir);

    let mut registry = nexcore_skills_engine::registry::SkillRegistry::default();
    registry
        .scan(dir)
        .map_err(|e| McpError::internal_error(format!("Skill scan failed: {e}"), None))?;

    let skill = registry.get(&params.skill_name).ok_or_else(|| {
        McpError::invalid_params(format!("Skill '{}' not found", params.skill_name), None)
    })?;

    // Get file modification time as proxy for last update
    let skill_path = Path::new(&skill.path).join("SKILL.md");
    let metadata = std::fs::metadata(&skill_path).map_err(|e| {
        McpError::internal_error(format!("Cannot stat {}: {e}", skill_path.display()), None)
    })?;

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    // Current quality snapshot
    let content = std::fs::read_to_string(&skill_path).unwrap_or_default();
    let sqi = nexcore_skills_engine::sqi::compute_sqi(&content).ok();
    let smst = nexcore_skills_engine::smst_v2::extract_smst_v2(&content);

    let response = json!({
        "skill_name": params.skill_name,
        "path": skill.path,
        "last_modified_epoch": modified,
        "current_snapshot": {
            "sqi_score": sqi.as_ref().map(|s| s.sqi),
            "sqi_grade": sqi.as_ref().map(|s| format!("{:?}", s.grade)),
            "smst_is_diamond": smst.is_diamond,
            "smst_total_score": smst.total_score,
            "tags": skill.tags,
            "compliance": format!("{:?}", skill.compliance),
        },
        "dependencies": {
            "upstream": skill.upstream,
            "downstream": skill.downstream,
            "see_also": skill.see_also,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
