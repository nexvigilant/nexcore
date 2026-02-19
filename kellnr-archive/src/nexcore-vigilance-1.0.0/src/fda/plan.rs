//! Credibility Assessment Plan (Step 4-7)
//!
//! ## T1 Grounding
//!
//! - **CredibilityPlan**: σ (Sequence) — 7 ordered steps
//!   - Each step executes in sequence
//!   - Results feed forward
//!
//! - **AssessmentStep**: N (Quantity) + κ (Comparison)
//!   - Step number (1-7) + completion status

use crate::fda::{
    ContextOfUse, CredibilityEvidence, DataDrift, EvidenceQuality, FitForUse, ModelRisk,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 7-step FDA credibility assessment framework
///
/// T1 Grounding: N (Quantity) — Step numbers 1-7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AssessmentStep {
    /// Step 1: Define question of interest
    DefineQuestion = 1,
    /// Step 2: Define context of use
    DefineContextOfUse = 2,
    /// Step 3: Assess AI model risk
    AssessRisk = 3,
    /// Step 4: Develop credibility plan
    DevelopPlan = 4,
    /// Step 5: Execute the plan
    ExecutePlan = 5,
    /// Step 6: Document results
    DocumentResults = 6,
    /// Step 7: Determine adequacy
    DetermineAdequacy = 7,
}

impl AssessmentStep {
    /// Returns step number (1-7)
    ///
    /// T1 Grounding: N (Quantity)
    pub fn number(&self) -> u8 {
        *self as u8
    }

    /// Returns the next step in sequence
    ///
    /// T1 Grounding: σ (Sequence)
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::DefineQuestion => Some(Self::DefineContextOfUse),
            Self::DefineContextOfUse => Some(Self::AssessRisk),
            Self::AssessRisk => Some(Self::DevelopPlan),
            Self::DevelopPlan => Some(Self::ExecutePlan),
            Self::ExecutePlan => Some(Self::DocumentResults),
            Self::DocumentResults => Some(Self::DetermineAdequacy),
            Self::DetermineAdequacy => None, // Final step
        }
    }

    /// Returns all steps in order
    ///
    /// T1 Grounding: σ (Sequence)
    pub fn all_steps() -> [Self; 7] {
        [
            Self::DefineQuestion,
            Self::DefineContextOfUse,
            Self::AssessRisk,
            Self::DevelopPlan,
            Self::ExecutePlan,
            Self::DocumentResults,
            Self::DetermineAdequacy,
        ]
    }
}

impl fmt::Display for AssessmentStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefineQuestion => write!(f, "Step 1: Define Question"),
            Self::DefineContextOfUse => write!(f, "Step 2: Define Context of Use"),
            Self::AssessRisk => write!(f, "Step 3: Assess Risk"),
            Self::DevelopPlan => write!(f, "Step 4: Develop Plan"),
            Self::ExecutePlan => write!(f, "Step 5: Execute Plan"),
            Self::DocumentResults => write!(f, "Step 6: Document Results"),
            Self::DetermineAdequacy => write!(f, "Step 7: Determine Adequacy"),
        }
    }
}

/// Status of plan execution
///
/// T1 Grounding: κ (Comparison) — State transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlanStatus {
    /// Plan is being developed
    Draft,
    /// Plan is ready but not started
    Ready,
    /// Plan is being executed
    InProgress,
    /// Plan executed, awaiting adequacy determination
    UnderReview,
    /// Plan passed adequacy check
    Approved,
    /// Plan failed adequacy check, needs revision
    NeedsRevision,
}

impl PlanStatus {
    /// Returns true if plan can be executed
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn can_execute(&self) -> bool {
        matches!(self, Self::Ready | Self::NeedsRevision)
    }

    /// Returns true if plan is complete
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Approved)
    }

    /// Returns true if plan needs work
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn needs_work(&self) -> bool {
        matches!(self, Self::Draft | Self::NeedsRevision)
    }
}

impl fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "Draft"),
            Self::Ready => write!(f, "Ready"),
            Self::InProgress => write!(f, "In Progress"),
            Self::UnderReview => write!(f, "Under Review"),
            Self::Approved => write!(f, "Approved"),
            Self::NeedsRevision => write!(f, "Needs Revision"),
        }
    }
}

/// Step 4-7: Complete credibility assessment plan
///
/// T1 Grounding: σ (Sequence) — Sequential execution of steps
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredibilityPlan {
    /// Context of use from Step 2
    context_of_use: ContextOfUse,
    /// Risk assessment from Step 3
    model_risk: ModelRisk,
    /// Current execution status
    status: PlanStatus,
    /// Current step
    current_step: AssessmentStep,
    /// Collected credibility evidence
    evidence: Vec<CredibilityEvidence>,
    /// Fit-for-use assessments
    fit_assessments: Vec<FitForUse>,
    /// Detected drift instances
    drift_detections: Vec<DataDrift>,
}

impl CredibilityPlan {
    /// Creates a new credibility plan from Steps 1-3
    pub fn new(context_of_use: ContextOfUse, model_risk: ModelRisk) -> Self {
        Self {
            context_of_use,
            model_risk,
            status: PlanStatus::Draft,
            current_step: AssessmentStep::DevelopPlan,
            evidence: Vec::new(),
            fit_assessments: Vec::new(),
            drift_detections: Vec::new(),
        }
    }

    pub fn context_of_use(&self) -> &ContextOfUse {
        &self.context_of_use
    }

    pub fn model_risk(&self) -> &ModelRisk {
        &self.model_risk
    }

    pub fn status(&self) -> PlanStatus {
        self.status
    }

    pub fn current_step(&self) -> AssessmentStep {
        self.current_step
    }

    pub fn evidence(&self) -> &[CredibilityEvidence] {
        &self.evidence
    }

    pub fn fit_assessments(&self) -> &[FitForUse] {
        &self.fit_assessments
    }

    pub fn drift_detections(&self) -> &[DataDrift] {
        &self.drift_detections
    }

    /// Marks plan as ready for execution
    pub fn mark_ready(&mut self) {
        self.status = PlanStatus::Ready;
    }

    /// Advances to next step in sequence
    ///
    /// T1 Grounding: σ (Sequence)
    pub fn advance_step(&mut self) -> Result<(), PlanError> {
        match self.current_step.next() {
            Some(next_step) => {
                self.current_step = next_step;
                if next_step == AssessmentStep::ExecutePlan {
                    self.status = PlanStatus::InProgress;
                }
                Ok(())
            }
            None => Err(PlanError::AlreadyComplete),
        }
    }

    /// Adds credibility evidence (Step 5)
    pub fn add_evidence(&mut self, evidence: CredibilityEvidence) {
        self.evidence.push(evidence);
    }

    /// Adds fit-for-use assessment (Step 5)
    pub fn add_fit_assessment(&mut self, fit: FitForUse) {
        self.fit_assessments.push(fit);
    }

    /// Records drift detection (Step 5)
    pub fn record_drift(&mut self, drift: DataDrift) {
        self.drift_detections.push(drift);
    }

    /// Completes execution phase (Step 5 → Step 6)
    pub fn complete_execution(&mut self) -> Result<(), PlanError> {
        if self.current_step != AssessmentStep::ExecutePlan {
            return Err(PlanError::InvalidStep);
        }
        self.current_step = AssessmentStep::DocumentResults;
        self.status = PlanStatus::UnderReview;
        Ok(())
    }

    /// Step 7: Determine adequacy of credibility evidence
    ///
    /// T1 Grounding: κ (Comparison) — Evidence count vs. risk threshold
    pub fn determine_adequacy(&mut self) -> Result<AdequacyDecision, PlanError> {
        if self.current_step != AssessmentStep::DetermineAdequacy {
            return Err(PlanError::InvalidStep);
        }

        let decision = self.evaluate_adequacy();

        match decision {
            AdequacyDecision::Adequate => {
                self.status = PlanStatus::Approved;
            }
            AdequacyDecision::Inadequate { .. } => {
                self.status = PlanStatus::NeedsRevision;
            }
        }

        Ok(decision)
    }

    /// Evaluates adequacy based on evidence quantity and quality
    ///
    /// T1 Grounding: N (Quantity) + κ (Comparison)
    fn evaluate_adequacy(&self) -> AdequacyDecision {
        let min_required = self.model_risk.level().min_evidence_count();
        let high_quality_count = self
            .evidence
            .iter()
            .filter(|e| e.quality() == EvidenceQuality::High)
            .count();

        let fit_pass_count = self
            .fit_assessments
            .iter()
            .filter(|f| f.is_adequate())
            .count();
        let critical_drift_count = self
            .drift_detections
            .iter()
            .filter(|d| d.requires_action())
            .count();

        // Fail conditions
        if high_quality_count < min_required {
            return AdequacyDecision::Inadequate {
                reason: format!(
                    "Insufficient high-quality evidence: {} < {}",
                    high_quality_count, min_required
                ),
            };
        }

        if self.fit_assessments.is_empty() {
            return AdequacyDecision::Inadequate {
                reason: "No fit-for-use assessments".into(),
            };
        }

        if fit_pass_count == 0 {
            return AdequacyDecision::Inadequate {
                reason: "No data passed fit-for-use assessment".into(),
            };
        }

        if critical_drift_count > 0 {
            return AdequacyDecision::Inadequate {
                reason: format!(
                    "Critical drift detected in {} features",
                    critical_drift_count
                ),
            };
        }

        AdequacyDecision::Adequate
    }

    /// Returns completion percentage (0-100)
    ///
    /// T1 Grounding: N (Quantity) + ∝ (Proportionality)
    pub fn completion_percentage(&self) -> u8 {
        let current = self.current_step.number();
        ((current as f64 / 7.0) * 100.0) as u8
    }
}

/// Step 7: Adequacy determination result
///
/// T1 Grounding: κ (Comparison) — Binary decision with rationale
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdequacyDecision {
    /// Evidence is adequate for COU
    Adequate,
    /// Evidence is inadequate, with reason
    Inadequate { reason: String },
}

impl AdequacyDecision {
    /// Returns true if adequate
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_adequate(&self) -> bool {
        matches!(self, Self::Adequate)
    }
}

impl fmt::Display for AdequacyDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adequate => write!(f, "ADEQUATE"),
            Self::Inadequate { reason } => write!(f, "INADEQUATE: {}", reason),
        }
    }
}

/// Errors in plan execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    InvalidStep,
    AlreadyComplete,
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStep => write!(f, "Invalid step for this operation"),
            Self::AlreadyComplete => write!(f, "Plan already at final step"),
        }
    }
}

impl std::error::Error for PlanError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fda::{
        DecisionConsequence, DecisionQuestion, EvidenceIntegration, EvidenceType, ModelInfluence,
        ModelPurpose, RegulatoryContext,
    };

    fn create_test_plan() -> CredibilityPlan {
        let question = DecisionQuestion::new("Is drug safe?")
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let purpose = ModelPurpose::new("AE data", "Signal scores", "PRR")
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let cou = ContextOfUse::new(
            question,
            purpose,
            EvidenceIntegration::Sole,
            RegulatoryContext::Postmarket,
        );
        let risk = ModelRisk::new(ModelInfluence::High, DecisionConsequence::High);
        CredibilityPlan::new(cou, risk)
    }

    #[test]
    fn test_assessment_step_ordering() {
        assert!(AssessmentStep::DefineQuestion < AssessmentStep::DetermineAdequacy);
    }

    #[test]
    fn test_assessment_step_next() {
        let step = AssessmentStep::DefineQuestion;
        assert_eq!(step.next(), Some(AssessmentStep::DefineContextOfUse));

        let final_step = AssessmentStep::DetermineAdequacy;
        assert_eq!(final_step.next(), None);
    }

    #[test]
    fn test_assessment_step_all() {
        let steps = AssessmentStep::all_steps();
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0], AssessmentStep::DefineQuestion);
        assert_eq!(steps[6], AssessmentStep::DetermineAdequacy);
    }

    #[test]
    fn test_plan_status_checks() {
        assert!(PlanStatus::Ready.can_execute());
        assert!(!PlanStatus::InProgress.can_execute());

        assert!(PlanStatus::Approved.is_complete());
        assert!(!PlanStatus::Draft.is_complete());

        assert!(PlanStatus::Draft.needs_work());
        assert!(!PlanStatus::Approved.needs_work());
    }

    #[test]
    fn test_credibility_plan_creation() {
        let plan = create_test_plan();
        assert_eq!(plan.status(), PlanStatus::Draft);
        assert_eq!(plan.current_step(), AssessmentStep::DevelopPlan);
        assert_eq!(plan.evidence().len(), 0);
    }

    #[test]
    fn test_credibility_plan_advance() {
        let mut plan = create_test_plan();
        let result = plan.advance_step();
        assert!(result.is_ok());
        assert_eq!(plan.current_step(), AssessmentStep::ExecutePlan);
        assert_eq!(plan.status(), PlanStatus::InProgress);
    }

    #[test]
    fn test_credibility_plan_add_evidence() {
        let mut plan = create_test_plan();
        let evidence = CredibilityEvidence::new(
            EvidenceType::ValidationMetrics,
            EvidenceQuality::High,
            "ROC AUC = 0.95",
        );
        plan.add_evidence(evidence);
        assert_eq!(plan.evidence().len(), 1);
    }

    #[test]
    fn test_credibility_plan_inadequate_evidence() {
        let mut plan = create_test_plan();
        plan.current_step = AssessmentStep::DetermineAdequacy;

        // High risk requires 8 high-quality evidence items
        // Add only 2
        for _ in 0..2 {
            plan.add_evidence(CredibilityEvidence::new(
                EvidenceType::ValidationMetrics,
                EvidenceQuality::High,
                "Test",
            ));
        }

        let decision = plan.evaluate_adequacy();
        assert!(!decision.is_adequate());
    }

    #[test]
    fn test_credibility_plan_adequate() {
        let mut plan = create_test_plan();
        plan.current_step = AssessmentStep::DetermineAdequacy;

        // Add required high-quality evidence (8 for high risk)
        for i in 0..8 {
            plan.add_evidence(CredibilityEvidence::new(
                EvidenceType::ValidationMetrics,
                EvidenceQuality::High,
                format!("Evidence {}", i),
            ));
        }

        // Add passing fit-for-use
        plan.add_fit_assessment(FitForUse::passing());

        let decision = plan.evaluate_adequacy();
        assert!(decision.is_adequate());
    }

    #[test]
    fn test_credibility_plan_completion_percentage() {
        let mut plan = create_test_plan();
        assert_eq!(plan.completion_percentage(), 57); // Step 4/7

        plan.current_step = AssessmentStep::DetermineAdequacy;
        assert_eq!(plan.completion_percentage(), 100); // Step 7/7
    }

    #[test]
    fn test_adequacy_decision_display() {
        let adequate = AdequacyDecision::Adequate;
        assert_eq!(adequate.to_string(), "ADEQUATE");

        let inadequate = AdequacyDecision::Inadequate {
            reason: "Missing data".into(),
        };
        let s = inadequate.to_string();
        assert!(s.contains("INADEQUATE"));
        assert!(s.contains("Missing data"));
    }
}
