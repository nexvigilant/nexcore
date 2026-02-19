//! Pipeline stage definitions and configuration.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Σ (Sum) | Stage variants as alternation |
//! | T1: σ (Sequence) | Stages execute in ordered pipeline |
//! | T1: μ (Mapping) | Config maps stage → parameters |
//! | T1: ∂ (Boundary) | Timeouts, allow_failure constraints |

use crate::types::StageId;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Pipeline stage type — each variant corresponds to a cargo/build operation.
///
/// Tier: T2-P (Σ + σ, dominant Σ)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    /// `cargo fmt --all -- --check`
    Fmt,
    /// `cargo check --workspace`
    Check,
    /// `cargo clippy --workspace -- -D warnings`
    Clippy,
    /// `cargo test --workspace` (or specific package)
    Test,
    /// `cargo build --workspace`
    Build,
    /// `cargo doc --workspace --no-deps`
    Docs,
    /// `cargo audit`
    Audit,
    /// `cargo llvm-cov`
    Coverage,
    /// Custom user-defined stage
    Custom(String),
}

impl PipelineStage {
    /// Display name for this stage.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Fmt => "fmt",
            Self::Check => "check",
            Self::Clippy => "clippy",
            Self::Test => "test",
            Self::Build => "build",
            Self::Docs => "docs",
            Self::Audit => "audit",
            Self::Coverage => "coverage",
            Self::Custom(name) => name,
        }
    }

    /// Default cargo arguments for this stage.
    #[must_use]
    pub fn default_args(&self) -> Vec<String> {
        match self {
            Self::Fmt => vec!["fmt".into(), "--all".into(), "--".into(), "--check".into()],
            Self::Check => vec!["check".into(), "--workspace".into()],
            Self::Clippy => vec![
                "clippy".into(),
                "--workspace".into(),
                "--".into(),
                "-D".into(),
                "warnings".into(),
            ],
            Self::Test => vec!["test".into(), "--workspace".into()],
            Self::Build => vec!["build".into(), "--workspace".into()],
            Self::Docs => vec!["doc".into(), "--workspace".into(), "--no-deps".into()],
            Self::Audit => vec!["audit".into()],
            Self::Coverage => vec!["llvm-cov".into()],
            Self::Custom(_) => vec![],
        }
    }

    /// Default timeout for this stage.
    #[must_use]
    pub fn default_timeout(&self) -> Duration {
        match self {
            Self::Fmt | Self::Check => Duration::from_secs(120),
            Self::Clippy => Duration::from_secs(300),
            Self::Test => Duration::from_secs(600),
            Self::Build => Duration::from_secs(600),
            Self::Docs => Duration::from_secs(300),
            Self::Audit => Duration::from_secs(60),
            Self::Coverage => Duration::from_secs(900),
            Self::Custom(_) => Duration::from_secs(300),
        }
    }
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Configuration for a single stage in a pipeline definition.
///
/// Tier: T2-C (μ + ∂ + → + λ, dominant μ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    /// Unique stage identifier within this pipeline.
    pub id: StageId,
    /// The type of stage.
    pub stage: PipelineStage,
    /// Maximum execution time.
    pub timeout: Duration,
    /// Whether failure of this stage allows the pipeline to continue.
    pub allow_failure: bool,
    /// Stage IDs that must complete before this stage starts (→ Causality).
    pub depends_on: Vec<StageId>,
    /// Override cargo arguments (empty = use stage defaults).
    pub cargo_args: Vec<String>,
}

impl StageConfig {
    /// Create a new stage config with defaults.
    #[must_use]
    pub fn new(id: impl Into<String>, stage: PipelineStage) -> Self {
        let timeout = stage.default_timeout();
        Self {
            id: StageId(id.into()),
            stage,
            timeout,
            allow_failure: false,
            depends_on: Vec::new(),
            cargo_args: Vec::new(),
        }
    }

    /// Set explicit dependencies.
    #[must_use]
    pub fn depends_on(mut self, deps: Vec<StageId>) -> Self {
        self.depends_on = deps;
        self
    }

    /// Set allow_failure flag.
    #[must_use]
    pub fn allow_failure(mut self, allow: bool) -> Self {
        self.allow_failure = allow;
        self
    }

    /// Set custom cargo arguments.
    #[must_use]
    pub fn cargo_args(mut self, args: Vec<String>) -> Self {
        self.cargo_args = args;
        self
    }

    /// Get the effective cargo arguments (custom or default).
    #[must_use]
    pub fn effective_args(&self) -> Vec<String> {
        if self.cargo_args.is_empty() {
            self.stage.default_args()
        } else {
            self.cargo_args.clone()
        }
    }
}
