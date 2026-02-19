//! Skeletal System MCP tools — structural knowledge framework.
//!
//! Maps Claude Code's knowledge structure to the skeleton:
//! - Skull: CLAUDE.md (protects the brain)
//! - Spine: Memory edits (vertebral structure)
//! - Ribs: User preferences (protect vital organs)
//! - Appendicular: Filesystem structure (limbs for interaction)
//!
//! ## T1 Primitive Grounding
//! - Structure: ×(Product) + σ(Sequence)
//! - Reinforcement: ν(Frequency) + π(Persistence)
//! - Joints: μ(Mapping) + ∂(Boundary)

use crate::params::skeletal::{
    SkeletalHealthParams, SkeletalStructureParams, SkeletalWolffsLawParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Assess skeletal health (structural knowledge integrity).
pub fn health(_params: SkeletalHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let claude_md = std::path::Path::new(&format!("{}/.claude/CLAUDE.md", home)).exists();
    let nexcore_claude_md = std::path::Path::new(&format!("{}/nexcore/CLAUDE.md", home)).exists();
    let settings = std::path::Path::new(&format!("{}/.claude/settings.json", home)).exists();
    let skills_dir = std::path::Path::new(&format!("{}/.claude/skills", home)).exists();

    let bone_count = [claude_md, nexcore_claude_md, settings, skills_dir]
        .iter()
        .filter(|&&b| b)
        .count();

    let health = if bone_count == 4 {
        "healthy"
    } else if bone_count >= 2 {
        "degraded"
    } else {
        "fractured"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "skeletal_health": {
                "status": health,
                "bone_inventory": {
                    "skull_claude_md": claude_md,
                    "skull_nexcore_claude_md": nexcore_claude_md,
                    "ribs_settings": settings,
                    "appendicular_skills": skills_dir,
                },
                "score": format!("{}/4 structural elements present", bone_count),
            },
            "analog": {
                "skull": "CLAUDE.md files — protect the brain (instructions)",
                "spine": "Memory edits — vertebral knowledge structure",
                "ribs": "Settings/preferences — protect vital organs",
                "appendicular": "Skills directory — limbs for interaction",
            },
        })
        .to_string(),
    )]))
}

/// Evaluate Wolff's Law reinforcement needs.
pub fn wolffs_law(params: SkeletalWolffsLawParams) -> Result<CallToolResult, McpError> {
    let domain = &params.domain;
    let corrections = params.correction_count.unwrap_or(0);

    // Wolff's Law: bone remodels under stress
    // More corrections = more stress = needs reinforcement (more CLAUDE.md entries)
    let stress_level = if corrections >= 5 {
        "high"
    } else if corrections >= 2 {
        "moderate"
    } else {
        "low"
    };

    let should_reinforce = corrections >= 3;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "wolffs_law": {
                "domain": domain,
                "correction_count": corrections,
                "stress_level": stress_level,
                "should_add_to_claude_md": should_reinforce,
                "recommendation": if should_reinforce {
                    format!("Domain '{}' has {} corrections — add explicit guidance to CLAUDE.md", domain, corrections)
                } else {
                    format!("Domain '{}' stress within normal limits", domain)
                },
            },
            "analog": {
                "stress": "Repeated corrections in same domain",
                "remodeling": "Adding CLAUDE.md entries where corrections concentrate",
                "osteoporosis": "Knowledge gaps from missing documentation",
            },
        })
        .to_string(),
    )]))
}

/// Get project skeleton structure snapshot.
pub fn structure(_params: SkeletalStructureParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // Count structural elements
    let skill_count = std::fs::read_dir(format!("{}/.claude/skills", home))
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_type().is_ok_and(|ft| ft.is_dir()))
                .count()
        })
        .unwrap_or(0);

    let hook_count = std::fs::read_dir(format!("{}/.claude/hooks/core-hooks/target/release", home))
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.file_type().is_ok_and(|ft| ft.is_file())
                        && !e.file_name().to_string_lossy().contains('.')
                        && !e.file_name().to_string_lossy().starts_with("lib")
                })
                .count()
        })
        .unwrap_or(0);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "skeleton": {
                "axial": {
                    "skull": "CLAUDE.md files (knowledge protection)",
                    "spine_vertebrae": "Memory edits (knowledge structure)",
                    "ribs": "Settings + preferences (organ protection)",
                    "sternum": "Hook infrastructure (central stability)",
                },
                "appendicular": {
                    "skills": skill_count,
                    "hooks": hook_count,
                    "description": "Interaction limbs — skills for reaching, hooks for grasping",
                },
                "joints": {
                    "ball_and_socket": "MCP protocol (full range of motion)",
                    "hinge": "Tool dispatch (one-axis: request → response)",
                    "fixed_suture": "Skill composition (immutable bonds)",
                },
                "marrow": {
                    "description": "Produces blood cells (new tools, hooks, skills)",
                    "red_production": "MCP tool generation",
                    "white_production": "Hook/validator generation",
                },
            },
        })
        .to_string(),
    )]))
}
