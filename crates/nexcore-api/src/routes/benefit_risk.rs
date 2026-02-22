//! QBRI Benefit-Risk Assessment endpoints

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

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

/// Benefit-risk router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/qbri/compute", post(qbri_compute))
        .route("/qbri/derive", post(qbri_derive))
        .route("/qbri/equation", get(qbri_equation))
}

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
