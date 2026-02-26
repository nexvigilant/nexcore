//! # NexVigilant Core — Skill Compiler
//!
//! Composes 2+ existing skills into a single executable compound skill.
//!
//! ## Pipeline
//!
//! ```text
//! compound.toml → parse → analyze → codegen → build → verify
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use nexcore_fs::dirs;

pub mod analyzer;
pub mod builder;
pub mod codegen;
pub mod error;
pub mod grounding;
pub mod spec;

use std::path::PathBuf;

use error::{CompilerError, Result};
use serde::Serialize;
use spec::{CompositionStrategy, CompoundSpec};

/// Result of a full compilation.
#[derive(Debug, Serialize)]
pub struct CompilationResult {
    /// Compound skill name.
    pub name: String,
    /// Path to the generated crate.
    pub crate_dir: PathBuf,
    /// Path to the compiled binary (if build was requested).
    pub binary_path: Option<PathBuf>,
    /// Path to the generated SKILL.md.
    pub skill_md: PathBuf,
    /// Analysis report.
    pub analysis: analyzer::AnalysisReport,
    /// Whether the compound skill is Diamond-compliant.
    pub diamond_compliant: bool,
}

/// Full compilation pipeline: parse → analyze → codegen → build.
///
/// # Errors
///
/// Returns `CompilerError` for parse failures, missing skills, or build failures.
pub fn compile(toml_text: &str, do_build: bool) -> Result<CompilationResult> {
    let spec = CompoundSpec::parse(toml_text)?;

    let analysis = analyzer::analyze(&spec)?;
    if !analysis.can_compile {
        return Err(CompilerError::InvalidSpec {
            message: format!("Analysis found blockers: {}", analysis.blockers.join("; ")),
        });
    }

    let output_dir = output_directory()?;
    let generated = codegen::generate(&spec, &output_dir)?;

    let binary_path = if do_build {
        let build_result = builder::build(&generated.root, &spec.compound.name)?;
        Some(build_result.binary_path)
    } else {
        None
    };

    Ok(CompilationResult {
        name: spec.compound.name.clone(),
        crate_dir: generated.root,
        binary_path,
        skill_md: generated.skill_md,
        analysis,
        diamond_compliant: true,
    })
}

/// Dry-run compatibility check without building.
///
/// # Errors
///
/// Returns `CompilerError` if fewer than 2 skills provided.
pub fn check(skill_names: &[String], strategy: &str) -> Result<analyzer::AnalysisReport> {
    if skill_names.len() < 2 {
        return Err(CompilerError::InsufficientSkills {
            count: skill_names.len(),
        });
    }

    let strategy_enum = match strategy {
        "parallel" => CompositionStrategy::Parallel,
        "feedback_loop" => CompositionStrategy::FeedbackLoop,
        _ => CompositionStrategy::Sequential,
    };

    let spec = CompoundSpec {
        compound: spec::CompoundMeta {
            name: "check".into(),
            description: "Compatibility check".into(),
            strategy: strategy_enum,
            tags: Vec::new(),
        },
        skills: skill_names
            .iter()
            .map(|name| spec::SkillEntry {
                name: name.clone(),
                required: true,
                timeout_seconds: 60,
            })
            .collect(),
        threading: None,
        feedback: None,
    };

    analyzer::analyze(&spec)
}

/// Build a compound spec TOML from parameters (for MCP integration).
///
/// # Errors
///
/// Returns `CompilerError` if fewer than 2 skills provided.
pub fn spec_from_params(skills: &[String], strategy: &str, name: &str) -> Result<String> {
    if skills.len() < 2 {
        return Err(CompilerError::InsufficientSkills {
            count: skills.len(),
        });
    }

    let skill_entries: Vec<String> = skills
        .iter()
        .map(|s| format!("[[skills]]\nname = \"{s}\"\nrequired = true\ntimeout_seconds = 60"))
        .collect();

    Ok(format!(
        "[compound]\nname = \"{name}\"\ndescription = \"Compiled compound skill: {name}\"\nstrategy = \"{strategy}\"\ntags = [\"compound\", \"generated\"]\n\n{skills}\n",
        name = name,
        strategy = strategy,
        skills = skill_entries.join("\n\n"),
    ))
}

/// Default output directory for generated compound skill crates.
fn output_directory() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("skills"))
        .ok_or_else(|| CompilerError::CodegenFailed {
            stage: "output_dir".into(),
            message: "Could not determine home directory".into(),
        })?;

    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
