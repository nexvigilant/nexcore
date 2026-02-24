//! Benefit-Risk Assessment endpoints
//!
//! Two complementary quantifiers under the same router:
//! - **QBRI** (Quantitative Benefit-Risk Index): Expert-judgment quantifier.
//! - **QBR** (Quantitative Benefit-Risk): Statistical-evidence quantifier (4 forms).

use axum::{
    Json, Router,
    http::{HeaderMap, HeaderValue},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

// ═══════════════════════════════════════════════════════════════════════════
// QBRI — Expert-Judgment Benefit-Risk Index (existing)
// ═══════════════════════════════════════════════════════════════════════════

/// QBRI computation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct QbriComputeRequest {
    /// Benefit effect size (0.0-1.0)
    pub benefit_effect: f64,
    /// Benefit p-value from trial (0.0-1.0)
    pub benefit_pvalue: f64,
    /// Unmet medical need (1-10)
    pub unmet_need: f64,
    /// Risk signal strength (0.0-1.0)
    pub risk_signal: f64,
    /// Probability of causal relationship (0.0-1.0)
    pub risk_probability: f64,
    /// Severity of adverse event (1-7)
    pub risk_severity: u8,
    /// Is the adverse event reversible?
    pub reversible: bool,
}

/// QBRI computation response
#[derive(Debug, Serialize, ToSchema)]
pub struct QbriComputeResponse {
    /// QBRI index value
    pub qbri: f64,
    /// Benefit score component
    pub benefit_score: f64,
    /// Risk score component
    pub risk_score: f64,
    /// Recommended decision
    pub decision: String,
    /// Confidence in assessment
    pub confidence: f64,
    /// Applied thresholds
    pub thresholds: ThresholdInfo,
}

/// Threshold information
#[derive(Debug, Serialize, ToSchema)]
pub struct ThresholdInfo {
    /// Approval threshold
    pub tau_approve: f64,
    /// Monitoring (REMS) threshold
    pub tau_monitor: f64,
    /// Uncertainty threshold
    pub tau_uncertain: f64,
}

/// QBRI threshold derivation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct QbriDeriveRequest {
    /// Use synthetic training data (true) or historical (false)
    #[serde(default = "default_true")]
    pub use_synthetic: bool,
}

fn default_true() -> bool {
    true
}

/// QBRI threshold derivation response
#[derive(Debug, Serialize, ToSchema)]
pub struct QbriDeriveResponse {
    /// Derived thresholds
    pub thresholds: ThresholdInfo,
    /// Classification accuracy on training set
    pub accuracy: f64,
    /// Number of drugs in training set
    pub n_drugs: usize,
    /// Decision boundary interpretation
    pub interpretation: DecisionBoundaries,
}

/// Decision boundary descriptions
#[derive(Debug, Serialize, ToSchema)]
pub struct DecisionBoundaries {
    /// Approve condition
    pub approve: String,
    /// REMS condition
    pub rems: String,
    /// More data condition
    pub more_data: String,
    /// Reject condition
    pub reject: String,
}

/// QBRI equation reference response
#[derive(Debug, Serialize, ToSchema)]
pub struct QbriEquationResponse {
    /// The QBRI equation
    pub equation: String,
    /// Variable descriptions
    pub variables: VariableDescriptions,
    /// Hypothesis thresholds
    pub thresholds: ThresholdInfo,
}

/// Variable descriptions for QBRI equation
#[derive(Debug, Serialize, ToSchema)]
pub struct VariableDescriptions {
    /// Benefit magnitude
    pub b: String,
    /// Probability of benefit
    pub pb: String,
    /// Unmet need
    pub ub: String,
    /// Risk signal
    pub r: String,
    /// Probability causal
    pub pr: String,
    /// Severity
    pub sr: String,
    /// Treatability
    pub tr: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// QBR — Statistical-Evidence Benefit-Risk (4 forms)
// ═══════════════════════════════════════════════════════════════════════════

// --- Shared sub-types for QBR API ---

/// 2×2 contingency table for QBR signal detection
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct QbrContingencyTable {
    /// Drug + Event count (cell a)
    pub a: u64,
    /// Drug + No-Event count (cell b)
    pub b: u64,
    /// No-Drug + Event count (cell c)
    pub c: u64,
    /// No-Drug + No-Event count (cell d)
    pub d: u64,
}

/// A value with propagated confidence
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct QbrMeasured {
    /// The computed value
    pub value: f64,
    /// Confidence in the measurement (0.0-1.0)
    pub confidence: f64,
}

/// Hill curve shape parameters for dose-response modeling
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct QbrHillCurveParams {
    /// Half-saturation constant (EC50/TC50). Must be positive.
    pub k_half: f64,
    /// Hill coefficient (cooperativity). Must be positive.
    pub n_hill: f64,
}

/// Dose range for therapeutic window integration
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct QbrIntegrationBounds {
    /// Lower dose bound (inclusive). Must be non-negative.
    pub dose_min: f64,
    /// Upper dose bound (inclusive). Must be > dose_min.
    pub dose_max: f64,
    /// Number of Simpson's rule intervals (must be even)
    #[serde(default = "default_qbr_intervals")]
    pub intervals: usize,
}

fn default_qbr_intervals() -> usize {
    1000
}

fn default_qbr_method() -> String {
    "ebgm".to_string()
}

// --- /qbr/compute ---

/// Full QBR computation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct QbrComputeRequest {
    /// Benefit outcome contingency tables
    pub benefit_tables: Vec<QbrContingencyTable>,
    /// Risk outcome contingency tables
    pub risk_tables: Vec<QbrContingencyTable>,
    /// Benefit weights for composite form (Form 3)
    #[serde(default)]
    pub benefit_weights: Option<Vec<QbrMeasured>>,
    /// Risk weights for composite form (Form 3)
    #[serde(default)]
    pub risk_weights: Option<Vec<QbrMeasured>>,
    /// Efficacy Hill curve parameters (for Form 4 therapeutic window)
    #[serde(default)]
    pub hill_efficacy: Option<QbrHillCurveParams>,
    /// Toxicity Hill curve parameters (for Form 4 therapeutic window)
    #[serde(default)]
    pub hill_toxicity: Option<QbrHillCurveParams>,
    /// Integration bounds for therapeutic window
    #[serde(default)]
    pub integration_bounds: Option<QbrIntegrationBounds>,
    /// Signal detection method: prr, ror, ic, ebgm (default: ebgm)
    #[serde(default = "default_qbr_method")]
    pub method: String,
}

/// Full QBR computation response — all applicable forms
#[derive(Debug, Serialize, ToSchema)]
pub struct QbrComputeResponse {
    /// Form 1: Simple benefit/risk signal ratio
    pub simple: QbrMeasured,
    /// Form 2: Bayesian EBGM ratio (only with EBGM method)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bayesian: Option<QbrMeasured>,
    /// Form 3: Composite weighted ratio (only with weights)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composite: Option<QbrMeasured>,
    /// Form 4: Therapeutic window (only with Hill parameters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub therapeutic_window: Option<QbrMeasured>,
    /// Method details for audit trail
    pub details: QbrMethodDetailsResponse,
}

/// QBR method details for audit transparency
#[derive(Debug, Serialize, ToSchema)]
pub struct QbrMethodDetailsResponse {
    /// Benefit signal strength
    pub benefit_signal: QbrMeasured,
    /// Risk signal strength
    pub risk_signal: QbrMeasured,
    /// EBGM lower bound (EB05) for benefit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_eb05: Option<f64>,
    /// EBGM upper bound (EB95) for risk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_eb95: Option<f64>,
    /// Worst-case Bayesian QBR (EB05_benefit / EB95_risk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worst_case_bayesian: Option<QbrMeasured>,
    /// Signal detection method used
    pub method: String,
}

// --- /qbr/simple ---

/// Simple QBR computation request (single table pair)
#[derive(Debug, Deserialize, ToSchema)]
pub struct QbrSimpleRequest {
    /// Benefit outcome contingency table
    pub benefit_table: QbrContingencyTable,
    /// Risk outcome contingency table
    pub risk_table: QbrContingencyTable,
    /// Signal detection method: prr, ror, ic, ebgm (default: ebgm)
    #[serde(default = "default_qbr_method")]
    pub method: String,
}

/// Simple QBR computation response
#[derive(Debug, Serialize, ToSchema)]
pub struct QbrSimpleResponse {
    /// Benefit/risk signal ratio
    pub qbr: QbrMeasured,
    /// Benefit signal strength
    pub benefit_signal: QbrMeasured,
    /// Risk signal strength
    pub risk_signal: QbrMeasured,
}

// --- /qbr/therapeutic-window ---

/// Therapeutic window computation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct QbrTherapeuticWindowRequest {
    /// Efficacy Hill curve parameters
    pub efficacy: QbrHillCurveParams,
    /// Toxicity Hill curve parameters
    pub toxicity: QbrHillCurveParams,
    /// Integration bounds. Defaults to dose 0.1-100.0, 1000 intervals.
    #[serde(default)]
    pub bounds: Option<QbrIntegrationBounds>,
}

/// Therapeutic window computation response
#[derive(Debug, Serialize, ToSchema)]
pub struct QbrTherapeuticWindowResponse {
    /// Normalized therapeutic window value
    pub therapeutic_window: QbrMeasured,
    /// Area under efficacy curve
    pub efficacy_auc: f64,
    /// Area under toxicity curve
    pub toxicity_auc: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// QBR CONVERSION HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn from_measured(m: &nexcore_constants::Measured<f64>) -> QbrMeasured {
    QbrMeasured {
        value: m.value,
        confidence: m.confidence.value(),
    }
}

fn to_measured(m: &QbrMeasured) -> nexcore_constants::Measured<f64> {
    nexcore_constants::Measured::new(m.value, nexcore_constants::Confidence::new(m.confidence))
}

fn to_ct(t: &QbrContingencyTable) -> nexcore_pv_core::signals::ContingencyTable {
    nexcore_pv_core::signals::ContingencyTable::new(t.a, t.b, t.c, t.d)
}

fn parse_qbr_method(s: &str) -> Result<nexcore_qbr::QbrSignalMethod, ApiError> {
    match s {
        "prr" => Ok(nexcore_qbr::QbrSignalMethod::Prr),
        "ror" => Ok(nexcore_qbr::QbrSignalMethod::Ror),
        "ic" => Ok(nexcore_qbr::QbrSignalMethod::Ic),
        "ebgm" => Ok(nexcore_qbr::QbrSignalMethod::Ebgm),
        other => Err(ApiError::new(
            "VALIDATION_ERROR",
            format!("Invalid method '{other}'. Must be one of: prr, ror, ic, ebgm"),
        )),
    }
}

fn qbr_error_to_api(e: nexcore_qbr::QbrError) -> ApiError {
    use nexcore_qbr::QbrError;
    match &e {
        QbrError::NoBenefitTables
        | QbrError::NoRiskTables
        | QbrError::WeightMismatch { .. }
        | QbrError::InvalidHillParams(_) => ApiError::new("VALIDATION_ERROR", e.to_string()),
        QbrError::SignalDetection(_) | QbrError::Integration(_) | QbrError::ZeroRiskSignal => {
            ApiError::new("COMPUTATION_ERROR", e.to_string())
        }
    }
}

fn qbr_confidence_header(confidence: f64) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(v) = HeaderValue::from_str(&format!("{confidence:.4}")) {
        headers.insert("x-qbr-confidence", v);
    }
    headers
}

// ═══════════════════════════════════════════════════════════════════════════
// ROUTER
// ═══════════════════════════════════════════════════════════════════════════

/// Benefit-risk router — QBRI + QBR coexist
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        // QBRI endpoints (expert-judgment)
        .route("/qbri/compute", post(qbri_compute))
        .route("/qbri/derive", post(qbri_derive))
        .route("/qbri/equation", get(qbri_equation))
        // QBR endpoints (statistical-evidence)
        .route("/qbr/compute", post(qbr_compute))
        .route("/qbr/simple", post(qbr_simple))
        .route("/qbr/therapeutic-window", post(qbr_therapeutic_window))
}

// ═══════════════════════════════════════════════════════════════════════════
// QBRI HANDLERS (unchanged)
// ═══════════════════════════════════════════════════════════════════════════

/// Compute QBRI from benefit and risk parameters
#[utoipa::path(
    post,
    path = "/api/v1/benefit-risk/qbri/compute",
    tag = "benefit-risk",
    request_body = QbriComputeRequest,
    responses(
        (status = 200, description = "QBRI computed successfully", body = QbriComputeResponse),
        (status = 400, description = "Invalid parameters", body = super::common::ApiError)
    )
)]
pub async fn qbri_compute(
    Json(req): Json<QbriComputeRequest>,
) -> Result<Json<QbriComputeResponse>, ApiError> {
    use nexcore_vigilance::pv::benefit_risk::{
        BenefitAssessment, QbriThresholds, RiskAssessment, compute_qbri,
    };

    let benefit =
        BenefitAssessment::from_trial(req.benefit_effect, req.benefit_pvalue, req.unmet_need);
    let risk = RiskAssessment::from_signal(
        req.risk_signal,
        req.risk_probability,
        req.risk_severity,
        req.reversible,
    );
    let thresholds = QbriThresholds::default();
    let result = compute_qbri(&benefit, &risk, &thresholds);

    Ok(Json(QbriComputeResponse {
        qbri: result.index,
        benefit_score: result.benefit_score,
        risk_score: result.risk_score,
        decision: format!("{:?}", result.decision),
        confidence: result.confidence,
        thresholds: ThresholdInfo {
            tau_approve: thresholds.tau_approve,
            tau_monitor: thresholds.tau_monitor,
            tau_uncertain: thresholds.tau_uncertain,
        },
    }))
}

/// Derive optimal QBRI thresholds from historical FDA decisions
#[utoipa::path(
    post,
    path = "/api/v1/benefit-risk/qbri/derive",
    tag = "benefit-risk",
    request_body = QbriDeriveRequest,
    responses(
        (status = 200, description = "Thresholds derived successfully", body = QbriDeriveResponse)
    )
)]
pub async fn qbri_derive(Json(req): Json<QbriDeriveRequest>) -> ApiResult<QbriDeriveResponse> {
    use nexcore_vigilance::pv::benefit_risk::{derive_thresholds, generate_synthetic_data};

    // Historical data source not yet available — synthetic is the only provider.
    // When historical FAERS/VAERS pipelines land, gate on req.use_synthetic here.
    let _use_synthetic = req.use_synthetic;
    let data = generate_synthetic_data();
    let result = derive_thresholds(&data);
    let t = &result.thresholds;

    Ok(Json(QbriDeriveResponse {
        thresholds: ThresholdInfo {
            tau_approve: t.tau_approve,
            tau_monitor: t.tau_monitor,
            tau_uncertain: t.tau_uncertain,
        },
        accuracy: result.accuracy,
        n_drugs: result.n_drugs,
        interpretation: DecisionBoundaries {
            approve: format!("QBRI > {:.2}", t.tau_approve),
            rems: format!("QBRI in [{:.2}, {:.2}]", t.tau_monitor, t.tau_approve),
            more_data: format!("QBRI in [{:.2}, {:.2}]", t.tau_uncertain, t.tau_monitor),
            reject: format!("QBRI < {:.2}", t.tau_uncertain),
        },
    }))
}

/// Get QBRI equation reference
#[utoipa::path(
    get,
    path = "/api/v1/benefit-risk/qbri/equation",
    tag = "benefit-risk",
    responses(
        (status = 200, description = "QBRI equation reference", body = QbriEquationResponse)
    )
)]
pub async fn qbri_equation() -> ApiResult<QbriEquationResponse> {
    Ok(Json(QbriEquationResponse {
        equation: "QBRI = (B * Pb * Ub) / (R * Pr * Sr * Tr)".to_string(),
        variables: VariableDescriptions {
            b: "Benefit magnitude (effect size)".to_string(),
            pb: "P(benefit) = 1 - pvalue".to_string(),
            ub: "Unmet medical need [1-10]".to_string(),
            r: "Risk signal strength".to_string(),
            pr: "P(causal relationship)".to_string(),
            sr: "Severity [1-7]".to_string(),
            tr: "Treatability (1=reversible, 2=irreversible)".to_string(),
        },
        thresholds: ThresholdInfo {
            tau_approve: 2.0,
            tau_monitor: 1.0,
            tau_uncertain: 0.5,
        },
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// QBR HANDLERS
// ═══════════════════════════════════════════════════════════════════════════

/// Compute full QBR with all available forms
#[utoipa::path(
    post,
    path = "/api/v1/benefit-risk/qbr/compute",
    tag = "benefit-risk",
    request_body = QbrComputeRequest,
    responses(
        (status = 200, description = "QBR computed successfully", body = QbrComputeResponse),
        (status = 400, description = "Invalid input parameters", body = super::common::ApiError),
        (status = 422, description = "Computation failure", body = super::common::ApiError)
    )
)]
pub async fn qbr_compute(
    Json(req): Json<QbrComputeRequest>,
) -> Result<(HeaderMap, Json<QbrComputeResponse>), ApiError> {
    use nexcore_qbr::{BenefitRiskInput, HillCurveParams, IntegrationBounds, compute_qbr};

    let method = parse_qbr_method(&req.method)?;

    let input = BenefitRiskInput {
        benefit_tables: req.benefit_tables.iter().map(to_ct).collect(),
        risk_tables: req.risk_tables.iter().map(to_ct).collect(),
        benefit_weights: req
            .benefit_weights
            .as_ref()
            .map(|ws| ws.iter().map(to_measured).collect()),
        risk_weights: req
            .risk_weights
            .as_ref()
            .map(|ws| ws.iter().map(to_measured).collect()),
        hill_efficacy: req.hill_efficacy.as_ref().map(|h| HillCurveParams {
            k_half: h.k_half,
            n_hill: h.n_hill,
        }),
        hill_toxicity: req.hill_toxicity.as_ref().map(|h| HillCurveParams {
            k_half: h.k_half,
            n_hill: h.n_hill,
        }),
        integration_bounds: req.integration_bounds.as_ref().map(|b| IntegrationBounds {
            dose_min: b.dose_min,
            dose_max: b.dose_max,
            intervals: b.intervals,
        }),
        method,
    };

    let qbr = compute_qbr(&input).map_err(qbr_error_to_api)?;
    let confidence = qbr.simple.confidence.value();

    let response = QbrComputeResponse {
        simple: from_measured(&qbr.simple),
        bayesian: qbr.bayesian.as_ref().map(from_measured),
        composite: qbr.composite.as_ref().map(from_measured),
        therapeutic_window: qbr.therapeutic_window.as_ref().map(from_measured),
        details: QbrMethodDetailsResponse {
            benefit_signal: from_measured(&qbr.details.benefit_signal),
            risk_signal: from_measured(&qbr.details.risk_signal),
            benefit_eb05: qbr.details.benefit_eb05,
            risk_eb95: qbr.details.risk_eb95,
            worst_case_bayesian: qbr.details.worst_case_bayesian.as_ref().map(from_measured),
            method: format!("{:?}", qbr.details.method).to_lowercase(),
        },
    };

    Ok((qbr_confidence_header(confidence), Json(response)))
}

/// Compute simple benefit-risk ratio from a single table pair
#[utoipa::path(
    post,
    path = "/api/v1/benefit-risk/qbr/simple",
    tag = "benefit-risk",
    request_body = QbrSimpleRequest,
    responses(
        (status = 200, description = "Simple QBR computed", body = QbrSimpleResponse),
        (status = 400, description = "Invalid input", body = super::common::ApiError),
        (status = 422, description = "Computation failure", body = super::common::ApiError)
    )
)]
pub async fn qbr_simple(
    Json(req): Json<QbrSimpleRequest>,
) -> Result<(HeaderMap, Json<QbrSimpleResponse>), ApiError> {
    use nexcore_qbr::signal::extract_signal_strength;

    let method = parse_qbr_method(&req.method)?;
    let benefit_ct = to_ct(&req.benefit_table);
    let risk_ct = to_ct(&req.risk_table);

    let qbr_ratio =
        nexcore_qbr::compute_simple(&benefit_ct, &risk_ct, method).map_err(qbr_error_to_api)?;
    let benefit_signal = extract_signal_strength(&benefit_ct, method).map_err(qbr_error_to_api)?;
    let risk_signal = extract_signal_strength(&risk_ct, method).map_err(qbr_error_to_api)?;

    let confidence = qbr_ratio.confidence.value();
    let response = QbrSimpleResponse {
        qbr: from_measured(&qbr_ratio),
        benefit_signal: from_measured(&benefit_signal),
        risk_signal: from_measured(&risk_signal),
    };

    Ok((qbr_confidence_header(confidence), Json(response)))
}

/// Compute therapeutic window from Hill curve parameters
#[utoipa::path(
    post,
    path = "/api/v1/benefit-risk/qbr/therapeutic-window",
    tag = "benefit-risk",
    request_body = QbrTherapeuticWindowRequest,
    responses(
        (status = 200, description = "Therapeutic window computed", body = QbrTherapeuticWindowResponse),
        (status = 400, description = "Invalid Hill parameters", body = super::common::ApiError),
        (status = 422, description = "Integration failure", body = super::common::ApiError)
    )
)]
pub async fn qbr_therapeutic_window(
    Json(req): Json<QbrTherapeuticWindowRequest>,
) -> Result<(HeaderMap, Json<QbrTherapeuticWindowResponse>), ApiError> {
    use nexcore_primitives::chemistry::cooperativity::hill_response;
    use nexcore_qbr::{
        HillCurveParams, IntegrationBounds, compute_therapeutic_window, simpson_integrate,
    };

    let efficacy = HillCurveParams {
        k_half: req.efficacy.k_half,
        n_hill: req.efficacy.n_hill,
    };
    let toxicity = HillCurveParams {
        k_half: req.toxicity.k_half,
        n_hill: req.toxicity.n_hill,
    };
    let bounds = req.bounds.as_ref().map_or(
        IntegrationBounds {
            dose_min: 0.1,
            dose_max: 100.0,
            intervals: 1000,
        },
        |b| IntegrationBounds {
            dose_min: b.dose_min,
            dose_max: b.dose_max,
            intervals: b.intervals,
        },
    );

    let tw = compute_therapeutic_window(&efficacy, &toxicity, &bounds).map_err(qbr_error_to_api)?;

    // Compute component AUCs for transparency
    let n = if bounds.intervals % 2 != 0 {
        bounds.intervals + 1
    } else {
        bounds.intervals
    };
    let eff_k = efficacy.k_half;
    let eff_n = efficacy.n_hill;
    let tox_k = toxicity.k_half;
    let tox_n = toxicity.n_hill;

    let efficacy_auc = simpson_integrate(
        |d| hill_response(d, eff_k, eff_n),
        bounds.dose_min,
        bounds.dose_max,
        n,
    )
    .map_err(qbr_error_to_api)?;

    let toxicity_auc = simpson_integrate(
        |d| hill_response(d, tox_k, tox_n),
        bounds.dose_min,
        bounds.dose_max,
        n,
    )
    .map_err(qbr_error_to_api)?;

    let confidence = tw.confidence.value();
    let response = QbrTherapeuticWindowResponse {
        therapeutic_window: from_measured(&tw),
        efficacy_auc,
        toxicity_auc,
    };

    Ok((qbr_confidence_header(confidence), Json(response)))
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn ct(a: u64, b: u64, c: u64, d: u64) -> QbrContingencyTable {
        QbrContingencyTable { a, b, c, d }
    }

    // ── QBR /compute ────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_qbr_compute_minimal() {
        let req = QbrComputeRequest {
            benefit_tables: vec![ct(20, 80, 5, 95)],
            risk_tables: vec![ct(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: "prr".to_string(),
        };
        let result = qbr_compute(Json(req)).await;
        assert!(result.is_ok(), "QBR compute minimal should succeed");
        if let Ok((headers, Json(resp))) = result {
            assert!(resp.simple.value > 0.0, "Simple ratio should be positive");
            assert!(resp.bayesian.is_none(), "PRR method → no Bayesian form");
            assert!(resp.composite.is_none(), "No weights → no composite");
            assert!(resp.therapeutic_window.is_none(), "No Hill → no window");
            assert!(
                headers.contains_key("x-qbr-confidence"),
                "QBR confidence header missing"
            );
        }
    }

    #[tokio::test]
    async fn test_qbr_compute_all_forms() {
        let req = QbrComputeRequest {
            benefit_tables: vec![ct(25, 75, 5, 95)],
            risk_tables: vec![ct(10, 90, 5, 95)],
            benefit_weights: Some(vec![QbrMeasured {
                value: 2.0,
                confidence: 0.9,
            }]),
            risk_weights: Some(vec![QbrMeasured {
                value: 1.0,
                confidence: 0.9,
            }]),
            hill_efficacy: Some(QbrHillCurveParams {
                k_half: 10.0,
                n_hill: 2.0,
            }),
            hill_toxicity: Some(QbrHillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            }),
            integration_bounds: Some(QbrIntegrationBounds {
                dose_min: 0.1,
                dose_max: 100.0,
                intervals: 1000,
            }),
            method: "ebgm".to_string(),
        };
        let result = qbr_compute(Json(req)).await;
        assert!(result.is_ok(), "QBR compute all forms should succeed");
        if let Ok((_, Json(resp))) = result {
            assert!(resp.simple.value > 0.0, "Form 1 present");
            assert!(resp.bayesian.is_some(), "Form 2 with EBGM");
            assert!(resp.composite.is_some(), "Form 3 with weights");
            assert!(resp.therapeutic_window.is_some(), "Form 4 with Hill");
            assert!(resp.details.benefit_signal.value > 0.0);
            assert!(resp.details.risk_signal.value > 0.0);
            assert_eq!(resp.details.method, "ebgm");
        }
    }

    #[tokio::test]
    async fn test_qbr_compute_empty_benefit_tables() {
        let req = QbrComputeRequest {
            benefit_tables: vec![],
            risk_tables: vec![ct(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: "prr".to_string(),
        };
        let result = qbr_compute(Json(req)).await;
        assert!(result.is_err(), "Empty benefit tables should fail");
        if let Err(err) = result {
            assert_eq!(err.code, "VALIDATION_ERROR");
        }
    }

    #[tokio::test]
    async fn test_qbr_compute_invalid_method() {
        let req = QbrComputeRequest {
            benefit_tables: vec![ct(20, 80, 5, 95)],
            risk_tables: vec![ct(10, 90, 5, 95)],
            benefit_weights: None,
            risk_weights: None,
            hill_efficacy: None,
            hill_toxicity: None,
            integration_bounds: None,
            method: "invalid_method".to_string(),
        };
        let result = qbr_compute(Json(req)).await;
        assert!(result.is_err(), "Invalid method should fail");
        if let Err(err) = result {
            assert_eq!(err.code, "VALIDATION_ERROR");
        }
    }

    // ── QBR /simple ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_qbr_simple_happy_path() {
        let req = QbrSimpleRequest {
            benefit_table: ct(20, 80, 5, 95),
            risk_table: ct(10, 90, 5, 95),
            method: "prr".to_string(),
        };
        let result = qbr_simple(Json(req)).await;
        assert!(result.is_ok(), "QBR simple should succeed");
        if let Ok((headers, Json(resp))) = result {
            assert!(resp.qbr.value > 0.0, "QBR ratio should be positive");
            assert!(resp.benefit_signal.value > 0.0);
            assert!(resp.risk_signal.value > 0.0);
            assert!(headers.contains_key("x-qbr-confidence"));
        }
    }

    #[tokio::test]
    async fn test_qbr_simple_all_methods() {
        for method in &["prr", "ror", "ic", "ebgm"] {
            let req = QbrSimpleRequest {
                benefit_table: ct(20, 80, 5, 95),
                risk_table: ct(10, 90, 5, 95),
                method: method.to_string(),
            };
            let result = qbr_simple(Json(req)).await;
            assert!(result.is_ok(), "QBR simple with {method} should succeed");
        }
    }

    // ── QBR /therapeutic-window ─────────────────────────────────────────

    #[tokio::test]
    async fn test_qbr_therapeutic_window_happy_path() {
        let req = QbrTherapeuticWindowRequest {
            efficacy: QbrHillCurveParams {
                k_half: 10.0,
                n_hill: 2.0,
            },
            toxicity: QbrHillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            },
            bounds: Some(QbrIntegrationBounds {
                dose_min: 0.1,
                dose_max: 100.0,
                intervals: 1000,
            }),
        };
        let result = qbr_therapeutic_window(Json(req)).await;
        assert!(result.is_ok(), "Therapeutic window should succeed");
        if let Ok((headers, Json(resp))) = result {
            assert!(
                resp.therapeutic_window.value > 0.0,
                "EC50=10 < TC50=80 → positive window"
            );
            assert!(resp.efficacy_auc > 0.0);
            assert!(resp.toxicity_auc > 0.0);
            assert!(headers.contains_key("x-qbr-confidence"));
        }
    }

    #[tokio::test]
    async fn test_qbr_therapeutic_window_default_bounds() {
        let req = QbrTherapeuticWindowRequest {
            efficacy: QbrHillCurveParams {
                k_half: 10.0,
                n_hill: 2.0,
            },
            toxicity: QbrHillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            },
            bounds: None,
        };
        let result = qbr_therapeutic_window(Json(req)).await;
        assert!(result.is_ok(), "Should succeed with default bounds");
    }

    #[tokio::test]
    async fn test_qbr_therapeutic_window_invalid_params() {
        let req = QbrTherapeuticWindowRequest {
            efficacy: QbrHillCurveParams {
                k_half: -5.0,
                n_hill: 2.0,
            },
            toxicity: QbrHillCurveParams {
                k_half: 80.0,
                n_hill: 2.0,
            },
            bounds: None,
        };
        let result = qbr_therapeutic_window(Json(req)).await;
        assert!(result.is_err(), "Negative k_half should fail");
        if let Err(err) = result {
            assert_eq!(err.code, "VALIDATION_ERROR");
        }
    }

    // ── Response header verification ────────────────────────────────────

    #[tokio::test]
    async fn test_qbr_confidence_header_format() {
        let req = QbrSimpleRequest {
            benefit_table: ct(20, 80, 5, 95),
            risk_table: ct(10, 90, 5, 95),
            method: "prr".to_string(),
        };
        let result = qbr_simple(Json(req)).await;
        assert!(result.is_ok());
        if let Ok((headers, _)) = result {
            let conf = headers.get("x-qbr-confidence");
            assert!(conf.is_some(), "x-qbr-confidence header must be present");
            if let Some(v) = conf {
                let s = v.to_str().unwrap_or("");
                let parsed: Result<f64, _> = s.parse();
                assert!(parsed.is_ok(), "Header value must be parseable as f64");
                if let Ok(val) = parsed {
                    assert!(val > 0.0 && val < 1.0, "Confidence should be in (0,1)");
                }
            }
        }
    }
}
