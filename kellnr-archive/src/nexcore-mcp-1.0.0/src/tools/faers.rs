//! FAERS tools: FDA Adverse Event Reporting System queries via OpenFDA API
//!
//! Provides real-time access to FAERS data for pharmacovigilance signal detection.
//! Reuses `nexcore-pv` for signal detection calculations.

use crate::params::{
    FaersCompareDrugsParams, FaersDrugEventsParams, FaersSearchParams, FaersSignalParams,
};
use nexcore_vigilance::pv::{
    ContingencyTable, SignalCriteria, calculate_chi_square, calculate_prr, calculate_ror,
};
use once_cell::sync::Lazy;
use reqwest::Client;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;

/// OpenFDA API base URL
const OPENFDA_BASE_URL: &str = "https://api.fda.gov/drug/event.json";

/// Lazy-initialized HTTP client for OpenFDA API
/// Reused across all tool calls to avoid 500-600µs client creation overhead per call.
/// Performance improvement: ~10x faster than creating new client each call.
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(200)
        .tcp_keepalive(Some(std::time::Duration::from_secs(60)))
        .build()
        .unwrap_or_else(|_| Client::new())
});

/// OpenFDA API response meta
///
/// Tier: T3 (Domain-specific OpenFDA response)
/// Grounds to T1 Concepts via Option fields
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct OpenFdaMeta {
    results: Option<OpenFdaResults>,
}

/// OpenFDA API response results metadata
///
/// Tier: T3 (Domain-specific OpenFDA response)
/// Grounds to T1 Concepts via Option<u64>
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct OpenFdaResults {
    total: Option<u64>,
}

/// OpenFDA API response structure
///
/// Tier: T3 (Domain-specific OpenFDA response)
/// Grounds to T1 Concepts via Option fields and JSON values
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct OpenFdaResponse {
    meta: Option<OpenFdaMeta>,
    results: Option<Vec<serde_json::Value>>,
}

/// OpenFDA count response
///
/// Tier: T3 (Domain-specific OpenFDA count)
/// Grounds to T1 Concepts via String and u64
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct CountResult {
    term: String,
    count: u64,
}

/// OpenFDA count response wrapper
///
/// Tier: T3 (Domain-specific OpenFDA response)
/// Grounds to T1 Concepts via Option and Vec
/// Ord: N/A (composite record)
#[derive(Debug, Deserialize)]
struct CountResponse {
    results: Option<Vec<CountResult>>,
}

/// Get total count from an OpenFDA search query
async fn get_count(search: &str) -> Result<u64, McpError> {
    let url = format!("{OPENFDA_BASE_URL}?search={search}&limit=1");

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("HTTP error: {e}"), None))?;

    if response.status().as_u16() == 404 {
        return Ok(0);
    }

    let data: OpenFdaResponse = response
        .json()
        .await
        .map_err(|e| McpError::internal_error(format!("JSON parse error: {e}"), None))?;

    Ok(data
        .meta
        .and_then(|m| m.results)
        .and_then(|r| r.total)
        .unwrap_or(0))
}

/// Search FAERS for adverse events
pub async fn search(params: FaersSearchParams) -> Result<CallToolResult, McpError> {
    let mut search_terms = Vec::new();

    if let Some(ref drug) = params.drug_name {
        let drug_upper = drug.to_uppercase();
        let drug_query = format!(
            "(patient.drug.medicinalproduct:{drug_upper}+OR+patient.drug.openfda.brand_name:{drug_upper}+OR+patient.drug.openfda.generic_name:{drug_upper})"
        );
        search_terms.push(drug_query);
    }

    if let Some(ref reaction) = params.reaction {
        let reaction_upper = reaction.to_uppercase();
        search_terms.push(format!(
            "patient.reaction.reactionmeddrapt:{reaction_upper}"
        ));
    }

    if params.serious.unwrap_or(false) {
        search_terms.push("serious:1".to_string());
    }

    if search_terms.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "Error: At least one search parameter (drug_name or reaction) is required.",
        )]));
    }

    let search = search_terms.join("+AND+");
    let limit = params.limit.unwrap_or(25).min(100);
    let url = format!("{OPENFDA_BASE_URL}?search={search}&limit={limit}");

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("HTTP error: {e}"), None))?;

    if response.status().as_u16() == 404 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"results": [], "total": 0, "message": "No results found"}).to_string(),
        )]));
    }

    if response.status().as_u16() == 429 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Rate limit exceeded. Try again later."}).to_string(),
        )]));
    }

    let data: OpenFdaResponse = response
        .json()
        .await
        .map_err(|e| McpError::internal_error(format!("JSON parse error: {e}"), None))?;

    let total = data
        .meta
        .and_then(|m| m.results)
        .and_then(|r| r.total)
        .unwrap_or(0);

    // ALLOC: Parse and simplify results for user output
    let events: Vec<serde_json::Value> = data
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(parse_event)
        .collect();

    let result = json!({
        "results": events,
        "total": total,
        "returned": events.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Parse a single event from API response
fn parse_event(result: serde_json::Value) -> Option<serde_json::Value> {
    let patient = result.get("patient")?;

    // Extract drug names
    let drugs: Vec<&str> = patient
        .get("drug")?
        .as_array()?
        .iter()
        .filter_map(|d| d.get("medicinalproduct")?.as_str())
        .take(5)
        .collect();

    // Extract reactions
    let reactions: Vec<&str> = patient
        .get("reaction")?
        .as_array()?
        .iter()
        .filter_map(|r| r.get("reactionmeddrapt")?.as_str())
        .take(10)
        .collect();

    Some(json!({
        "report_id": result.get("safetyreportid").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "receive_date": result.get("receivedate").and_then(|v| v.as_str()),
        "drugs": drugs,
        "reactions": reactions,
        "serious": result.get("serious").and_then(|v| v.as_str()) == Some("1"),
        "death": result.get("seriousnessdeath").and_then(|v| v.as_str()) == Some("1"),
        "country": result.get("occurcountry").and_then(|v| v.as_str()),
    }))
}

/// Get top adverse events for a drug
pub async fn drug_events(params: FaersDrugEventsParams) -> Result<CallToolResult, McpError> {
    let drug_upper = params.drug_name.to_uppercase();
    let top_n = params.top_n.unwrap_or(20).min(100);

    let url = format!(
        "{OPENFDA_BASE_URL}?search=patient.drug.medicinalproduct:{drug_upper}&count=patient.reaction.reactionmeddrapt.exact&limit={top_n}"
    );

    let response = HTTP_CLIENT
        .get(&url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("HTTP error: {e}"), None))?;

    if response.status().as_u16() == 404 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"drug": params.drug_name, "events": [], "message": "No data found"}).to_string(),
        )]));
    }

    let data: CountResponse = response
        .json()
        .await
        .map_err(|e| McpError::internal_error(format!("JSON parse error: {e}"), None))?;

    let events: Vec<serde_json::Value> = data
        .results
        .unwrap_or_default()
        .iter()
        .map(|r| json!({"event": r.term, "count": r.count}))
        .collect();

    let total_reports: u64 = events.iter().filter_map(|e| e.get("count")?.as_u64()).sum();

    let result = json!({
        "drug": params.drug_name,
        "events": events,
        "total_reports": total_reports,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get 2x2 contingency table counts for signal detection
/// Uses tokio::join! to run all 4 HTTP requests concurrently (~4x speedup)
async fn get_contingency_counts(
    drug_name: &str,
    event_name: &str,
) -> Result<(u64, u64, u64, u64), McpError> {
    let drug_upper = drug_name.to_uppercase();
    let event_upper = event_name.to_uppercase();

    // Build all search queries
    let search_a = format!(
        "patient.drug.medicinalproduct:{drug_upper}+AND+patient.reaction.reactionmeddrapt:{event_upper}"
    );
    let search_drug = format!("patient.drug.medicinalproduct:{drug_upper}");
    let search_event = format!("patient.reaction.reactionmeddrapt:{event_upper}");
    let search_total = "receivedate:[20040101+TO+20261231]";

    // Execute all 4 HTTP requests concurrently
    let (a_res, drug_res, event_res, total_res) = tokio::join!(
        get_count(&search_a),
        get_count(&search_drug),
        get_count(&search_event),
        get_count(search_total)
    );

    let a = a_res?;
    let drug_total = drug_res?;
    let event_total = event_res?;
    let total = total_res?;

    // Calculate 2x2 table
    let b = drug_total.saturating_sub(a);
    let c = event_total.saturating_sub(a);
    let d = total
        .saturating_sub(drug_total)
        .saturating_sub(event_total)
        .saturating_add(a);

    Ok((a, b, c, d))
}

/// Quick signal check for drug-event pair
pub async fn signal_check(params: FaersSignalParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = get_contingency_counts(&params.drug_name, &params.event_name).await?;

    // Use nexcore-pv for calculations (reuse existing tested code)
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let prr_result = calculate_prr(&table, &criteria);
    let ror_result = calculate_ror(&table, &criteria);
    let chi_sq = calculate_chi_square(&table);

    // Evans criteria: n≥3, PRR≥2, χ²≥3.841
    let signal_detected = a >= 3 && prr_result.point_estimate >= 2.0 && chi_sq >= 3.841;

    let result = json!({
        "drug": params.drug_name,
        "event": params.event_name,
        "case_count": a,
        "prr": (prr_result.point_estimate * 1000.0).round() / 1000.0,
        "ror": (ror_result.point_estimate * 1000.0).round() / 1000.0,
        "chi_square": (chi_sq * 1000.0).round() / 1000.0,
        "signal_detected": signal_detected,
        "signal_criteria": "Evans: n≥3, PRR≥2, χ²≥3.841",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Full disproportionality analysis for drug-event pair
pub async fn disproportionality(params: FaersSignalParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = get_contingency_counts(&params.drug_name, &params.event_name).await?;

    // Use nexcore-pv for calculations
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let prr_result = calculate_prr(&table, &criteria);
    let ror_result = calculate_ror(&table, &criteria);
    let chi_sq = calculate_chi_square(&table);

    // Evans criteria
    let signal_detected = a >= 3 && prr_result.point_estimate >= 2.0 && chi_sq >= 3.841;

    let result = json!({
        "drug": params.drug_name,
        "event": params.event_name,
        "contingency_table": {"a": a, "b": b, "c": c, "d": d},
        "prr": (prr_result.point_estimate * 1000.0).round() / 1000.0,
        "prr_ci_lower": (prr_result.lower_ci * 1000.0).round() / 1000.0,
        "prr_ci_upper": (prr_result.upper_ci * 1000.0).round() / 1000.0,
        "ror": (ror_result.point_estimate * 1000.0).round() / 1000.0,
        "ror_ci_lower": (ror_result.lower_ci * 1000.0).round() / 1000.0,
        "chi_square": (chi_sq * 1000.0).round() / 1000.0,
        "is_signal": signal_detected,
        "signal_criteria": "Evans: PRR≥2, χ²≥3.841, n≥3",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compare adverse event profiles between two drugs
/// O(n²) - set intersection/difference on bounded top_n ≤ 50 events
pub async fn compare_drugs(params: FaersCompareDrugsParams) -> Result<CallToolResult, McpError> {
    let top_n = params.top_n.unwrap_or(15).min(50);

    // Fetch both drug event profiles
    let (data1, data2) = fetch_drug_events_pair(&params.drug1, &params.drug2, top_n).await?;

    // Compute set operations using HashSet O(n) operations
    let result = compute_drug_comparison(&params.drug1, &params.drug2, &data1, &data2);

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Fetch event counts for two drugs concurrently
async fn fetch_drug_events_pair(
    drug1: &str,
    drug2: &str,
    top_n: usize,
) -> Result<(CountResponse, CountResponse), McpError> {
    let drug1_upper = drug1.to_uppercase();
    let drug2_upper = drug2.to_uppercase();

    let url1 = format!(
        "{OPENFDA_BASE_URL}?search=patient.drug.medicinalproduct:{drug1_upper}&count=patient.reaction.reactionmeddrapt.exact&limit={top_n}"
    );
    let url2 = format!(
        "{OPENFDA_BASE_URL}?search=patient.drug.medicinalproduct:{drug2_upper}&count=patient.reaction.reactionmeddrapt.exact&limit={top_n}"
    );

    // Execute both HTTP requests concurrently
    let (resp1, resp2) = tokio::join!(HTTP_CLIENT.get(&url1).send(), HTTP_CLIENT.get(&url2).send());

    let resp1 = resp1.map_err(|e| McpError::internal_error(format!("HTTP error: {e}"), None))?;
    let resp2 = resp2.map_err(|e| McpError::internal_error(format!("HTTP error: {e}"), None))?;

    let data1: CountResponse = if resp1.status().as_u16() == 404 {
        CountResponse { results: None }
    } else {
        resp1
            .json()
            .await
            .map_err(|e| McpError::internal_error(format!("JSON error: {e}"), None))?
    };

    let data2: CountResponse = if resp2.status().as_u16() == 404 {
        CountResponse { results: None }
    } else {
        resp2
            .json()
            .await
            .map_err(|e| McpError::internal_error(format!("JSON error: {e}"), None))?
    };

    Ok((data1, data2))
}

/// Compute comparison between two drug profiles
fn compute_drug_comparison(
    drug1: &str,
    drug2: &str,
    data1: &CountResponse,
    data2: &CountResponse,
) -> serde_json::Value {
    // Extract event names
    let events1: HashSet<&str> = data1
        .results
        .as_ref()
        .map(|r| r.iter().map(|x| x.term.as_str()).collect())
        .unwrap_or_default();
    let events2: HashSet<&str> = data2
        .results
        .as_ref()
        .map(|r| r.iter().map(|x| x.term.as_str()).collect())
        .unwrap_or_default();

    // Set operations
    let common: Vec<&str> = events1.intersection(&events2).copied().collect();
    let unique1: Vec<&str> = events1.difference(&events2).copied().collect();
    let unique2: Vec<&str> = events2.difference(&events1).copied().collect();

    json!({
        "drug1": drug1,
        "drug1_top_events": data1.results.as_ref().map(|r|
            r.iter().map(|x| json!({"event": x.term, "count": x.count})).collect::<Vec<_>>()
        ).unwrap_or_default(),
        "drug2": drug2,
        "drug2_top_events": data2.results.as_ref().map(|r|
            r.iter().map(|x| json!({"event": x.term, "count": x.count})).collect::<Vec<_>>()
        ).unwrap_or_default(),
        "common_events": common,
        "unique_to_drug1": unique1,
        "unique_to_drug2": unique2,
    })
}
