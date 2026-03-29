//! Station client — bridges StationRegistry configs with Observatory quality metrics.
//!
//! ## Ownership model: owned registry + generic feed
//!
//! StationClient owns the StationRegistry (cheap Vec<StationConfig>) and borrows
//! nothing. This avoids lifetime parameters and keeps the client as a simple value
//! type. If callers need to share a registry across multiple clients, they clone
//! before passing — that's the caller's concern, not the client's.
//!
//! ## Resolution logic (4 cases)
//!
//! | Registry | Feed | Result |
//! |----------|------|--------|
//! | has config | has metrics | Real config + computed confidence |
//! | no config | has metrics | DomainNotCovered error |
//! | has config | no metrics | Config with confidence 0.0, TrustTier::Experimental |
//! | no config | no metrics | DomainNotCovered error |

use nexcore_constants::Measured;

use crate::confidence::compute_confidence;
use crate::error::StationError;
use crate::feed::ObservatoryFeed;
use crate::registry::StationRegistry;
use crate::telemetry;
use crate::types::{GapInfo, ResolutionRequest, ResolutionResponse, TrustTier};

/// Serialize a StationConfig to JSON, surfacing errors instead of silently defaulting.
fn serialize_config(
    config: &crate::config::StationConfig,
    domain: &str,
) -> Result<serde_json::Value, StationError> {
    serde_json::to_value(config).map_err(|e| StationError::SerializationFailed {
        domain: domain.into(),
        detail: e.to_string(),
    })
}

/// Client for the Station resolution API.
///
/// Generic over `F: ObservatoryFeed` so tests can inject stubs while production
/// uses the live Observatory feed.
pub struct StationClient<F> {
    registry: StationRegistry,
    feed: F,
}

impl<F: ObservatoryFeed> StationClient<F> {
    /// Create a new station client with a registry and Observatory feed.
    pub fn new(registry: StationRegistry, feed: F) -> Self {
        Self { registry, feed }
    }

    /// Resolve the best tool for a domain.
    ///
    /// Looks up the domain in the StationRegistry for config data, then queries
    /// the ObservatoryFeed for quality metrics to compute confidence and trust tier.
    pub fn resolve(&self, request: &ResolutionRequest) -> Result<ResolutionResponse, StationError> {
        let domain = &request.domain;
        let trace_id = telemetry::new_trace_id();
        telemetry::emit_resolve_start(&trace_id, domain);
        let started = std::time::Instant::now();
        let registry_entry = self.registry.by_domain(domain);
        let quality =
            telemetry::with_trace_id(&trace_id, || self.feed.fetch_quality_metrics(domain));

        let (case_id, result) = match (registry_entry, quality) {
            // Case 1: domain in both registry and feed — full resolution
            (Some(station_config), Ok(metrics)) => {
                let confidence_score = compute_confidence(
                    metrics.schema_complete,
                    metrics.selector_present,
                    metrics.verified,
                );

                let trust_tier = if metrics.verified {
                    TrustTier::Verified
                } else {
                    TrustTier::Experimental
                };

                match serialize_config(station_config, domain) {
                    Ok(config_value) => {
                        let primary_tool = station_config
                            .tools
                            .first()
                            .map(|t| t.name.clone())
                            .unwrap_or_default();

                        (
                            1,
                            Ok(ResolutionResponse {
                                tool_name: primary_tool,
                                config: config_value,
                                confidence: Measured::certain(confidence_score),
                                trust_tier,
                                verified_at: None,
                                gap: None,
                            }),
                        )
                    }
                    Err(e) => (1, Err(e)),
                }
            }

            // Case 2: domain in feed but NOT in registry — gap response
            (None, Ok(_metrics)) => (
                2,
                Err(StationError::DomainNotCovered {
                    domain: domain.clone(),
                }),
            ),

            // Case 3: domain in registry but NOT in feed — config exists, no quality data
            (Some(station_config), Err(_)) => match serialize_config(station_config, domain) {
                Ok(config_value) => {
                    let primary_tool = station_config
                        .tools
                        .first()
                        .map(|t| t.name.clone())
                        .unwrap_or_default();

                    (
                        3,
                        Ok(ResolutionResponse {
                            tool_name: primary_tool,
                            config: config_value,
                            confidence: Measured::certain(0.0),
                            trust_tier: TrustTier::Experimental,
                            verified_at: None,
                            gap: Some(GapInfo {
                                domain: domain.clone(),
                                priority: "MED".into(),
                                reason:
                                    "Config exists but no Observatory quality metrics available"
                                        .into(),
                            }),
                        }),
                    )
                }
                Err(e) => (3, Err(e)),
            },

            // Case 4: domain in neither — not covered
            (None, Err(_)) => (
                4,
                Err(StationError::DomainNotCovered {
                    domain: domain.clone(),
                }),
            ),
        };

        let latency = started.elapsed().as_millis();
        match &result {
            Ok(resp) => telemetry::emit_resolve_finish(
                &trace_id,
                domain,
                telemetry::ResolutionMetadata {
                    case: case_id,
                    confidence: Some(resp.confidence.value),
                    trust_tier: Some(&resp.trust_tier),
                    latency_ms: latency,
                    error: None,
                },
            ),
            Err(err) => telemetry::emit_resolve_finish(
                &trace_id,
                domain,
                telemetry::ResolutionMetadata {
                    case: case_id,
                    confidence: None,
                    trust_tier: None,
                    latency_ms: latency,
                    error: Some(err.to_string()),
                },
            ),
        };

        result
    }

    /// Access the backing registry.
    #[must_use]
    pub fn registry(&self) -> &StationRegistry {
        &self.registry
    }

    /// Access the Observatory feed.
    #[must_use]
    pub fn feed(&self) -> &F {
        &self.feed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StationBuilder;
    use crate::config::PvVertical;
    use crate::feed::{AlwaysOkFeed, NullObservatoryFeed, StubObservatoryFeed};

    /// Build a test registry with DailyMed and FAERS entries.
    fn test_registry() -> StationRegistry {
        let mut reg = StationRegistry::new();
        reg.add(
            StationBuilder::new(PvVertical::DailyMed, "DailyMed")
                .description("Drug labels")
                .extract_tool("search-drugs", "Search drugs", "/search.cfm")
                .extract_tool("get-label", "Get drug label", "/drugInfo.cfm")
                .build(),
        );
        reg.add(
            StationBuilder::new(PvVertical::Faers, "FAERS")
                .description("Adverse events")
                .extract_tool("search-events", "Search events", "/drug/event.json")
                .build(),
        );
        reg
    }

    // -----------------------------------------------------------------------
    // Tests using StubObservatoryFeed (existing behavior preserved)
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_domain_in_both_registry_and_feed() {
        let client = StationClient::new(test_registry(), StubObservatoryFeed);
        let resp = client
            .resolve(&ResolutionRequest {
                domain: "dailymed.nlm.nih.gov".into(),
                task_hint: None,
            })
            .expect("should resolve");

        assert_eq!(resp.tool_name, "search-drugs");
        assert_ne!(resp.config, serde_json::Value::Null);
        assert!(resp.confidence.value > 0.5);
        assert!(resp.gap.is_none());
    }

    #[test]
    fn resolve_domain_in_feed_not_registry() {
        let client = StationClient::new(test_registry(), AlwaysOkFeed);
        let result = client.resolve(&ResolutionRequest {
            domain: "some-random-domain.org".into(),
            task_hint: None,
        });

        assert!(result.is_err());
        match result {
            Err(StationError::DomainNotCovered { domain }) => {
                assert_eq!(domain, "some-random-domain.org");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn resolve_domain_in_registry_not_feed() {
        // FAERS (api.fda.gov) is NOW in StubObservatoryFeed (D3 fix),
        // so this test uses NullObservatoryFeed to test Case 3 stably.
        let client = StationClient::new(test_registry(), NullObservatoryFeed);
        let resp = client
            .resolve(&ResolutionRequest {
                domain: "api.fda.gov".into(),
                task_hint: None,
            })
            .expect("should resolve with experimental tier");

        assert_eq!(resp.tool_name, "search-events");
        assert_ne!(resp.config, serde_json::Value::Null);
        assert!((resp.confidence.value - 0.0).abs() < f64::EPSILON);
        assert_eq!(resp.trust_tier, TrustTier::Experimental);
        assert!(resp.gap.is_some());
    }

    #[test]
    fn resolve_domain_in_neither() {
        let client = StationClient::new(test_registry(), StubObservatoryFeed);
        let result = client.resolve(&ResolutionRequest {
            domain: "unknown.example.com".into(),
            task_hint: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn registry_accessor() {
        let client = StationClient::new(test_registry(), StubObservatoryFeed);
        assert_eq!(client.registry().config_count(), 2);
    }

    #[test]
    fn config_field_contains_real_tools() {
        let client = StationClient::new(test_registry(), StubObservatoryFeed);
        let resp = client
            .resolve(&ResolutionRequest {
                domain: "dailymed.nlm.nih.gov".into(),
                task_hint: None,
            })
            .expect("should resolve");

        let tools = resp.config.get("tools");
        assert!(tools.is_some(), "config should contain tools array");
        let tools_arr = tools.and_then(|t| t.as_array());
        assert!(tools_arr.is_some());
        assert_eq!(tools_arr.map(|a| a.len()), Some(2));
    }

    // -----------------------------------------------------------------------
    // Tests using NullObservatoryFeed (D2 fix — stub-decoupled)
    // -----------------------------------------------------------------------

    #[test]
    fn null_feed_registry_only_returns_experimental() {
        // Case 3 via NullObservatoryFeed — no dependency on stub domain list
        let client = StationClient::new(test_registry(), NullObservatoryFeed);
        let resp = client
            .resolve(&ResolutionRequest {
                domain: "dailymed.nlm.nih.gov".into(),
                task_hint: None,
            })
            .expect("should resolve with experimental");

        assert_eq!(resp.trust_tier, TrustTier::Experimental);
        assert!((resp.confidence.value - 0.0).abs() < f64::EPSILON);
        assert!(resp.gap.is_some());
        assert_ne!(resp.config, serde_json::Value::Null);
    }

    #[test]
    fn null_feed_neither_returns_error() {
        // Case 4 via NullObservatoryFeed
        let client = StationClient::new(test_registry(), NullObservatoryFeed);
        let result = client.resolve(&ResolutionRequest {
            domain: "unknown.example.com".into(),
            task_hint: None,
        });
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // D3 fix: api.fda.gov now has quality metrics in StubObservatoryFeed
    // -----------------------------------------------------------------------

    #[test]
    fn api_fda_gov_resolves_with_verified_metrics() {
        // api.fda.gov is in both registry (FAERS vertical) and stub feed
        let client = StationClient::new(test_registry(), StubObservatoryFeed);
        let resp = client
            .resolve(&ResolutionRequest {
                domain: "api.fda.gov".into(),
                task_hint: None,
            })
            .expect("should resolve");

        assert_eq!(resp.trust_tier, TrustTier::Verified);
        assert!((resp.confidence.value - 0.65).abs() < f64::EPSILON);
        assert!(resp.gap.is_none());
    }

    #[cfg(all(feature = "live-feed", feature = "integration"))]
    #[test]
    fn live_resolve_dailymed_integration() {
        use crate::feed::HttpObservatoryFeed;

        let feed = HttpObservatoryFeed::new().expect("live feed client should build");
        let client = StationClient::new(test_registry(), feed);
        let result = client.resolve(&ResolutionRequest {
            domain: "dailymed.nlm.nih.gov".into(),
            task_hint: None,
        });

        assert!(result.is_ok(), "live resolve failed: {result:?}");
        let resp = result.expect("checked above");
        assert_eq!(resp.tool_name, "search-drugs");
        assert_ne!(resp.config, serde_json::Value::Null);
        assert!(resp.confidence.value >= 0.0);
    }
}
