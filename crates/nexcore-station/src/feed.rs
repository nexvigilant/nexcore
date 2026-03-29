//! Observatory feed trait and stub implementation.

use serde::{Deserialize, Serialize};

use crate::error::StationError;
#[cfg(feature = "live-feed")]
use crate::telemetry;

/// Coverage data for a single domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCoverage {
    /// The domain name.
    pub domain: String,
    /// Whether a config exists for this domain.
    pub has_config: bool,
    /// Number of tools in the config.
    pub tool_count: u32,
    /// Number of verified tools in the config.
    pub verified_tool_count: u32,
    /// Whether this is a PV target domain.
    pub is_pv_target: bool,
}

/// Quality metrics for a domain's config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// The domain name.
    pub domain: String,
    /// Whether all tools have complete schemas.
    pub schema_complete: bool,
    /// Whether CSS selectors are present.
    pub selector_present: bool,
    /// Whether the config has been human-verified.
    pub verified: bool,
    /// Composite quality score.
    pub composite_score: f64,
}

/// Trait for fetching Observatory data.
pub trait ObservatoryFeed {
    /// Fetch coverage data for all known domains.
    fn fetch_domain_coverage(&self) -> Result<Vec<DomainCoverage>, StationError>;

    /// Fetch quality metrics for a specific domain.
    fn fetch_quality_metrics(&self, domain: &str) -> Result<QualityMetrics, StationError>;
}

/// Stub implementation returning hardcoded PV target data.
pub struct StubObservatoryFeed;

impl ObservatoryFeed for StubObservatoryFeed {
    fn fetch_domain_coverage(&self) -> Result<Vec<DomainCoverage>, StationError> {
        Ok(vec![
            DomainCoverage {
                domain: "dailymed.nlm.nih.gov".into(),
                has_config: true,
                tool_count: 7,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "api.fda.gov".into(),
                has_config: true,
                tool_count: 7,
                verified_tool_count: 7,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "accessdata.fda.gov".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "clinicaltrials.gov".into(),
                has_config: true,
                tool_count: 15,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "pubmed.ncbi.nlm.nih.gov".into(),
                has_config: true,
                tool_count: 15,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "open.fda.gov".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "eudravigilance.ema.europa.eu".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "vigiaccess.org".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "www.ema.europa.eu".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "meddra.org".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
            DomainCoverage {
                domain: "ich.org".into(),
                has_config: false,
                tool_count: 0,
                verified_tool_count: 0,
                is_pv_target: true,
            },
        ])
    }

    fn fetch_quality_metrics(&self, domain: &str) -> Result<QualityMetrics, StationError> {
        match domain {
            "dailymed.nlm.nih.gov" => Ok(QualityMetrics {
                domain: domain.into(),
                schema_complete: true,
                selector_present: true,
                verified: false,
                composite_score: 0.70,
            }),
            // api.fda.gov (openFDA REST API) — live proxy with full schemas
            "api.fda.gov" => Ok(QualityMetrics {
                domain: domain.into(),
                schema_complete: true,
                selector_present: false,
                verified: true,
                composite_score: 0.65,
            }),
            // accessdata.fda.gov (drug approvals, FAERS web UI) — stub, not yet live
            "accessdata.fda.gov" => Ok(QualityMetrics {
                domain: domain.into(),
                schema_complete: false,
                selector_present: false,
                verified: false,
                composite_score: 0.0,
            }),
            "clinicaltrials.gov" => Ok(QualityMetrics {
                domain: domain.into(),
                schema_complete: true,
                selector_present: true,
                verified: false,
                composite_score: 0.70,
            }),
            "pubmed.ncbi.nlm.nih.gov" => Ok(QualityMetrics {
                domain: domain.into(),
                schema_complete: true,
                selector_present: false,
                verified: false,
                composite_score: 0.35,
            }),
            _ => Err(StationError::DomainNotCovered {
                domain: domain.into(),
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// HttpObservatoryFeed — live feed from WebMCP Hub API
// ---------------------------------------------------------------------------

/// Live Observatory feed fetching quality metrics from the WebMCP Hub API.
///
/// ## API Shape (audited from Observatory notebook)
///
/// ```text
/// GET https://webmcp-hub.com/api/configs/lookup?domain={domain}
/// → {
///     "configs": [{
///       "id": "...",
///       "verified": true/false,
///       "tools": [{
///         "execution": { "selector": "..." },
///         "inputSchema": { "properties": {...} },
///         ...
///       }],
///       "verifiedToolNames": [...],
///       ...
///     }]
///   }
/// ```
///
/// Quality signal derivation:
/// - `schema_complete` = any tool has non-empty `inputSchema.properties`
/// - `selector_present` = any tool has truthy `execution.selector`
/// - `verified` = config-level `verified` field
#[cfg(feature = "live-feed")]
#[derive(Clone)]
pub struct HttpObservatoryFeed {
    client: reqwest::blocking::Client,
    base_url: String,
}

#[cfg(feature = "live-feed")]
impl HttpObservatoryFeed {
    /// Hub API base URL.
    const DEFAULT_BASE: &'static str = "https://webmcp-hub.com/api/configs/lookup";
    const PV_TARGET_DOMAINS: [&'static str; 10] = [
        "dailymed.nlm.nih.gov",
        "accessdata.fda.gov",
        "clinicaltrials.gov",
        "www.ema.europa.eu",
        "pubmed.ncbi.nlm.nih.gov",
        "who-umc.org",
        "drugbank.com",
        "rxnav.nlm.nih.gov",
        "vigiaccess.org",
        "open.fda.gov",
    ];

    /// Create a new feed pointing at the default Hub API.
    pub fn new() -> Result<Self, StationError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("NexVigilant-Station/1.0")
            .build()
            .map_err(|e| StationError::HttpError {
                reason: format!("failed to build HTTP client: {e}"),
            })?;
        Ok(Self {
            client,
            base_url: Self::DEFAULT_BASE.into(),
        })
    }

    /// Create a feed pointing at a custom base URL (for testing).
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, StationError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("NexVigilant-Station/1.0")
            .build()
            .map_err(|e| StationError::HttpError {
                reason: format!("failed to build HTTP client: {e}"),
            })?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    /// Parse the Hub API response into quality metrics for a domain.
    fn parse_hub_response(
        domain: &str,
        body: &serde_json::Value,
    ) -> Result<QualityMetrics, StationError> {
        let configs = body
            .get("configs")
            .and_then(|c| c.as_array())
            .ok_or_else(|| StationError::DomainNotCovered {
                domain: domain.into(),
            })?;

        if configs.is_empty() {
            return Err(StationError::DomainNotCovered {
                domain: domain.into(),
            });
        }

        // Use the first config (primary)
        let cfg = &configs[0];
        let verified = cfg
            .get("verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let tools = cfg
            .get("tools")
            .and_then(|t| t.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        let schema_complete = tools.iter().any(|tool| {
            tool.get("inputSchema")
                .and_then(|s| s.get("properties"))
                .and_then(|p| p.as_object())
                .is_some_and(|obj| !obj.is_empty())
        });

        let selector_present = tools.iter().any(|tool| {
            tool.get("execution")
                .and_then(|e| e.get("selector"))
                .and_then(|s| s.as_str())
                .is_some_and(|s| !s.is_empty())
        });

        use crate::confidence;
        let composite_score =
            confidence::compute_confidence(schema_complete, selector_present, verified);

        Ok(QualityMetrics {
            domain: domain.into(),
            schema_complete,
            selector_present,
            verified,
            composite_score,
        })
    }

    /// Parse the Hub API response into domain coverage stats.
    fn parse_coverage_response(domain: &str, body: &serde_json::Value) -> DomainCoverage {
        let configs = body
            .get("configs")
            .and_then(|c| c.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        let (tool_count, verified_tool_count) = if let Some(cfg) = configs.first() {
            let tools_len = cfg
                .get("tools")
                .and_then(|t| t.as_array())
                .map_or(0_usize, |arr| arr.len());
            let verified_len = cfg
                .get("verifiedToolNames")
                .and_then(|v| v.as_array())
                .map_or(0_usize, |arr| arr.len());
            (
                u32::try_from(tools_len).unwrap_or(u32::MAX),
                u32::try_from(verified_len).unwrap_or(u32::MAX),
            )
        } else {
            (0, 0)
        };

        DomainCoverage {
            domain: domain.into(),
            has_config: !configs.is_empty(),
            tool_count,
            verified_tool_count,
            is_pv_target: true,
        }
    }

    fn fetch_lookup_json(
        &self,
        trace_id: &str,
        domain: &str,
    ) -> Result<serde_json::Value, StationError> {
        let url = format!("{}?domain={}", self.base_url, urlencoding::encode(domain));
        let started = std::time::Instant::now();
        telemetry::emit_feed_http(
            "feed_http_start",
            trace_id,
            domain,
            telemetry::HttpFeedMetadata::default(),
        );

        let response = self.client.get(&url).send().map_err(|e| {
            let reason = format!("GET {url} failed: {e}");
            telemetry::emit_feed_http(
                "feed_http_error",
                trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    http_status: None,
                    error: Some(reason.clone()),
                },
            );
            StationError::HttpError { reason }
        })?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let reason = format!("GET {url} returned {}", response.status());
            telemetry::emit_feed_http(
                "feed_http_error",
                trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    http_status: Some(status_code),
                    error: Some(reason.clone()),
                },
            );
            return Err(StationError::HttpError { reason });
        }

        let status_code = response.status().as_u16();
        let parsed = response.json().map_err(|e| {
            let reason = format!("failed to parse response JSON: {e}");
            telemetry::emit_feed_http(
                "feed_http_parse_error",
                trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    http_status: Some(status_code),
                    error: Some(reason.clone()),
                },
            );
            StationError::HttpError { reason }
        });

        if parsed.is_ok() {
            telemetry::emit_feed_http(
                "feed_http_success",
                trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    http_status: Some(status_code),
                    error: None,
                },
            );
        }

        parsed
    }
}

#[cfg(feature = "live-feed")]
impl ObservatoryFeed for HttpObservatoryFeed {
    fn fetch_domain_coverage(&self) -> Result<Vec<DomainCoverage>, StationError> {
        let mut handles = Vec::with_capacity(Self::PV_TARGET_DOMAINS.len());
        for domain in Self::PV_TARGET_DOMAINS {
            let worker = self.clone();
            let domain_owned = domain.to_string();
            let trace_id = telemetry::current_trace_id().unwrap_or_else(telemetry::new_trace_id);
            let handle = std::thread::spawn(move || {
                let body = worker.fetch_lookup_json(&trace_id, &domain_owned)?;
                Ok::<DomainCoverage, StationError>(Self::parse_coverage_response(
                    &domain_owned,
                    &body,
                ))
            });
            handles.push(handle);
        }

        let mut coverage = Vec::with_capacity(Self::PV_TARGET_DOMAINS.len());
        for handle in handles {
            match handle.join() {
                Ok(Ok(item)) => coverage.push(item),
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(StationError::ResolutionFailed {
                        domain: "station_coverage".into(),
                        detail: "coverage worker thread panicked".into(),
                    });
                }
            }
        }
        Ok(coverage)
    }

    fn fetch_quality_metrics(&self, domain: &str) -> Result<QualityMetrics, StationError> {
        let trace_id = telemetry::current_trace_id().unwrap_or_else(telemetry::new_trace_id);
        let started = std::time::Instant::now();
        let body = self.fetch_lookup_json(&trace_id, domain)?;
        let parsed = Self::parse_hub_response(domain, &body);
        match &parsed {
            Ok(_) => telemetry::emit_feed_http(
                "feed_quality_metrics",
                &trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    ..Default::default()
                },
            ),
            Err(e) => telemetry::emit_feed_http(
                "feed_quality_error",
                &trace_id,
                domain,
                telemetry::HttpFeedMetadata {
                    latency_ms: Some(started.elapsed().as_millis()),
                    error: Some(e.to_string()),
                    ..Default::default()
                },
            ),
        };
        parsed
    }
}

// ---------------------------------------------------------------------------
// NullObservatoryFeed — test double that always returns DomainNotCovered
// ---------------------------------------------------------------------------

/// Test double that always returns `DomainNotCovered` for any domain.
///
/// Used in integration tests to decouple test assertions from
/// StubObservatoryFeed's specific domain coverage.
#[cfg(test)]
pub(crate) struct NullObservatoryFeed;

#[cfg(test)]
impl ObservatoryFeed for NullObservatoryFeed {
    fn fetch_domain_coverage(&self) -> Result<Vec<DomainCoverage>, StationError> {
        Ok(Vec::new())
    }

    fn fetch_quality_metrics(&self, domain: &str) -> Result<QualityMetrics, StationError> {
        Err(StationError::DomainNotCovered {
            domain: domain.into(),
        })
    }
}

// ---------------------------------------------------------------------------
// AlwaysOkFeed — test double that returns Ok metrics for any domain
// ---------------------------------------------------------------------------

/// Test double that always returns Ok quality metrics for any domain.
///
/// Used to test Case 2 (feed has data, registry does not) without
/// depending on StubObservatoryFeed's specific domain list.
#[cfg(test)]
pub(crate) struct AlwaysOkFeed;

#[cfg(test)]
impl ObservatoryFeed for AlwaysOkFeed {
    fn fetch_domain_coverage(&self) -> Result<Vec<DomainCoverage>, StationError> {
        Ok(Vec::new())
    }

    fn fetch_quality_metrics(&self, domain: &str) -> Result<QualityMetrics, StationError> {
        Ok(QualityMetrics {
            domain: domain.into(),
            schema_complete: true,
            selector_present: true,
            verified: false,
            composite_score: 0.70,
        })
    }
}

#[cfg(all(test, feature = "live-feed"))]
mod live_tests {
    use super::*;

    #[test]
    fn parse_hub_response_with_tools() {
        let body = serde_json::json!({
            "configs": [{
                "id": "test-id",
                "verified": true,
                "tools": [{
                    "execution": { "selector": "#main" },
                    "inputSchema": {
                        "properties": { "query": { "type": "string" } }
                    }
                }]
            }]
        });

        let metrics =
            HttpObservatoryFeed::parse_hub_response("test.com", &body).expect("should parse");
        assert!(metrics.schema_complete);
        assert!(metrics.selector_present);
        assert!(metrics.verified);
        assert!((metrics.composite_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_hub_response_empty_configs() {
        let body = serde_json::json!({ "configs": [] });
        let result = HttpObservatoryFeed::parse_hub_response("test.com", &body);
        assert!(result.is_err());
    }

    #[test]
    fn parse_hub_response_no_schemas() {
        let body = serde_json::json!({
            "configs": [{
                "verified": false,
                "tools": [{
                    "execution": {},
                    "inputSchema": {}
                }]
            }]
        });

        let metrics =
            HttpObservatoryFeed::parse_hub_response("test.com", &body).expect("should parse");
        assert!(!metrics.schema_complete);
        assert!(!metrics.selector_present);
        assert!(!metrics.verified);
        assert!((metrics.composite_score - 0.0).abs() < f64::EPSILON);
    }
}

#[cfg(all(test, feature = "live-feed", feature = "integration"))]
mod integration_tests {
    use super::*;

    #[test]
    fn live_fetch_dailymed() {
        let feed = HttpObservatoryFeed::new().expect("client should build");
        let metrics = feed.fetch_quality_metrics("dailymed.nlm.nih.gov");
        // This test hits the live Hub API — it may fail if the API is down.
        // Gated behind "integration" feature so it doesn't run in CI.
        assert!(
            metrics.is_ok(),
            "live fetch for dailymed failed: {metrics:?}"
        );
        let m = metrics.expect("checked above");
        assert_eq!(m.domain, "dailymed.nlm.nih.gov");
    }
}
