/// Signal detection API client — ∂ Boundary (threshold detection) + N Quantity (2x2 table)
///
/// Calls /api/v1/pv/signal/complete on nexcore-api
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{url, ApiError};

/// 2x2 contingency table input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInput {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
}

/// Individual metric result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricResult {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub threshold: f64,
    #[serde(default)]
    pub signal: bool,
    #[serde(default)]
    pub interpretation: String,
}

/// Complete signal detection response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalResponse {
    #[serde(default)]
    pub metrics: Vec<MetricResult>,
    #[serde(default)]
    pub overall_signal: bool,
    #[serde(default)]
    pub summary: String,
}

/// Run complete signal detection (PRR, ROR, IC, EBGM, Chi-squared)
pub async fn detect_signal(input: &SignalInput) -> Result<SignalResponse, ApiError> {
    let endpoint = url(&format!(
        "/api/v1/pv/signal/complete?a={}&b={}&c={}&d={}",
        input.a, input.b, input.c, input.d
    ));

    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: SignalResponse = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("Signal API returned {}", resp.status()),
        })
    }
}
