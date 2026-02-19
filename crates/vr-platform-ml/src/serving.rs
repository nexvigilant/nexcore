//! Multi-model inference router for the platform ML engine.
//!
//! Routes inference requests to the appropriate model endpoint based on
//! explicit model selection, platform defaults, or popularity-based fallback.

use serde::{Deserialize, Serialize};
use vr_core::{Money, TenantId};

/// A request for model inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// The specific model to use (or empty for platform default).
    pub model_id: String,
    /// Input feature vector for prediction.
    pub inputs: Vec<f64>,
    /// The tenant making the request (for billing and isolation).
    pub tenant_id: TenantId,
}

/// Response from a model inference call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Predicted values (class probabilities, regression output, etc.).
    pub predictions: Vec<f64>,
    /// The version of the model that produced these predictions.
    pub model_version: String,
    /// Wall-clock latency in milliseconds.
    pub latency_ms: u64,
    /// Optional prediction confidence (model-dependent).
    pub confidence: Option<f64>,
}

/// A registered model endpoint available for serving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEndpoint {
    /// Unique model identifier.
    pub model_id: String,
    /// Model type category (e.g., "activity_prediction", "admet", "toxicity").
    pub model_type: String,
    /// Deployed version tag.
    pub version: String,
    /// Path to the model artifact (S3, GCS, or local path).
    pub artifact_path: String,
    /// Whether this is a platform-provided model (vs. tenant-trained).
    pub is_platform_model: bool,
    /// Per-prediction price (None for included-in-subscription models).
    pub price_per_prediction: Option<Money>,
    /// Rolling average inference latency in milliseconds.
    pub avg_latency_ms: f64,
    /// Total number of inference requests served.
    pub request_count: u64,
}

/// Select the best model endpoint for an inference request.
///
/// Selection priority:
/// 1. If `preferred_model_id` is Some and matches an endpoint, use it.
/// 2. Otherwise, find a platform model matching `model_type`.
/// 3. If no platform model, fall back to the endpoint with highest `request_count`
///    among those matching `model_type`.
///
/// Returns None if no endpoint matches the model_type.
#[must_use]
pub fn select_model<'a>(
    endpoints: &'a [ModelEndpoint],
    preferred_model_id: Option<&str>,
    model_type: &str,
) -> Option<&'a ModelEndpoint> {
    // Priority 1: Explicit model ID match.
    if let Some(preferred_id) = preferred_model_id {
        let found = endpoints.iter().find(|e| e.model_id == preferred_id);
        if found.is_some() {
            return found;
        }
    }

    // Filter to endpoints matching the requested model_type.
    let type_matches: Vec<&ModelEndpoint> = endpoints
        .iter()
        .filter(|e| e.model_type == model_type)
        .collect();

    if type_matches.is_empty() {
        return None;
    }

    // Priority 2: Platform model for this type.
    let platform_model = type_matches.iter().find(|e| e.is_platform_model);
    if let Some(pm) = platform_model {
        return Some(pm);
    }

    // Priority 3: Highest request_count (most popular/proven).
    type_matches.into_iter().max_by_key(|e| e.request_count)
}

/// Estimate the total cost for a batch of predictions.
///
/// If the endpoint has no per-prediction price (included in subscription),
/// returns Money::usd(0).
#[must_use]
pub fn estimate_batch_cost(endpoint: &ModelEndpoint, batch_size: u64) -> Money {
    match endpoint.price_per_prediction {
        Some(price) => price.times(batch_size),
        None => Money::usd(0),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use vr_core::Currency;

    fn make_endpoint(
        model_id: &str,
        model_type: &str,
        is_platform: bool,
        request_count: u64,
        price_cents: Option<u64>,
    ) -> ModelEndpoint {
        ModelEndpoint {
            model_id: model_id.to_string(),
            model_type: model_type.to_string(),
            version: "v1.0.0".to_string(),
            artifact_path: format!("gs://models/{model_id}/v1"),
            is_platform_model: is_platform,
            price_per_prediction: price_cents.map(Money::usd),
            avg_latency_ms: 15.0,
            request_count,
        }
    }

    #[test]
    fn select_by_explicit_id() {
        let endpoints = vec![
            make_endpoint("model-a", "admet", false, 100, Some(5)),
            make_endpoint("model-b", "admet", true, 500, None),
        ];
        let selected = select_model(&endpoints, Some("model-a"), "admet");
        assert_eq!(selected.unwrap().model_id, "model-a");
    }

    #[test]
    fn select_platform_model_as_fallback() {
        let endpoints = vec![
            make_endpoint("tenant-model", "admet", false, 1000, Some(3)),
            make_endpoint("platform-admet", "admet", true, 500, None),
        ];
        let selected = select_model(&endpoints, None, "admet");
        assert_eq!(selected.unwrap().model_id, "platform-admet");
    }

    #[test]
    fn select_most_popular_when_no_platform() {
        let endpoints = vec![
            make_endpoint("model-x", "toxicity", false, 100, Some(5)),
            make_endpoint("model-y", "toxicity", false, 300, Some(5)),
            make_endpoint("model-z", "toxicity", false, 200, Some(5)),
        ];
        let selected = select_model(&endpoints, None, "toxicity");
        assert_eq!(selected.unwrap().model_id, "model-y");
    }

    #[test]
    fn select_returns_none_for_no_match() {
        let endpoints = vec![make_endpoint("model-a", "admet", true, 500, None)];
        let selected = select_model(&endpoints, None, "toxicity");
        assert!(selected.is_none());
    }

    #[test]
    fn select_explicit_id_overrides_type() {
        let endpoints = vec![
            make_endpoint("special", "admet", false, 10, Some(10)),
            make_endpoint("platform", "toxicity", true, 1000, None),
        ];
        // Even though model_type is "toxicity", explicit ID wins
        let selected = select_model(&endpoints, Some("special"), "toxicity");
        assert_eq!(selected.unwrap().model_id, "special");
    }

    #[test]
    fn select_explicit_id_not_found_falls_through() {
        let endpoints = vec![make_endpoint("platform-admet", "admet", true, 500, None)];
        // Preferred ID doesn't exist, falls through to platform model
        let selected = select_model(&endpoints, Some("nonexistent"), "admet");
        assert_eq!(selected.unwrap().model_id, "platform-admet");
    }

    #[test]
    fn batch_cost_with_price() {
        let endpoint = make_endpoint("model-a", "admet", false, 100, Some(5));
        let cost = estimate_batch_cost(&endpoint, 1000);
        assert_eq!(cost.cents(), 5000); // 5 cents * 1000 = $50.00
    }

    #[test]
    fn batch_cost_included_in_subscription() {
        let endpoint = make_endpoint("platform-admet", "admet", true, 500, None);
        let cost = estimate_batch_cost(&endpoint, 10_000);
        assert_eq!(cost.cents(), 0);
    }

    #[test]
    fn batch_cost_zero_batch() {
        let endpoint = make_endpoint("model-a", "admet", false, 100, Some(5));
        let cost = estimate_batch_cost(&endpoint, 0);
        assert_eq!(cost.cents(), 0);
    }

    #[test]
    fn inference_request_serialization() {
        let req = InferenceRequest {
            model_id: "admet-v2".to_string(),
            inputs: vec![0.5, 1.2, -0.3],
            tenant_id: TenantId::new(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: InferenceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.model_id, "admet-v2");
        assert_eq!(back.inputs.len(), 3);
    }

    #[test]
    fn inference_response_serialization() {
        let resp = InferenceResponse {
            predictions: vec![0.85, 0.15],
            model_version: "v2.1.0".to_string(),
            latency_ms: 12,
            confidence: Some(0.92),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: InferenceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.predictions.len(), 2);
        assert_eq!(back.latency_ms, 12);
    }
}
