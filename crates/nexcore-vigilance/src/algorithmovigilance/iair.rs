//! # IAIR Schema (ToV §55)
//!
//! Individual Algorithm Incident Report - the foundational data unit for algorithmovigilance,
//! analogous to the ICSR (Individual Case Safety Report) in pharmacovigilance.
//!
//! # Eight-Block Structure
//!
//! | Block | Title | Content |
//! |-------|-------|---------|
//! | A | Metadata | Report ID, timestamps, sender |
//! | B | Algorithm Identification | Name, vendor, version, FDA status |
//! | C | Patient Characteristics | Demographics, conditions |
//! | D | Incident Description | Timeline, algorithm output, clinician action |
//! | E | Outcome Information | Severity, interventions |
//! | F | Causality Assessment | ACA scoring integration |
//! | G | Signal Indicators | Signal flag, drift indicators |
//! | H | Administrative | Regulatory reporting, corrective actions |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::algorithmovigilance::iair::{
//!     IairReport, BlockA, BlockB, BlockE, IncidentCategory, OutcomeSeverity,
//!     FdaClearanceStatus, AlgorithmCategory, DeploymentContext,
//! };
//!
//! let report = IairReport::new("IAIR-2026-00001")
//!     .with_block_a(BlockA::new("Hospital A", "Dr. Smith"))
//!     .with_block_b(BlockB::new("SepsisPredictor", "1.2.3", "AI Corp")
//!         .with_fda_status(FdaClearanceStatus::Cleared510k)
//!         .with_category(AlgorithmCategory::RiskPrediction))
//!     .with_block_e(BlockE::new(OutcomeSeverity::TemporaryHarm)
//!         .with_incident_category(IncidentCategory::FalseNegative));
//!
//! assert!(report.is_valid());
//! ```

use serde::{Deserialize, Serialize};

use super::scoring::{AcaScoringResult, GroundTruthStandard};

// ============================================================================
// BLOCK B: ALGORITHM IDENTIFICATION (T2-P/T2-C)
// ============================================================================

/// FDA clearance status for algorithm (ToV §55.3).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum FdaClearanceStatus {
    /// 510(k) cleared device.
    Cleared510k = 1,
    /// De Novo authorized.
    DeNovo = 2,
    /// PMA approved.
    Pma = 3,
    /// Exempt from FDA clearance.
    Exempt = 4,
    /// Not cleared/under review.
    #[default]
    NotCleared = 5,
    /// Breakthrough device designation.
    Breakthrough = 6,
}

impl std::fmt::Display for FdaClearanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cleared510k => write!(f, "510(k) Cleared"),
            Self::DeNovo => write!(f, "De Novo"),
            Self::Pma => write!(f, "PMA Approved"),
            Self::Exempt => write!(f, "Exempt"),
            Self::NotCleared => write!(f, "Not Cleared"),
            Self::Breakthrough => write!(f, "Breakthrough Device"),
        }
    }
}

/// Algorithm category (ToV §55.3).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AlgorithmCategory {
    /// Diagnostic aid (image analysis, pattern detection).
    #[default]
    DiagnosticAid = 1,
    /// Treatment recommendation.
    TreatmentRecommendation = 2,
    /// Risk prediction (sepsis, deterioration).
    RiskPrediction = 3,
    /// Workflow optimization.
    WorkflowOptimization = 4,
    /// Clinical decision support.
    ClinicalDecisionSupport = 5,
    /// Triage/prioritization.
    Triage = 6,
    /// Monitoring/surveillance.
    Monitoring = 7,
}

impl std::fmt::Display for AlgorithmCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DiagnosticAid => write!(f, "Diagnostic Aid"),
            Self::TreatmentRecommendation => write!(f, "Treatment Recommendation"),
            Self::RiskPrediction => write!(f, "Risk Prediction"),
            Self::WorkflowOptimization => write!(f, "Workflow Optimization"),
            Self::ClinicalDecisionSupport => write!(f, "Clinical Decision Support"),
            Self::Triage => write!(f, "Triage"),
            Self::Monitoring => write!(f, "Monitoring"),
        }
    }
}

/// Deployment context (ToV §55.3).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DeploymentContext {
    /// Embedded in EHR system.
    #[default]
    EhrEmbedded = 1,
    /// Standalone application.
    Standalone = 2,
    /// Cloud-based service.
    CloudService = 3,
    /// Edge device (point of care).
    EdgeDevice = 4,
    /// Mobile application.
    MobileApp = 5,
}

impl std::fmt::Display for DeploymentContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EhrEmbedded => write!(f, "EHR Embedded"),
            Self::Standalone => write!(f, "Standalone"),
            Self::CloudService => write!(f, "Cloud Service"),
            Self::EdgeDevice => write!(f, "Edge Device"),
            Self::MobileApp => write!(f, "Mobile App"),
        }
    }
}

/// Clinical domain (ToV §55.3).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ClinicalDomain {
    /// Cardiology.
    Cardiology = 1,
    /// Oncology.
    Oncology = 2,
    /// Radiology/imaging.
    #[default]
    Radiology = 3,
    /// Emergency medicine.
    Emergency = 4,
    /// Primary care.
    PrimaryCare = 5,
    /// Critical care/ICU.
    CriticalCare = 6,
    /// Pathology.
    Pathology = 7,
    /// Ophthalmology.
    Ophthalmology = 8,
    /// Dermatology.
    Dermatology = 9,
    /// Neurology.
    Neurology = 10,
    /// Other/multiple domains.
    Other = 255,
}

impl std::fmt::Display for ClinicalDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cardiology => write!(f, "Cardiology"),
            Self::Oncology => write!(f, "Oncology"),
            Self::Radiology => write!(f, "Radiology"),
            Self::Emergency => write!(f, "Emergency Medicine"),
            Self::PrimaryCare => write!(f, "Primary Care"),
            Self::CriticalCare => write!(f, "Critical Care"),
            Self::Pathology => write!(f, "Pathology"),
            Self::Ophthalmology => write!(f, "Ophthalmology"),
            Self::Dermatology => write!(f, "Dermatology"),
            Self::Neurology => write!(f, "Neurology"),
            Self::Other => write!(f, "Other"),
        }
    }
}

// ============================================================================
// BLOCK D: INCIDENT CLASSIFICATION (T2-P)
// ============================================================================

/// Incident category codes (ToV §55.4).
///
/// # Tier: T2-P
///
/// Maps incident types to ToV harm types.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncidentCategory {
    /// IC-FN: False Negative Error → Type C (Off-Target Harm).
    FalseNegative = 1,
    /// IC-FP: False Positive Error → Type B (Excessive/Cumulative Harm).
    FalsePositive = 2,
    /// IC-BIAS: Bias Manifestation → Type H (Population Harm).
    Bias = 3,
    /// IC-DRIFT: Performance Drift Impact → Type F (Saturation/Chronic Harm).
    Drift = 4,
    /// IC-WORKFLOW: Workflow Disruption → Type G (Systemic/Interaction Harm).
    Workflow = 5,
    /// IC-INTERACTION: System Interaction Error → Type D (Cascade/Interaction Harm).
    Interaction = 6,
}

impl IncidentCategory {
    /// Get the incident code (e.g., "IC-FN").
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::FalseNegative => "IC-FN",
            Self::FalsePositive => "IC-FP",
            Self::Bias => "IC-BIAS",
            Self::Drift => "IC-DRIFT",
            Self::Workflow => "IC-WORKFLOW",
            Self::Interaction => "IC-INTERACTION",
        }
    }

    /// Get the corresponding ToV harm type letter.
    #[must_use]
    pub const fn tov_harm_type(&self) -> char {
        match self {
            Self::FalseNegative => 'C', // Off-Target
            Self::FalsePositive => 'B', // Excessive
            Self::Bias => 'H',          // Population
            Self::Drift => 'F',         // Saturation
            Self::Workflow => 'G',      // Systemic
            Self::Interaction => 'D',   // Cascade
        }
    }
}

impl std::fmt::Display for IncidentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FalseNegative => write!(f, "IC-FN: False Negative Error"),
            Self::FalsePositive => write!(f, "IC-FP: False Positive Error"),
            Self::Bias => write!(f, "IC-BIAS: Bias Manifestation"),
            Self::Drift => write!(f, "IC-DRIFT: Performance Drift"),
            Self::Workflow => write!(f, "IC-WORKFLOW: Workflow Disruption"),
            Self::Interaction => write!(f, "IC-INTERACTION: System Interaction Error"),
        }
    }
}

// ============================================================================
// BLOCK E: OUTCOME SEVERITY (T2-P)
// ============================================================================

/// Outcome severity levels (ToV §55.5).
///
/// # Tier: T2-P
///
/// Mapped to ToV severity v ∈ [0, 1].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OutcomeSeverity {
    /// No harm, but potential identified (v = 0, S > 0).
    NearMiss = 0,
    /// Incident occurred, no patient impact (v = 0).
    NoHarm = 1,
    /// Reversible adverse effect (v ∈ (0, 0.5)).
    TemporaryHarm = 2,
    /// Additional treatment needed (v ∈ (0.3, 0.7)).
    InterventionRequired = 3,
    /// Required inpatient admission (v ∈ (0.5, 0.8)).
    Hospitalization = 4,
    /// Irreversible adverse effect (v ∈ (0.5, 0.8)).
    PermanentHarm = 5,
    /// Immediate risk of death (v ∈ (0.8, 1.0)).
    LifeThreatening = 6,
    /// Patient died (v = 1.0).
    Death = 7,
}

impl OutcomeSeverity {
    /// Get ToV severity midpoint v.
    #[must_use]
    pub const fn tov_severity_midpoint(&self) -> f64 {
        match self {
            Self::NearMiss | Self::NoHarm => 0.0,
            Self::TemporaryHarm => 0.25,
            Self::InterventionRequired => 0.5,
            Self::Hospitalization => 0.65,
            Self::PermanentHarm => 0.65,
            Self::LifeThreatening => 0.9,
            Self::Death => 1.0,
        }
    }

    /// Get ToV severity range [min, max].
    #[must_use]
    pub const fn tov_severity_range(&self) -> (f64, f64) {
        match self {
            Self::NearMiss | Self::NoHarm => (0.0, 0.0),
            Self::TemporaryHarm => (0.0, 0.5),
            Self::InterventionRequired => (0.3, 0.7),
            Self::Hospitalization => (0.5, 0.8),
            Self::PermanentHarm => (0.5, 0.8),
            Self::LifeThreatening => (0.8, 1.0),
            Self::Death => (1.0, 1.0),
        }
    }

    /// Check if this represents harm (v > 0).
    #[must_use]
    pub const fn is_harmful(&self) -> bool {
        !matches!(self, Self::NearMiss | Self::NoHarm)
    }

    /// Check if this is serious harm (requires intervention or worse).
    #[must_use]
    pub const fn is_serious(&self) -> bool {
        matches!(
            self,
            Self::InterventionRequired
                | Self::Hospitalization
                | Self::PermanentHarm
                | Self::LifeThreatening
                | Self::Death
        )
    }
}

impl Default for OutcomeSeverity {
    fn default() -> Self {
        Self::NoHarm
    }
}

impl std::fmt::Display for OutcomeSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NearMiss => write!(f, "Near Miss"),
            Self::NoHarm => write!(f, "No Harm"),
            Self::TemporaryHarm => write!(f, "Temporary Harm"),
            Self::InterventionRequired => write!(f, "Intervention Required"),
            Self::Hospitalization => write!(f, "Hospitalization"),
            Self::PermanentHarm => write!(f, "Permanent Harm"),
            Self::LifeThreatening => write!(f, "Life-Threatening"),
            Self::Death => write!(f, "Death"),
        }
    }
}

// ============================================================================
// BLOCK STRUCTURES (T2-C / T3)
// ============================================================================

/// Block A: Metadata (ToV §55.2).
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockA {
    /// Sending institution.
    pub sender_organization: String,
    /// Reporter name/ID (de-identified).
    pub reporter_id: String,
    /// Report creation timestamp (ISO 8601).
    pub created_at: Option<String>,
    /// Last modification timestamp.
    pub modified_at: Option<String>,
    /// Report version.
    pub version: u32,
}

impl BlockA {
    /// Create new Block A.
    #[must_use]
    pub fn new(sender: impl Into<String>, reporter: impl Into<String>) -> Self {
        Self {
            sender_organization: sender.into(),
            reporter_id: reporter.into(),
            created_at: None,
            modified_at: None,
            version: 1,
        }
    }

    /// Set creation timestamp.
    #[must_use]
    pub fn with_created_at(mut self, timestamp: impl Into<String>) -> Self {
        self.created_at = Some(timestamp.into());
        self
    }
}

/// Block B: Algorithm Identification (ToV §55.3).
///
/// # Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockB {
    /// Algorithm commercial or internal name.
    pub algorithm_name: String,
    /// Specific version at incident time.
    pub algorithm_version: String,
    /// Developer/vendor organization.
    pub vendor_name: String,
    /// FDA clearance status.
    pub fda_status: FdaClearanceStatus,
    /// FDA clearance/approval number (if applicable).
    pub clearance_number: Option<String>,
    /// Algorithm functional category.
    pub category: AlgorithmCategory,
    /// Clinical domain of use.
    pub clinical_domain: ClinicalDomain,
    /// Deployment context.
    pub deployment_context: DeploymentContext,
    /// Model architecture (if known).
    pub model_architecture: Option<String>,
}

impl BlockB {
    /// Create new Block B with required fields.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        vendor: impl Into<String>,
    ) -> Self {
        Self {
            algorithm_name: name.into(),
            algorithm_version: version.into(),
            vendor_name: vendor.into(),
            fda_status: FdaClearanceStatus::default(),
            clearance_number: None,
            category: AlgorithmCategory::default(),
            clinical_domain: ClinicalDomain::default(),
            deployment_context: DeploymentContext::default(),
            model_architecture: None,
        }
    }

    /// Set FDA status.
    #[must_use]
    pub const fn with_fda_status(mut self, status: FdaClearanceStatus) -> Self {
        self.fda_status = status;
        self
    }

    /// Set clearance number.
    #[must_use]
    pub fn with_clearance_number(mut self, number: impl Into<String>) -> Self {
        self.clearance_number = Some(number.into());
        self
    }

    /// Set algorithm category.
    #[must_use]
    pub const fn with_category(mut self, category: AlgorithmCategory) -> Self {
        self.category = category;
        self
    }

    /// Set clinical domain.
    #[must_use]
    pub const fn with_clinical_domain(mut self, domain: ClinicalDomain) -> Self {
        self.clinical_domain = domain;
        self
    }

    /// Set deployment context.
    #[must_use]
    pub const fn with_deployment_context(mut self, context: DeploymentContext) -> Self {
        self.deployment_context = context;
        self
    }

    /// Set model architecture.
    #[must_use]
    pub fn with_architecture(mut self, arch: impl Into<String>) -> Self {
        self.model_architecture = Some(arch.into());
        self
    }
}

/// Block C: Patient Characteristics (ToV §55.2).
///
/// # Tier: T2-C
///
/// De-identified patient demographics for parameter θ context.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockC {
    /// Age range (e.g., "65-74").
    pub age_range: Option<String>,
    /// Sex (Male/Female/Other/Unknown).
    pub sex: Option<String>,
    /// Relevant medical conditions.
    pub conditions: Vec<String>,
    /// Relevant medications.
    pub medications: Vec<String>,
}

impl BlockC {
    /// Create new Block C.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set age range.
    #[must_use]
    pub fn with_age_range(mut self, range: impl Into<String>) -> Self {
        self.age_range = Some(range.into());
        self
    }

    /// Add a medical condition.
    #[must_use]
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }
}

/// Block D: Incident Description (ToV §55.2).
///
/// # Tier: T3
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockD {
    /// Incident date (ISO 8601).
    pub incident_date: Option<String>,
    /// Algorithm output/recommendation.
    pub algorithm_output: Option<String>,
    /// Clinician action taken.
    pub clinician_action: Option<String>,
    /// Whether clinician followed algorithm.
    pub followed_algorithm: Option<bool>,
    /// Narrative description.
    pub narrative: Option<String>,
    /// Contributing factors.
    pub contributing_factors: Vec<String>,
}

impl BlockD {
    /// Create new Block D.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set incident date.
    #[must_use]
    pub fn with_incident_date(mut self, date: impl Into<String>) -> Self {
        self.incident_date = Some(date.into());
        self
    }

    /// Set algorithm output.
    #[must_use]
    pub fn with_algorithm_output(mut self, output: impl Into<String>) -> Self {
        self.algorithm_output = Some(output.into());
        self
    }

    /// Set clinician action.
    #[must_use]
    pub fn with_clinician_action(mut self, action: impl Into<String>) -> Self {
        self.clinician_action = Some(action.into());
        self
    }

    /// Set whether algorithm was followed.
    #[must_use]
    pub const fn with_followed_algorithm(mut self, followed: bool) -> Self {
        self.followed_algorithm = Some(followed);
        self
    }

    /// Set narrative description.
    #[must_use]
    pub fn with_narrative(mut self, narrative: impl Into<String>) -> Self {
        self.narrative = Some(narrative.into());
        self
    }
}

/// Block E: Outcome Information (ToV §55.5).
///
/// # Tier: T3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockE {
    /// Outcome severity.
    pub severity: OutcomeSeverity,
    /// Incident category.
    pub incident_category: Option<IncidentCategory>,
    /// Outcome description.
    pub description: Option<String>,
    /// Interventions required.
    pub interventions: Vec<String>,
}

impl BlockE {
    /// Create new Block E with severity.
    #[must_use]
    pub fn new(severity: OutcomeSeverity) -> Self {
        Self {
            severity,
            incident_category: None,
            description: None,
            interventions: Vec::new(),
        }
    }

    /// Set incident category.
    #[must_use]
    pub const fn with_incident_category(mut self, category: IncidentCategory) -> Self {
        self.incident_category = Some(category);
        self
    }

    /// Set description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add an intervention.
    #[must_use]
    pub fn with_intervention(mut self, intervention: impl Into<String>) -> Self {
        self.interventions.push(intervention.into());
        self
    }
}

/// Block F: Causality Assessment (ToV §55.6).
///
/// # Tier: T3
///
/// Captures complete ACA assessment from §53-§54.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockF {
    /// Complete ACA scoring result.
    pub aca_result: Option<AcaScoringResult>,
    /// Ground truth details.
    pub ground_truth_determination: Option<String>,
    /// Ground truth standard.
    pub ground_truth_standard: GroundTruthStandard,
    /// Additional causality notes.
    pub notes: Option<String>,
}

impl BlockF {
    /// Create new Block F.
    #[must_use]
    pub fn new() -> Self {
        Self {
            aca_result: None,
            ground_truth_determination: None,
            ground_truth_standard: GroundTruthStandard::None,
            notes: None,
        }
    }

    /// Set ACA result.
    #[must_use]
    pub fn with_aca_result(mut self, result: AcaScoringResult) -> Self {
        self.aca_result = Some(result);
        self
    }

    /// Set ground truth determination.
    #[must_use]
    pub fn with_ground_truth(
        mut self,
        determination: impl Into<String>,
        standard: GroundTruthStandard,
    ) -> Self {
        self.ground_truth_determination = Some(determination.into());
        self.ground_truth_standard = standard;
        self
    }
}

impl Default for BlockF {
    fn default() -> Self {
        Self::new()
    }
}

/// Block G: Signal Indicators (ToV §55.2).
///
/// # Tier: T2-C
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BlockG {
    /// Whether this case flagged as signal.
    pub signal_flag: bool,
    /// Drift indicators detected.
    pub drift_indicators: Vec<String>,
    /// Subgroup analysis results.
    pub subgroup_analysis: Option<String>,
    /// Similar cases identified.
    pub similar_case_count: Option<u32>,
}

impl BlockG {
    /// Create new Block G.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set signal flag.
    #[must_use]
    pub const fn with_signal_flag(mut self, flag: bool) -> Self {
        self.signal_flag = flag;
        self
    }

    /// Add drift indicator.
    #[must_use]
    pub fn with_drift_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.drift_indicators.push(indicator.into());
        self
    }

    /// Set similar case count.
    #[must_use]
    pub const fn with_similar_cases(mut self, count: u32) -> Self {
        self.similar_case_count = Some(count);
        self
    }
}

/// Block H: Administrative (ToV §55.2).
///
/// # Tier: T2-C
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockH {
    /// Regulatory reporting required.
    pub regulatory_reporting_required: bool,
    /// Regulatory submission date.
    pub submission_date: Option<String>,
    /// Vendor notified.
    pub vendor_notified: bool,
    /// Corrective actions taken.
    pub corrective_actions: Vec<String>,
    /// Case status.
    pub case_status: CaseStatus,
}

impl BlockH {
    /// Create new Block H.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set regulatory reporting required.
    #[must_use]
    pub const fn with_regulatory_required(mut self, required: bool) -> Self {
        self.regulatory_reporting_required = required;
        self
    }

    /// Set vendor notified.
    #[must_use]
    pub const fn with_vendor_notified(mut self, notified: bool) -> Self {
        self.vendor_notified = notified;
        self
    }

    /// Add corrective action.
    #[must_use]
    pub fn with_corrective_action(mut self, action: impl Into<String>) -> Self {
        self.corrective_actions.push(action.into());
        self
    }

    /// Set case status.
    #[must_use]
    pub const fn with_status(mut self, status: CaseStatus) -> Self {
        self.case_status = status;
        self
    }
}

/// Case processing status.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CaseStatus {
    /// Newly received.
    #[default]
    New = 1,
    /// Under initial review.
    UnderReview = 2,
    /// Awaiting additional information.
    AwaitingInfo = 3,
    /// Assessment complete.
    Assessed = 4,
    /// Closed.
    Closed = 5,
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::UnderReview => write!(f, "Under Review"),
            Self::AwaitingInfo => write!(f, "Awaiting Information"),
            Self::Assessed => write!(f, "Assessed"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}

// ============================================================================
// IAIR REPORT (T3)
// ============================================================================

/// Complete IAIR Report (ToV §55).
///
/// # Tier: T3
///
/// Domain-specific composite containing all 8 blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IairReport {
    /// Report ID (e.g., "IAIR-2026-00001").
    pub report_id: String,
    /// Schema version (GAV-SPEC-001).
    pub schema_version: String,
    /// Block A: Metadata.
    pub block_a: Option<BlockA>,
    /// Block B: Algorithm Identification.
    pub block_b: Option<BlockB>,
    /// Block C: Patient Characteristics.
    pub block_c: Option<BlockC>,
    /// Block D: Incident Description.
    pub block_d: Option<BlockD>,
    /// Block E: Outcome Information.
    pub block_e: Option<BlockE>,
    /// Block F: Causality Assessment.
    pub block_f: Option<BlockF>,
    /// Block G: Signal Indicators.
    pub block_g: Option<BlockG>,
    /// Block H: Administrative.
    pub block_h: Option<BlockH>,
}

impl IairReport {
    /// Create new IAIR report with ID.
    #[must_use]
    pub fn new(report_id: impl Into<String>) -> Self {
        Self {
            report_id: report_id.into(),
            schema_version: "GAV-SPEC-001".to_string(),
            block_a: None,
            block_b: None,
            block_c: None,
            block_d: None,
            block_e: None,
            block_f: None,
            block_g: None,
            block_h: None,
        }
    }

    /// Set Block A.
    #[must_use]
    pub fn with_block_a(mut self, block: BlockA) -> Self {
        self.block_a = Some(block);
        self
    }

    /// Set Block B.
    #[must_use]
    pub fn with_block_b(mut self, block: BlockB) -> Self {
        self.block_b = Some(block);
        self
    }

    /// Set Block C.
    #[must_use]
    pub fn with_block_c(mut self, block: BlockC) -> Self {
        self.block_c = Some(block);
        self
    }

    /// Set Block D.
    #[must_use]
    pub fn with_block_d(mut self, block: BlockD) -> Self {
        self.block_d = Some(block);
        self
    }

    /// Set Block E.
    #[must_use]
    pub fn with_block_e(mut self, block: BlockE) -> Self {
        self.block_e = Some(block);
        self
    }

    /// Set Block F.
    #[must_use]
    pub fn with_block_f(mut self, block: BlockF) -> Self {
        self.block_f = Some(block);
        self
    }

    /// Set Block G.
    #[must_use]
    pub fn with_block_g(mut self, block: BlockG) -> Self {
        self.block_g = Some(block);
        self
    }

    /// Set Block H.
    #[must_use]
    pub fn with_block_h(mut self, block: BlockH) -> Self {
        self.block_h = Some(block);
        self
    }

    /// Check if report has minimum required blocks (A, B, E).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.block_a.is_some() && self.block_b.is_some() && self.block_e.is_some()
    }

    /// Check if report is complete (all blocks present).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.block_a.is_some()
            && self.block_b.is_some()
            && self.block_c.is_some()
            && self.block_d.is_some()
            && self.block_e.is_some()
            && self.block_f.is_some()
            && self.block_g.is_some()
            && self.block_h.is_some()
    }

    /// Get outcome severity if Block E is present.
    #[must_use]
    pub fn severity(&self) -> Option<OutcomeSeverity> {
        self.block_e.as_ref().map(|e| e.severity)
    }

    /// Get ACA category if Block F has result.
    #[must_use]
    pub fn aca_category(&self) -> Option<super::scoring::AcaCausalityCategory> {
        self.block_f
            .as_ref()
            .and_then(|f| f.aca_result.as_ref())
            .map(|r| r.category)
    }

    /// Check if this is a serious incident (severity >= InterventionRequired).
    #[must_use]
    pub fn is_serious(&self) -> bool {
        self.block_e
            .as_ref()
            .map(|e| e.severity.is_serious())
            .unwrap_or(false)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_status_display() {
        assert_eq!(
            format!("{}", FdaClearanceStatus::Cleared510k),
            "510(k) Cleared"
        );
        assert_eq!(format!("{}", FdaClearanceStatus::DeNovo), "De Novo");
    }

    #[test]
    fn test_incident_category_codes() {
        assert_eq!(IncidentCategory::FalseNegative.code(), "IC-FN");
        assert_eq!(IncidentCategory::Bias.code(), "IC-BIAS");
        assert_eq!(IncidentCategory::FalseNegative.tov_harm_type(), 'C');
        assert_eq!(IncidentCategory::Bias.tov_harm_type(), 'H');
    }

    #[test]
    fn test_outcome_severity_ranges() {
        assert!(!OutcomeSeverity::NearMiss.is_harmful());
        assert!(!OutcomeSeverity::NoHarm.is_harmful());
        assert!(OutcomeSeverity::TemporaryHarm.is_harmful());
        assert!(OutcomeSeverity::Death.is_harmful());

        assert!(!OutcomeSeverity::TemporaryHarm.is_serious());
        assert!(OutcomeSeverity::Hospitalization.is_serious());
        assert!(OutcomeSeverity::Death.is_serious());
    }

    #[test]
    fn test_outcome_severity_tov_values() {
        assert_eq!(OutcomeSeverity::NoHarm.tov_severity_midpoint(), 0.0);
        assert_eq!(OutcomeSeverity::Death.tov_severity_midpoint(), 1.0);

        let (min, max) = OutcomeSeverity::LifeThreatening.tov_severity_range();
        assert!((min - 0.8).abs() < f64::EPSILON);
        assert!((max - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_block_b_builder() {
        let block = BlockB::new("TestAlgo", "1.0.0", "TestCorp")
            .with_fda_status(FdaClearanceStatus::Cleared510k)
            .with_clearance_number("K123456")
            .with_category(AlgorithmCategory::RiskPrediction)
            .with_clinical_domain(ClinicalDomain::CriticalCare)
            .with_architecture("Transformer");

        assert_eq!(block.algorithm_name, "TestAlgo");
        assert_eq!(block.fda_status, FdaClearanceStatus::Cleared510k);
        assert_eq!(block.clearance_number, Some("K123456".to_string()));
        assert_eq!(block.category, AlgorithmCategory::RiskPrediction);
    }

    #[test]
    fn test_iair_report_validity() {
        let minimal = IairReport::new("IAIR-TEST-001")
            .with_block_a(BlockA::new("Hospital", "Reporter"))
            .with_block_b(BlockB::new("Algo", "1.0", "Vendor"))
            .with_block_e(BlockE::new(OutcomeSeverity::TemporaryHarm));

        assert!(minimal.is_valid());
        assert!(!minimal.is_complete());
    }

    #[test]
    fn test_iair_report_complete() {
        let complete = IairReport::new("IAIR-TEST-002")
            .with_block_a(BlockA::new("Hospital", "Reporter"))
            .with_block_b(BlockB::new("Algo", "1.0", "Vendor"))
            .with_block_c(BlockC::new().with_age_range("65-74"))
            .with_block_d(BlockD::new().with_incident_date("2026-02-02"))
            .with_block_e(BlockE::new(OutcomeSeverity::Hospitalization))
            .with_block_f(BlockF::new())
            .with_block_g(BlockG::new().with_signal_flag(true))
            .with_block_h(BlockH::new().with_regulatory_required(true));

        assert!(complete.is_valid());
        assert!(complete.is_complete());
        assert!(complete.is_serious());
    }

    #[test]
    fn test_case_status_lifecycle() {
        let mut block_h = BlockH::new();
        assert_eq!(block_h.case_status, CaseStatus::New);

        block_h = block_h.with_status(CaseStatus::UnderReview);
        assert_eq!(block_h.case_status, CaseStatus::UnderReview);

        block_h = block_h.with_status(CaseStatus::Closed);
        assert_eq!(block_h.case_status, CaseStatus::Closed);
    }

    #[test]
    fn test_schema_version() {
        let report = IairReport::new("TEST");
        assert_eq!(report.schema_version, "GAV-SPEC-001");
    }
}
