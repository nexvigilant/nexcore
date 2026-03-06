//! Observatory feed — integration point for live quality metrics.
//!
//! The Observatory layer measures WebMCP Hub coverage, schema quality,
//! and verification status. This module defines the trait contract and
//! a stub implementation for development/testing.

use serde::{Deserialize, Serialize};

/// Coverage data for a single domain from the Observatory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCoverage {
    /// The domain (e.g., "dailymed.nlm.nih.gov").
    pub domain: String,
    /// Primary tool name for this domain.
    pub primary_tool: String,
    /// Full config snapshot as JSON.
    pub config_snapshot: serde_json::Value,
    /// Whether the tool has a complete outputSchema.
    pub schema_complete: bool,
    /// Whether the tool has verified CSS selectors.
    pub selector_present: bool,
    /// Whether the tool has been verified against live data.
    pub verified: bool,
    /// ISO 8601 timestamp of last verification.
    pub verified_at: Option<String>,
    /// Total tool count for this domain's config.
    pub tool_count: usize,
}

/// Quality metrics for a domain from the Observatory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// The domain.
    pub domain: String,
    /// Schema completeness ratio (0.0–1.0).
    pub schema_completeness: f64,
    /// Selector coverage ratio (0.0–1.0).
    pub selector_coverage: f64,
    /// Number of tools with outputSchema.
    pub tools_with_schema: usize,
    /// Total tools.
    pub total_tools: usize,
}

/// Trait for Observatory data feeds.
///
/// Implementations fetch real-time quality metrics from the Observatory layer.
/// The stub implementation returns hardcoded data for development; the real
/// implementation will query the Observatory notebook or a cached metrics store.
pub trait ObservatoryFeed {
    /// Fetch coverage data for a specific domain.
    /// Returns `None` if the domain has no config on the hub.
    fn fetch_domain_coverage(&self, domain: &str) -> Option<DomainCoverage>;

    /// Fetch quality metrics for a specific domain.
    /// Returns `None` if no metrics are available.
    fn fetch_quality_metrics(&self, domain: &str) -> Option<QualityMetrics>;
}

/// Stub Observatory feed for development and testing.
///
/// Returns hardcoded data for the 10 PV target domains based on
/// the Observatory notebook's current scan results.
pub struct StubObservatoryFeed;

impl ObservatoryFeed for StubObservatoryFeed {
    fn fetch_domain_coverage(&self, domain: &str) -> Option<DomainCoverage> {
        // Live proxy domains (6 live as of 2026-03-06)
        match domain {
            "dailymed.nlm.nih.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-drugs".into(),
                config_snapshot: serde_json::json!({
                    "domain": "dailymed.nlm.nih.gov",
                    "tools": ["search-drugs", "get-drug-label", "get-adverse-reactions"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 3,
            }),
            "api.fda.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-adverse-events".into(),
                config_snapshot: serde_json::json!({
                    "domain": "api.fda.gov",
                    "tools": ["search-adverse-events", "get-drug-counts", "get-event-outcomes",
                              "get-event-timeline", "get-reporter-breakdown", "get-drug-characterization",
                              "get-indication-counts"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 7,
            }),
            "clinicaltrials.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-trials".into(),
                config_snapshot: serde_json::json!({
                    "domain": "clinicaltrials.gov",
                    "tools": ["search-trials", "get-trial", "get-safety-endpoints",
                              "get-serious-adverse-events", "compare-trial-arms"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 5,
            }),
            "pubmed.ncbi.nlm.nih.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-articles".into(),
                config_snapshot: serde_json::json!({
                    "domain": "pubmed.ncbi.nlm.nih.gov",
                    "tools": ["search-articles", "get-abstract", "get-citations",
                              "search-case-reports", "search-signal-literature"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 5,
            }),
            "rxnav.nlm.nih.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "get-rxcui".into(),
                config_snapshot: serde_json::json!({
                    "domain": "rxnav.nlm.nih.gov",
                    "tools": ["search-drugs", "get-rxcui", "get-interactions",
                              "get-ingredients", "get-drug-classes", "get-ndc"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 6,
            }),
            "open-vigil.fr" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "compute-disproportionality".into(),
                config_snapshot: serde_json::json!({
                    "domain": "open-vigil.fr",
                    "tools": ["compute-disproportionality", "get-top-drugs",
                              "get-top-reactions", "get-case-demographics"]
                }),
                schema_complete: true,
                selector_present: true,
                verified: true,
                verified_at: Some("2026-03-06T00:00:00Z".into()),
                tool_count: 4,
            }),
            // Stub configs (not yet live — experimental tier)
            "accessdata.fda.gov" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-approvals".into(),
                config_snapshot: serde_json::json!({
                    "domain": "accessdata.fda.gov",
                    "tools": ["search-approvals", "search-recalls", "get-orange-book",
                              "get-rems", "get-labeling-changes", "get-approval-history"]
                }),
                schema_complete: true,
                selector_present: false,
                verified: false,
                verified_at: None,
                tool_count: 6,
            }),
            "www.ema.europa.eu" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-medicines".into(),
                config_snapshot: serde_json::json!({
                    "domain": "www.ema.europa.eu",
                    "tools": ["search-medicines", "get-epar", "get-safety-signals",
                              "get-psur-assessment", "get-rmp-summary", "get-referral"]
                }),
                schema_complete: true,
                selector_present: false,
                verified: false,
                verified_at: None,
                tool_count: 6,
            }),
            "vigiaccess.org" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-reports".into(),
                config_snapshot: serde_json::json!({
                    "domain": "vigiaccess.org",
                    "tools": ["search-reports", "get-adverse-reactions",
                              "get-age-distribution", "get-region-distribution",
                              "get-reporter-distribution"]
                }),
                schema_complete: true,
                selector_present: false,
                verified: false,
                verified_at: None,
                tool_count: 5,
            }),
            "www.meddra.org" => Some(DomainCoverage {
                domain: domain.into(),
                primary_tool: "search-terms".into(),
                config_snapshot: serde_json::json!({
                    "domain": "www.meddra.org",
                    "tools": ["search-terms", "get-term-hierarchy", "get-soc-terms", "get-smq"]
                }),
                schema_complete: true,
                selector_present: false,
                verified: false,
                verified_at: None,
                tool_count: 4,
            }),
            _ => None,
        }
    }

    fn fetch_quality_metrics(&self, domain: &str) -> Option<QualityMetrics> {
        // Derive from coverage data — in production these would be independent signals
        let coverage = self.fetch_domain_coverage(domain)?;
        Some(QualityMetrics {
            domain: domain.into(),
            schema_completeness: if coverage.schema_complete {
                1.0
            } else {
                0.0
            },
            selector_coverage: if coverage.selector_present {
                1.0
            } else {
                0.0
            },
            tools_with_schema: coverage.tool_count,
            total_tools: coverage.tool_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_covers_all_live_proxies() {
        let feed = StubObservatoryFeed;
        let live_domains = [
            "dailymed.nlm.nih.gov",
            "api.fda.gov",
            "clinicaltrials.gov",
            "pubmed.ncbi.nlm.nih.gov",
            "rxnav.nlm.nih.gov",
            "open-vigil.fr",
        ];
        for domain in &live_domains {
            let cov = feed.fetch_domain_coverage(domain);
            assert!(cov.is_some(), "Missing coverage for live domain: {domain}");
            let cov = cov.as_ref();
            assert!(
                cov.map_or(false, |c| c.verified),
                "Live domain {domain} should be verified"
            );
        }
    }

    #[test]
    fn stub_covers_accessdata_and_ema() {
        let feed = StubObservatoryFeed;
        assert!(
            feed.fetch_domain_coverage("accessdata.fda.gov")
                .is_some()
        );
        assert!(
            feed.fetch_domain_coverage("www.ema.europa.eu").is_some()
        );
    }

    #[test]
    fn stub_returns_none_for_unknown() {
        let feed = StubObservatoryFeed;
        assert!(feed.fetch_domain_coverage("unknown.com").is_none());
        assert!(feed.fetch_quality_metrics("unknown.com").is_none());
    }

    #[test]
    fn quality_metrics_derive_from_coverage() {
        let feed = StubObservatoryFeed;
        let m = feed
            .fetch_quality_metrics("api.fda.gov")
            .expect("should have metrics");
        assert!((m.schema_completeness - 1.0).abs() < f64::EPSILON);
        assert!((m.selector_coverage - 1.0).abs() < f64::EPSILON);
    }
}
