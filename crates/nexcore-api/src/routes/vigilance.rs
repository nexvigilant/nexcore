//! Theory of Vigilance endpoints

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiResult;

/// Safety margin calculation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SafetyMarginRequest {
    /// Current state vector
    pub state: Vec<f64>,
    /// Safety boundary
    pub boundary: Vec<f64>,
}

/// Safety margin response
#[derive(Debug, Serialize, ToSchema)]
pub struct SafetyMarginResponse {
    /// Distance to safety boundary d(s)
    pub distance: f64,
    /// Safety status
    pub status: String,
    /// Normalized margin (0.0 = at boundary, 1.0 = max safe)
    pub normalized: f64,
}

/// Risk score calculation request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RiskScoreRequest {
    /// Severity (1-5)
    pub severity: u8,
    /// Likelihood (1-5)
    pub likelihood: u8,
    /// Detectability (1-5)
    pub detectability: u8,
    /// Optional: number of affected patients
    pub affected_count: Option<u32>,
}

/// Risk score response
#[derive(Debug, Serialize, ToSchema)]
pub struct RiskScoreResponse {
    /// Raw risk score
    pub score: f64,
    /// Risk level category
    pub level: String,
    /// Recommended action
    pub action: String,
    /// Priority ranking (1 = highest)
    pub priority: u8,
}

/// Harm type from Theory of Vigilance taxonomy
#[derive(Debug, Serialize, ToSchema)]
pub struct HarmType {
    /// Harm type code (A-H)
    pub code: String,
    /// Harm type name
    pub name: String,
    /// Description
    pub description: String,
    /// Example manifestations
    pub examples: Vec<String>,
}

/// Map event to ToV request
#[derive(Debug, Deserialize, ToSchema)]
#[allow(dead_code)]
pub struct MapToTovRequest {
    /// Event description
    pub event: String,
    /// Event category (optional)
    pub category: Option<String>,
}

/// Map event to ToV response
#[derive(Debug, Serialize, ToSchema)]
pub struct MapToTovResponse {
    /// Matched harm types
    pub harm_types: Vec<String>,
    /// Relevant axioms
    pub axioms: Vec<String>,
    /// Confidence score
    pub confidence: f64,
}

/// Vigilance router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/safety-margin", post(safety_margin))
        .route("/risk-score", post(risk_score))
        .route("/harm-types", get(harm_types))
        .route("/map-to-tov", post(map_to_tov))
}

/// Calculate safety margin d(s) from current state to boundary
#[utoipa::path(
    post,
    path = "/api/v1/vigilance/safety-margin",
    tag = "vigilance",
    request_body = SafetyMarginRequest,
    responses(
        (status = 200, description = "Safety margin calculated", body = SafetyMarginResponse)
    )
)]
pub async fn safety_margin(
    Json(req): Json<SafetyMarginRequest>,
) -> ApiResult<SafetyMarginResponse> {
    // Calculate Euclidean distance to boundary
    let distance: f64 = req
        .state
        .iter()
        .zip(req.boundary.iter())
        .map(|(s, b)| (s - b).powi(2))
        .sum::<f64>()
        .sqrt();

    let max_distance = req.boundary.iter().map(|b| b.abs()).sum::<f64>().max(1.0);
    let normalized = (distance / max_distance).min(1.0);

    let status = if distance > 0.5 {
        "SAFE"
    } else if distance > 0.1 {
        "WARNING"
    } else {
        "CRITICAL"
    };

    Ok(Json(SafetyMarginResponse {
        distance,
        status: status.to_string(),
        normalized,
    }))
}

/// Calculate Guardian-AV risk score
#[utoipa::path(
    post,
    path = "/api/v1/vigilance/risk-score",
    tag = "vigilance",
    request_body = RiskScoreRequest,
    responses(
        (status = 200, description = "Risk score calculated", body = RiskScoreResponse)
    )
)]
pub async fn risk_score(Json(req): Json<RiskScoreRequest>) -> ApiResult<RiskScoreResponse> {
    // RPN-style calculation: Severity x Likelihood x (6 - Detectability)
    let detectability_factor = 6.0 - f64::from(req.detectability.min(5));
    let raw_score = f64::from(req.severity) * f64::from(req.likelihood) * detectability_factor;

    // Normalize to 0-100 scale (max raw = 5 * 5 * 5 = 125)
    let score = (raw_score / 125.0) * 100.0;

    // Apply affected count multiplier if provided
    let final_score = if let Some(count) = req.affected_count {
        score * (1.0 + (count as f64).ln().max(0.0) / 10.0)
    } else {
        score
    }
    .min(100.0);

    let (level, action, priority) = if final_score >= 80.0 {
        ("CRITICAL", "Immediate intervention required", 1)
    } else if final_score >= 60.0 {
        ("HIGH", "Urgent action within 24 hours", 2)
    } else if final_score >= 40.0 {
        ("MEDIUM", "Action within 1 week", 3)
    } else if final_score >= 20.0 {
        ("LOW", "Monitor and review", 4)
    } else {
        ("MINIMAL", "Document and continue monitoring", 5)
    };

    Ok(Json(RiskScoreResponse {
        score: final_score,
        level: level.to_string(),
        action: action.to_string(),
        priority,
    }))
}

/// List all harm types from ToV taxonomy
#[utoipa::path(
    get,
    path = "/api/v1/vigilance/harm-types",
    tag = "vigilance",
    responses(
        (status = 200, description = "Harm types list", body = Vec<HarmType>)
    )
)]
pub async fn harm_types() -> Json<Vec<HarmType>> {
    let types = vec![
        HarmType {
            code: "A".to_string(),
            name: "Dose-Related (Augmented)".to_string(),
            description: "Predictable, dose-dependent adverse effects".to_string(),
            examples: vec![
                "Bleeding with anticoagulants".to_string(),
                "Hypoglycemia with insulin".to_string(),
            ],
        },
        HarmType {
            code: "B".to_string(),
            name: "Non-Dose-Related (Bizarre)".to_string(),
            description: "Unpredictable, idiosyncratic reactions".to_string(),
            examples: vec![
                "Anaphylaxis".to_string(),
                "Stevens-Johnson syndrome".to_string(),
            ],
        },
        HarmType {
            code: "C".to_string(),
            name: "Dose and Time-Related (Chronic)".to_string(),
            description: "Effects from prolonged exposure".to_string(),
            examples: vec![
                "Osteoporosis with corticosteroids".to_string(),
                "Tardive dyskinesia".to_string(),
            ],
        },
        HarmType {
            code: "D".to_string(),
            name: "Time-Related (Delayed)".to_string(),
            description: "Effects appearing after discontinuation".to_string(),
            examples: vec!["Carcinogenesis".to_string(), "Teratogenesis".to_string()],
        },
        HarmType {
            code: "E".to_string(),
            name: "Withdrawal (End of Use)".to_string(),
            description: "Effects from drug discontinuation".to_string(),
            examples: vec![
                "Opioid withdrawal".to_string(),
                "Rebound hypertension".to_string(),
            ],
        },
        HarmType {
            code: "F".to_string(),
            name: "Unexpected Failure".to_string(),
            description: "Therapeutic failure".to_string(),
            examples: vec![
                "Antibiotic resistance".to_string(),
                "Contraceptive failure".to_string(),
            ],
        },
        HarmType {
            code: "G".to_string(),
            name: "Genetic".to_string(),
            description: "Genetically-determined reactions".to_string(),
            examples: vec![
                "G6PD deficiency reactions".to_string(),
                "CYP2D6 poor metabolizers".to_string(),
            ],
        },
        HarmType {
            code: "H".to_string(),
            name: "Hypersensitivity".to_string(),
            description: "Immune-mediated reactions".to_string(),
            examples: vec![
                "Drug-induced lupus".to_string(),
                "Serum sickness".to_string(),
            ],
        },
    ];

    Json(types)
}

/// Map an event to Theory of Vigilance axioms and harm types
#[utoipa::path(
    post,
    path = "/api/v1/vigilance/map-to-tov",
    tag = "vigilance",
    request_body = MapToTovRequest,
    responses(
        (status = 200, description = "ToV mapping", body = MapToTovResponse)
    )
)]
pub async fn map_to_tov(Json(req): Json<MapToTovRequest>) -> ApiResult<MapToTovResponse> {
    let event_lower = req.event.to_lowercase();

    // Simple keyword-based mapping (production would use ML)
    let mut harm_types = Vec::new();
    let mut axioms = Vec::new();
    let mut confidence = 0.5;

    if event_lower.contains("dose") || event_lower.contains("overdose") {
        harm_types.push("A".to_string());
        axioms.push("Axiom 4: Safety Manifold".to_string());
        confidence = 0.8;
    }
    if event_lower.contains("allergy") || event_lower.contains("anaphyl") {
        harm_types.push("B".to_string());
        harm_types.push("H".to_string());
        axioms.push("Axiom 5: Emergence".to_string());
        confidence = 0.85;
    }
    if event_lower.contains("withdraw") || event_lower.contains("discontin") {
        harm_types.push("E".to_string());
        axioms.push("Axiom 3: Conservation".to_string());
        confidence = 0.75;
    }
    if event_lower.contains("fail") || event_lower.contains("ineffect") {
        harm_types.push("F".to_string());
        axioms.push("Axiom 1: Decomposition".to_string());
        confidence = 0.7;
    }

    // Default if no matches
    if harm_types.is_empty() {
        harm_types.push("B".to_string()); // Default to idiosyncratic
        axioms.push("Axiom 2: Hierarchy".to_string());
        confidence = 0.3;
    }

    Ok(Json(MapToTovResponse {
        harm_types,
        axioms,
        confidence,
    }))
}
