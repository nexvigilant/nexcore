//! # FDA 21 CFR 314.80 Primitive Structures
//!
//! Rust implementation of primitives extracted from FDA pharmacovigilance
//! regulations. Demonstrates T1→T2→T3 composition patterns.
//!
//! ## Primitive Tiers
//! - **T1**: Universal (domain-independent)
//! - **T2-P**: Cross-Domain Primitive (atomic, multiple domains)
//! - **T2-C**: Cross-Domain Composite (built from T1/T2-P)
//! - **T3**: Domain-Specific (single domain)

pub mod cioms_audit;

use std::time::Duration;

// ============================================================================
// T1 UNIVERSAL PRIMITIVES
// ============================================================================

/// T1: Threshold - boundary value triggering state change
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold<T> {
    pub value: T,
    pub comparison: Comparison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
    Equal,
}

impl<T: PartialOrd> Threshold<T> {
    pub fn is_crossed(&self, actual: &T) -> bool {
        match self.comparison {
            Comparison::GreaterThan => actual > &self.value,
            Comparison::GreaterOrEqual => actual >= &self.value,
            Comparison::LessThan => actual < &self.value,
            Comparison::LessOrEqual => actual <= &self.value,
            Comparison::Equal => actual == &self.value,
        }
    }
}

/// T1: Duration - time extent between points
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBound {
    pub duration: Duration,
    pub from_event: EventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    InitialReceipt,
    NewInformation,
    EventOccurrence,
}

impl TimeBound {
    pub const FIFTEEN_DAYS: Self = Self {
        duration: Duration::from_secs(15 * 24 * 60 * 60),
        from_event: EventType::InitialReceipt,
    };

    pub const SEVEN_DAYS: Self = Self {
        duration: Duration::from_secs(7 * 24 * 60 * 60),
        from_event: EventType::InitialReceipt,
    };
}

/// T1: Classification - category assignment based on properties
pub trait Classifiable {
    type Category;
    fn classify(&self) -> Self::Category;
}

/// T1: Sequence - ordered operations
#[derive(Debug, Clone)]
pub struct Sequence<T> {
    steps: Vec<T>,
    current: usize,
}

impl<T> Sequence<T> {
    pub fn new(steps: Vec<T>) -> Self {
        Self { steps, current: 0 }
    }

    pub fn current_step(&self) -> Option<&T> {
        self.steps.get(self.current)
    }

    pub fn advance(&mut self) -> bool {
        if self.current < self.steps.len() - 1 {
            self.current += 1;
            true
        } else {
            false
        }
    }
}

// ============================================================================
// T2-P CROSS-DOMAIN PRIMITIVES (Atomic, Multiple Domains)
// ============================================================================

/// T2-P: Harm - negative change to valued state
#[derive(Debug, Clone)]
pub struct Harm {
    pub description: String,
    pub severity: ClinicalSeverity,
    pub reversibility: Reversibility,
}

/// Clinical severity — magnitude of deviation from desired health state.
///
/// Tier: T2-P (N + ∝ — quantity with irreversibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClinicalSeverity {
    Mild,
    Moderate,
    Severe,
    LifeThreatening,
    Fatal,
}

/// Backward-compatible alias.
#[deprecated(note = "use ClinicalSeverity — F2 equivocation fix")]
pub type Severity = ClinicalSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reversibility {
    FullyReversible,
    PartiallyReversible,
    Irreversible,
    Unknown,
}

/// T2-P: Expectedness - degree outcome was anticipated from prior knowledge
#[derive(Debug, Clone)]
pub struct Expectedness {
    pub known_in_knowledge_base: bool,
    pub knowledge_base: KnowledgeBase,
}

#[derive(Debug, Clone)]
pub enum KnowledgeBase {
    ProductLabeling(String),  // FDA labeling
    Runbook(String),          // Cloud/SRE
    ModelCard(String),        // AI Safety
    Prospectus(String),       // Finance
}

impl Expectedness {
    pub fn is_unexpected(&self) -> bool {
        !self.known_in_knowledge_base
    }
}

/// T2-P: Obligation - binding requirement to perform action
#[derive(Debug, Clone)]
pub struct Obligation {
    pub action: ObligationAction,
    pub deadline: Option<TimeBound>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ObligationAction {
    Report,
    Investigate,
    Review,
    Notify,
    Document,
}

/// T2-P: Signal - pattern distinguishable from noise
#[derive(Debug, Clone)]
pub struct Signal {
    pub pattern: String,
    pub strength: f64,        // Signal-to-noise ratio
    pub confidence: f64,      // Statistical confidence
}

/// T2-P: Source - origin point of information
#[derive(Debug, Clone)]
pub enum Source {
    Domestic,
    Foreign,
    Clinical,
    Epidemiological,
    Literature,
    Spontaneous,
}

/// T2-P: Evidence - information supporting conclusion
#[derive(Debug, Clone)]
pub struct Evidence {
    pub source: Source,
    pub quality: EvidenceQuality,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceQuality {
    Anecdotal,
    CaseReport,
    CaseSeries,
    Observational,
    Controlled,
    Randomized,
}

// ============================================================================
// T2-C CROSS-DOMAIN COMPOSITES (Built from primitives)
// ============================================================================

/// T2-C: Serious Adverse Experience
/// Composition: HARM + SEVERITY[high] + THRESHOLD[medical outcomes]
#[derive(Debug, Clone)]
pub struct SeriousAdverseExperience {
    pub harm: Harm,
    pub outcome: SeriousOutcome,
    pub onset_date: Option<String>,
}

/// FDA's seriousness threshold criteria
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeriousOutcome {
    Death,
    LifeThreatening,
    Hospitalization,
    Disability,
    CongenitalAnomaly,
    RequiredIntervention,
}

impl SeriousAdverseExperience {
    /// Check if harm crosses seriousness threshold
    pub fn is_serious(&self) -> bool {
        // All SeriousOutcome variants are serious by definition
        true
    }
}

/// T2-C: Unexpected Adverse Experience
/// Composition: EXPECTEDNESS[false] + KNOWLEDGE_BASE[labeling]
#[derive(Debug, Clone)]
pub struct UnexpectedAdverseExperience {
    pub harm: Harm,
    pub expectedness: Expectedness,
    pub specificity_note: Option<String>, // e.g., "cerebral thromboembolism vs generic CVA"
}

impl UnexpectedAdverseExperience {
    pub fn is_unexpected(&self) -> bool {
        self.expectedness.is_unexpected()
    }
}

/// T2-C: Alert Report (15-Day Report)
/// Composition: OBLIGATION + DURATION[15 days] + SERIOUS + UNEXPECTED
#[derive(Debug, Clone)]
pub struct AlertReport {
    pub serious_experience: SeriousAdverseExperience,
    pub unexpected_experience: UnexpectedAdverseExperience,
    pub deadline: TimeBound,
    pub receipt_date: String,
    pub status: ReportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportStatus {
    Pending,
    Submitted,
    FollowUpRequired,
    Closed,
}

impl AlertReport {
    pub fn new_fifteen_day(
        serious: SeriousAdverseExperience,
        unexpected: UnexpectedAdverseExperience,
        receipt_date: String,
    ) -> Self {
        Self {
            serious_experience: serious,
            unexpected_experience: unexpected,
            deadline: TimeBound::FIFTEEN_DAYS,
            receipt_date,
            status: ReportStatus::Pending,
        }
    }

    pub fn requires_alert(&self) -> bool {
        self.serious_experience.is_serious() && self.unexpected_experience.is_unexpected()
    }
}

/// T2-C: Reasonable Possibility
/// Composition: CAUSE + EVIDENCE + THRESHOLD[non-exclusion]
#[derive(Debug, Clone)]
pub struct ReasonablePossibility {
    pub drug: String,
    pub event: String,
    pub evidence: Vec<Evidence>,
    pub can_be_ruled_out: bool,
}

impl ReasonablePossibility {
    /// "Reasonable possibility" = cannot be ruled out
    pub fn exists(&self) -> bool {
        !self.can_be_ruled_out
    }
}

// ============================================================================
// T3 DOMAIN-SPECIFIC (FDA Pharmacovigilance)
// ============================================================================

/// T3: Product Labeling - FDA-approved drug documentation
#[derive(Debug, Clone)]
pub struct ProductLabeling {
    pub drug_name: String,
    pub ndc: String,
    pub known_adverse_reactions: Vec<String>,
    pub contraindications: Vec<String>,
    pub warnings: Vec<String>,
}

impl ProductLabeling {
    pub fn contains_reaction(&self, reaction: &str) -> bool {
        self.known_adverse_reactions
            .iter()
            .any(|r| r.to_lowercase().contains(&reaction.to_lowercase()))
    }
}

/// T3: Applicant - FDA approval holder
#[derive(Debug, Clone)]
pub struct Applicant {
    pub name: String,
    pub nda_number: String,
    pub contact: String,
}

/// T3: Postmarketing Study
#[derive(Debug, Clone)]
pub struct PostmarketingStudy {
    pub study_id: String,
    pub study_type: StudyType,
    pub under_ind: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudyType {
    Clinical,
    Epidemiological,
    Surveillance,
    Observational,
}

// ============================================================================
// REGULATORY GRAMMAR IMPLEMENTATION
// ============================================================================

/// The 314.80 regulatory grammar as a state machine
///
/// ```text
/// REPORTABLE_EVENT := HARM ∧ (SERIOUS ∨ UNEXPECTED)
/// ALERT_TRIGGER := SERIOUS ∧ UNEXPECTED
/// ALERT_OBLIGATION := ALERT_TRIGGER → NOTIFICATION within DURATION[15 days]
/// ```
#[derive(Debug)]
pub struct RegulatoryEvaluator {
    pub labeling: ProductLabeling,
}

impl RegulatoryEvaluator {
    pub fn new(labeling: ProductLabeling) -> Self {
        Self { labeling }
    }

    /// Evaluate if an adverse experience requires 15-day alert
    pub fn requires_alert_report(&self, harm: &Harm, outcome: Option<SeriousOutcome>) -> AlertDecision {
        let is_serious = outcome.is_some();
        let is_unexpected = !self.labeling.contains_reaction(&harm.description);

        AlertDecision {
            is_serious,
            is_unexpected,
            requires_alert: is_serious && is_unexpected,
            deadline: if is_serious && is_unexpected {
                Some(TimeBound::FIFTEEN_DAYS)
            } else {
                None
            },
        }
    }

    /// Evaluate causality using "reasonable possibility" standard
    pub fn evaluate_causality(&self, possibility: &ReasonablePossibility) -> CausalityDecision {
        CausalityDecision {
            drug: possibility.drug.clone(),
            event: possibility.event.clone(),
            reasonable_possibility: possibility.exists(),
            evidence_count: possibility.evidence.len(),
            reporting_required: possibility.exists(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertDecision {
    pub is_serious: bool,
    pub is_unexpected: bool,
    pub requires_alert: bool,
    pub deadline: Option<TimeBound>,
}

#[derive(Debug, Clone)]
pub struct CausalityDecision {
    pub drug: String,
    pub event: String,
    pub reasonable_possibility: bool,
    pub evidence_count: usize,
    pub reporting_required: bool,
}

// ============================================================================
// CROSS-DOMAIN TRANSFER EXAMPLES
// ============================================================================

pub mod cross_domain {
    use super::*;

    /// Cloud/SRE instantiation of the same primitives
    pub mod cloud {
        #![allow(dead_code)]

        /// Maps to SERIOUS_ADE
        #[derive(Debug, Clone)]
        pub struct CriticalIncident {
            pub severity: IncidentSeverity,
            pub impact: ServiceImpact,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum IncidentSeverity {
            Sev1,  // Maps to Fatal/LifeThreatening
            Sev2,  // Maps to Severe
            Sev3,  // Maps to Moderate
            Sev4,  // Maps to Mild
        }

        #[derive(Debug, Clone)]
        pub struct ServiceImpact {
            pub revenue_loss: bool,
            pub user_facing: bool,
            pub data_loss: bool,
        }

        /// Maps to UNEXPECTED_ADE (not in runbook)
        #[derive(Debug, Clone)]
        pub struct NovelFailure {
            pub in_runbook: bool,
            pub failure_mode: String,
        }

        /// Maps to ALERT_REPORT
        #[derive(Debug, Clone)]
        pub struct PagerDutyAlert {
            pub incident: CriticalIncident,
            pub sla_deadline: std::time::Duration,
        }
    }

    /// AI Safety instantiation
    pub mod ai_safety {
        use super::*;

        /// Maps to SERIOUS_ADE
        #[derive(Debug, Clone)]
        pub struct HighSeverityAiIncident {
            pub harm_type: AiHarmType,
            pub reversibility: Reversibility,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum AiHarmType {
            Bias,
            Misinformation,
            PrivacyViolation,
            Manipulation,
            PhysicalHarm,
        }

        /// Maps to UNEXPECTED_ADE (not in model card)
        #[derive(Debug, Clone)]
        pub struct OutOfDistributionFailure {
            pub documented_in_model_card: bool,
            pub failure_description: String,
        }

        /// Maps to PRODUCT_LABELING
        #[derive(Debug, Clone)]
        pub struct ModelCard {
            pub model_name: String,
            pub known_limitations: Vec<String>,
            pub intended_use: String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serious_unexpected_triggers_alert() {
        let labeling = ProductLabeling {
            drug_name: "TestDrug".into(),
            ndc: "12345-678-90".into(),
            known_adverse_reactions: vec!["headache".into(), "nausea".into()],
            contraindications: vec![],
            warnings: vec![],
        };

        let evaluator = RegulatoryEvaluator::new(labeling);

        // Known reaction (expected) + serious = NO alert
        let known_harm = Harm {
            description: "headache".into(),
            severity: Severity::Severe,
            reversibility: Reversibility::FullyReversible,
        };
        let decision1 = evaluator.requires_alert_report(&known_harm, Some(SeriousOutcome::Hospitalization));
        assert!(decision1.is_serious);
        assert!(!decision1.is_unexpected);
        assert!(!decision1.requires_alert);

        // Unknown reaction (unexpected) + serious = YES alert
        let unknown_harm = Harm {
            description: "cerebral thromboembolism".into(),
            severity: Severity::LifeThreatening,
            reversibility: Reversibility::PartiallyReversible,
        };
        let decision2 = evaluator.requires_alert_report(&unknown_harm, Some(SeriousOutcome::LifeThreatening));
        assert!(decision2.is_serious);
        assert!(decision2.is_unexpected);
        assert!(decision2.requires_alert);
    }

    #[test]
    fn test_reasonable_possibility() {
        let possibility = ReasonablePossibility {
            drug: "TestDrug".into(),
            event: "liver failure".into(),
            evidence: vec![
                Evidence {
                    source: Source::Clinical,
                    quality: EvidenceQuality::CaseReport,
                    description: "Temporal association".into(),
                },
            ],
            can_be_ruled_out: false,
        };

        assert!(possibility.exists());

        let ruled_out = ReasonablePossibility {
            can_be_ruled_out: true,
            ..possibility
        };
        assert!(!ruled_out.exists());
    }

    #[test]
    fn test_threshold_crossing() {
        let fifteen_day_threshold = Threshold {
            value: 15,
            comparison: Comparison::LessOrEqual,
        };

        assert!(fifteen_day_threshold.is_crossed(&10)); // 10 days - within deadline
        assert!(fifteen_day_threshold.is_crossed(&15)); // 15 days - at deadline
        assert!(!fifteen_day_threshold.is_crossed(&16)); // 16 days - past deadline
    }
}
