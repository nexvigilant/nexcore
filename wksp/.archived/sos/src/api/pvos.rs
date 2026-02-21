/// PVOS API client — κ Comparison (KSB scoring) + Σ Sum (aggregated counts)
///
/// Calls /api/v1/academy/ksb/domains on nexcore-api
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use super::{url, ApiError};

/// A KSB domain (one of 15 PVOS layers)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KsbDomain {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ksb_count: u32,
    #[serde(default)]
    pub dominant_primitive: String,
    #[serde(default)]
    pub transfer_confidence: f64,
    #[serde(default)]
    pub pvos_layer: String,
}

/// Fetch all 15 KSB domains
pub async fn fetch_ksb_domains() -> Result<Vec<KsbDomain>, ApiError> {
    let endpoint = url("/api/v1/academy/ksb/domains");
    let resp = Request::get(&endpoint).send().await?;

    if resp.ok() {
        let body = resp.text().await?;
        let parsed: Vec<KsbDomain> = serde_json::from_str(&body)?;
        Ok(parsed)
    } else {
        Err(ApiError {
            message: format!("PVOS API returned {}", resp.status()),
        })
    }
}
