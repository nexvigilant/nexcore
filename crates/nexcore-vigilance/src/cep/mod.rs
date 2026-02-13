//! # Cognitive Evolution Pipeline (CEP)
//!
//! An 8-stage method for cross-domain knowledge discovery with feedback loops.
//! Patent: NV-2026-001
//!
//! ## Pipeline Stages
//!
//! | Stage | Name | Purpose |
//! |-------|------|---------|
//! | 1 | SEE | Observe phenomena without prejudice |
//! | 2 | SPEAK | Articulate into structured vocabulary |
//! | 3 | DECOMPOSE | Extract T1/T2/T3 primitives via DAG analysis |
//! | 4 | COMPOSE | Synthesize novel structures from primitives |
//! | 5 | TRANSLATE | Bidirectional cross-domain mapping |
//! | 6 | VALIDATE | Verify coverage, minimality, independence |
//! | 7 | DEPLOY | Generate operational artifacts |
//! | 8 | IMPROVE | Aggregate feedback for next cycle |
//!
//! ## Key Innovation
//!
//! Feedback loop from IMPROVE → SEE enables continuous knowledge refinement.

mod feedback;
mod pipeline;
mod stages;
mod validation;

pub use pipeline::{CepPipeline, PipelineConfig, PipelineExecution, PipelineOutput, PipelineState};

pub use stages::{
    Anomaly,
    Artifact,
    // Stage trait
    CepStage,
    // Stage 4: COMPOSE
    ComposeStage,
    CompositeStructure,
    CompositionResult,
    // Stage 3: DECOMPOSE
    DecomposeStage,
    DependencyGraph,
    // Stage 7: DEPLOY
    DeployStage,
    DeploymentRecord,
    EdgeCase,
    ExtractionResult,
    FeedbackChannel,
    GlossaryEntry,
    GrammarRule,
    // Stage 8: IMPROVE
    ImproveStage,
    ImprovementPlan,
    ImprovementVector,
    LevelDistribution,
    Metric,
    Pattern,
    PerceptionRecord,
    Phenomenon,
    // Stage 1: SEE
    SeeStage,
    // Stage 2: SPEAK
    SpeakStage,
    StageId,
    StageOutput,
    Template,
    // Stage 5: TRANSLATE
    TranslateStage, // Uses domain_discovery::translation types
    // Stage 6: VALIDATE
    ValidateStage,
    ValidationMetrics,
    ValidationReport,
    VocabularyRecord,
};

pub use validation::{
    CoverageScore, ExtractionValidation, IndependenceScore, MinimalityScore, ValidationResult,
    ValidationThresholds,
};

pub use feedback::{FeedbackAggregator, FeedbackPriority, FeedbackSignal, FeedbackSource};

/// CEP version
pub const VERSION: &str = "1.0.0";

/// Default validation thresholds (from patent)
pub const DEFAULT_COVERAGE_THRESHOLD: f64 = 0.95;
pub const DEFAULT_MINIMALITY_THRESHOLD: f64 = 0.90;
pub const DEFAULT_INDEPENDENCE_THRESHOLD: f64 = 0.90;
