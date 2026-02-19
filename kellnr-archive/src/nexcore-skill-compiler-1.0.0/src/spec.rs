//! Compound skill specification types parsed from `compound.toml`.

use serde::Deserialize;

use crate::error::{CompilerError, Result};

/// Top-level compound skill specification.
#[derive(Debug, Deserialize)]
pub struct CompoundSpec {
    /// Compound metadata.
    pub compound: CompoundMeta,
    /// Ordered list of sub-skills.
    pub skills: Vec<SkillEntry>,
    /// Threading / result-passing configuration.
    pub threading: Option<ThreadingConfig>,
    /// Feedback loop configuration (only for `FeedbackLoop` strategy).
    pub feedback: Option<FeedbackConfig>,
}

impl CompoundSpec {
    /// Parse and validate a compound spec from TOML text.
    pub fn parse(toml_text: &str) -> Result<Self> {
        let spec: Self = toml::from_str(toml_text)?;
        spec.validate()?;
        Ok(spec)
    }

    /// Validate invariants.
    fn validate(&self) -> Result<()> {
        if self.skills.len() < 2 {
            return Err(CompilerError::InsufficientSkills {
                count: self.skills.len(),
            });
        }

        if self.compound.name.is_empty() {
            return Err(CompilerError::InvalidSpec {
                message: "compound.name must not be empty".into(),
            });
        }

        if matches!(self.compound.strategy, CompositionStrategy::FeedbackLoop)
            && self.feedback.is_none()
        {
            return Err(CompilerError::InvalidSpec {
                message: "feedback_loop strategy requires [feedback] section".into(),
            });
        }

        for (i, skill) in self.skills.iter().enumerate() {
            if skill.name.is_empty() {
                return Err(CompilerError::InvalidSpec {
                    message: format!("skills[{i}].name must not be empty"),
                });
            }
            if skill.timeout_seconds == 0 {
                return Err(CompilerError::InvalidSpec {
                    message: format!("skills[{i}].timeout_seconds must be > 0"),
                });
            }
        }

        Ok(())
    }
}

/// Compound-level metadata.
#[derive(Debug, Deserialize)]
pub struct CompoundMeta {
    /// Name of the compound skill (kebab-case).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Composition strategy.
    #[serde(default)]
    pub strategy: CompositionStrategy,
    /// Tags for discovery.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// How sub-skills are composed.
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CompositionStrategy {
    /// Execute skills one after another, threading results.
    #[default]
    #[serde(rename = "sequential")]
    Sequential,
    /// Execute all skills concurrently.
    #[serde(rename = "parallel")]
    Parallel,
    /// Iterate until convergence.
    #[serde(rename = "feedback_loop")]
    FeedbackLoop,
}

impl std::fmt::Display for CompositionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sequential => write!(f, "sequential"),
            Self::Parallel => write!(f, "parallel"),
            Self::FeedbackLoop => write!(f, "feedback_loop"),
        }
    }
}

/// A sub-skill entry in the compound spec.
#[derive(Debug, Deserialize)]
pub struct SkillEntry {
    /// Skill name (must match registry or filesystem).
    pub name: String,
    /// Whether failure of this skill should abort the pipeline.
    #[serde(default = "default_true")]
    pub required: bool,
    /// Per-skill timeout.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

/// Threading configuration for result passing between skills.
#[derive(Debug, Deserialize)]
pub struct ThreadingConfig {
    /// How results are accumulated.
    #[serde(default)]
    pub mode: ThreadingMode,
    /// Merge strategy for the accumulator.
    #[serde(default)]
    pub merge_strategy: MergeStrategy,
}

/// Result-passing mode.
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ThreadingMode {
    /// Results accumulate in a shared `HashMap<String, Value>`.
    #[default]
    #[serde(rename = "json_accumulator")]
    JsonAccumulator,
}

/// How per-skill results are merged into the accumulator.
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Recursively merge JSON objects.
    #[default]
    #[serde(rename = "deep_merge")]
    DeepMerge,
    /// Later values overwrite earlier ones.
    #[serde(rename = "overwrite")]
    Overwrite,
}

/// Feedback loop parameters.
#[derive(Debug, Deserialize)]
pub struct FeedbackConfig {
    /// Maximum iterations before forced stop.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// JSON path to check for convergence.
    #[serde(default)]
    pub convergence_field: String,
    /// Threshold value for convergence.
    #[serde(default = "default_threshold")]
    pub convergence_threshold: f64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    60
}

fn default_max_iterations() -> u32 {
    5
}

fn default_threshold() -> f64 {
    0.85
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_spec() {
        let toml = r#"
[compound]
name = "test-compound"
description = "Two skills"
strategy = "sequential"

[[skills]]
name = "skill-a"

[[skills]]
name = "skill-b"
"#;
        let spec = CompoundSpec::parse(toml).unwrap_or_else(|e| panic!("parse failed: {e}"));
        assert_eq!(spec.compound.name, "test-compound");
        assert_eq!(spec.skills.len(), 2);
        assert!(spec.skills[0].required);
        assert_eq!(spec.skills[0].timeout_seconds, 60);
    }

    #[test]
    fn reject_single_skill() {
        let toml = r#"
[compound]
name = "solo"

[[skills]]
name = "only-one"
"#;
        let err = CompoundSpec::parse(toml).unwrap_err();
        assert!(err.to_string().contains("Insufficient skills"));
    }

    #[test]
    fn feedback_requires_config() {
        let toml = r#"
[compound]
name = "looper"
strategy = "feedback_loop"

[[skills]]
name = "a"

[[skills]]
name = "b"
"#;
        let err = CompoundSpec::parse(toml).unwrap_err();
        assert!(err.to_string().contains("feedback"));
    }

    #[test]
    fn parse_full_spec() {
        let toml = r#"
[compound]
name = "pv-pipeline"
description = "Signal → causality → report"
strategy = "sequential"
tags = ["pv", "pipeline"]

[[skills]]
name = "signal-detector"
required = true
timeout_seconds = 120

[[skills]]
name = "causality-assessor"
required = true
timeout_seconds = 60

[[skills]]
name = "report-generator"
required = false
timeout_seconds = 30

[threading]
mode = "json_accumulator"
merge_strategy = "deep_merge"
"#;
        let spec = CompoundSpec::parse(toml).unwrap_or_else(|e| panic!("parse failed: {e}"));
        assert_eq!(spec.skills.len(), 3);
        assert!(!spec.skills[2].required);
        assert_eq!(spec.skills[0].timeout_seconds, 120);
        assert!(spec.threading.is_some());
    }
}
