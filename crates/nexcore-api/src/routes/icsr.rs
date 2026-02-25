//! ICSR (Individual Case Safety Report) endpoints — Tier 4.
//!
//! ## Endpoints
//!
//! - `POST /icsr/build`    — Construct an E2B(R3)-compliant ICSR from input fields
//! - `POST /icsr/validate` — Validate an ICSR against E2B structural requirements
//!
//! All types from `nexcore_pv_core::icsr` are already `Serialize + Deserialize`,
//! so these handlers are thin JSON adapters over the existing builder/types.

use axum::{Json, Router, routing::post};
use nexcore_pv_core::icsr::{
    Assessor, CausalityAssessment, CausalityMethod, CausalityResult, Dosage, Drug, DrugAction,
    DrugRole, Icsr, Patient, Reaction, ReactionOutcome, ReportInfo, ReportSource, ReportType,
    Route as DrugRoute, Seriousness, Sex,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

// ── Build request/response types ─────────────────────────────────────────────

/// Request to construct an ICSR.
///
/// All fields mirror the E2B(R3) ICSR structure. The `case_id` and at least one
/// drug and one reaction are required. All other fields have sensible defaults.
#[derive(Debug, Deserialize, ToSchema)]
pub struct IcsrBuildRequest {
    /// Unique case identifier (E2B: C.1.1)
    pub case_id: String,
    /// Patient demographics
    #[serde(default)]
    pub patient: Option<PatientDto>,
    /// Suspect and concomitant drugs (at least one required)
    pub drugs: Vec<DrugDto>,
    /// Adverse reactions (at least one required)
    pub reactions: Vec<ReactionDto>,
    /// Causality assessments linking drugs to reactions
    #[serde(default)]
    pub causality: Vec<CausalityDto>,
    /// Report metadata
    #[serde(default)]
    pub report: Option<ReportInfoDto>,
    /// Seriousness criteria
    #[serde(default)]
    pub seriousness: Option<SeriousnessDto>,
}

/// Patient demographics DTO.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct PatientDto {
    /// Age at onset (years)
    pub age: Option<f64>,
    /// Biological sex: `"Male"`, `"Female"`, `"Unknown"`
    #[serde(default = "default_sex")]
    pub sex: String,
    /// Weight in kg
    pub weight_kg: Option<f64>,
    /// Relevant medical history terms
    #[serde(default)]
    pub medical_history: Vec<String>,
}

fn default_sex() -> String {
    "Unknown".to_string()
}

/// Drug information DTO.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DrugDto {
    /// Drug name (generic preferred)
    pub name: String,
    /// Role: `"Suspect"`, `"Concomitant"`, `"Interacting"`, `"Treatment"`
    #[serde(default = "default_drug_role")]
    pub role: String,
    /// Dosage information
    #[serde(default)]
    pub dosage: Option<DosageDto>,
    /// Start date (ISO 8601)
    #[serde(default)]
    pub start_date: Option<String>,
    /// End date (ISO 8601)
    #[serde(default)]
    pub end_date: Option<String>,
    /// Indication (MedDRA preferred term)
    #[serde(default)]
    pub indication: Option<String>,
    /// Action taken: `"Withdrawn"`, `"DoseReduced"`, `"DoseIncreased"`,
    /// `"Unchanged"`, `"Unknown"`, `"NotApplicable"`
    #[serde(default = "default_drug_action")]
    pub action: String,
}

fn default_drug_role() -> String {
    "Suspect".to_string()
}

fn default_drug_action() -> String {
    "Unknown".to_string()
}

/// Dosage DTO.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DosageDto {
    /// Numeric dose value
    pub value: f64,
    /// Unit (mg, mcg, mL, etc.)
    pub unit: String,
    /// Route: `"Oral"`, `"Intravenous"`, `"Intramuscular"`, `"Subcutaneous"`,
    /// `"Topical"`, `"Inhalation"`, `"Rectal"`, `"Ophthalmic"`, `"Other"`, `"Unknown"`
    #[serde(default = "default_route")]
    pub route: String,
    /// Frequency (e.g., `"QD"`, `"BID"`)
    #[serde(default)]
    pub frequency: Option<String>,
}

fn default_route() -> String {
    "Unknown".to_string()
}

/// Reaction DTO.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReactionDto {
    /// MedDRA preferred term
    pub term: String,
    /// MedDRA code (if known)
    #[serde(default)]
    pub meddra_code: Option<u64>,
    /// Outcome: `"Recovered"`, `"Recovering"`, `"NotRecovered"`,
    /// `"RecoveredWithSequelae"`, `"Fatal"`, `"Unknown"`
    #[serde(default = "default_outcome")]
    pub outcome: String,
    /// Time to onset from drug start (days)
    #[serde(default)]
    pub onset_days: Option<f64>,
    /// Duration (days)
    #[serde(default)]
    pub duration_days: Option<f64>,
}

fn default_outcome() -> String {
    "Unknown".to_string()
}

/// Causality assessment DTO.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CausalityDto {
    /// Index into the drugs array
    pub drug_index: usize,
    /// Index into the reactions array
    pub reaction_index: usize,
    /// Method: `"Naranjo"`, `"WhoUmc"`, `"Rucam"`, `"ClinicalJudgment"`, `"Algorithmic"`
    pub method: String,
    /// Result: `"Certain"`, `"Probable"`, `"Possible"`, `"Unlikely"`, `"Unassessable"`
    pub result: String,
    /// Assessor: `"Reporter"`, `"Sponsor"`, `"RegulatoryAuthority"`, `"Algorithm"`
    #[serde(default = "default_assessor")]
    pub assessor: String,
}

fn default_assessor() -> String {
    "Algorithm".to_string()
}

/// Report metadata DTO.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct ReportInfoDto {
    /// Report type: `"Spontaneous"`, `"StudyReport"`, `"Literature"`, `"Other"`
    #[serde(default = "default_report_type")]
    pub report_type: String,
    /// Source: `"HealthcareProfessional"`, `"Consumer"`, `"Lawyer"`, `"Other"`
    #[serde(default = "default_report_source")]
    pub source: String,
    /// Country (ISO 3166-1 alpha-2)
    #[serde(default)]
    pub country: Option<String>,
    /// Date of receipt (ISO 8601)
    #[serde(default)]
    pub receipt_date: Option<String>,
    /// Date of most recent info (ISO 8601)
    #[serde(default)]
    pub latest_date: Option<String>,
}

fn default_report_type() -> String {
    "Spontaneous".to_string()
}

fn default_report_source() -> String {
    "HealthcareProfessional".to_string()
}

/// Seriousness criteria DTO.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub struct SeriousnessDto {
    #[serde(default)]
    pub death: bool,
    #[serde(default)]
    pub life_threatening: bool,
    #[serde(default)]
    pub hospitalization: bool,
    #[serde(default)]
    pub disability: bool,
    #[serde(default)]
    pub congenital_anomaly: bool,
    #[serde(default)]
    pub other_medically_important: bool,
}

/// Response containing the constructed ICSR.
#[derive(Debug, Serialize, ToSchema)]
pub struct IcsrBuildResponse {
    /// The fully constructed ICSR
    pub icsr: IcsrOutputDto,
    /// Whether the ICSR is considered serious (any seriousness criterion is true)
    pub is_serious: bool,
    /// Count of seriousness criteria met
    pub seriousness_criteria_count: u8,
}

/// Serializable ICSR output matching E2B(R3) structure.
#[derive(Debug, Serialize, ToSchema)]
pub struct IcsrOutputDto {
    pub case_id: String,
    pub patient: PatientDto,
    pub drugs: Vec<DrugDto>,
    pub reactions: Vec<ReactionDto>,
    pub causality: Vec<CausalityDto>,
    pub report: ReportInfoDto,
    pub seriousness: SeriousnessDto,
}

// ── Validate types ───────────────────────────────────────────────────────────

/// Request to validate an ICSR.
///
/// Accepts the same structure as `IcsrBuildRequest` — checks it against E2B
/// structural requirements and returns any violations.
#[derive(Debug, Deserialize, ToSchema)]
pub struct IcsrValidateRequest {
    /// Unique case identifier
    pub case_id: String,
    #[serde(default)]
    pub patient: Option<PatientDto>,
    #[serde(default)]
    pub drugs: Vec<DrugDto>,
    #[serde(default)]
    pub reactions: Vec<ReactionDto>,
    #[serde(default)]
    pub causality: Vec<CausalityDto>,
    #[serde(default)]
    pub report: Option<ReportInfoDto>,
    #[serde(default)]
    pub seriousness: Option<SeriousnessDto>,
}

/// Validation response.
#[derive(Debug, Serialize, ToSchema)]
pub struct IcsrValidateResponse {
    /// Whether the ICSR passes all validation checks
    pub valid: bool,
    /// List of validation issues (empty when valid)
    pub issues: Vec<ValidationIssue>,
    /// Number of issues found
    pub issue_count: usize,
}

/// A single validation issue.
#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationIssue {
    /// Severity: `"error"` (blocks submission) or `"warning"` (advisory)
    pub severity: String,
    /// E2B field reference (e.g., `"C.1.1"`, `"G.k"`)
    pub field: String,
    /// Human-readable description
    pub message: String,
}

// ── DTO ↔ Domain conversions ─────────────────────────────────────────────────

fn parse_sex(s: &str) -> Sex {
    match s {
        "Male" => Sex::Male,
        "Female" => Sex::Female,
        _ => Sex::Unknown,
    }
}

fn sex_to_str(s: Sex) -> String {
    match s {
        Sex::Male => "Male",
        Sex::Female => "Female",
        Sex::Unknown => "Unknown",
    }
    .to_string()
}

fn parse_drug_role(s: &str) -> DrugRole {
    match s {
        "Suspect" => DrugRole::Suspect,
        "Concomitant" => DrugRole::Concomitant,
        "Interacting" => DrugRole::Interacting,
        "Treatment" => DrugRole::Treatment,
        _ => DrugRole::Suspect,
    }
}

fn drug_role_to_str(r: DrugRole) -> String {
    match r {
        DrugRole::Suspect => "Suspect",
        DrugRole::Concomitant => "Concomitant",
        DrugRole::Interacting => "Interacting",
        DrugRole::Treatment => "Treatment",
    }
    .to_string()
}

fn parse_drug_action(s: &str) -> DrugAction {
    match s {
        "Withdrawn" => DrugAction::Withdrawn,
        "DoseReduced" => DrugAction::DoseReduced,
        "DoseIncreased" => DrugAction::DoseIncreased,
        "Unchanged" => DrugAction::Unchanged,
        "NotApplicable" => DrugAction::NotApplicable,
        _ => DrugAction::Unknown,
    }
}

fn drug_action_to_str(a: DrugAction) -> String {
    match a {
        DrugAction::Withdrawn => "Withdrawn",
        DrugAction::DoseReduced => "DoseReduced",
        DrugAction::DoseIncreased => "DoseIncreased",
        DrugAction::Unchanged => "Unchanged",
        DrugAction::NotApplicable => "NotApplicable",
        DrugAction::Unknown => "Unknown",
    }
    .to_string()
}

fn parse_route(s: &str) -> DrugRoute {
    match s {
        "Oral" => DrugRoute::Oral,
        "Intravenous" => DrugRoute::Intravenous,
        "Intramuscular" => DrugRoute::Intramuscular,
        "Subcutaneous" => DrugRoute::Subcutaneous,
        "Topical" => DrugRoute::Topical,
        "Inhalation" => DrugRoute::Inhalation,
        "Rectal" => DrugRoute::Rectal,
        "Ophthalmic" => DrugRoute::Ophthalmic,
        "Other" => DrugRoute::Other,
        _ => DrugRoute::Unknown,
    }
}

fn route_to_str(r: DrugRoute) -> String {
    match r {
        DrugRoute::Oral => "Oral",
        DrugRoute::Intravenous => "Intravenous",
        DrugRoute::Intramuscular => "Intramuscular",
        DrugRoute::Subcutaneous => "Subcutaneous",
        DrugRoute::Topical => "Topical",
        DrugRoute::Inhalation => "Inhalation",
        DrugRoute::Rectal => "Rectal",
        DrugRoute::Ophthalmic => "Ophthalmic",
        DrugRoute::Other => "Other",
        DrugRoute::Unknown => "Unknown",
    }
    .to_string()
}

fn parse_outcome(s: &str) -> ReactionOutcome {
    match s {
        "Recovered" => ReactionOutcome::Recovered,
        "Recovering" => ReactionOutcome::Recovering,
        "NotRecovered" => ReactionOutcome::NotRecovered,
        "RecoveredWithSequelae" => ReactionOutcome::RecoveredWithSequelae,
        "Fatal" => ReactionOutcome::Fatal,
        _ => ReactionOutcome::Unknown,
    }
}

fn outcome_to_str(o: ReactionOutcome) -> String {
    match o {
        ReactionOutcome::Recovered => "Recovered",
        ReactionOutcome::Recovering => "Recovering",
        ReactionOutcome::NotRecovered => "NotRecovered",
        ReactionOutcome::RecoveredWithSequelae => "RecoveredWithSequelae",
        ReactionOutcome::Fatal => "Fatal",
        ReactionOutcome::Unknown => "Unknown",
    }
    .to_string()
}

fn parse_causality_method(s: &str) -> CausalityMethod {
    match s {
        "Naranjo" => CausalityMethod::Naranjo,
        "WhoUmc" => CausalityMethod::WhoUmc,
        "Rucam" => CausalityMethod::Rucam,
        "ClinicalJudgment" => CausalityMethod::ClinicalJudgment,
        _ => CausalityMethod::Algorithmic,
    }
}

fn causality_method_to_str(m: CausalityMethod) -> String {
    match m {
        CausalityMethod::Naranjo => "Naranjo",
        CausalityMethod::WhoUmc => "WhoUmc",
        CausalityMethod::Rucam => "Rucam",
        CausalityMethod::ClinicalJudgment => "ClinicalJudgment",
        CausalityMethod::Algorithmic => "Algorithmic",
    }
    .to_string()
}

fn parse_causality_result(s: &str) -> CausalityResult {
    match s {
        "Certain" => CausalityResult::Certain,
        "Probable" => CausalityResult::Probable,
        "Possible" => CausalityResult::Possible,
        "Unlikely" => CausalityResult::Unlikely,
        _ => CausalityResult::Unassessable,
    }
}

fn causality_result_to_str(r: CausalityResult) -> String {
    match r {
        CausalityResult::Certain => "Certain",
        CausalityResult::Probable => "Probable",
        CausalityResult::Possible => "Possible",
        CausalityResult::Unlikely => "Unlikely",
        CausalityResult::Unassessable => "Unassessable",
    }
    .to_string()
}

fn parse_assessor(s: &str) -> Assessor {
    match s {
        "Reporter" => Assessor::Reporter,
        "Sponsor" => Assessor::Sponsor,
        "RegulatoryAuthority" => Assessor::RegulatoryAuthority,
        _ => Assessor::Algorithm,
    }
}

fn assessor_to_str(a: Assessor) -> String {
    match a {
        Assessor::Reporter => "Reporter",
        Assessor::Sponsor => "Sponsor",
        Assessor::RegulatoryAuthority => "RegulatoryAuthority",
        Assessor::Algorithm => "Algorithm",
    }
    .to_string()
}

fn parse_report_type(s: &str) -> ReportType {
    match s {
        "Spontaneous" => ReportType::Spontaneous,
        "StudyReport" => ReportType::StudyReport,
        "Literature" => ReportType::Literature,
        _ => ReportType::Other,
    }
}

fn report_type_to_str(t: ReportType) -> String {
    match t {
        ReportType::Spontaneous => "Spontaneous",
        ReportType::StudyReport => "StudyReport",
        ReportType::Literature => "Literature",
        ReportType::Other => "Other",
    }
    .to_string()
}

fn parse_report_source(s: &str) -> ReportSource {
    match s {
        "HealthcareProfessional" => ReportSource::HealthcareProfessional,
        "Consumer" => ReportSource::Consumer,
        "Lawyer" => ReportSource::Lawyer,
        _ => ReportSource::Other,
    }
}

fn report_source_to_str(s: ReportSource) -> String {
    match s {
        ReportSource::HealthcareProfessional => "HealthcareProfessional",
        ReportSource::Consumer => "Consumer",
        ReportSource::Lawyer => "Lawyer",
        ReportSource::Other => "Other",
    }
    .to_string()
}

/// Convert a built `Icsr` back to the DTO for serialization.
fn icsr_to_output(icsr: &Icsr) -> IcsrOutputDto {
    IcsrOutputDto {
        case_id: icsr.case_id.as_str().to_string(),
        patient: PatientDto {
            age: icsr.patient.age,
            sex: sex_to_str(icsr.patient.sex),
            weight_kg: icsr.patient.weight_kg,
            medical_history: icsr.patient.medical_history.clone(),
        },
        drugs: icsr
            .drugs
            .iter()
            .map(|d| DrugDto {
                name: d.name.clone(),
                role: drug_role_to_str(d.role),
                dosage: d.dosage.as_ref().map(|dos| DosageDto {
                    value: dos.value,
                    unit: dos.unit.clone(),
                    route: route_to_str(dos.route),
                    frequency: dos.frequency.clone(),
                }),
                start_date: d.start_date.clone(),
                end_date: d.end_date.clone(),
                indication: d.indication.clone(),
                action: drug_action_to_str(d.action),
            })
            .collect(),
        reactions: icsr
            .reactions
            .iter()
            .map(|r| ReactionDto {
                term: r.term.clone(),
                meddra_code: r.meddra_code,
                outcome: outcome_to_str(r.outcome),
                onset_days: r.onset_days,
                duration_days: r.duration_days,
            })
            .collect(),
        causality: icsr
            .causality
            .iter()
            .map(|c| CausalityDto {
                drug_index: c.drug_index,
                reaction_index: c.reaction_index,
                method: causality_method_to_str(c.method),
                result: causality_result_to_str(c.result),
                assessor: assessor_to_str(c.assessor),
            })
            .collect(),
        report: ReportInfoDto {
            report_type: report_type_to_str(icsr.report.report_type),
            source: report_source_to_str(icsr.report.source),
            country: icsr.report.country.clone(),
            receipt_date: icsr.report.receipt_date.clone(),
            latest_date: icsr.report.latest_date.clone(),
        },
        seriousness: SeriousnessDto {
            death: icsr.seriousness.death,
            life_threatening: icsr.seriousness.life_threatening,
            hospitalization: icsr.seriousness.hospitalization,
            disability: icsr.seriousness.disability,
            congenital_anomaly: icsr.seriousness.congenital_anomaly,
            other_medically_important: icsr.seriousness.other_medically_important,
        },
    }
}

// ── Router ───────────────────────────────────────────────────────────────────

/// ICSR router. Nested under `/api/v1/icsr`.
pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/build", post(icsr_build))
        .route("/validate", post(icsr_validate))
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Construct an E2B(R3)-compliant ICSR from input fields.
///
/// Accepts patient, drugs, reactions, causality assessments, report metadata,
/// and seriousness criteria. Returns the fully constructed ICSR with computed
/// fields (seriousness assessment, criteria count).
///
/// ## Validation
/// - `case_id` must not be empty
/// - At least one drug required
/// - At least one reaction required
#[utoipa::path(
    post,
    path = "/api/v1/icsr/build",
    tag = "icsr",
    request_body = IcsrBuildRequest,
    responses(
        (status = 200, description = "ICSR constructed successfully", body = IcsrBuildResponse),
        (status = 400, description = "Validation error", body = ApiError),
    )
)]
pub async fn icsr_build(Json(req): Json<IcsrBuildRequest>) -> ApiResult<IcsrBuildResponse> {
    // Input validation
    if req.case_id.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "case_id must not be empty",
        ));
    }
    if req.drugs.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "At least one drug is required",
        ));
    }
    if req.reactions.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "At least one reaction is required",
        ));
    }

    // Build patient
    let patient = match &req.patient {
        Some(p) => Patient {
            age: p.age,
            sex: parse_sex(&p.sex),
            weight_kg: p.weight_kg,
            medical_history: p.medical_history.clone(),
        },
        None => Patient::default(),
    };

    // Build drugs
    let drugs: Vec<Drug> = req
        .drugs
        .iter()
        .map(|d| Drug {
            name: d.name.clone(),
            role: parse_drug_role(&d.role),
            dosage: d.dosage.as_ref().map(|dos| Dosage {
                value: dos.value,
                unit: dos.unit.clone(),
                route: parse_route(&dos.route),
                frequency: dos.frequency.clone(),
            }),
            start_date: d.start_date.clone(),
            end_date: d.end_date.clone(),
            indication: d.indication.clone(),
            action: parse_drug_action(&d.action),
        })
        .collect();

    // Build reactions
    let reactions: Vec<Reaction> = req
        .reactions
        .iter()
        .map(|r| Reaction {
            term: r.term.clone(),
            meddra_code: r.meddra_code,
            outcome: parse_outcome(&r.outcome),
            onset_days: r.onset_days,
            duration_days: r.duration_days,
        })
        .collect();

    // Build causality
    let causality: Vec<CausalityAssessment> = req
        .causality
        .iter()
        .filter(|c| c.drug_index < drugs.len() && c.reaction_index < reactions.len())
        .map(|c| CausalityAssessment {
            drug_index: c.drug_index,
            reaction_index: c.reaction_index,
            method: parse_causality_method(&c.method),
            result: parse_causality_result(&c.result),
            assessor: parse_assessor(&c.assessor),
        })
        .collect();

    // Build report info
    let report = match &req.report {
        Some(r) => ReportInfo {
            report_type: parse_report_type(&r.report_type),
            source: parse_report_source(&r.source),
            country: r.country.clone(),
            receipt_date: r.receipt_date.clone(),
            latest_date: r.latest_date.clone(),
        },
        None => ReportInfo::default(),
    };

    // Build seriousness
    let seriousness = match &req.seriousness {
        Some(s) => Seriousness {
            death: s.death,
            life_threatening: s.life_threatening,
            hospitalization: s.hospitalization,
            disability: s.disability,
            congenital_anomaly: s.congenital_anomaly,
            other_medically_important: s.other_medically_important,
        },
        None => Seriousness::default(),
    };

    let icsr = Icsr {
        case_id: nexcore_pv_core::icsr::CaseId::new(&req.case_id),
        patient,
        drugs,
        reactions,
        causality,
        report,
        seriousness,
    };

    let is_serious = icsr.seriousness.is_serious();
    let criteria_count = icsr.seriousness.criteria_count();
    let output = icsr_to_output(&icsr);

    Ok(Json(IcsrBuildResponse {
        icsr: output,
        is_serious,
        seriousness_criteria_count: criteria_count,
    }))
}

/// Validate an ICSR against E2B(R3) structural requirements.
///
/// Checks for required fields, structural consistency, and E2B compliance.
/// Returns a list of issues categorized as `"error"` (blocks submission) or
/// `"warning"` (advisory only).
///
/// ## Validation Rules
///
/// **Errors** (block E2B submission):
/// - Case ID must not be empty (C.1.1)
/// - At least one drug required (G.k)
/// - At least one reaction required (E.i)
/// - Drug names must not be empty (G.k.2)
/// - Reaction terms must not be empty (E.i.1)
/// - Seriousness must be assessed if serious criteria are indicated (E.i.3)
///
/// **Warnings** (advisory):
/// - Patient information absent (D)
/// - No causality assessment provided (G.k.9)
/// - Report metadata missing (A/C)
/// - Fatal outcome without death seriousness criterion
/// - Causality index out of bounds
#[utoipa::path(
    post,
    path = "/api/v1/icsr/validate",
    tag = "icsr",
    request_body = IcsrValidateRequest,
    responses(
        (status = 200, description = "Validation results", body = IcsrValidateResponse),
    )
)]
pub async fn icsr_validate(
    Json(req): Json<IcsrValidateRequest>,
) -> ApiResult<IcsrValidateResponse> {
    let mut issues = Vec::new();

    // ── Errors (block submission) ────────────────────────────────────────

    if req.case_id.trim().is_empty() {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            field: "C.1.1".to_string(),
            message: "Case ID (C.1.1) must not be empty".to_string(),
        });
    }

    if req.drugs.is_empty() {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            field: "G.k".to_string(),
            message: "At least one drug (G.k) is required".to_string(),
        });
    }

    if req.reactions.is_empty() {
        issues.push(ValidationIssue {
            severity: "error".to_string(),
            field: "E.i".to_string(),
            message: "At least one reaction (E.i) is required".to_string(),
        });
    }

    for (i, drug) in req.drugs.iter().enumerate() {
        if drug.name.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                field: format!("G.k.2[{i}]"),
                message: format!("Drug name (G.k.2) at index {i} must not be empty"),
            });
        }
    }

    for (i, reaction) in req.reactions.iter().enumerate() {
        if reaction.term.trim().is_empty() {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                field: format!("E.i.1[{i}]"),
                message: format!("Reaction term (E.i.1) at index {i} must not be empty"),
            });
        }
    }

    // Check seriousness consistency: if any seriousness criterion is set, confirm
    // it's internally consistent.
    if let Some(s) = &req.seriousness {
        let has_fatal_reaction = req.reactions.iter().any(|r| r.outcome == "Fatal");
        if has_fatal_reaction && !s.death {
            issues.push(ValidationIssue {
                severity: "error".to_string(),
                field: "E.i.3".to_string(),
                message: "Reaction with Fatal outcome requires death seriousness criterion (E.i.3.2a) to be true".to_string(),
            });
        }
    }

    // ── Warnings (advisory) ──────────────────────────────────────────────

    if req.patient.is_none() {
        issues.push(ValidationIssue {
            severity: "warning".to_string(),
            field: "D".to_string(),
            message:
                "Patient information (Section D) not provided — recommended for E2B completeness"
                    .to_string(),
        });
    }

    if req.causality.is_empty() && !req.drugs.is_empty() && !req.reactions.is_empty() {
        issues.push(ValidationIssue {
            severity: "warning".to_string(),
            field: "G.k.9".to_string(),
            message:
                "No causality assessment (G.k.9) provided — recommended for regulatory submission"
                    .to_string(),
        });
    }

    if req.report.is_none() {
        issues.push(ValidationIssue {
            severity: "warning".to_string(),
            field: "A/C".to_string(),
            message:
                "Report metadata (Sections A/C) not provided — recommended for E2B completeness"
                    .to_string(),
        });
    }

    if req.seriousness.is_none() && !req.reactions.is_empty() {
        issues.push(ValidationIssue {
            severity: "warning".to_string(),
            field: "E.i.3".to_string(),
            message: "Seriousness criteria (E.i.3) not assessed — required for serious cases"
                .to_string(),
        });
    }

    // Check causality index bounds
    for (i, c) in req.causality.iter().enumerate() {
        if c.drug_index >= req.drugs.len() {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                field: format!("G.k.9[{i}].drug_index"),
                message: format!(
                    "Causality assessment {i} references drug index {} but only {} drugs provided",
                    c.drug_index,
                    req.drugs.len()
                ),
            });
        }
        if c.reaction_index >= req.reactions.len() {
            issues.push(ValidationIssue {
                severity: "warning".to_string(),
                field: format!("G.k.9[{i}].reaction_index"),
                message: format!(
                    "Causality assessment {i} references reaction index {} but only {} reactions provided",
                    c.reaction_index,
                    req.reactions.len()
                ),
            });
        }
    }

    let issue_count = issues.len();
    let valid = !issues.iter().any(|i| i.severity == "error");

    Ok(Json(IcsrValidateResponse {
        valid,
        issues,
        issue_count,
    }))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icsr_build_minimal() {
        let req = IcsrBuildRequest {
            case_id: "CASE-001".to_string(),
            patient: None,
            drugs: vec![DrugDto {
                name: "ASPIRIN".to_string(),
                role: "Suspect".to_string(),
                dosage: None,
                start_date: None,
                end_date: None,
                indication: None,
                action: "Unknown".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Gastrointestinal haemorrhage".to_string(),
                meddra_code: None,
                outcome: "Recovered".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![],
            report: None,
            seriousness: None,
        };
        let result = icsr_build(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert_eq!(resp.icsr.case_id, "CASE-001");
        assert!(!resp.is_serious);
        assert_eq!(resp.seriousness_criteria_count, 0);
    }

    #[tokio::test]
    async fn test_icsr_build_serious() {
        let req = IcsrBuildRequest {
            case_id: "CASE-002".to_string(),
            patient: Some(PatientDto {
                age: Some(65.0),
                sex: "Male".to_string(),
                weight_kg: Some(80.0),
                medical_history: vec!["Hypertension".to_string()],
            }),
            drugs: vec![DrugDto {
                name: "ROFECOXIB".to_string(),
                role: "Suspect".to_string(),
                dosage: Some(DosageDto {
                    value: 25.0,
                    unit: "mg".to_string(),
                    route: "Oral".to_string(),
                    frequency: Some("QD".to_string()),
                }),
                start_date: Some("2024-01-15".to_string()),
                end_date: Some("2024-03-01".to_string()),
                indication: Some("Osteoarthritis".to_string()),
                action: "Withdrawn".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Myocardial infarction".to_string(),
                meddra_code: Some(10028596),
                outcome: "Fatal".to_string(),
                onset_days: Some(45.0),
                duration_days: None,
            }],
            causality: vec![CausalityDto {
                drug_index: 0,
                reaction_index: 0,
                method: "Naranjo".to_string(),
                result: "Probable".to_string(),
                assessor: "Algorithm".to_string(),
            }],
            report: Some(ReportInfoDto {
                report_type: "Spontaneous".to_string(),
                source: "HealthcareProfessional".to_string(),
                country: Some("US".to_string()),
                receipt_date: Some("2024-03-02".to_string()),
                latest_date: None,
            }),
            seriousness: Some(SeriousnessDto {
                death: true,
                life_threatening: false,
                hospitalization: true,
                disability: false,
                congenital_anomaly: false,
                other_medically_important: false,
            }),
        };
        let result = icsr_build(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert!(resp.is_serious);
        assert_eq!(resp.seriousness_criteria_count, 2);
        assert_eq!(resp.icsr.drugs[0].name, "ROFECOXIB");
    }

    #[tokio::test]
    async fn test_icsr_build_empty_case_id() {
        let req = IcsrBuildRequest {
            case_id: "".to_string(),
            patient: None,
            drugs: vec![DrugDto {
                name: "X".to_string(),
                role: "Suspect".to_string(),
                dosage: None,
                start_date: None,
                end_date: None,
                indication: None,
                action: "Unknown".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Y".to_string(),
                meddra_code: None,
                outcome: "Unknown".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![],
            report: None,
            seriousness: None,
        };
        let result = icsr_build(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_icsr_build_no_drugs() {
        let req = IcsrBuildRequest {
            case_id: "CASE-X".to_string(),
            patient: None,
            drugs: vec![],
            reactions: vec![ReactionDto {
                term: "Y".to_string(),
                meddra_code: None,
                outcome: "Unknown".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![],
            report: None,
            seriousness: None,
        };
        let result = icsr_build(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_icsr_validate_valid() {
        let req = IcsrValidateRequest {
            case_id: "CASE-001".to_string(),
            patient: Some(PatientDto {
                age: Some(45.0),
                sex: "Female".to_string(),
                weight_kg: None,
                medical_history: vec![],
            }),
            drugs: vec![DrugDto {
                name: "IBUPROFEN".to_string(),
                role: "Suspect".to_string(),
                dosage: None,
                start_date: None,
                end_date: None,
                indication: None,
                action: "Unknown".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Rash".to_string(),
                meddra_code: None,
                outcome: "Recovered".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![CausalityDto {
                drug_index: 0,
                reaction_index: 0,
                method: "Naranjo".to_string(),
                result: "Possible".to_string(),
                assessor: "Reporter".to_string(),
            }],
            report: Some(ReportInfoDto {
                report_type: "Spontaneous".to_string(),
                source: "HealthcareProfessional".to_string(),
                country: Some("US".to_string()),
                receipt_date: None,
                latest_date: None,
            }),
            seriousness: Some(SeriousnessDto::default()),
        };
        let result = icsr_validate(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert!(resp.valid, "Expected valid ICSR, issues: {:?}", resp.issues);
    }

    #[tokio::test]
    async fn test_icsr_validate_empty_case_id() {
        let req = IcsrValidateRequest {
            case_id: "".to_string(),
            patient: None,
            drugs: vec![],
            reactions: vec![],
            causality: vec![],
            report: None,
            seriousness: None,
        };
        let result = icsr_validate(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert!(!resp.valid);
        assert!(resp.issues.iter().any(|i| i.field == "C.1.1"));
        assert!(resp.issues.iter().any(|i| i.field == "G.k"));
        assert!(resp.issues.iter().any(|i| i.field == "E.i"));
    }

    #[tokio::test]
    async fn test_icsr_validate_fatal_without_death_seriousness() {
        let req = IcsrValidateRequest {
            case_id: "CASE-003".to_string(),
            patient: None,
            drugs: vec![DrugDto {
                name: "DRUG-A".to_string(),
                role: "Suspect".to_string(),
                dosage: None,
                start_date: None,
                end_date: None,
                indication: None,
                action: "Unknown".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Cardiac arrest".to_string(),
                meddra_code: None,
                outcome: "Fatal".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![],
            report: None,
            seriousness: Some(SeriousnessDto {
                death: false,
                hospitalization: true,
                ..Default::default()
            }),
        };
        let result = icsr_validate(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert!(!resp.valid);
        assert!(
            resp.issues
                .iter()
                .any(|i| i.field == "E.i.3" && i.severity == "error"),
            "Expected E.i.3 error for fatal without death criterion"
        );
    }

    #[tokio::test]
    async fn test_icsr_validate_out_of_bounds_causality() {
        let req = IcsrValidateRequest {
            case_id: "CASE-004".to_string(),
            patient: None,
            drugs: vec![DrugDto {
                name: "DRUG-A".to_string(),
                role: "Suspect".to_string(),
                dosage: None,
                start_date: None,
                end_date: None,
                indication: None,
                action: "Unknown".to_string(),
            }],
            reactions: vec![ReactionDto {
                term: "Headache".to_string(),
                meddra_code: None,
                outcome: "Recovered".to_string(),
                onset_days: None,
                duration_days: None,
            }],
            causality: vec![CausalityDto {
                drug_index: 5, // out of bounds
                reaction_index: 0,
                method: "Naranjo".to_string(),
                result: "Possible".to_string(),
                assessor: "Algorithm".to_string(),
            }],
            report: None,
            seriousness: None,
        };
        let result = icsr_validate(Json(req)).await;
        assert!(result.is_ok());
        let resp = result.unwrap_or_else(|e| panic!("{}", e.message));
        assert!(
            resp.issues.iter().any(|i| i.field.contains("drug_index")),
            "Expected out-of-bounds drug_index warning"
        );
    }
}
