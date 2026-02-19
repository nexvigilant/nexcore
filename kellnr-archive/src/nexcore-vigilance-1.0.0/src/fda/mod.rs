//! FDA AI Credibility Assessment Framework (January 2025)
//!
//! Implementation of FDA Draft Guidance: "Considerations for the Use of Artificial
//! Intelligence to Support Regulatory Decision-Making for Drug and Biological Products"
//!
//! ## T1 Primitive Foundation
//!
//! | Type | T1 Grounding |
//! |------|--------------|
//! | ContextOfUse | λ (Location) + μ (Mapping) |
//! | ModelRisk | κ (Comparison) + N (Quantity) |
//! | CredibilityEvidence | ∃ (Existence) + κ (Comparison) |
//! | FitForUse | ∃ (Existence) + κ (Comparison) |
//! | DataDrift | ∂ (Boundary) + ν (Frequency) |
//! | CredibilityPlan | σ (Sequence) |

pub mod drift;
pub mod evidence;
pub mod metrics;
pub mod plan;
pub mod risk;
pub mod types;

pub use drift::{DataDrift, DriftDetector, DriftError, DriftMagnitude, DriftSeverity, DriftType};
pub use evidence::{
    CredibilityEvidence, EvidenceQuality, EvidenceType, FitForUse, Relevance, Reliability,
};
pub use metrics::{
    AssessmentMetrics, CredibilityInput, CredibilityRating, CredibilityScore, DriftHistory,
    DriftMeasurement, EvidenceDistribution, RiskDistribution,
};
pub use plan::{AdequacyDecision, AssessmentStep, CredibilityPlan, PlanError, PlanStatus};
pub use risk::{DecisionConsequence, ModelInfluence, ModelRisk, RiskLevel};
pub use types::{
    ContextOfUse, DecisionQuestion, EvidenceIntegration, ModelPurpose, RegulatoryContext,
    ValidationError,
};
