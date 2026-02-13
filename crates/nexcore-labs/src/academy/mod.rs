//! # Academy Domain - Learning & Quality
//!
//! Consolidated from `nexcore-academy` into the master vigilance crate.
//! Quality scoring, KSB validation, capability prioritization, and competency framework.
//!
//! ## Constructive Epistemology
//!
//! This module applies the Construction Pipeline (SEE → SPEAK → DECOMPOSE →
//! COMPOSE → TRANSLATE → VALIDATE → DEPLOY → IMPROVE) to extract pure
//! algorithms from the Academy module.
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Weight constants, thresholds
//! - **L1 Atoms**: Score calculations (<20 LOC)
//! - **L2 Molecules**: Composite validators (<50 LOC)
//!
//! ## Modules
//!
//! - [`quality`] - Quality scoring with weighted agents
//! - [`ksb`] - Knowledge-Skills-Behaviors component types
//!   - [`ksb::research`] - KSB research pipeline (quality, metrics, grading)
//! - [`capability`] - Priority scoring for strategic capabilities
//! - [`job`] - Type-safe job state machine
//! - [`competency`] - Pharmacovigilance competency framework
//!   - 15 Competency Domains in 4 thematic clusters
//!   - 20 EPAs (Entrustable Professional Activities)
//!   - 8 CPAs (Critical Practice Areas)
//!   - User competency profiles and metrics
//!
//! ## Safety Axioms
//!
//! - **Score Bounds**: Quality scores constrained to [0.0, 100.0] range
//! - **Weight Invariant**: Quality weights sum to 1.0 (compile-time verified)
//! - **State Machine**: Job transitions are type-safe (impossible states = compile error)
//! - **Level Ordering**: Competency levels form a strict total order (L1 < L2 < ... < L5++)
//! - **Gateway Invariant**: CPA8 requires EPA10 Level 4+

pub mod capability;
pub mod competency;
pub mod job;
pub mod ksb;
pub mod quality;

// Re-export key types at module root
pub use capability::{CapabilityScore, PriorityScore};
pub use job::{JobStage, JobStatus};
pub use ksb::research::{
    BloomLevel, LearningObjective, PipelineResult, QualityIndicators, ResearchGrade,
    ResearchMetrics, ResearchQuality, Trend, TrendImpact,
};
pub use ksb::{ComponentType, DifficultyLevel, KsbComponent};
pub use quality::{QualityScore, QualityWeights, ValidationSeverity};

// Re-export competency types
pub use competency::{
    // Domain types
    AchievedBehavioralAnchor,
    // Profile types
    AiGatewayStatus,
    BehavioralAnchor,
    CompetencyAssessment,
    CompetencyLevel,
    CompetencyMetrics,
    // CPA types
    Cpa,
    CpaDefinition,
    CpaProgress,
    CpaProgressionLevel,
    CpaStatus,
    DevelopmentGoal,
    DevelopmentPlan,
    Domain,
    DomainDefinition,
    DomainProgress,
    // EPA types
    EntrustmentLevel,
    EntrustmentLevelDefinition,
    EntrustmentRecord,
    Epa,
    EpaCategory,
    EpaContribution,
    EpaDefinition,
    EpaProgress,
    EvidenceType,
    LearningMilestone,
    ThematicCluster,
    UserCompetencyProfile,
};
