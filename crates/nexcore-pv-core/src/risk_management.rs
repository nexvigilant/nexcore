//! # Risk Management Programs (REMS/RMP)
//!
//! Models the lifecycle of risk minimization strategies per FDA (REMS)
//! and EMA (RMP) requirements.
//!
//! ## Lifecycle States
//!
//! ```text
//! Draft → Submitted → Approved → Active → Modified → Closed
//!                   ↘ Rejected                ↗
//! ```
//!
//! ## REMS Components (FDA)
//!
//! - Medication Guide
//! - Communication Plan
//! - Elements to Assure Safe Use (ETASU)
//! - Implementation System
//! - Timetable for Assessment
//!
//! ## RMP Components (EMA)
//!
//! - Safety Specification (Part I-III)
//! - Pharmacovigilance Plan
//! - Risk Minimization Measures
//! - Summary of the RMP
//!
//! ## T1 Grounding
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|-------------|--------|
//! | Program state | State | ς |
//! | Lifecycle transitions | Causality | → |
//! | Assessment schedule | Frequency | ν |
//! | Safety boundaries | Boundary | ∂ |

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// RISK MANAGEMENT PROGRAM
// ═══════════════════════════════════════════════════════════════════════════

/// A risk management program (REMS or RMP).
///
/// # Tier: T3 (Domain-Specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementProgram {
    /// Program identifier (e.g., "REMS-2024-001")
    pub id: String,
    /// Drug or product name
    pub drug_name: String,
    /// Regulatory authority framework
    pub framework: RmFramework,
    /// Current lifecycle state
    pub state: RmState,
    /// Risk minimization measures
    pub measures: Vec<RiskMeasure>,
    /// Assessment schedule (months between assessments)
    pub assessment_interval_months: u16,
    /// Number of assessments completed
    pub assessments_completed: u16,
}

/// Regulatory framework governing the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RmFramework {
    /// FDA Risk Evaluation and Mitigation Strategy
    Rems,
    /// EMA Risk Management Plan
    Rmp,
    /// Health Canada Risk Management Framework
    HcRmf,
    /// TGA Risk Management Plan (Australia)
    TgaRmp,
}

/// Lifecycle state of a risk management program.
///
/// State machine: Draft → Submitted → {Approved, Rejected} → Active → {Modified, Closed}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RmState {
    /// Initial drafting
    Draft,
    /// Submitted to regulatory authority
    Submitted,
    /// Approved by authority
    Approved,
    /// Rejected — requires revision
    Rejected,
    /// Active and enforced
    Active,
    /// Modified (new version submitted)
    Modified,
    /// Closed — no longer required
    Closed,
}

impl RmState {
    /// Valid transitions from current state.
    #[must_use]
    pub fn valid_transitions(&self) -> &[RmState] {
        match self {
            Self::Draft => &[Self::Submitted],
            Self::Submitted => &[Self::Approved, Self::Rejected],
            Self::Rejected => &[Self::Draft],
            Self::Approved => &[Self::Active],
            Self::Active => &[Self::Modified, Self::Closed],
            Self::Modified => &[Self::Submitted],
            Self::Closed => &[],
        }
    }

    /// Whether a transition to `target` is valid from current state.
    #[must_use]
    pub fn can_transition_to(&self, target: RmState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RISK MEASURES
// ═══════════════════════════════════════════════════════════════════════════

/// A specific risk minimization measure.
///
/// # Tier: T2-C (composed from T1: Boundary + Causality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMeasure {
    /// Measure type
    pub kind: MeasureKind,
    /// Description of the measure
    pub description: String,
    /// Whether this measure is currently active
    pub active: bool,
    /// Effectiveness metric (0.0 = no effect, 1.0 = fully effective)
    pub effectiveness: Option<f64>,
}

/// Categories of risk minimization measures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureKind {
    // ── Routine measures ──
    /// Product labeling (SmPC, PIL)
    Labeling,
    /// Packaging design (e.g., unit-dose blister)
    Packaging,
    /// Legal status (prescription-only, restricted)
    LegalStatus,

    // ── Additional measures (ETASU for REMS) ──
    /// Medication Guide distributed to patients
    MedicationGuide,
    /// Healthcare provider communication plan
    CommunicationPlan,
    /// Prescriber certification/enrollment required
    PrescriberCertification,
    /// Pharmacy certification required
    PharmacyCertification,
    /// Patient enrollment required before dispensing
    PatientEnrollment,
    /// Required lab test before/during treatment
    MonitoringRequirement { test: String },
    /// Restricted distribution program
    RestrictedDistribution,
    /// Patient registry
    PatientRegistry,
    /// Pregnancy prevention program
    PregnancyPrevention,
}

impl RiskManagementProgram {
    /// Create a new program in Draft state.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        drug_name: impl Into<String>,
        framework: RmFramework,
    ) -> Self {
        Self {
            id: id.into(),
            drug_name: drug_name.into(),
            framework,
            state: RmState::Draft,
            measures: Vec::new(),
            assessment_interval_months: 18,
            assessments_completed: 0,
        }
    }

    /// Add a risk measure.
    pub fn add_measure(&mut self, measure: RiskMeasure) {
        self.measures.push(measure);
    }

    /// Attempt state transition. Returns `Err` with reason if invalid.
    pub fn transition(&mut self, target: RmState) -> Result<(), String> {
        if self.state.can_transition_to(target) {
            self.state = target;
            Ok(())
        } else {
            Err(format!(
                "Invalid transition: {:?} → {:?}. Valid: {:?}",
                self.state,
                target,
                self.state.valid_transitions()
            ))
        }
    }

    /// Count active measures.
    #[must_use]
    pub fn active_measure_count(&self) -> usize {
        self.measures.iter().filter(|m| m.active).count()
    }

    /// Whether program is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.state == RmState::Closed
    }

    /// Whether an assessment is overdue given months since last.
    #[must_use]
    pub fn assessment_overdue(&self, months_since_last: u16) -> bool {
        self.state == RmState::Active && months_since_last > self.assessment_interval_months
    }

    /// Whether program includes ETASU (Elements to Assure Safe Use).
    #[must_use]
    pub fn has_etasu(&self) -> bool {
        self.measures.iter().any(|m| {
            matches!(
                m.kind,
                MeasureKind::PrescriberCertification
                    | MeasureKind::PharmacyCertification
                    | MeasureKind::PatientEnrollment
                    | MeasureKind::RestrictedDistribution
                    | MeasureKind::MonitoringRequirement { .. }
            )
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rems() -> RiskManagementProgram {
        let mut rems =
            RiskManagementProgram::new("REMS-2024-001", "Thalidomide", RmFramework::Rems);
        rems.add_measure(RiskMeasure {
            kind: MeasureKind::MedicationGuide,
            description: "Patient medication guide".into(),
            active: true,
            effectiveness: Some(0.75),
        });
        rems.add_measure(RiskMeasure {
            kind: MeasureKind::PregnancyPrevention,
            description: "THALOMID REMS pregnancy prevention".into(),
            active: true,
            effectiveness: Some(0.95),
        });
        rems.add_measure(RiskMeasure {
            kind: MeasureKind::PrescriberCertification,
            description: "Prescriber must be certified in THALOMID REMS".into(),
            active: true,
            effectiveness: Some(0.90),
        });
        rems
    }

    #[test]
    fn lifecycle_happy_path() {
        let mut rems = sample_rems();
        assert_eq!(rems.state, RmState::Draft);

        assert!(rems.transition(RmState::Submitted).is_ok());
        assert!(rems.transition(RmState::Approved).is_ok());
        assert!(rems.transition(RmState::Active).is_ok());

        assert_eq!(rems.state, RmState::Active);
        assert!(!rems.is_terminal());
    }

    #[test]
    fn invalid_transition_blocked() {
        let mut rems = sample_rems();
        // Cannot go directly from Draft to Active
        let result = rems.transition(RmState::Active);
        assert!(result.is_err());
        assert_eq!(rems.state, RmState::Draft);
    }

    #[test]
    fn rejection_loop() {
        let mut rems = sample_rems();
        assert!(rems.transition(RmState::Submitted).is_ok());
        assert!(rems.transition(RmState::Rejected).is_ok());
        // Can go back to Draft for revision
        assert!(rems.transition(RmState::Draft).is_ok());
        assert_eq!(rems.state, RmState::Draft);
    }

    #[test]
    fn modification_cycle() {
        let mut rems = sample_rems();
        assert!(rems.transition(RmState::Submitted).is_ok());
        assert!(rems.transition(RmState::Approved).is_ok());
        assert!(rems.transition(RmState::Active).is_ok());
        assert!(rems.transition(RmState::Modified).is_ok());
        // Modified goes back to Submitted
        assert!(rems.transition(RmState::Submitted).is_ok());
    }

    #[test]
    fn closed_is_terminal() {
        let mut rems = sample_rems();
        assert!(rems.transition(RmState::Submitted).is_ok());
        assert!(rems.transition(RmState::Approved).is_ok());
        assert!(rems.transition(RmState::Active).is_ok());
        assert!(rems.transition(RmState::Closed).is_ok());

        assert!(rems.is_terminal());
        assert!(rems.state.valid_transitions().is_empty());
    }

    #[test]
    fn etasu_detection() {
        let rems = sample_rems();
        assert!(rems.has_etasu()); // has PrescriberCertification
    }

    #[test]
    fn active_measure_count() {
        let rems = sample_rems();
        assert_eq!(rems.active_measure_count(), 3);
    }

    #[test]
    fn assessment_overdue_check() {
        let mut rems = sample_rems();
        assert!(rems.transition(RmState::Submitted).is_ok());
        assert!(rems.transition(RmState::Approved).is_ok());
        assert!(rems.transition(RmState::Active).is_ok());

        assert!(!rems.assessment_overdue(12)); // 12 < 18
        assert!(rems.assessment_overdue(24)); // 24 > 18
    }

    #[test]
    fn rmp_framework() {
        let rmp = RiskManagementProgram::new("RMP-EU-001", "Rosiglitazone", RmFramework::Rmp);
        assert_eq!(rmp.framework, RmFramework::Rmp);
        assert_eq!(rmp.state, RmState::Draft);
    }
}
