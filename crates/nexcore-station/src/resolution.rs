//! Resolution API types — the gate surface for PV AI agents.
//!
//! An agent sends a `ResolutionRequest` (domain + intent) and receives a
//! `ResolutionResponse` with the best tool, its config, a confidence score,
//! and a trust tier derived from Observatory quality metrics.

use nexcore_tov_grounded::Measured;
use serde::{Deserialize, Serialize};

use crate::confidence::compute_confidence;
use crate::telemetry;

/// Trust tier for a resolved tool.
///
/// Non-exhaustive to allow future tiers (e.g., `Deprecated`, `Beta`)
/// without breaking downstream consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    /// Tool has been tested against live data and passes quality gates.
    Verified,
    /// Tool exists but has not been verified against live data.
    Experimental,
    /// No tool available for the requested domain/intent.
    Unavailable,
}

/// Information about a coverage gap — returned when no tool is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapInfo {
    /// Domain that was requested but has no coverage.
    pub domain: String,
    /// Priority of filling this gap (from Observatory gap analysis).
    pub priority: GapPriority,
    /// Estimated time to availability, if known.
    pub eta: Option<String>,
}

/// Gap priority levels from Observatory analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GapPriority {
    /// Critical PV domain — should be filled immediately.
    High,
    /// Important but not blocking core PV workflows.
    Medium,
    /// Nice to have — low agent traffic expected.
    Low,
}

/// A request to resolve the best tool for a PV task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRequest {
    /// Target domain (e.g., "dailymed.nlm.nih.gov").
    pub domain: String,
    /// Agent's intent (e.g., "get adverse reactions for metformin").
    pub intent: String,
}

/// A resolved tool recommendation from Station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResponse {
    /// Name of the resolved tool (e.g., "get-adverse-reactions").
    pub tool_name: String,
    /// Full tool config as JSON (compatible with WebMCP Hub format).
    pub config: serde_json::Value,
    /// Confidence score: weighted sum of schema, selector, and verification signals.
    pub confidence: Measured<f64>,
    /// Trust tier derived from verification status.
    pub trust_tier: TrustTier,
    /// ISO 8601 timestamp of last verification, if verified.
    pub verified_at: Option<String>,
    /// Gap information — populated only when `trust_tier == Unavailable`.
    pub gap: Option<GapInfo>,
}

/// Station client — resolves PV tool requests against Observatory data.
pub struct StationClient<F> {
    feed: F,
}

impl<F: crate::observatory::ObservatoryFeed> StationClient<F> {
    /// Create a new Station client backed by the given Observatory feed.
    pub fn new(feed: F) -> Self {
        Self { feed }
    }

    /// Resolve a tool for the given request.
    ///
    /// Queries the Observatory feed for domain coverage, computes confidence,
    /// and returns the best available tool or a structured gap response.
    pub fn resolve(&self, request: &ResolutionRequest) -> ResolutionResponse {
        let trace_id = telemetry::new_trace_id();
        telemetry::emit_resolve_start(&trace_id, &request.domain);
        let started = std::time::Instant::now();
        let coverage = self.feed.fetch_domain_coverage(&request.domain);

        let (case_id, response) = match coverage {
            Some(cov) => {
                let confidence = compute_confidence(
                    cov.schema_complete,
                    cov.selector_present,
                    cov.verified,
                );

                let trust_tier = if cov.verified {
                    TrustTier::Verified
                } else {
                    TrustTier::Experimental
                };

                (
                    1,
                    ResolutionResponse {
                        tool_name: cov.primary_tool,
                        config: cov.config_snapshot,
                        confidence,
                        trust_tier,
                        verified_at: cov.verified_at,
                        gap: None,
                    },
                )
            }
            None => (
                4,
                ResolutionResponse {
                    tool_name: String::new(),
                    config: serde_json::Value::Null,
                    confidence: Measured::certain(0.0),
                    trust_tier: TrustTier::Unavailable,
                    verified_at: None,
                    gap: Some(GapInfo {
                        domain: request.domain.clone(),
                        priority: GapPriority::Medium,
                        eta: None,
                    }),
                },
            ),
        };

        telemetry::emit_resolve_finish(
            &trace_id,
            &request.domain,
            case_id,
            Some(response.confidence.value),
            None,
            started.elapsed().as_millis(),
            None,
        );

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observatory::StubObservatoryFeed;

    #[test]
    fn resolve_known_domain() {
        let client = StationClient::new(StubObservatoryFeed);
        let resp = client.resolve(&ResolutionRequest {
            domain: "dailymed.nlm.nih.gov".into(),
            intent: "get drug label".into(),
        });
        assert_eq!(resp.trust_tier, TrustTier::Verified);
        assert!(resp.confidence.value > 0.5);
        assert!(resp.gap.is_none());
        assert!(!resp.tool_name.is_empty());
    }

    #[test]
    fn resolve_unknown_domain_returns_gap() {
        let client = StationClient::new(StubObservatoryFeed);
        let resp = client.resolve(&ResolutionRequest {
            domain: "unknown.example.com".into(),
            intent: "anything".into(),
        });
        assert_eq!(resp.trust_tier, TrustTier::Unavailable);
        assert!((resp.confidence.value - 0.0).abs() < f64::EPSILON);
        assert!(resp.gap.is_some());
        let gap = resp.gap.as_ref().map(|g| &g.domain);
        assert_eq!(gap, Some(&"unknown.example.com".to_string()));
    }

    #[test]
    fn trust_tier_serialization_roundtrip() {
        // Validates #[non_exhaustive] enum survives serde roundtrip.
        let tier = TrustTier::Verified;
        let json = serde_json::to_string(&tier).expect("serialize");
        assert_eq!(json, "\"verified\"");
        let back: TrustTier = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, TrustTier::Verified);
    }
}
