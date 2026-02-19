//! # Individual Case Safety Report (ICSR)
//!
//! The atomic unit of pharmacovigilance. Every signal detection, causality
//! assessment, and periodic report operates on ICSRs.
//!
//! Modeled after ICH E2B(R3) core elements with extensions for computational PV.
//!
//! ## E2B(R3) Coverage
//!
//! ```text
//! ICSR
//! ├── Patient (age, sex, weight, medical_history)
//! ├── Drugs[] (name, role, dose, route, dates)
//! ├── Reactions[] (term, outcome, onset, duration)
//! ├── Causality (naranjo, who_umc, reporter)
//! └── Report (source, country, date, seriousness)
//! ```
//!
//! ## T1 Grounding
//!
//! | ICSR Component | T1 Primitive | Symbol |
//! |----------------|-------------|--------|
//! | Case identity | Existence | ∃ |
//! | Drug→Reaction | Causality | → |
//! | Onset sequence | Sequence | σ |
//! | Severity level | Comparison | κ |
//! | Case state | State | ς |

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// CORE ICSR TYPE
// ═══════════════════════════════════════════════════════════════════════════

/// Individual Case Safety Report — the atomic unit of pharmacovigilance.
///
/// Conforms to ICH E2B(R3) core data elements.
///
/// # Tier: T3 (Domain-Specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icsr {
    /// Unique case identifier (E2B: C.1.1)
    pub case_id: CaseId,
    /// Patient information (E2B: D)
    pub patient: Patient,
    /// Suspect and concomitant drugs (E2B: G)
    pub drugs: Vec<Drug>,
    /// Adverse reactions/events (E2B: E)
    pub reactions: Vec<Reaction>,
    /// Causality assessments (E2B: G.k.9)
    pub causality: Vec<CausalityAssessment>,
    /// Report metadata (E2B: A, C)
    pub report: ReportInfo,
    /// Seriousness criteria (E2B: E.i.3)
    pub seriousness: Seriousness,
}

/// Unique case identifier.
///
/// # Tier: T2-P (wraps String for domain safety)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaseId(String);

impl CaseId {
    /// Create new case ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get raw ID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PATIENT (E2B Section D)
// ═══════════════════════════════════════════════════════════════════════════

/// Patient demographics and medical history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    /// Age at onset (years, None if unknown)
    pub age: Option<f64>,
    /// Biological sex
    pub sex: Sex,
    /// Weight in kg (None if unknown)
    pub weight_kg: Option<f64>,
    /// Relevant medical history (MedDRA preferred terms)
    pub medical_history: Vec<String>,
}

impl Default for Patient {
    fn default() -> Self {
        Self {
            age: None,
            sex: Sex::Unknown,
            weight_kg: None,
            medical_history: Vec::new(),
        }
    }
}

/// Biological sex per E2B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

// ═══════════════════════════════════════════════════════════════════════════
// DRUG (E2B Section G)
// ═══════════════════════════════════════════════════════════════════════════

/// Drug information per E2B G.k.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    /// Drug name (generic preferred)
    pub name: String,
    /// Role in the adverse event
    pub role: DrugRole,
    /// Dose and route
    pub dosage: Option<Dosage>,
    /// Start date (ISO 8601)
    pub start_date: Option<String>,
    /// End date (ISO 8601)
    pub end_date: Option<String>,
    /// Indication (MedDRA preferred term)
    pub indication: Option<String>,
    /// Action taken after event
    pub action: DrugAction,
}

/// Drug's role in the adverse event.
///
/// # Tier: T2-P (maps to FAERS DrugRole)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrugRole {
    /// Primary suspect
    Suspect,
    /// Concomitant medication
    Concomitant,
    /// Interacting drug
    Interacting,
    /// Treatment for the event
    Treatment,
}

/// Action taken with drug after event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrugAction {
    /// Withdrawn
    Withdrawn,
    /// Dose reduced
    DoseReduced,
    /// Dose increased
    DoseIncreased,
    /// Unchanged
    Unchanged,
    /// Unknown
    Unknown,
    /// Not applicable
    NotApplicable,
}

impl Default for DrugAction {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Dosage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dosage {
    /// Numeric dose value
    pub value: f64,
    /// Unit (mg, mcg, mL, etc.)
    pub unit: String,
    /// Route of administration
    pub route: Route,
    /// Frequency (e.g., "QD", "BID")
    pub frequency: Option<String>,
}

/// Route of administration (common routes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Route {
    Oral,
    Intravenous,
    Intramuscular,
    Subcutaneous,
    Topical,
    Inhalation,
    Rectal,
    Ophthalmic,
    Other,
    Unknown,
}

impl Default for Route {
    fn default() -> Self {
        Self::Unknown
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// REACTION (E2B Section E)
// ═══════════════════════════════════════════════════════════════════════════

/// Adverse reaction/event per E2B E.i.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// MedDRA preferred term
    pub term: String,
    /// MedDRA code (if known)
    pub meddra_code: Option<u64>,
    /// Outcome
    pub outcome: ReactionOutcome,
    /// Time to onset from drug start (days, None if unknown)
    pub onset_days: Option<f64>,
    /// Duration (days, None if ongoing/unknown)
    pub duration_days: Option<f64>,
}

/// Reaction outcome per E2B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReactionOutcome {
    Recovered,
    Recovering,
    NotRecovered,
    RecoveredWithSequelae,
    Fatal,
    Unknown,
}

impl Default for ReactionOutcome {
    fn default() -> Self {
        Self::Unknown
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CAUSALITY (E2B G.k.9)
// ═══════════════════════════════════════════════════════════════════════════

/// Causality assessment linking a drug to a reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityAssessment {
    /// Drug index (position in Icsr.drugs)
    pub drug_index: usize,
    /// Reaction index (position in Icsr.reactions)
    pub reaction_index: usize,
    /// Assessment method
    pub method: CausalityMethod,
    /// Assessment result category
    pub result: CausalityResult,
    /// Assessor (reporter, sponsor, regulatory)
    pub assessor: Assessor,
}

/// Causality assessment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CausalityMethod {
    /// Naranjo algorithm (10 questions)
    Naranjo,
    /// WHO-UMC system (6 categories)
    WhoUmc,
    /// RUCAM for hepatotoxicity
    Rucam,
    /// Clinical judgment
    ClinicalJudgment,
    /// Algorithm-based (NexCore UCAS)
    Algorithmic,
}

/// Causality result — unified across methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CausalityResult {
    /// Not assessable / unclassifiable
    Unassessable,
    /// Unlikely / doubtful
    Unlikely,
    /// Possible
    Possible,
    /// Probable / likely
    Probable,
    /// Certain / definite
    Certain,
}

/// Who performed the assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Assessor {
    Reporter,
    Sponsor,
    RegulatoryAuthority,
    Algorithm,
}

// ═══════════════════════════════════════════════════════════════════════════
// REPORT METADATA (E2B Sections A, C)
// ═══════════════════════════════════════════════════════════════════════════

/// Report metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInfo {
    /// Report type
    pub report_type: ReportType,
    /// Source of report
    pub source: ReportSource,
    /// Country (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Date of receipt (ISO 8601)
    pub receipt_date: Option<String>,
    /// Date of most recent info (ISO 8601)
    pub latest_date: Option<String>,
}

impl Default for ReportInfo {
    fn default() -> Self {
        Self {
            report_type: ReportType::Spontaneous,
            source: ReportSource::HealthcareProfessional,
            country: None,
            receipt_date: None,
            latest_date: None,
        }
    }
}

/// Report type per E2B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    Spontaneous,
    StudyReport,
    Literature,
    Other,
}

/// Source of the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportSource {
    HealthcareProfessional,
    Consumer,
    Lawyer,
    Other,
}

// ═══════════════════════════════════════════════════════════════════════════
// SERIOUSNESS (E2B E.i.3)
// ═══════════════════════════════════════════════════════════════════════════

/// Seriousness criteria per ICH E2A.
///
/// An event is serious if ANY criterion is true.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Seriousness {
    /// Results in death
    pub death: bool,
    /// Life-threatening
    pub life_threatening: bool,
    /// Requires hospitalization or prolongs existing hospitalization
    pub hospitalization: bool,
    /// Results in persistent or significant disability/incapacity
    pub disability: bool,
    /// Congenital anomaly/birth defect
    pub congenital_anomaly: bool,
    /// Other medically important condition
    pub other_medically_important: bool,
}

impl Seriousness {
    /// Check if the event is serious (any criterion true).
    #[must_use]
    pub fn is_serious(&self) -> bool {
        self.death
            || self.life_threatening
            || self.hospitalization
            || self.disability
            || self.congenital_anomaly
            || self.other_medically_important
    }

    /// Count how many seriousness criteria are met.
    #[must_use]
    pub fn criteria_count(&self) -> u8 {
        self.death as u8
            + self.life_threatening as u8
            + self.hospitalization as u8
            + self.disability as u8
            + self.congenital_anomaly as u8
            + self.other_medically_important as u8
    }

    /// Most severe criterion met.
    #[must_use]
    pub fn most_severe(&self) -> Option<SeriousnessCriterion> {
        if self.death {
            Some(SeriousnessCriterion::Death)
        } else if self.life_threatening {
            Some(SeriousnessCriterion::LifeThreatening)
        } else if self.hospitalization {
            Some(SeriousnessCriterion::Hospitalization)
        } else if self.disability {
            Some(SeriousnessCriterion::Disability)
        } else if self.congenital_anomaly {
            Some(SeriousnessCriterion::CongenitalAnomaly)
        } else if self.other_medically_important {
            Some(SeriousnessCriterion::OtherMedicallyImportant)
        } else {
            None
        }
    }
}

/// Individual seriousness criterion (for enumeration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SeriousnessCriterion {
    Death,
    LifeThreatening,
    Hospitalization,
    Disability,
    CongenitalAnomaly,
    OtherMedicallyImportant,
}

// ═══════════════════════════════════════════════════════════════════════════
// BUILDER
// ═══════════════════════════════════════════════════════════════════════════

/// Builder for ICSR construction.
pub struct IcsrBuilder {
    case_id: CaseId,
    patient: Patient,
    drugs: Vec<Drug>,
    reactions: Vec<Reaction>,
    causality: Vec<CausalityAssessment>,
    report: ReportInfo,
    seriousness: Seriousness,
}

impl IcsrBuilder {
    /// Start building with a case ID.
    pub fn new(case_id: impl Into<String>) -> Self {
        Self {
            case_id: CaseId::new(case_id),
            patient: Patient::default(),
            drugs: Vec::new(),
            reactions: Vec::new(),
            causality: Vec::new(),
            report: ReportInfo::default(),
            seriousness: Seriousness::default(),
        }
    }

    /// Set patient info.
    pub fn patient(mut self, patient: Patient) -> Self {
        self.patient = patient;
        self
    }

    /// Add a suspect drug.
    pub fn suspect_drug(mut self, name: impl Into<String>) -> Self {
        self.drugs.push(Drug {
            name: name.into(),
            role: DrugRole::Suspect,
            dosage: None,
            start_date: None,
            end_date: None,
            indication: None,
            action: DrugAction::default(),
        });
        self
    }

    /// Add a reaction.
    pub fn reaction(mut self, term: impl Into<String>, outcome: ReactionOutcome) -> Self {
        self.reactions.push(Reaction {
            term: term.into(),
            meddra_code: None,
            outcome,
            onset_days: None,
            duration_days: None,
        });
        self
    }

    /// Set seriousness.
    pub fn seriousness(mut self, seriousness: Seriousness) -> Self {
        self.seriousness = seriousness;
        self
    }

    /// Set report info.
    pub fn report(mut self, report: ReportInfo) -> Self {
        self.report = report;
        self
    }

    /// Add causality assessment.
    pub fn causality(mut self, assessment: CausalityAssessment) -> Self {
        self.causality.push(assessment);
        self
    }

    /// Build the ICSR.
    #[must_use]
    pub fn build(self) -> Icsr {
        Icsr {
            case_id: self.case_id,
            patient: self.patient,
            drugs: self.drugs,
            reactions: self.reactions,
            causality: self.causality,
            report: self.report,
            seriousness: self.seriousness,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_minimal_icsr() {
        let icsr = IcsrBuilder::new("CASE-001")
            .suspect_drug("ASPIRIN")
            .reaction("Gastrointestinal haemorrhage", ReactionOutcome::Recovered)
            .build();

        assert_eq!(icsr.case_id.as_str(), "CASE-001");
        assert_eq!(icsr.drugs.len(), 1);
        assert_eq!(icsr.reactions.len(), 1);
        assert!(!icsr.seriousness.is_serious());
    }

    #[test]
    fn build_serious_icsr() {
        let icsr = IcsrBuilder::new("CASE-002")
            .suspect_drug("ROFECOXIB")
            .reaction("Myocardial infarction", ReactionOutcome::Fatal)
            .seriousness(Seriousness {
                death: true,
                ..Default::default()
            })
            .build();

        assert!(icsr.seriousness.is_serious());
        assert_eq!(icsr.seriousness.criteria_count(), 1);
        assert_eq!(
            icsr.seriousness.most_severe(),
            Some(SeriousnessCriterion::Death)
        );
    }

    #[test]
    fn seriousness_multiple_criteria() {
        let s = Seriousness {
            hospitalization: true,
            disability: true,
            ..Default::default()
        };
        assert!(s.is_serious());
        assert_eq!(s.criteria_count(), 2);
        assert_eq!(s.most_severe(), Some(SeriousnessCriterion::Hospitalization));
    }

    #[test]
    fn case_id_display() {
        let id = CaseId::new("US-FDA-2024-12345");
        assert_eq!(format!("{id}"), "US-FDA-2024-12345");
    }

    #[test]
    fn drug_role_variants() {
        let suspect = DrugRole::Suspect;
        let concomitant = DrugRole::Concomitant;
        assert_ne!(suspect, concomitant);
    }

    #[test]
    fn icsr_with_causality() {
        let icsr = IcsrBuilder::new("CASE-003")
            .suspect_drug("IBUPROFEN")
            .reaction("Renal failure acute", ReactionOutcome::Recovering)
            .causality(CausalityAssessment {
                drug_index: 0,
                reaction_index: 0,
                method: CausalityMethod::Naranjo,
                result: CausalityResult::Probable,
                assessor: Assessor::Algorithm,
            })
            .build();

        assert_eq!(icsr.causality.len(), 1);
        assert_eq!(icsr.causality[0].result, CausalityResult::Probable);
    }

    #[test]
    fn causality_ordering() {
        assert!(CausalityResult::Certain > CausalityResult::Probable);
        assert!(CausalityResult::Probable > CausalityResult::Possible);
        assert!(CausalityResult::Possible > CausalityResult::Unlikely);
    }
}
