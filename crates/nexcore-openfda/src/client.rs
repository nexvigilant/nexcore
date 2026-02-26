//! Generalized OpenFDA HTTP client with V33 contingency cache.
//!
//! # Design
//!
//! - Single `OpenFdaClient` serves ALL openFDA endpoints via `fetch<T>`.
//! - Response cache keyed by full URL, storing raw JSON text with TTL.
//! - V33 contingency: on API failure, falls back to stale cached response.
//! - Rate-limit detection: 429 responses surface as `OpenFdaError::RateLimited`.
//!
//! # Example
//!
//! ```rust,ignore
//! use nexcore_openfda::client::{OpenFdaClient, QueryParams};
//! use nexcore_openfda::types::drug::DrugEvent;
//!
//! let client = OpenFdaClient::new()?;
//! let params = QueryParams {
//!     search: Some("patient.drug.openfda.brand_name:\"ASPIRIN\"".to_string()),
//!     limit: Some(10),
//!     ..Default::default()
//! };
//! let response = client.fetch::<DrugEvent>("/drug/event.json", &params).await?;
//! println!("total events: {}", response.meta.results.total);
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use nexcore_chrono::DateTime;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use crate::error::OpenFdaError;
use crate::types::common::OpenFdaResponse;

// =============================================================================
// Constants
// =============================================================================

/// OpenFDA REST API base URL.
const OPENFDA_BASE_URL: &str = "https://api.fda.gov";

/// Default HTTP request timeout.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Hard upper limit on results per request (FDA enforces this).
pub const MAX_LIMIT: u32 = 1_000;

/// Cache TTL — 15 minutes.
const CACHE_TTL_SECS: i64 = 900;

// =============================================================================
// QueryParams
// =============================================================================

/// Parameters for an openFDA API request.
///
/// Maps directly to the query string parameters accepted by all openFDA endpoints.
#[derive(Debug, Clone, Default)]
pub struct QueryParams {
    /// Elasticsearch query string (openFDA search syntax).
    ///
    /// Examples:
    /// - `patient.drug.openfda.brand_name:"ASPIRIN"`
    /// - `serious:1+AND+receiptdate:[20220101+TO+20221231]`
    pub search: Option<String>,
    /// Maximum records to return (capped at [`MAX_LIMIT`]).
    pub limit: Option<u32>,
    /// Number of records to skip (pagination).
    pub skip: Option<u32>,
    /// Sort order (e.g., `"receiptdate:desc"`).
    pub sort: Option<String>,
    /// Count field — returns term-count aggregation instead of records.
    pub count: Option<String>,
}

impl QueryParams {
    /// Create params with a search term and limit.
    #[must_use]
    pub fn search(term: impl Into<String>, limit: u32) -> Self {
        Self {
            search: Some(term.into()),
            limit: Some(limit.min(MAX_LIMIT)),
            ..Default::default()
        }
    }

    /// Create params for a term-count aggregation (no records returned).
    #[must_use]
    pub fn count(field: impl Into<String>) -> Self {
        Self {
            count: Some(field.into()),
            limit: Some(MAX_LIMIT),
            ..Default::default()
        }
    }
}

// =============================================================================
// Cache
// =============================================================================

struct CacheEntry {
    /// Raw JSON response body.
    body: String,
    fetched_at: DateTime,
}

impl CacheEntry {
    fn is_fresh(&self) -> bool {
        DateTime::now()
            .signed_duration_since(self.fetched_at)
            .num_seconds()
            < CACHE_TTL_SECS
    }
}

// =============================================================================
// OpenFdaClient
// =============================================================================

/// Async client for all openFDA REST endpoints.
///
/// Thread-safe; clone or wrap in `Arc` for sharing across tasks.
pub struct OpenFdaClient {
    client: reqwest::Client,
    /// URL → raw JSON cache.
    cache: Arc<RwLock<BTreeMap<String, CacheEntry>>>,
    api_key: Option<String>,
}

impl OpenFdaClient {
    /// Create a new client with default settings.
    ///
    /// # Errors
    ///
    /// Returns [`OpenFdaError::ClientBuild`] if the underlying HTTP client fails to initialise.
    pub fn new() -> Result<Self, OpenFdaError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent("nexcore-openfda/1.0 (NexVigilant Pharmacovigilance)")
            .build()
            .map_err(OpenFdaError::ClientBuild)?;

        Ok(Self {
            client,
            cache: Arc::new(RwLock::new(BTreeMap::new())),
            api_key: None,
        })
    }

    /// Create a new client with an API key for higher rate limits.
    ///
    /// # Errors
    ///
    /// Returns [`OpenFdaError::ClientBuild`] if the HTTP client fails to initialise.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self, OpenFdaError> {
        let mut c = Self::new()?;
        c.api_key = Some(api_key.into());
        Ok(c)
    }

    /// Fetch results from any openFDA endpoint and deserialize into `T`.
    ///
    /// Implements **V33 contingency**: on network failure, falls back to the
    /// most recent cached response (even if stale) rather than returning an error.
    ///
    /// # Errors
    ///
    /// Returns error only when the API call fails AND no cached response exists.
    pub async fn fetch<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &QueryParams,
    ) -> Result<OpenFdaResponse<T>, nexcore_error::NexError> {
        let url = self.build_url(endpoint, params);

        // --- Fresh cache hit ---
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&url) {
                if entry.is_fresh() {
                    tracing::debug!(url = %url, "openFDA cache hit (fresh)");
                    let response: OpenFdaResponse<T> =
                        serde_json::from_str(&entry.body).map_err(OpenFdaError::ParseError)?;
                    return Ok(response);
                }
            }
        }

        // --- Live fetch ---
        let body = match self.do_fetch(&url).await {
            Ok(body) => body,
            Err(api_err) => {
                // V33 contingency: fall back to stale cache on any API error.
                let cache = self.cache.read().await;
                if let Some(entry) = cache.get(&url) {
                    tracing::warn!(
                        error = %api_err,
                        cache_age_secs = DateTime::now()
                            .signed_duration_since(entry.fetched_at)
                            .num_seconds(),
                        "openFDA API failed — serving stale cache (V33 contingency)"
                    );
                    let response: OpenFdaResponse<T> =
                        serde_json::from_str(&entry.body).map_err(OpenFdaError::ParseError)?;
                    return Ok(response);
                }
                return Err(nexcore_error::NexError::from(api_err));
            }
        };

        // Parse and cache.
        let response: OpenFdaResponse<T> =
            serde_json::from_str(&body).map_err(OpenFdaError::ParseError)?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(
                url,
                CacheEntry {
                    body,
                    fetched_at: DateTime::now(),
                },
            );
        }

        Ok(response)
    }

    /// Clear all cached responses.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Number of entries currently in the cache.
    pub async fn cache_len(&self) -> usize {
        self.cache.read().await.len()
    }

    // -------------------------------------------------------------------------
    // Test helpers

    /// Inject a synthetic cache entry (test-only, used to seed the cache without a real HTTP call).
    #[cfg(test)]
    pub async fn inject_cache_entry(&self, url: impl Into<String>, body: impl Into<String>) {
        let mut cache = self.cache.write().await;
        cache.insert(
            url.into(),
            CacheEntry {
                body: body.into(),
                fetched_at: DateTime::now(),
            },
        );
    }

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    async fn do_fetch(&self, url: &str) -> Result<String, OpenFdaError> {
        tracing::debug!(url = %url, "openFDA live fetch");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(OpenFdaError::NetworkError)?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after_secs = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(OpenFdaError::RateLimited { retry_after_secs });
        }

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(OpenFdaError::InvalidResponse {
                status: status.as_u16(),
                message,
            });
        }

        response.text().await.map_err(OpenFdaError::NetworkError)
    }

    fn build_url(&self, endpoint: &str, params: &QueryParams) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(ref search) = params.search {
            parts.push(format!("search={search}"));
        }
        if let Some(limit) = params.limit {
            parts.push(format!("limit={}", limit.min(MAX_LIMIT)));
        }
        if let Some(skip) = params.skip {
            parts.push(format!("skip={skip}"));
        }
        if let Some(ref sort) = params.sort {
            parts.push(format!("sort={sort}"));
        }
        if let Some(ref count) = params.count {
            parts.push(format!("count={count}"));
        }
        if let Some(ref key) = self.api_key {
            parts.push(format!("api_key={key}"));
        }

        if parts.is_empty() {
            format!("{OPENFDA_BASE_URL}{endpoint}")
        } else {
            format!("{OPENFDA_BASE_URL}{endpoint}?{}", parts.join("&"))
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_creation_succeeds() {
        let result = OpenFdaClient::new();
        assert!(result.is_ok());
    }

    #[test]
    fn build_url_no_params() {
        let client = OpenFdaClient::new().unwrap_or_else(|e| panic!("{e}"));
        let params = QueryParams::default();
        let url = client.build_url("/drug/event.json", &params);
        assert_eq!(url, "https://api.fda.gov/drug/event.json");
    }

    #[test]
    fn build_url_with_search_and_limit() {
        let client = OpenFdaClient::new().unwrap_or_else(|e| panic!("{e}"));
        let params = QueryParams {
            search: Some("aspirin".to_string()),
            limit: Some(10),
            ..Default::default()
        };
        let url = client.build_url("/drug/event.json", &params);
        assert!(url.contains("search=aspirin"));
        assert!(url.contains("limit=10"));
        assert!(url.contains("api.fda.gov/drug/event.json"));
    }

    #[test]
    fn build_url_limit_capped_at_max() {
        let client = OpenFdaClient::new().unwrap_or_else(|e| panic!("{e}"));
        let params = QueryParams {
            limit: Some(99_999),
            ..Default::default()
        };
        let url = client.build_url("/drug/event.json", &params);
        assert!(url.contains("limit=1000"));
        assert!(!url.contains("99999"));
    }

    #[test]
    fn build_url_with_api_key() {
        let client = OpenFdaClient::with_api_key("test_key_abc").unwrap_or_else(|e| panic!("{e}"));
        let params = QueryParams::default();
        let url = client.build_url("/drug/event.json", &params);
        assert!(url.contains("api_key=test_key_abc"));
    }

    #[test]
    fn build_url_all_params() {
        let client = OpenFdaClient::new().unwrap_or_else(|e| panic!("{e}"));
        let params = QueryParams {
            search: Some("serious:1".to_string()),
            limit: Some(50),
            skip: Some(100),
            sort: Some("receiptdate:desc".to_string()),
            count: None,
        };
        let url = client.build_url("/drug/event.json", &params);
        assert!(url.contains("search=serious:1"));
        assert!(url.contains("limit=50"));
        assert!(url.contains("skip=100"));
        assert!(url.contains("sort=receiptdate:desc"));
    }

    #[test]
    fn query_params_search_constructor() {
        let p = QueryParams::search("aspirin", 25);
        assert_eq!(p.search.as_deref(), Some("aspirin"));
        assert_eq!(p.limit, Some(25));
    }

    #[test]
    fn query_params_count_constructor() {
        let p = QueryParams::count("patient.reaction.reactionmeddrapt.exact");
        assert!(p.count.is_some());
        assert_eq!(p.limit, Some(MAX_LIMIT));
    }

    #[tokio::test]
    async fn cache_starts_empty() {
        let client = OpenFdaClient::new().unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(client.cache_len().await, 0);
    }

    #[tokio::test]
    async fn clear_cache_empties_non_empty_cache() {
        let client_result = OpenFdaClient::new();
        assert!(client_result.is_ok(), "client construction must succeed");
        let client = client_result.ok().unwrap_or_else(|| {
            // Safety: asserted is_ok() above; this branch is unreachable in practice.
            std::process::abort()
        });
        // Seed the cache with two synthetic entries.
        client
            .inject_cache_entry("https://api.fda.gov/drug/event.json?search=aspirin", "{}")
            .await;
        client
            .inject_cache_entry(
                "https://api.fda.gov/device/510k.json?search=pacemaker",
                "{}",
            )
            .await;
        assert_eq!(
            client.cache_len().await,
            2,
            "cache must have 2 entries before clear"
        );
        // Now clear and verify all entries are removed.
        client.clear_cache().await;
        assert_eq!(
            client.cache_len().await,
            0,
            "cache must be empty after clear"
        );
    }
}
