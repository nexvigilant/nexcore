//! FAERS (FDA Adverse Event Reporting System) routes
//!
//! Proxies queries to the openFDA drug/event API and returns structured results.
//! Signal detection reuses `nexcore_vigilance::pv::signals` for consistency
//! with the PV signal routes.

use axum::{
    Json, Router,
    extract::Query,
    routing::{get, post},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::ToSchema;

use super::common::ApiError;

// ── Constants ────────────────────────────────

const OPENFDA_BASE_URL: &str = "https://api.fda.gov/drug/event.json";

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(4)
        .build()
        .unwrap_or_default()
});

// ── Request types ────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct FaersSearchQuery {
    /// Drug name, reaction term, or free-text query
    pub query: String,
    /// Max results (default 25, max 100)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FaersDrugEventsQuery {
    /// Drug name (generic or brand)
    pub drug: String,
    /// Max events to return (default 20, max 100)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FaersSignalCheckRequest {
    /// Drug name
    pub drug: String,
    /// Adverse event (MedDRA preferred term)
    pub event: String,
}

// ── Response types (match frontend TypeScript interfaces) ──

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersSearchResponse {
    pub results: Vec<FaersResult>,
    pub total: u64,
    pub query: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersResult {
    pub safetyreportid: String,
    pub receivedate: String,
    pub serious: u8,
    pub patient: FaersPatient,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersPatient {
    pub drug: Vec<FaersDrug>,
    pub reaction: Vec<FaersReaction>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersDrug {
    pub medicinalproduct: String,
    pub drugcharacterization: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersReaction {
    pub reactionmeddrapt: String,
    pub reactionoutcome: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersDrugEventsResponse {
    pub drug: String,
    pub events: Vec<FaersEventCount>,
    pub total_reports: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersEventCount {
    pub event: String,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersSignalCheckResponse {
    pub drug: String,
    pub event: String,
    pub signal_detected: bool,
    pub prr: f64,
    pub ror: f64,
    pub case_count: u64,
}

// ── OpenFDA deserialization ──────────────────

#[derive(Debug, Deserialize)]
struct OpenFdaResponse {
    meta: Option<OpenFdaMeta>,
    results: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct OpenFdaMeta {
    results: Option<OpenFdaMetaResults>,
}

#[derive(Debug, Deserialize)]
struct OpenFdaMetaResults {
    total: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenFdaCountResult {
    term: String,
    count: u64,
}

#[derive(Debug, Deserialize)]
struct OpenFdaCountResponse {
    results: Option<Vec<OpenFdaCountResult>>,
}

// ── Router ───────────────────────────────────

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/search", get(search))
        .route("/drug-events", get(drug_events))
        .route("/signal-check", post(signal_check))
        .route("/signal-graph", get(signal_graph))
}

// ── Handlers ─────────────────────────────────

/// Search FAERS adverse event reports
#[utoipa::path(
    get,
    path = "/api/v1/faers/search",
    tag = "faers",
    params(
        ("query" = String, Query, description = "Drug name, reaction, or free-text query"),
        ("limit" = Option<usize>, Query, description = "Max results (default 25, max 100)")
    ),
    responses(
        (status = 200, description = "Search results", body = FaersSearchResponse),
        (status = 400, description = "Missing query", body = ApiError),
        (status = 502, description = "openFDA upstream error", body = ApiError)
    )
)]
pub async fn search(
    Query(params): Query<FaersSearchQuery>,
) -> Result<Json<FaersSearchResponse>, ApiError> {
    if params.query.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "query parameter is required",
        ));
    }

    let query_upper = params.query.trim().to_uppercase();
    let limit = params.limit.unwrap_or(25).min(100);

    // Search across drug names (generic + brand)
    let search = format!(
        "(patient.drug.medicinalproduct:{query_upper}\
         +OR+patient.drug.openfda.brand_name:{query_upper}\
         +OR+patient.drug.openfda.generic_name:{query_upper})"
    );
    let url = format!("{OPENFDA_BASE_URL}?search={search}&limit={limit}");

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::new("UPSTREAM_ERROR", format!("openFDA request failed: {e}")))?;

    if response.status().as_u16() == 404 {
        return Ok(Json(FaersSearchResponse {
            results: vec![],
            total: 0,
            query: params.query,
        }));
    }

    if response.status().as_u16() == 429 {
        return Err(ApiError::new(
            "RATE_LIMITED",
            "openFDA rate limit exceeded. Try again later.",
        ));
    }

    if !response.status().is_success() {
        return Err(ApiError::new(
            "UPSTREAM_ERROR",
            format!("openFDA returned status {}", response.status()),
        ));
    }

    let data: OpenFdaResponse = response.json().await.map_err(|e| {
        ApiError::new(
            "PARSE_ERROR",
            format!("Failed to parse openFDA response: {e}"),
        )
    })?;

    let total = data
        .meta
        .and_then(|m| m.results)
        .and_then(|r| r.total)
        .unwrap_or(0);

    let results = data
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|raw| parse_event_result(&raw))
        .collect();

    Ok(Json(FaersSearchResponse {
        results,
        total,
        query: params.query,
    }))
}

/// Get top adverse events for a drug
#[utoipa::path(
    get,
    path = "/api/v1/faers/drug-events",
    tag = "faers",
    params(
        ("drug" = String, Query, description = "Drug name (generic or brand)"),
        ("limit" = Option<usize>, Query, description = "Top N events (default 20, max 100)")
    ),
    responses(
        (status = 200, description = "Drug event counts", body = FaersDrugEventsResponse),
        (status = 400, description = "Missing drug name", body = ApiError),
        (status = 502, description = "openFDA upstream error", body = ApiError)
    )
)]
pub async fn drug_events(
    Query(params): Query<FaersDrugEventsQuery>,
) -> Result<Json<FaersDrugEventsResponse>, ApiError> {
    if params.drug.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "drug parameter is required",
        ));
    }

    let drug_upper = params.drug.trim().to_uppercase();
    let limit = params.limit.unwrap_or(20).min(100);

    // Use openFDA count API to aggregate reaction terms
    let url = format!(
        "{OPENFDA_BASE_URL}?search=patient.drug.medicinalproduct:{drug_upper}\
         &count=patient.reaction.reactionmeddrapt.exact&limit={limit}"
    );

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::new("UPSTREAM_ERROR", format!("openFDA request failed: {e}")))?;

    if response.status().as_u16() == 404 {
        return Ok(Json(FaersDrugEventsResponse {
            drug: params.drug,
            events: vec![],
            total_reports: 0,
        }));
    }

    if !response.status().is_success() {
        return Err(ApiError::new(
            "UPSTREAM_ERROR",
            format!("openFDA returned status {}", response.status()),
        ));
    }

    let data: OpenFdaCountResponse = response.json().await.map_err(|e| {
        ApiError::new(
            "PARSE_ERROR",
            format!("Failed to parse openFDA response: {e}"),
        )
    })?;

    let raw_events = data.results.unwrap_or_default();
    let total_reports: u64 = raw_events.iter().map(|r| r.count).sum();

    let events: Vec<FaersEventCount> = raw_events
        .into_iter()
        .map(|r| {
            let pct = if total_reports > 0 {
                (r.count as f64 / total_reports as f64) * 100.0
            } else {
                0.0
            };
            FaersEventCount {
                event: r.term,
                count: r.count,
                percentage: (pct * 10.0).round() / 10.0,
            }
        })
        .collect();

    Ok(Json(FaersDrugEventsResponse {
        drug: params.drug,
        events,
        total_reports,
    }))
}

/// Check for a disproportionality signal between a drug and event
#[utoipa::path(
    post,
    path = "/api/v1/faers/signal-check",
    tag = "faers",
    request_body = FaersSignalCheckRequest,
    responses(
        (status = 200, description = "Signal check result", body = FaersSignalCheckResponse),
        (status = 400, description = "Missing parameters", body = ApiError),
        (status = 502, description = "openFDA upstream error", body = ApiError)
    )
)]
pub async fn signal_check(
    Json(req): Json<FaersSignalCheckRequest>,
) -> Result<Json<FaersSignalCheckResponse>, ApiError> {
    if req.drug.trim().is_empty() || req.event.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "Both drug and event are required",
        ));
    }

    let drug_upper = req.drug.trim().to_uppercase();
    let event_upper = req.event.trim().to_uppercase();

    // Build 2x2 contingency table from 4 concurrent openFDA queries
    let (a, b, c, d) = get_contingency_counts(&drug_upper, &event_upper).await?;

    // Signal detection using nexcore-vigilance
    use nexcore_vigilance::pv::signals::{
        ContingencyTable, SignalCriteria, calculate_prr, calculate_ror,
    };

    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();

    let prr_result = calculate_prr(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;
    let ror_result = calculate_ror(&table, &criteria)
        .map_err(|e| ApiError::new("CALCULATION_ERROR", e.to_string()))?;

    // Evans criteria: n >= 3, PRR >= 2.0, chi-square >= 3.841
    let chi_sq = prr_result.chi_square.unwrap_or(0.0);
    let signal_detected = a >= 3 && prr_result.point_estimate >= 2.0 && chi_sq >= 3.841;

    Ok(Json(FaersSignalCheckResponse {
        drug: req.drug,
        event: req.event,
        signal_detected,
        prr: (prr_result.point_estimate * 1000.0).round() / 1000.0,
        ror: (ror_result.point_estimate * 1000.0).round() / 1000.0,
        case_count: a,
    }))
}

// ── Internal helpers ─────────────────────────

/// Parse a raw openFDA event result into our structured type
fn parse_event_result(raw: &serde_json::Value) -> Option<FaersResult> {
    let patient = raw.get("patient")?;

    let drugs: Vec<FaersDrug> = patient
        .get("drug")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .take(10)
                .filter_map(|d| {
                    Some(FaersDrug {
                        medicinalproduct: d.get("medicinalproduct")?.as_str()?.to_string(),
                        drugcharacterization: d
                            .get("drugcharacterization")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let reactions: Vec<FaersReaction> = patient
        .get("reaction")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .take(20)
                .filter_map(|r| {
                    Some(FaersReaction {
                        reactionmeddrapt: r.get("reactionmeddrapt")?.as_str()?.to_string(),
                        reactionoutcome: r
                            .get("reactionoutcome")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let serious = raw
        .get("serious")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);

    Some(FaersResult {
        safetyreportid: raw
            .get("safetyreportid")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        receivedate: raw
            .get("receivedate")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        serious,
        patient: FaersPatient {
            drug: drugs,
            reaction: reactions,
        },
    })
}

/// Query openFDA for a count (total matching reports)
async fn get_openfda_count(search: &str) -> Result<u64, ApiError> {
    let url = format!("{OPENFDA_BASE_URL}?search={search}&limit=1");

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::new("UPSTREAM_ERROR", format!("openFDA request failed: {e}")))?;

    // 404 = no results = 0
    if response.status().as_u16() == 404 {
        return Ok(0);
    }

    if !response.status().is_success() {
        return Err(ApiError::new(
            "UPSTREAM_ERROR",
            format!("openFDA returned status {}", response.status()),
        ));
    }

    let data: OpenFdaResponse = response
        .json()
        .await
        .map_err(|e| ApiError::new("PARSE_ERROR", e.to_string()))?;

    Ok(data
        .meta
        .and_then(|m| m.results)
        .and_then(|r| r.total)
        .unwrap_or(0))
}

/// Build a 2x2 contingency table from 4 concurrent openFDA queries
async fn get_contingency_counts(drug: &str, event: &str) -> Result<(u64, u64, u64, u64), ApiError> {
    let search_a = format!(
        "patient.drug.medicinalproduct:{drug}+AND+patient.reaction.reactionmeddrapt:{event}"
    );
    let search_drug = format!("patient.drug.medicinalproduct:{drug}");
    let search_event = format!("patient.reaction.reactionmeddrapt:{event}");
    let search_total = "receivedate:[20040101+TO+20261231]";

    // 4 concurrent API calls
    let (a_res, drug_res, event_res, total_res) = tokio::join!(
        get_openfda_count(&search_a),
        get_openfda_count(&search_drug),
        get_openfda_count(&search_event),
        get_openfda_count(search_total),
    );

    let a = a_res?;
    let drug_total = drug_res?;
    let event_total = event_res?;
    let total = total_res?;

    let b = drug_total.saturating_sub(a);
    let c = event_total.saturating_sub(a);
    let d = total
        .saturating_sub(drug_total)
        .saturating_sub(event_total)
        .saturating_add(a);

    Ok((a, b, c, d))
}

// ── Signal Graph ─────────────────────────────
// Single endpoint returning drug events + signal detection for Observatory 3D.

#[derive(Debug, Deserialize, ToSchema)]
pub struct FaersSignalGraphQuery {
    /// Drug name (generic or brand)
    pub drug: String,
    /// Max events to return (default 15, max 50)
    pub limit: Option<usize>,
    /// Number of top events to run signal detection on (default 5, max 10)
    pub signal_top_n: Option<usize>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersEventWithSignal {
    pub term: String,
    pub count: u64,
    pub prr: Option<f64>,
    pub ror: Option<f64>,
    pub signal_detected: bool,
    pub case_count: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FaersSignalGraphResponse {
    pub drug: String,
    pub events: Vec<FaersEventWithSignal>,
    pub total_reports: u64,
    pub signal_count: usize,
}

/// Get drug events with signal detection in a single call.
///
/// Fetches top adverse events for a drug, then runs disproportionality
/// analysis on the top N events concurrently. Returns everything needed
/// to build a signal graph visualization.
#[utoipa::path(
    get,
    path = "/api/v1/faers/signal-graph",
    tag = "faers",
    params(
        ("drug" = String, Query, description = "Drug name (generic or brand)"),
        ("limit" = Option<usize>, Query, description = "Max events (default 15, max 50)"),
        ("signal_top_n" = Option<usize>, Query, description = "Events to check for signals (default 5, max 10)")
    ),
    responses(
        (status = 200, description = "Drug events with signal detection", body = FaersSignalGraphResponse),
        (status = 400, description = "Missing drug name", body = ApiError),
        (status = 502, description = "openFDA upstream error", body = ApiError)
    )
)]
pub async fn signal_graph(
    Query(params): Query<FaersSignalGraphQuery>,
) -> Result<Json<FaersSignalGraphResponse>, ApiError> {
    if params.drug.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "drug parameter is required",
        ));
    }

    let drug_upper = params.drug.trim().to_uppercase();
    let limit = params.limit.unwrap_or(15).min(50);
    let signal_top_n = params.signal_top_n.unwrap_or(5).min(10);

    // Step 1: Get top adverse events (1 OpenFDA call)
    let url = format!(
        "{OPENFDA_BASE_URL}?search=patient.drug.medicinalproduct:{drug_upper}\
         &count=patient.reaction.reactionmeddrapt.exact&limit={limit}"
    );

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::new("UPSTREAM_ERROR", format!("openFDA request failed: {e}")))?;

    if response.status().as_u16() == 404 {
        return Ok(Json(FaersSignalGraphResponse {
            drug: params.drug,
            events: vec![],
            total_reports: 0,
            signal_count: 0,
        }));
    }

    if !response.status().is_success() {
        return Err(ApiError::new(
            "UPSTREAM_ERROR",
            format!("openFDA returned status {}", response.status()),
        ));
    }

    let data: OpenFdaCountResponse = response.json().await.map_err(|e| {
        ApiError::new("PARSE_ERROR", format!("Failed to parse openFDA response: {e}"))
    })?;

    let raw_events = data.results.unwrap_or_default();
    let total_reports: u64 = raw_events.iter().map(|r| r.count).sum();

    // Step 2: Run signal detection on top N events concurrently
    use nexcore_vigilance::pv::signals::{
        ContingencyTable, SignalCriteria, calculate_prr, calculate_ror,
    };

    let top_terms: Vec<String> = raw_events
        .iter()
        .take(signal_top_n)
        .map(|ev| ev.term.to_uppercase())
        .collect();
    let signal_futures: Vec<_> = top_terms
        .iter()
        .map(|term| get_contingency_counts(&drug_upper, term))
        .collect();

    let signal_results = futures::future::join_all(signal_futures).await;

    // Step 3: Build combined response
    let mut events = Vec::with_capacity(raw_events.len());
    let mut signal_count = 0usize;

    for (i, raw) in raw_events.iter().enumerate() {
        let (prr, ror, detected, case_n) = if i < signal_results.len() {
            match &signal_results[i] {
                Ok((a, b, c, d)) => {
                    let table = ContingencyTable::new(*a, *b, *c, *d);
                    let criteria = SignalCriteria::evans();
                    let prr_r = calculate_prr(&table, &criteria);
                    let ror_r = calculate_ror(&table, &criteria);
                    let (prr_val, chi_sq) = match prr_r {
                        Ok(r) => (r.point_estimate, r.chi_square.unwrap_or(0.0)),
                        Err(_) => (0.0, 0.0),
                    };
                    let ror_val = ror_r.map(|r| r.point_estimate).unwrap_or(0.0);
                    let det = *a >= 3 && prr_val >= 2.0 && chi_sq >= 3.841;
                    (Some(round3(prr_val)), Some(round3(ror_val)), det, Some(*a))
                }
                Err(_) => (None, None, false, None),
            }
        } else {
            (None, None, false, None)
        };

        if detected {
            signal_count += 1;
        }

        events.push(FaersEventWithSignal {
            term: raw.term.clone(),
            count: raw.count,
            prr,
            ror,
            signal_detected: detected,
            case_count: case_n,
        });
    }

    Ok(Json(FaersSignalGraphResponse {
        drug: params.drug,
        events,
        total_reports,
        signal_count,
    }))
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}
