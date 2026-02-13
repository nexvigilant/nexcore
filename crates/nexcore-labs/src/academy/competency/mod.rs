//! # Competency Framework Module
//!
//! Pharmacovigilance competency-based education framework.
//!
//! ## Framework Components
//!
//! - **15 Competency Domains** organized in 4 thematic clusters
//! - **20 EPAs** (Entrustable Professional Activities) - 10 core + 10 executive
//! - **8 CPAs** (Critical Practice Areas) - integration points for EPAs
//! - **7 Competency Levels** - L1 (Novice) to L5++ (Industry Leader)
//!
//! ## UACA Hierarchy
//!
//! - **L0 Quarks**: Level orderings, cluster mappings
//! - **L1 Atoms**: Level comparisons, next-level functions (<20 LOC)
//! - **L2 Molecules**: Readiness calculations, eligibility checks (<50 LOC)
//!
//! ## Safety Axioms
//!
//! - **Level Ordering**: Levels form a strict total order (L1 < L2 < ... < L5++)
//! - **Cluster Invariant**: Each domain belongs to exactly one thematic cluster
//! - **Gateway Invariant**: CPA8 requires EPA10 Level 4+

pub mod cpa;
pub mod domain;
pub mod epa;
pub mod profile;

// Re-export key types
pub use cpa::{Cpa, CpaDefinition, CpaProgress, CpaProgressionLevel, CpaStatus, EpaContribution};
pub use domain::{
    AchievedBehavioralAnchor, BehavioralAnchor, CompetencyLevel, Domain, DomainDefinition,
    DomainProgress, EvidenceType, ThematicCluster,
};
pub use epa::{
    EntrustmentLevel, EntrustmentLevelDefinition, EntrustmentRecord, Epa, EpaCategory,
    EpaDefinition, EpaProgress,
};
pub use profile::{
    AiGatewayStatus, CompetencyAssessment, CompetencyMetrics, DevelopmentGoal, DevelopmentPlan,
    LearningMilestone, UserCompetencyProfile,
};
