//! Pipeline definitions — named, reusable build pipeline templates.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: σ (Sequence) | Ordered stage execution |
//! | T1: μ (Mapping) | Name → definition lookup |
//! | T1: → (Causality) | depends_on triggers |
//! | T1: ρ (Recursion) | DAG dependency resolution |
//! | T1: ∂ (Boundary) | Timeout constraints |

use crate::pipeline::stage::{PipelineStage, StageConfig};
use crate::types::StageId;
use serde::{Deserialize, Serialize};

/// A named pipeline definition: an ordered set of stages with dependencies.
///
/// Tier: T3 (σ + μ + ∂ + → + ρ + ς, dominant σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    /// Pipeline name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Ordered stage configurations.
    pub stages: Vec<StageConfig>,
}

impl PipelineDefinition {
    /// Create a new pipeline definition.
    #[must_use]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            stages: Vec::new(),
        }
    }

    /// Add a stage.
    #[must_use]
    pub fn with_stage(mut self, config: StageConfig) -> Self {
        self.stages.push(config);
        self
    }

    /// Get all stage IDs.
    #[must_use]
    pub fn stage_ids(&self) -> Vec<StageId> {
        self.stages.iter().map(|s| s.id.clone()).collect()
    }

    /// Resolve the next runnable stages (all dependencies satisfied).
    #[must_use]
    pub fn next_runnable(&self, completed: &[StageId]) -> Vec<&StageConfig> {
        self.stages
            .iter()
            .filter(|s| {
                // Not yet completed
                !completed.contains(&s.id)
                    // All dependencies are in completed
                    && s.depends_on.iter().all(|dep| completed.contains(dep))
            })
            .collect()
    }

    /// The built-in "validate" pipeline: `fmt → (clippy | test | docs) → build → coverage`.
    #[must_use]
    pub fn validate() -> Self {
        let fmt_id = StageId("fmt".into());
        let clippy_id = StageId("clippy".into());
        let test_id = StageId("test".into());
        let docs_id = StageId("docs".into());
        let build_id = StageId("build".into());
        let coverage_id = StageId("coverage".into());

        Self::new(
            "validate",
            "Full CI: fmt → (clippy | test | docs) → build → coverage",
        )
        .with_stage(StageConfig::new("fmt", PipelineStage::Fmt))
        .with_stage(
            StageConfig::new("clippy", PipelineStage::Clippy).depends_on(vec![fmt_id.clone()]),
        )
        .with_stage(StageConfig::new("test", PipelineStage::Test).depends_on(vec![fmt_id.clone()]))
        .with_stage(
            StageConfig::new("docs", PipelineStage::Docs)
                .depends_on(vec![fmt_id.clone()])
                .allow_failure(true),
        )
        .with_stage(
            StageConfig::new("build", PipelineStage::Build).depends_on(vec![
                clippy_id.clone(),
                test_id.clone(),
                docs_id.clone(),
            ]),
        )
        .with_stage(
            StageConfig::new("coverage", PipelineStage::Coverage)
                .depends_on(vec![build_id])
                .allow_failure(true),
        )
    }

    /// The built-in "validate-quick" pipeline: `check → (clippy | test-core)`.
    ///
    /// Clippy and test-core run in parallel after check passes.
    #[must_use]
    pub fn validate_quick() -> Self {
        let check_id = StageId("check".into());

        Self::new(
            "validate-quick",
            "Quick validation: check → (clippy | test-core)",
        )
        .with_stage(StageConfig::new("check", PipelineStage::Check))
        .with_stage(
            StageConfig::new("clippy", PipelineStage::Clippy).depends_on(vec![check_id.clone()]),
        )
        .with_stage(
            StageConfig::new("test-core", PipelineStage::Test)
                .depends_on(vec![check_id])
                .cargo_args(vec![
                    "test".into(),
                    "-p".into(),
                    "nexcore-vigilance".into(),
                    "--lib".into(),
                ]),
        )
    }

    /// Look up a built-in pipeline by name.
    #[must_use]
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            "validate" => Some(Self::validate()),
            "validate-quick" => Some(Self::validate_quick()),
            _ => None,
        }
    }

    /// List all built-in pipeline names.
    #[must_use]
    pub fn builtin_names() -> Vec<&'static str> {
        vec!["validate", "validate-quick"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_pipeline_has_6_stages() {
        let p = PipelineDefinition::validate();
        assert_eq!(p.stages.len(), 6);
        assert_eq!(p.name, "validate");
    }

    #[test]
    fn validate_quick_pipeline_has_3_stages() {
        let p = PipelineDefinition::validate_quick();
        assert_eq!(p.stages.len(), 3);
        assert_eq!(p.name, "validate-quick");
    }

    #[test]
    fn builtin_lookup() {
        assert!(PipelineDefinition::builtin("validate").is_some());
        assert!(PipelineDefinition::builtin("validate-quick").is_some());
        assert!(PipelineDefinition::builtin("nonexistent").is_none());
    }

    #[test]
    fn builtin_names_match() {
        let names = PipelineDefinition::builtin_names();
        assert_eq!(names.len(), 2);
        for name in &names {
            assert!(PipelineDefinition::builtin(name).is_some());
        }
    }

    #[test]
    fn validate_first_wave_is_fmt() {
        let p = PipelineDefinition::validate();
        let completed: Vec<StageId> = vec![];
        let runnable = p.next_runnable(&completed);
        assert_eq!(runnable.len(), 1);
        assert_eq!(runnable[0].id.0, "fmt");
    }

    #[test]
    fn validate_second_wave_is_parallel() {
        let p = PipelineDefinition::validate();
        let completed = vec![StageId("fmt".into())];
        let runnable = p.next_runnable(&completed);
        // clippy, test, docs should all be runnable after fmt
        assert_eq!(runnable.len(), 3);
        let names: Vec<&str> = runnable.iter().map(|s| s.id.0.as_str()).collect();
        assert!(names.contains(&"clippy"));
        assert!(names.contains(&"test"));
        assert!(names.contains(&"docs"));
    }

    #[test]
    fn validate_quick_parallel_wave() {
        let p = PipelineDefinition::validate_quick();

        // Wave 1: only check
        let wave1 = p.next_runnable(&[]);
        assert_eq!(wave1.len(), 1);
        assert_eq!(wave1[0].id.0, "check");

        // Wave 2: clippy AND test-core in parallel after check
        let wave2 = p.next_runnable(&[StageId("check".into())]);
        assert_eq!(wave2.len(), 2);
        let names: Vec<&str> = wave2.iter().map(|s| s.id.0.as_str()).collect();
        assert!(names.contains(&"clippy"));
        assert!(names.contains(&"test-core"));
    }

    #[test]
    fn dry_run_validate() {
        let p = PipelineDefinition::validate();
        let waves = crate::pipeline::executor::dry_run(&p);
        assert_eq!(waves.len(), 4); // fmt | clippy+test+docs | build | coverage
        assert_eq!(waves[0].len(), 1); // fmt
        assert_eq!(waves[1].len(), 3); // parallel wave
        assert_eq!(waves[2].len(), 1); // build
        assert_eq!(waves[3].len(), 1); // coverage
    }

    #[test]
    fn dry_run_validate_quick() {
        let p = PipelineDefinition::validate_quick();
        let waves = crate::pipeline::executor::dry_run(&p);
        assert_eq!(waves.len(), 2); // check | clippy+test-core
        assert_eq!(waves[0].len(), 1); // check
        assert_eq!(waves[1].len(), 2); // clippy + test-core in parallel
    }

    #[test]
    fn stage_config_builder() {
        let config = StageConfig::new("my-stage", PipelineStage::Test)
            .allow_failure(true)
            .depends_on(vec![StageId("prev".into())])
            .cargo_args(vec!["test".into(), "-p".into(), "my-crate".into()]);

        assert!(config.allow_failure);
        assert_eq!(config.depends_on.len(), 1);
        assert_eq!(config.effective_args(), vec!["test", "-p", "my-crate"]);
    }

    #[test]
    fn stage_config_default_args() {
        let config = StageConfig::new("fmt", PipelineStage::Fmt);
        let args = config.effective_args();
        assert_eq!(args, vec!["fmt", "--all", "--", "--check"]);
    }

    #[test]
    fn pipeline_definition_serde_roundtrip() {
        let p = PipelineDefinition::validate_quick();
        let json = serde_json::to_string(&p);
        assert!(json.is_ok());
        let back: Result<PipelineDefinition, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(back.is_ok());
        let back = back.unwrap_or_else(|_| PipelineDefinition::new("x", "x"));
        assert_eq!(back.name, "validate-quick");
        assert_eq!(back.stages.len(), 3);
    }
}
