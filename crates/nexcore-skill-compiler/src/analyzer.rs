//! Compatibility analysis for compound skill composition.
//!
//! Verifies that all referenced skills exist and are executable,
//! then builds a dependency report.

use nexcore_skill_exec::CompositeExecutor;
use serde::Serialize;

use crate::error::Result;
use crate::spec::CompoundSpec;

/// Result of analyzing a compound spec for compatibility.
#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    /// Non-fatal issues.
    pub warnings: Vec<String>,
    /// Fatal issues that prevent compilation.
    pub blockers: Vec<String>,
    /// Per-skill info discovered during analysis.
    pub skill_infos: Vec<SkillAnalysis>,
    /// Whether compilation can proceed.
    pub can_compile: bool,
}

/// Per-skill analysis result.
#[derive(Debug, Serialize)]
pub struct SkillAnalysis {
    /// Skill name.
    pub name: String,
    /// Whether the skill was found on disk.
    pub found: bool,
    /// Whether the skill has at least one execution method.
    pub executable: bool,
    /// Discovered execution methods.
    pub methods: Vec<String>,
    /// Whether this skill is marked required.
    pub required: bool,
}

/// Analyze a compound spec for compatibility.
///
/// Checks that every referenced skill exists and has executable methods.
pub fn analyze(spec: &CompoundSpec) -> Result<AnalysisReport> {
    let executor = CompositeExecutor::new();
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();
    let mut skill_infos = Vec::new();

    for entry in &spec.skills {
        match executor.discover_skill(&entry.name) {
            Ok(info) => {
                let methods: Vec<String> = info
                    .execution_methods
                    .iter()
                    .map(|m| format!("{m:?}"))
                    .collect();

                let executable = !info.execution_methods.is_empty();
                if !executable {
                    let msg = format!("Skill '{}' has no execution methods", entry.name);
                    if entry.required {
                        blockers.push(msg);
                    } else {
                        warnings.push(msg);
                    }
                }

                skill_infos.push(SkillAnalysis {
                    name: entry.name.clone(),
                    found: true,
                    executable,
                    methods,
                    required: entry.required,
                });
            }
            Err(_) => {
                let msg = format!("Skill not found: {}", entry.name);
                if entry.required {
                    blockers.push(msg);
                } else {
                    warnings.push(msg);
                }

                skill_infos.push(SkillAnalysis {
                    name: entry.name.clone(),
                    found: false,
                    executable: false,
                    methods: Vec::new(),
                    required: entry.required,
                });
            }
        }
    }

    // Check for duplicate skill names
    let mut seen = std::collections::HashSet::new();
    for entry in &spec.skills {
        if !seen.insert(&entry.name) {
            blockers.push(format!("Duplicate skill entry: {}", entry.name));
        }
    }

    let can_compile = blockers.is_empty();

    Ok(AnalysisReport {
        warnings,
        blockers,
        skill_infos,
        can_compile,
    })
}

/// Validate that a set of skill names can be composed.
///
/// Lightweight check that returns a list of missing skills.
pub fn check_skills_exist(skill_names: &[String]) -> Vec<String> {
    let executor = CompositeExecutor::new();
    skill_names
        .iter()
        .filter(|name| executor.discover_skill(name).is_err())
        .cloned()
        .collect()
}
