//! Pharmacovigilance signal detection endpoints

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

/// Contingency table for signal detection (2x2 table)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ContingencyTableRequest {
    /// Drug + Event count
    pub a: u64,
    /// Drug + No Event count
    pub b: u64,
    /// No Drug + Event count
    pub c: u64,
    /// No Drug + No Event count
    pub d: u64,
}

/// Complete signal detection response
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalCompleteResponse {
    /// PRR value
    pub prr: f64,
    /// PRR lower 95% CI
    pub prr_ci_lower: f64,
    /// PRR upper 95% CI
    pub prr_ci_upper: f64,
    /// PRR signal detected
    pub prr_signal: bool,
    /// ROR value
    pub ror: f64,
    /// ROR lower 95% CI
    pub ror_ci_lower: f64,
    /// ROR upper 95% CI
    pub ror_ci_upper: f64,
    /// ROR signal detected
    pub ror_signal: bool,
    /// IC value (Information Component)
    pub ic: f64,
    /// IC lower 95% CI
    pub ic_ci_lower: f64,
    /// IC signal detected
    pub ic_signal: bool,
    /// EBGM value
    pub ebgm: f64,
    /// EB05 (lower 5% bound)
    pub eb05: f64,
    /// EBGM signal detected
    pub ebgm_signal: bool,
    /// Chi-square statistic
    pub chi_square: f64,
    /// Overall signal detected (any method)
    pub signal_detected: bool,
}

/// Single signal metric response
#[derive(Debug, Serialize, ToSchema)]
pub struct SignalMetricResponse {
    /// Metric value
    pub value: f64,
    /// Lower confidence interval
    pub ci_lower: f64,
    /// Upper confidence interval
    pub ci_upper: f64,
    /// Signal detected
    pub signal: bool,
}

/// Naranjo causality assessment request
#[derive(Debug, Deserialize, ToSchema)]
pub struct NaranjoRequest {
    /// Temporal relationship (1=yes, 0=unknown, -1=no)
    pub temporal: i32,
    /// Improved after withdrawal (dechallenge)
    pub dechallenge: i32,
    /// Recurred on re-exposure (rechallenge)
    pub rechallenge: i32,
    /// Alternative causes ruled out (-1=yes causes exist, 1=no alternatives, 0=unknown)
    pub alternatives: i32,
    /// Previously reported in literature
    pub previous: i32,
}

/// Naranjo causality response
#[derive(Debug, Serialize, ToSchema)]
pub struct NaranjoResponse {
    /// Total score
    pub score: i32,
    /// Causality category
    pub category: String,
    /// Score interpretation
    pub interpretation: String,
}

/// WHO-UMC causality assessment request
#[derive(Debug, Deserialize, ToSchema)]
pub struct WhoUmcRequest {
    /// Temporal relationship plausible
    pub temporal_plausible: bool,
    /// Positive dechallenge
    pub dechallenge_positive: bool,
    /// Positive rechallenge
    pub rechallenge_positive: bool,
    /// Alternative explanations ruled out
    pub alternatives_ruled_out: bool,
    /// Pharmacologically plausible
    pub pharmacologically_plausible: bool,
}

/// WHO-UMC causality response
#[derive(Debug, Serialize, ToSchema)]
pub struct WhoUmcResponse {
    /// Causality category
    pub category: String,
    /// Category description
    pub description: String,
}

/// ICH E2A seriousness assessment request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SeriousnessRequest {
    /// Patient died
    #[serde(default)]
    pub death: bool,
    /// Life-threatening at time of event
    #[serde(default)]
    pub life_threatening: bool,
    /// Required hospitalization
    #[serde(default)]
    pub hospitalization: bool,
    /// Resulted in disability/incapacity
    #[serde(default)]
    pub disability: bool,
    /// Congenital anomaly/birth defect
    #[serde(default)]
    pub congenital_anomaly: bool,
    /// Other medically important condition
    #[serde(default)]
    pub other_medically_important: bool,
}

/// Seriousness assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct SeriousnessResponse {
    /// Whether the event is serious per ICH E2A
    pub is_serious: bool,
    /// Criteria that were met
    pub criteria_met: Vec<String>,
    /// Primary (most severe) criterion
    pub primary_criterion: Option<String>,
    /// Whether expedited reporting is required
    pub requires_expedited: bool,
    /// Reporting deadline
    pub reporting_deadline: Option<String>,
}

/// RSI-based expectedness assessment request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ExpectednessRequest {
    /// Adverse event term
    pub event_term: String,
    /// Product name
    pub product_name: String,
    /// Whether event is listed in RSI
    pub listed_in_rsi: bool,
    /// Whether this is a known class effect
    #[serde(default)]
    pub is_class_effect: bool,
}

/// Expectedness assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct ExpectednessResponse {
    /// Whether the event is expected
    pub is_expected: bool,
    /// Expectedness category
    pub category: String,
    /// Assessment confidence
    pub confidence: f64,
    /// Whether expedited reporting required (if also serious)
    pub requires_expedited_if_serious: bool,
}

/// Combined assessment request (causality + seriousness + expectedness)
#[derive(Debug, Deserialize, ToSchema)]
pub struct CombinedRequest {
    /// Temporal relationship (1=yes, 0=unknown, -1=no)
    pub temporal: i32,
    /// Dechallenge result
    pub dechallenge: i32,
    /// Rechallenge result
    pub rechallenge: i32,
    /// Alternative causes
    pub alternatives: i32,
    /// Biological plausibility
    pub plausibility: i32,
    /// Seriousness criteria
    pub seriousness: SeriousnessRequest,
    /// Expectedness criteria
    pub expectedness: ExpectednessRequest,
}

/// Combined assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct CombinedResponse {
    /// Causality category
    pub causality_category: String,
    /// Is serious
    pub is_serious: bool,
    /// Is expected
    pub is_expected: bool,
    /// Requires expedited reporting
    pub requires_expedited: bool,
    /// Reporting deadline
    pub deadline: String,
    /// Regulatory rationale
    pub rationale: String,
}

// --- RUCAM types ---

/// RUCAM (Roussel Uclaf Causality Assessment Method) request
///
/// Assesses drug-induced liver injury causality across 7 criteria.
/// Tier: T3 (→ Causality dominant, κ Comparison for scoring)
#[derive(Debug, Deserialize, ToSchema)]
pub struct RucamRequest {
    /// Days from drug start to reaction onset
    pub time_to_onset: u32,
    /// Type of liver reaction pattern
    pub reaction_type: RucamReactionType,
    /// Was drug withdrawn?
    pub drug_withdrawn: bool,
    /// Days from withdrawal to improvement (if withdrawn)
    #[serde(default)]
    pub time_to_improvement: Option<u32>,
    /// Percentage decrease in liver values after withdrawal
    #[serde(default)]
    pub percentage_decrease: Option<f64>,
    /// Patient age in years
    pub age: u32,
    /// Alcohol use
    #[serde(default)]
    pub alcohol: bool,
    /// Pregnancy
    #[serde(default)]
    pub pregnancy: bool,
    /// Concomitant hepatotoxic drug information
    #[serde(default)]
    pub concomitant_drugs: RucamConcomitantDrugs,
    /// Alternative causes investigation
    #[serde(default)]
    pub alternative_causes: RucamAlternativeCauses,
    /// Previous hepatotoxicity information
    #[serde(default)]
    pub previous_hepatotoxicity: RucamPreviousHepatotoxicity,
    /// Was rechallenge performed?
    #[serde(default)]
    pub rechallenge_performed: bool,
    /// Result of rechallenge
    #[serde(default)]
    pub rechallenge_result: Option<RucamRechallengeResult>,
}

/// Liver reaction type
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum RucamReactionType {
    /// ALT-predominant
    Hepatocellular,
    /// ALP-predominant
    Cholestatic,
    /// Both elevated
    Mixed,
}

/// Concomitant drug details
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RucamConcomitantDrugs {
    /// Count of known hepatotoxic drugs
    #[serde(default)]
    pub hepatotoxic_count: u32,
    /// Known drug-drug interactions
    #[serde(default)]
    pub interactions: bool,
}

/// Serology result for alternative cause investigation
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub enum RucamSerologyResult {
    /// Test result was positive
    Positive,
    /// Test result was negative
    Negative,
    /// Test was not performed
    #[default]
    NotDone,
}

/// Yes/No/NotApplicable for alternative causes
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub enum RucamYesNoNa {
    /// Present
    Yes,
    /// Absent
    No,
    /// Not applicable
    #[default]
    NotApplicable,
}

/// Alternative causes investigation (non-drug causes of liver injury)
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RucamAlternativeCauses {
    /// Hepatitis A virus serology
    #[serde(default)]
    pub hepatitis_a: RucamSerologyResult,
    /// Hepatitis B virus serology
    #[serde(default)]
    pub hepatitis_b: RucamSerologyResult,
    /// Hepatitis C virus serology
    #[serde(default)]
    pub hepatitis_c: RucamSerologyResult,
    /// CMV or EBV serology
    #[serde(default)]
    pub cmv_ebv: RucamSerologyResult,
    /// Biliary ultrasound/sonography
    #[serde(default)]
    pub biliary_sonography: RucamSerologyResult,
    /// History of alcohol abuse
    #[serde(default)]
    pub alcoholism: RucamYesNoNa,
    /// Pre-existing liver complications
    #[serde(default)]
    pub underlying_complications: RucamYesNoNa,
}

/// Previous hepatotoxicity information
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RucamPreviousHepatotoxicity {
    /// Drug is labeled as hepatotoxic
    #[serde(default)]
    pub labeled_hepatotoxic: bool,
    /// Published case reports exist
    #[serde(default)]
    pub published_reports: bool,
    /// This specific reaction type is documented
    #[serde(default)]
    pub reaction_known: bool,
}

/// Rechallenge result
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum RucamRechallengeResult {
    /// Reaction recurred
    Positive,
    /// No reaction
    Negative,
    /// Inconclusive
    NotConclusive,
}

/// RUCAM score breakdown by assessment area
#[derive(Debug, Serialize, ToSchema)]
pub struct RucamBreakdownResponse {
    /// Score for temporal relationship
    pub temporal_relationship: i32,
    /// Score for course of reaction (dechallenge)
    pub course_of_reaction: i32,
    /// Score for risk factors
    pub risk_factors: i32,
    /// Score for concomitant drugs
    pub concomitant_drugs: i32,
    /// Score for alternative causes exclusion
    pub alternative_causes: i32,
    /// Score for previous hepatotoxicity info
    pub previous_information: i32,
    /// Score for rechallenge
    pub rechallenge: i32,
}

/// RUCAM causality assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct RucamResponse {
    /// Total RUCAM score (-4 to +14)
    pub total_score: i32,
    /// Causality category
    pub category: String,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
    /// Score breakdown by criterion
    pub breakdown: RucamBreakdownResponse,
}

// --- Full Naranjo types ---

/// Full Naranjo causality assessment request (10 questions)
///
/// Original 10-question Naranjo scale. Each field scored as the weight
/// for that question (positive=yes, 0=unknown, negative=no).
/// Tier: T3 (→ Causality dominant, κ Comparison for threshold)
#[derive(Debug, Deserialize, ToSchema)]
pub struct NaranjoFullRequest {
    /// Q1: Are there previous conclusive reports? (+1 yes, 0 unknown, 0 no)
    pub previous_reports: i8,
    /// Q2: Did the event appear after drug administration? (+2 yes, 0 unknown, -1 no)
    pub after_drug: i8,
    /// Q3: Did it improve on withdrawal (dechallenge)? (+1 yes, 0 unknown, 0 no)
    pub improved_on_dechallenge: i8,
    /// Q4: Did it recur on re-exposure (rechallenge)? (+2 yes, 0 unknown, -1 no)
    pub recurred_on_rechallenge: i8,
    /// Q5: Are there alternative causes? (-1 yes, 0 unknown, +2 no)
    pub alternative_causes: i8,
    /// Q6: Did reaction appear on placebo? (-1 yes, 0 unknown, +1 no)
    pub reaction_on_placebo: i8,
    /// Q7: Was drug detected in blood/fluids? (+1 yes, 0 unknown, 0 no)
    pub detected_in_fluids: i8,
    /// Q8: Was there dose-response? (+1 yes, 0 unknown, 0 no)
    pub dose_response: i8,
    /// Q9: Similar reaction in previous exposure? (+1 yes, 0 unknown, 0 no)
    pub previous_similar_reaction: i8,
    /// Q10: Was there objective evidence? (+1 yes, 0 unknown, 0 no)
    pub objective_evidence: i8,
}

/// Full Naranjo causality response
#[derive(Debug, Serialize, ToSchema)]
pub struct NaranjoFullResponse {
    /// Total score (-4 to +13)
    pub score: i32,
    /// Causality category
    pub category: String,
    /// Score interpretation
    pub interpretation: String,
    /// Individual question scores
    pub question_scores: Vec<i32>,
}

// --- Full WHO-UMC types ---

/// Full WHO-UMC causality assessment request
///
/// Comprehensive WHO-UMC assessment with all evidence dimensions.
/// Tier: T3 (→ Causality dominant, κ Comparison for category)
#[derive(Debug, Deserialize, ToSchema)]
pub struct WhoUmcFullRequest {
    /// Temporal relationship exists
    pub has_temporal_relationship: bool,
    /// Strength of temporal relationship
    #[serde(default)]
    pub temporal_strength: WhoUmcTemporalStrengthDto,
    /// Was dechallenge performed?
    #[serde(default)]
    pub dechallenge_performed: bool,
    /// Result of dechallenge
    #[serde(default)]
    pub dechallenge_result: Option<ChallengeResultDto>,
    /// Was rechallenge performed?
    #[serde(default)]
    pub rechallenge_performed: bool,
    /// Result of rechallenge
    #[serde(default)]
    pub rechallenge_result: Option<ChallengeResultDto>,
    /// Alternative causes present
    #[serde(default)]
    pub alternative_causes_present: bool,
    /// Likelihood of alternatives
    #[serde(default)]
    pub alternatives_likelihood: AlternativesLikelihoodDto,
    /// Is the relationship biologically plausible?
    #[serde(default)]
    pub biologically_plausible: bool,
    /// Strength of biological plausibility
    #[serde(default)]
    pub plausibility_strength: PlausibilityStrengthDto,
    /// Previously reported in literature
    #[serde(default)]
    pub previously_reported: bool,
    /// Known adverse reaction for this drug
    #[serde(default)]
    pub known_adverse_reaction: bool,
    /// Data is complete
    #[serde(default = "default_true")]
    pub data_complete: bool,
    /// Data is sufficient for assessment
    #[serde(default = "default_true")]
    pub data_sufficient: bool,
}

fn default_true() -> bool {
    true
}

/// Temporal relationship strength
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub enum WhoUmcTemporalStrengthDto {
    /// Strong temporal relationship
    Strong,
    /// Moderate temporal relationship
    #[default]
    Moderate,
    /// Weak temporal relationship
    Weak,
    /// No temporal relationship
    None,
}

/// Challenge (dechallenge/rechallenge) result
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum ChallengeResultDto {
    /// Event resolved/recurred as expected
    Positive,
    /// Event did not resolve/recur
    Negative,
    /// Inconclusive
    Inconclusive,
}

/// Likelihood of alternative causes
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub enum AlternativesLikelihoodDto {
    /// No alternatives identified
    #[default]
    None,
    /// Possible alternatives
    Possible,
    /// Probable alternatives
    Probable,
    /// Certain alternatives
    Certain,
}

/// Biological plausibility strength
#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
pub enum PlausibilityStrengthDto {
    /// High plausibility
    High,
    /// Moderate plausibility
    #[default]
    Moderate,
    /// Low plausibility
    Low,
    /// Unknown plausibility
    Unknown,
}

/// Full WHO-UMC criteria flags response
#[derive(Debug, Serialize, ToSchema)]
pub struct WhoUmcCriteriaResponse {
    /// Temporal relationship met
    pub temporal_relationship: bool,
    /// Dechallenge criterion met
    pub dechallenge: bool,
    /// Rechallenge criterion met
    pub rechallenge: bool,
    /// Alternative causes ruled out
    pub alternative_causes: bool,
    /// Biological plausibility met
    pub biological_plausibility: bool,
}

/// Full WHO-UMC causality response
#[derive(Debug, Serialize, ToSchema)]
pub struct WhoUmcFullResponse {
    /// Causality category
    pub category: String,
    /// Internal numeric score
    pub score: i32,
    /// Human-readable rationale
    pub rationale: String,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
    /// Individual criteria met
    pub criteria: WhoUmcCriteriaResponse,
}

/// PV router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/signal/complete", post(signal_complete))
        .route("/signal/prr", post(signal_prr))
        .route("/signal/ror", post(signal_ror))
        .route("/signal/ic", post(signal_ic))
        .route("/signal/ebgm", post(signal_ebgm))
        .route("/chi-square", post(chi_square))
        .route("/naranjo", post(naranjo))
        .route("/naranjo/full", post(naranjo_full))
        .route("/who-umc", post(who_umc))
        .route("/who-umc/full", post(who_umc_full))
        .route("/rucam", post(rucam))
        .route("/seriousness", post(seriousness))
        .route("/expectedness", post(expectedness))
        .route("/combined", post(combined))
        .route("/ucas", post(ucas))
}

// --- UCAS types ---

/// UCAS criterion response value
///
/// Serialises as `"yes"`, `"no"`, or `"unknown"` to match the domain type.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UcasCriterionDto {
    /// Criterion is met
    Yes,
    /// Criterion is not met
    No,
    /// Insufficient information to determine
    Unknown,
}

/// Universal Causality Assessment Scale (UCAS) request — ToV §36
///
/// Eight domain-agnostic criteria adapted from WHO-UMC and Naranjo.
/// Each field accepts `"yes"`, `"no"`, or `"unknown"`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UcasRequest {
    /// Temporal relationship: harm occurred after exposure with plausible latency
    pub temporal: UcasCriterionDto,
    /// Dechallenge: harm improved when intervention was removed
    pub dechallenge: UcasCriterionDto,
    /// Rechallenge: harm recurred when intervention was reintroduced
    pub rechallenge: UcasCriterionDto,
    /// Mechanistic plausibility: a known biological mechanism exists
    pub mechanistic_plausibility: UcasCriterionDto,
    /// Alternative explanations: other plausible causes are present
    pub alternative_explanations: UcasCriterionDto,
    /// Dose-response: a relationship between dose intensity and severity exists
    pub dose_response: UcasCriterionDto,
    /// Prior evidence: this association has been previously reported
    pub prior_evidence: UcasCriterionDto,
    /// Specificity: harm is characteristic of this intervention class
    pub specificity: UcasCriterionDto,
}

/// Per-criterion score breakdown entry
#[derive(Debug, Serialize, ToSchema)]
pub struct UcasBreakdownEntry {
    /// Criterion name
    pub name: String,
    /// Response given
    pub response: String,
    /// Points awarded for this criterion
    pub score: i32,
    /// Maximum points possible for this criterion
    pub max_points: i32,
}

/// UCAS causality assessment response
#[derive(Debug, Serialize, ToSchema)]
pub struct UcasResponse {
    /// Total UCAS score (-3 to +14)
    pub score: i32,
    /// Causality category (Certain, Probable, Possible, Unlikely, Unassessable)
    pub category: String,
    /// Recognition component R for ToV signal equation S = U × R × T
    pub recognition_r: f64,
    /// Per-criterion score breakdown (8 entries)
    pub breakdown: Vec<UcasBreakdownEntry>,
    /// Assessment confidence (0.0–1.0)
    pub confidence: f64,
}

/// Calculate complete signal detection (PRR, ROR, IC, EBGM, Chi-square)
#[utoipa::path(
    post,
    path = "/api/v1/pv/signal/complete",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "Signal analysis complete", body = SignalCompleteResponse),
        (status = 400, description = "Invalid table values", body = super::common::ApiError)
    )
)]
pub async fn signal_complete(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalCompleteResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{
        ContingencyTable, SignalCriteria, calculate_ebgm, calculate_ic, calculate_prr,
        calculate_ror,
    };

    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let criteria = SignalCriteria::evans();

    let prr = calculate_prr(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;
    let ror = calculate_ror(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;
    let ic = calculate_ic(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;
    let ebgm = calculate_ebgm(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    let signal_detected = prr.is_signal || ror.is_signal || ic.is_signal || ebgm.is_signal;

    Ok(Json(SignalCompleteResponse {
        prr: prr.point_estimate,
        prr_ci_lower: prr.lower_ci,
        prr_ci_upper: prr.upper_ci,
        prr_signal: prr.is_signal,
        ror: ror.point_estimate,
        ror_ci_lower: ror.lower_ci,
        ror_ci_upper: ror.upper_ci,
        ror_signal: ror.is_signal,
        ic: ic.point_estimate,
        ic_ci_lower: ic.lower_ci,
        ic_signal: ic.is_signal,
        ebgm: ebgm.point_estimate,
        eb05: ebgm.lower_ci,
        ebgm_signal: ebgm.is_signal,
        chi_square: prr.chi_square.unwrap_or(0.0),
        signal_detected,
    }))
}

/// Calculate Proportional Reporting Ratio (PRR)
#[utoipa::path(
    post,
    path = "/api/v1/pv/signal/prr",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "PRR calculated", body = SignalMetricResponse)
    )
)]
pub async fn signal_prr(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalMetricResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, calculate_prr};

    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let result = calculate_prr(&table, &SignalCriteria::evans())
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    Ok(Json(SignalMetricResponse {
        value: result.point_estimate,
        ci_lower: result.lower_ci,
        ci_upper: result.upper_ci,
        signal: result.is_signal,
    }))
}

/// Calculate Reporting Odds Ratio (ROR)
#[utoipa::path(
    post,
    path = "/api/v1/pv/signal/ror",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "ROR calculated", body = SignalMetricResponse)
    )
)]
pub async fn signal_ror(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalMetricResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, calculate_ror};

    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let result = calculate_ror(&table, &SignalCriteria::evans())
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    Ok(Json(SignalMetricResponse {
        value: result.point_estimate,
        ci_lower: result.lower_ci,
        ci_upper: result.upper_ci,
        signal: result.is_signal,
    }))
}

/// Calculate Information Component (IC)
#[utoipa::path(
    post,
    path = "/api/v1/pv/signal/ic",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "IC calculated", body = SignalMetricResponse)
    )
)]
pub async fn signal_ic(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalMetricResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, calculate_ic};

    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let result = calculate_ic(&table, &SignalCriteria::evans())
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    Ok(Json(SignalMetricResponse {
        value: result.point_estimate,
        ci_lower: result.lower_ci,
        ci_upper: result.upper_ci,
        signal: result.is_signal,
    }))
}

/// Calculate Empirical Bayes Geometric Mean (EBGM)
#[utoipa::path(
    post,
    path = "/api/v1/pv/signal/ebgm",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "EBGM calculated", body = SignalMetricResponse)
    )
)]
pub async fn signal_ebgm(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalMetricResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, calculate_ebgm};

    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let result = calculate_ebgm(&table, &SignalCriteria::evans())
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    Ok(Json(SignalMetricResponse {
        value: result.point_estimate,
        ci_lower: result.lower_ci,
        ci_upper: result.upper_ci,
        signal: result.is_signal,
    }))
}

/// Calculate Chi-square test statistic
#[utoipa::path(
    post,
    path = "/api/v1/pv/chi-square",
    tag = "pv",
    request_body = ContingencyTableRequest,
    responses(
        (status = 200, description = "Chi-square calculated", body = SignalMetricResponse)
    )
)]
pub async fn chi_square(
    Json(req): Json<ContingencyTableRequest>,
) -> Result<Json<SignalMetricResponse>, ApiError> {
    use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria, calculate_prr};

    // Chi-square is included in PRR result
    let table = ContingencyTable::new(req.a, req.b, req.c, req.d);
    let result = calculate_prr(&table, &SignalCriteria::evans())
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    let chi_sq = result.chi_square.unwrap_or(0.0);

    Ok(Json(SignalMetricResponse {
        value: chi_sq,
        ci_lower: 0.0,
        ci_upper: chi_sq,
        signal: chi_sq >= 3.841, // p < 0.05 threshold
    }))
}

/// Naranjo causality assessment (5-question quick version)
#[utoipa::path(
    post,
    path = "/api/v1/pv/naranjo",
    tag = "pv",
    request_body = NaranjoRequest,
    responses(
        (status = 200, description = "Causality assessed", body = NaranjoResponse)
    )
)]
pub async fn naranjo(Json(req): Json<NaranjoRequest>) -> ApiResult<NaranjoResponse> {
    let result = nexcore_vigilance::pv::causality::calculate_naranjo_quick(
        req.temporal,
        req.dechallenge,
        req.rechallenge,
        req.alternatives,
        req.previous,
    );

    // Convert category enum to string
    let category = format!("{:?}", result.category);
    let interpretation = match result.score {
        9..=13 => "Definite causality",
        5..=8 => "Probable causality",
        1..=4 => "Possible causality",
        _ => "Doubtful causality",
    };

    Ok(Json(NaranjoResponse {
        score: result.score,
        category,
        interpretation: interpretation.to_string(),
    }))
}

/// WHO-UMC causality assessment (5-question quick version)
#[utoipa::path(
    post,
    path = "/api/v1/pv/who-umc",
    tag = "pv",
    request_body = WhoUmcRequest,
    responses(
        (status = 200, description = "Causality assessed", body = WhoUmcResponse)
    )
)]
pub async fn who_umc(Json(req): Json<WhoUmcRequest>) -> ApiResult<WhoUmcResponse> {
    // Convert bools to i32 for the API
    let temporal = if req.temporal_plausible { 1 } else { 0 };
    let dechallenge = if req.dechallenge_positive { 1 } else { 0 };
    let rechallenge = if req.rechallenge_positive { 1 } else { 0 };
    let alternatives = if req.alternatives_ruled_out { 1 } else { 0 };
    let plausibility = if req.pharmacologically_plausible {
        1
    } else {
        0
    };

    let result = nexcore_vigilance::pv::causality::calculate_who_umc_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        plausibility,
    );

    Ok(Json(WhoUmcResponse {
        category: format!("{:?}", result.category),
        description: result.description.clone(),
    }))
}

/// ICH E2A seriousness assessment
#[utoipa::path(
    post,
    path = "/api/v1/pv/seriousness",
    tag = "pv",
    request_body = SeriousnessRequest,
    responses(
        (status = 200, description = "Seriousness assessed", body = SeriousnessResponse)
    )
)]
pub async fn seriousness(Json(req): Json<SeriousnessRequest>) -> ApiResult<SeriousnessResponse> {
    use nexcore_vigilance::pv::classification::{SeriousnessInput, assess_seriousness};

    let input = SeriousnessInput {
        death: req.death,
        life_threatening: req.life_threatening,
        hospitalization: req.hospitalization,
        hospitalization_type: None,
        disability: req.disability,
        congenital_anomaly: req.congenital_anomaly,
        other_medically_important: req.other_medically_important,
        medical_justification: None,
        required_intervention: false,
    };

    let result = assess_seriousness(&input);

    Ok(Json(SeriousnessResponse {
        is_serious: result.is_serious,
        criteria_met: result.criteria_met.iter().map(|c| c.to_string()).collect(),
        primary_criterion: result.primary_criterion.map(|c| c.to_string()),
        requires_expedited: result.regulatory_impact.requires_expedited,
        reporting_deadline: result.regulatory_impact.reporting_deadline,
    }))
}

/// RSI-based expectedness assessment
#[utoipa::path(
    post,
    path = "/api/v1/pv/expectedness",
    tag = "pv",
    request_body = ExpectednessRequest,
    responses(
        (status = 200, description = "Expectedness assessed", body = ExpectednessResponse)
    )
)]
pub async fn expectedness(Json(req): Json<ExpectednessRequest>) -> ApiResult<ExpectednessResponse> {
    use nexcore_vigilance::pv::classification::{ExpectednessInput, assess_expectedness};

    let input = ExpectednessInput {
        event_term: req.event_term,
        product_name: req.product_name,
        listed_in_rsi: req.listed_in_rsi,
        is_class_effect: req.is_class_effect,
        ..Default::default()
    };

    let result = assess_expectedness(&input);

    Ok(Json(ExpectednessResponse {
        is_expected: result.is_expected,
        category: format!("{}", result.category),
        confidence: result.confidence,
        requires_expedited_if_serious: result.regulatory_impact.requires_expedited,
    }))
}

/// Combined assessment: causality + seriousness + expectedness → regulatory recommendation
#[utoipa::path(
    post,
    path = "/api/v1/pv/combined",
    tag = "pv",
    request_body = CombinedRequest,
    responses(
        (status = 200, description = "Combined assessment complete", body = CombinedResponse)
    )
)]
pub async fn combined(Json(req): Json<CombinedRequest>) -> ApiResult<CombinedResponse> {
    use nexcore_vigilance::pv::classification::{
        CombinedAssessmentInput, ExpectednessInput, SeriousnessInput, assess_combined,
    };

    let input = CombinedAssessmentInput {
        temporal: req.temporal,
        dechallenge: req.dechallenge,
        rechallenge: req.rechallenge,
        alternatives: req.alternatives,
        plausibility: req.plausibility,
        seriousness: SeriousnessInput {
            death: req.seriousness.death,
            life_threatening: req.seriousness.life_threatening,
            hospitalization: req.seriousness.hospitalization,
            hospitalization_type: None,
            disability: req.seriousness.disability,
            congenital_anomaly: req.seriousness.congenital_anomaly,
            other_medically_important: req.seriousness.other_medically_important,
            medical_justification: None,
            required_intervention: false,
        },
        expectedness: ExpectednessInput {
            event_term: req.expectedness.event_term,
            product_name: req.expectedness.product_name,
            listed_in_rsi: req.expectedness.listed_in_rsi,
            is_class_effect: req.expectedness.is_class_effect,
            ..Default::default()
        },
    };

    let result = assess_combined(&input);

    Ok(Json(CombinedResponse {
        causality_category: format!("{:?}", result.causality.category),
        is_serious: result.seriousness.is_serious,
        is_expected: result.expectedness.is_expected,
        requires_expedited: result.requires_expedited,
        deadline: result.deadline.to_string(),
        rationale: result.rationale,
    }))
}

/// RUCAM hepatotoxicity causality assessment
///
/// Assesses drug-induced liver injury (DILI) causality using 7 criteria:
/// temporal relationship, course of reaction, risk factors, concomitant drugs,
/// alternative causes, previous hepatotoxicity info, and rechallenge.
#[utoipa::path(
    post,
    path = "/api/v1/pv/rucam",
    tag = "pv",
    request_body = RucamRequest,
    responses(
        (status = 200, description = "RUCAM causality assessed", body = RucamResponse),
        (status = 400, description = "Invalid input", body = super::common::ApiError)
    )
)]
pub async fn rucam(Json(req): Json<RucamRequest>) -> ApiResult<RucamResponse> {
    use nexcore_pv_core::causality::{
        AlternativeCauses, ConcomitantDrugs, PreviousHepatotoxicity, ReactionType,
        RechallengeResult, RucamInput, SerologyResult, YesNoNa, calculate_rucam,
    };

    // Map DTO enums to domain enums
    let reaction_type = match req.reaction_type {
        RucamReactionType::Hepatocellular => ReactionType::Hepatocellular,
        RucamReactionType::Cholestatic => ReactionType::Cholestatic,
        RucamReactionType::Mixed => ReactionType::Mixed,
    };

    let rechallenge_result = req.rechallenge_result.map(|r| match r {
        RucamRechallengeResult::Positive => RechallengeResult::Positive,
        RucamRechallengeResult::Negative => RechallengeResult::Negative,
        RucamRechallengeResult::NotConclusive => RechallengeResult::NotConclusive,
    });

    let map_serology = |s: &RucamSerologyResult| match s {
        RucamSerologyResult::Positive => SerologyResult::Positive,
        RucamSerologyResult::Negative => SerologyResult::Negative,
        RucamSerologyResult::NotDone => SerologyResult::NotDone,
    };

    let map_yes_no = |y: &RucamYesNoNa| match y {
        RucamYesNoNa::Yes => YesNoNa::Yes,
        RucamYesNoNa::No => YesNoNa::No,
        RucamYesNoNa::NotApplicable => YesNoNa::NotApplicable,
    };

    let input = RucamInput {
        time_to_onset: req.time_to_onset,
        reaction_type,
        drug_withdrawn: req.drug_withdrawn,
        time_to_improvement: req.time_to_improvement,
        percentage_decrease: req.percentage_decrease,
        age: req.age,
        alcohol: req.alcohol,
        pregnancy: req.pregnancy,
        concomitant_drugs: ConcomitantDrugs {
            hepatotoxic_count: req.concomitant_drugs.hepatotoxic_count,
            interactions: req.concomitant_drugs.interactions,
        },
        alternative_causes: AlternativeCauses {
            hepatitis_a: map_serology(&req.alternative_causes.hepatitis_a),
            hepatitis_b: map_serology(&req.alternative_causes.hepatitis_b),
            hepatitis_c: map_serology(&req.alternative_causes.hepatitis_c),
            cmv_ebv: map_serology(&req.alternative_causes.cmv_ebv),
            biliary_sonography: map_serology(&req.alternative_causes.biliary_sonography),
            alcoholism: map_yes_no(&req.alternative_causes.alcoholism),
            underlying_complications: map_yes_no(&req.alternative_causes.underlying_complications),
        },
        previous_hepatotoxicity: PreviousHepatotoxicity {
            labeled_hepatotoxic: req.previous_hepatotoxicity.labeled_hepatotoxic,
            published_reports: req.previous_hepatotoxicity.published_reports,
            reaction_known: req.previous_hepatotoxicity.reaction_known,
        },
        rechallenge_performed: req.rechallenge_performed,
        rechallenge_result,
    };

    let result = calculate_rucam(&input);

    Ok(Json(RucamResponse {
        total_score: result.total_score,
        category: result.category.to_string(),
        confidence: result.confidence,
        breakdown: RucamBreakdownResponse {
            temporal_relationship: result.breakdown.temporal_relationship,
            course_of_reaction: result.breakdown.course_of_reaction,
            risk_factors: result.breakdown.risk_factors,
            concomitant_drugs: result.breakdown.concomitant_drugs,
            alternative_causes: result.breakdown.alternative_causes,
            previous_information: result.breakdown.previous_information,
            rechallenge: result.breakdown.rechallenge,
        },
    }))
}

/// Full Naranjo causality assessment (10-question original scale)
///
/// The complete 10-question Naranjo ADR Probability Scale as published in
/// Clin Pharmacol Ther 1981;30:239-45.
#[utoipa::path(
    post,
    path = "/api/v1/pv/naranjo/full",
    tag = "pv",
    request_body = NaranjoFullRequest,
    responses(
        (status = 200, description = "Full Naranjo assessment complete", body = NaranjoFullResponse)
    )
)]
pub async fn naranjo_full(Json(req): Json<NaranjoFullRequest>) -> ApiResult<NaranjoFullResponse> {
    use nexcore_pv_core::causality::{NaranjoInput, calculate_naranjo};

    let input = NaranjoInput {
        previous_reports: req.previous_reports,
        after_drug: req.after_drug,
        improved_on_dechallenge: req.improved_on_dechallenge,
        recurred_on_rechallenge: req.recurred_on_rechallenge,
        alternative_causes: req.alternative_causes,
        reaction_on_placebo: req.reaction_on_placebo,
        detected_in_fluids: req.detected_in_fluids,
        dose_response: req.dose_response,
        previous_similar_reaction: req.previous_similar_reaction,
        objective_evidence: req.objective_evidence,
    };

    let result = calculate_naranjo(&input);

    let interpretation = match result.score {
        9..=13 => "Definite causality",
        5..=8 => "Probable causality",
        1..=4 => "Possible causality",
        _ => "Doubtful causality",
    };

    Ok(Json(NaranjoFullResponse {
        score: result.score,
        category: format!("{}", result.category),
        interpretation: interpretation.to_string(),
        question_scores: result.question_scores,
    }))
}

/// Universal Causality Assessment Scale (UCAS) — ToV §36
///
/// Domain-agnostic causality assessment across 8 evidence dimensions.
/// Scores range from -3 (all negative) to +14 (all positive).  The
/// `recognition_r` field integrates directly with the ToV signal
/// equation S = U × R × T via a sigmoid function.
///
/// Scoring table:
///
/// | Criterion           | Yes | Unknown | No  |
/// |---------------------|-----|---------|-----|
/// | Temporal            | +2  | 0       | -1  |
/// | Dechallenge         | +2  | 0       | 0   |
/// | Rechallenge         | +3  | 0       | 0   |
/// | Mechanism           | +2  | 0       | 0   |
/// | Alternatives (inv.) | -2  | 0       | +1  |
/// | Dose-Response       | +2  | 0       | 0   |
/// | Prior Evidence      | +1  | 0       | 0   |
/// | Specificity         | +1  | 0       | 0   |
#[utoipa::path(
    post,
    path = "/api/v1/pv/ucas",
    tag = "pv",
    request_body = UcasRequest,
    responses(
        (status = 200, description = "UCAS causality assessed", body = UcasResponse),
        (status = 400, description = "Invalid input", body = super::common::ApiError)
    )
)]
pub async fn ucas(Json(req): Json<UcasRequest>) -> ApiResult<UcasResponse> {
    use nexcore_pv_core::causality::{CriterionResponse, UcasInput, calculate_ucas};

    let map = |dto: &UcasCriterionDto| -> CriterionResponse {
        match dto {
            UcasCriterionDto::Yes => CriterionResponse::Yes,
            UcasCriterionDto::No => CriterionResponse::No,
            UcasCriterionDto::Unknown => CriterionResponse::Unknown,
        }
    };

    let input = UcasInput {
        temporal_relationship: map(&req.temporal),
        dechallenge: map(&req.dechallenge),
        rechallenge: map(&req.rechallenge),
        mechanistic_plausibility: map(&req.mechanistic_plausibility),
        alternative_explanations: map(&req.alternative_explanations),
        dose_response: map(&req.dose_response),
        prior_evidence: map(&req.prior_evidence),
        specificity: map(&req.specificity),
    };

    let result = calculate_ucas(&input);

    let breakdown = result
        .breakdown
        .iter()
        .map(|b| UcasBreakdownEntry {
            name: b.name.clone(),
            response: format!("{:?}", b.response).to_lowercase(),
            score: b.score.value(),
            max_points: b.max_points,
        })
        .collect();

    Ok(Json(UcasResponse {
        score: result.score.value(),
        category: result.category.to_string(),
        recognition_r: result.recognition_r,
        breakdown,
        confidence: result.confidence,
    }))
}

/// Full WHO-UMC causality assessment
///
/// Comprehensive WHO-Uppsala Monitoring Centre assessment with all evidence
/// dimensions: temporal, dechallenge, rechallenge, alternatives, plausibility.
#[utoipa::path(
    post,
    path = "/api/v1/pv/who-umc/full",
    tag = "pv",
    request_body = WhoUmcFullRequest,
    responses(
        (status = 200, description = "Full WHO-UMC assessment complete", body = WhoUmcFullResponse)
    )
)]
pub async fn who_umc_full(Json(req): Json<WhoUmcFullRequest>) -> ApiResult<WhoUmcFullResponse> {
    use nexcore_pv_core::causality::who_umc::{
        AlternativesLikelihood, ChallengeResult, PlausibilityStrength, WhoUmcInput,
        WhoUmcTemporalStrength, assess_who_umc_full as assess_full,
    };

    let temporal_strength = match req.temporal_strength {
        WhoUmcTemporalStrengthDto::Strong => WhoUmcTemporalStrength::Strong,
        WhoUmcTemporalStrengthDto::Moderate => WhoUmcTemporalStrength::Moderate,
        WhoUmcTemporalStrengthDto::Weak => WhoUmcTemporalStrength::Weak,
        WhoUmcTemporalStrengthDto::None => WhoUmcTemporalStrength::None,
    };

    let map_challenge = |c: ChallengeResultDto| match c {
        ChallengeResultDto::Positive => ChallengeResult::Positive,
        ChallengeResultDto::Negative => ChallengeResult::Negative,
        ChallengeResultDto::Inconclusive => ChallengeResult::Inconclusive,
    };

    let alternatives_likelihood = match req.alternatives_likelihood {
        AlternativesLikelihoodDto::None => AlternativesLikelihood::None,
        AlternativesLikelihoodDto::Possible => AlternativesLikelihood::Possible,
        AlternativesLikelihoodDto::Probable => AlternativesLikelihood::Probable,
        AlternativesLikelihoodDto::Certain => AlternativesLikelihood::Certain,
    };

    let plausibility_strength = match req.plausibility_strength {
        PlausibilityStrengthDto::High => PlausibilityStrength::High,
        PlausibilityStrengthDto::Moderate => PlausibilityStrength::Moderate,
        PlausibilityStrengthDto::Low => PlausibilityStrength::Low,
        PlausibilityStrengthDto::Unknown => PlausibilityStrength::Unknown,
    };

    let input = WhoUmcInput {
        has_temporal_relationship: req.has_temporal_relationship,
        temporal_strength,
        dechallenge_performed: req.dechallenge_performed,
        dechallenge_result: req.dechallenge_result.map(map_challenge),
        rechallenge_performed: req.rechallenge_performed,
        rechallenge_result: req.rechallenge_result.map(map_challenge),
        alternative_causes_present: req.alternative_causes_present,
        alternatives_likelihood,
        biologically_plausible: req.biologically_plausible,
        plausibility_strength,
        previously_reported: req.previously_reported,
        known_adverse_reaction: req.known_adverse_reaction,
        data_complete: req.data_complete,
        data_sufficient: req.data_sufficient,
    };

    let result = assess_full(&input);

    Ok(Json(WhoUmcFullResponse {
        category: result.category.to_string(),
        score: result.score,
        rationale: result.rationale,
        confidence: result.confidence,
        criteria: WhoUmcCriteriaResponse {
            temporal_relationship: result.criteria.temporal_relationship,
            dechallenge: result.criteria.dechallenge,
            rechallenge: result.criteria.rechallenge,
            alternative_causes: result.criteria.alternative_causes,
            biological_plausibility: result.criteria.biological_plausibility,
        },
    }))
}
