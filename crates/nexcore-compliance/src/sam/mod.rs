//! SAM.gov Exclusions API Client
//!
//! Async Rust client for querying federal exclusions data.
//!
//! ## API Endpoint
//!
//! Production: `https://api.sam.gov/entity-information/v4/exclusions`
//!
//! ## Rate Limits
//!
//! - Public (no key): 10 requests/day
//! - Registered: 1,000 requests/day
//! - Federal systems: 10,000 requests/day

use std::time::Duration;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// SAM.gov API production base URL.
const SAM_BASE_URL: &str = "https://api.sam.gov/entity-information/v4";

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default page size for paginated requests.
const DEFAULT_PAGE_SIZE: u32 = 10;

// =============================================================================
// Error Types
// =============================================================================

/// Errors from SAM.gov API operations.
#[derive(Debug, nexcore_error::Error)]
pub enum SamError {
    /// Failed to build HTTP client.
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(#[source] reqwest::Error),

    /// Network request failed.
    #[error("SAM.gov API request failed: {0}")]
    NetworkError(#[source] reqwest::Error),

    /// Invalid HTTP response status.
    #[error("SAM.gov returned HTTP {status}: {message}")]
    InvalidResponse { status: u16, message: String },

    /// Failed to parse response JSON.
    #[error("Failed to parse SAM.gov response: {0}")]
    ParseError(#[source] reqwest::Error),

    /// Rate limited by SAM.gov.
    #[error("SAM.gov rate limit exceeded")]
    RateLimited,

    /// Missing API key for authenticated endpoint.
    #[error("SAM.gov API key required for this operation")]
    ApiKeyRequired,
}

// =============================================================================
// Exclusion Types (T2-P: Cross-Domain Primitives)
// =============================================================================

/// Classification of excluded entity.
///
/// T2-P: Reusable across compliance domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExclusionClassification {
    /// Individual person.
    Individual,
    /// Business firm.
    Firm,
    /// Maritime vessel.
    Vessel,
    /// Special entity designation.
    #[serde(rename = "special entity designation")]
    SpecialEntityDesignation,
}

impl ExclusionClassification {
    /// Convert to API query parameter value.
    #[must_use]
    pub fn as_query_param(&self) -> &'static str {
        match self {
            Self::Individual => "Individual",
            Self::Firm => "Firm",
            Self::Vessel => "Vessel",
            Self::SpecialEntityDesignation => "Special Entity Designation",
        }
    }
}

/// Type of exclusion action.
///
/// T2-P: Maps to regulatory compliance categories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExclusionType {
    /// Proceedings pending against entity.
    #[serde(rename = "Ineligible (Proceedings Pending)")]
    IneligiblePending,
    /// Proceedings completed, entity excluded.
    #[serde(rename = "Ineligible (Proceedings Completed)")]
    IneligibleCompleted,
    /// Prohibition or restriction on entity.
    #[serde(rename = "Prohibition/Restriction")]
    ProhibitionRestriction,
    /// Entity voluntarily excluded.
    #[serde(rename = "Voluntary Exclusion")]
    VoluntaryExclusion,
    /// Unknown or other type.
    #[serde(other)]
    Unknown,
}

// =============================================================================
// Response Types
// =============================================================================

/// SAM.gov exclusions API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionsResponse {
    /// Total number of records matching query.
    #[serde(default)]
    pub total_records: u64,
    /// Array of exclusion records.
    #[serde(default)]
    pub exclusion_data: Vec<Exclusion>,
}

/// A single exclusion record from SAM.gov.
///
/// T3: Domain-specific to federal procurement compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exclusion {
    /// Exclusion classification (Individual, Firm, etc.).
    #[serde(default)]
    pub classification: Option<String>,
    /// Name of excluded entity.
    #[serde(default)]
    pub name: Option<String>,
    /// Unique Entity Identifier (UEI) in SAM.
    #[serde(default, rename = "ueiSAM")]
    pub uei_sam: Option<String>,
    /// CAGE code if applicable.
    #[serde(default)]
    pub cage_code: Option<String>,
    /// National Provider Identifier (for healthcare).
    #[serde(default)]
    pub npi: Option<String>,
    /// Type of exclusion.
    #[serde(default)]
    pub exclusion_type: Option<String>,
    /// Exclusion program.
    #[serde(default)]
    pub exclusion_program: Option<String>,
    /// Agency that issued exclusion.
    #[serde(default)]
    pub excluding_agency_code: Option<String>,
    /// Agency name.
    #[serde(default)]
    pub excluding_agency_name: Option<String>,
    /// Date exclusion was activated.
    #[serde(default)]
    pub activation_date: Option<String>,
    /// Date exclusion terminates.
    #[serde(default)]
    pub termination_date: Option<String>,
    /// Record creation date.
    #[serde(default)]
    pub creation_date: Option<String>,
    /// Record last update date.
    #[serde(default)]
    pub update_date: Option<String>,
    /// Primary address.
    #[serde(default)]
    pub address: Option<ExclusionAddress>,
}

/// Address information for excluded entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExclusionAddress {
    /// Address line 1.
    #[serde(default)]
    pub address_line1: Option<String>,
    /// Address line 2.
    #[serde(default)]
    pub address_line2: Option<String>,
    /// City.
    #[serde(default)]
    pub city: Option<String>,
    /// State or province.
    #[serde(default)]
    pub state_province: Option<String>,
    /// ZIP or postal code.
    #[serde(default)]
    pub zip_code: Option<String>,
    /// Country.
    #[serde(default)]
    pub country: Option<String>,
}

// =============================================================================
// Query Builder
// =============================================================================

/// Builder for exclusion queries.
#[derive(Debug, Clone, Default)]
pub struct ExclusionQuery {
    /// Filter by classification.
    classification: Option<ExclusionClassification>,
    /// Filter by entity name (partial match).
    name: Option<String>,
    /// Filter by UEI.
    uei: Option<String>,
    /// Filter by CAGE code.
    cage_code: Option<String>,
    /// Filter by excluding agency code.
    agency_code: Option<String>,
    /// Filter by state/province.
    state: Option<String>,
    /// Page number (1-based).
    page: u32,
    /// Page size (max 10).
    size: u32,
}

impl ExclusionQuery {
    /// Create a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self {
            page: 1,
            size: DEFAULT_PAGE_SIZE,
            ..Default::default()
        }
    }

    /// Filter by classification type.
    #[must_use]
    pub fn classification(mut self, classification: ExclusionClassification) -> Self {
        self.classification = Some(classification);
        self
    }

    /// Filter by entity name (partial match).
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Filter by UEI.
    #[must_use]
    pub fn uei(mut self, uei: impl Into<String>) -> Self {
        self.uei = Some(uei.into());
        self
    }

    /// Filter by CAGE code.
    #[must_use]
    pub fn cage_code(mut self, cage_code: impl Into<String>) -> Self {
        self.cage_code = Some(cage_code.into());
        self
    }

    /// Filter by excluding agency code.
    #[must_use]
    pub fn agency_code(mut self, agency_code: impl Into<String>) -> Self {
        self.agency_code = Some(agency_code.into());
        self
    }

    /// Filter by state/province.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Set page number (1-based).
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page.max(1);
        self
    }

    /// Set page size (max 10).
    #[must_use]
    pub fn size(mut self, size: u32) -> Self {
        self.size = size.min(10).max(1);
        self
    }

    /// Build query parameters for URL.
    fn build_params(&self, api_key: &str) -> Vec<(&'static str, String)> {
        let mut params = vec![("api_key", api_key.to_string())];

        if let Some(ref classification) = self.classification {
            params.push((
                "classification",
                classification.as_query_param().to_string(),
            ));
        }
        if let Some(ref name) = self.name {
            params.push(("exclusionName", name.clone()));
        }
        if let Some(ref uei) = self.uei {
            params.push(("ueiSAM", uei.clone()));
        }
        if let Some(ref cage_code) = self.cage_code {
            params.push(("cageCode", cage_code.clone()));
        }
        if let Some(ref agency_code) = self.agency_code {
            params.push(("excludingAgencyCode", agency_code.clone()));
        }
        if let Some(ref state) = self.state {
            params.push(("stateProvince", state.clone()));
        }

        params.push(("page", self.page.to_string()));
        params.push(("size", self.size.to_string()));

        params
    }
}

// =============================================================================
// SAM.gov Client
// =============================================================================

/// Async client for SAM.gov Exclusions API.
///
/// ## Example
///
/// ```rust,ignore
/// use nexcore_compliance::sam::{SamClient, ExclusionQuery, ExclusionClassification};
///
/// let client = SamClient::new("your-api-key")?;
/// let query = ExclusionQuery::new()
///     .classification(ExclusionClassification::Firm)
///     .state("CA");
/// let exclusions = client.query_exclusions(&query).await?;
/// ```
pub struct SamClient {
    /// HTTP client.
    client: reqwest::Client,
    /// API key for authentication.
    api_key: String,
}

impl SamClient {
    /// Create a new SAM.gov client with API key.
    ///
    /// # Errors
    ///
    /// Returns `SamError::ClientBuild` if the HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>) -> Result<Self, SamError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent("nexcore-compliance/1.0")
            .build()
            .map_err(SamError::ClientBuild)?;

        Ok(Self {
            client,
            api_key: api_key.into(),
        })
    }

    /// Query exclusions from SAM.gov.
    ///
    /// # Errors
    ///
    /// Returns error if the API request fails or response cannot be parsed.
    pub async fn query_exclusions(
        &self,
        query: &ExclusionQuery,
    ) -> Result<ExclusionsResponse, SamError> {
        let url = format!("{}/exclusions", SAM_BASE_URL);
        let params = query.build_params(&self.api_key);

        tracing::debug!(url = %url, params = ?params, "Querying SAM.gov exclusions");

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(SamError::NetworkError)?;

        let status = response.status();

        // Handle rate limiting
        if status.as_u16() == 429 {
            return Err(SamError::RateLimited);
        }

        // Handle other errors
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SamError::InvalidResponse {
                status: status.as_u16(),
                message,
            });
        }

        // Parse response
        response
            .json::<ExclusionsResponse>()
            .await
            .map_err(SamError::ParseError)
    }

    /// Check if an entity is excluded by UEI.
    ///
    /// Returns `Some(Exclusion)` if entity is found in exclusions list.
    ///
    /// # Errors
    ///
    /// Returns error if the API request fails.
    pub async fn check_uei(&self, uei: &str) -> Result<Option<Exclusion>, SamError> {
        let query = ExclusionQuery::new().uei(uei);
        let response = self.query_exclusions(&query).await?;
        Ok(response.exclusion_data.into_iter().next())
    }

    /// Check if an entity is excluded by CAGE code.
    ///
    /// Returns `Some(Exclusion)` if entity is found in exclusions list.
    ///
    /// # Errors
    ///
    /// Returns error if the API request fails.
    pub async fn check_cage(&self, cage_code: &str) -> Result<Option<Exclusion>, SamError> {
        let query = ExclusionQuery::new().cage_code(cage_code);
        let response = self.query_exclusions(&query).await?;
        Ok(response.exclusion_data.into_iter().next())
    }
}

// =============================================================================
// Compliance Signal Integration
// =============================================================================

impl Exclusion {
    /// Check if exclusion is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        // If no termination date, assume active
        let Some(ref term_date) = self.termination_date else {
            return true;
        };

        // Parse termination date and compare to now
        if let Ok(term) = DateTime::parse_from_rfc3339(term_date) {
            term > DateTime::now()
        } else {
            // If unparseable, assume active for safety
            true
        }
    }

    /// Get compliance risk level based on exclusion type.
    ///
    /// Returns a score from 0.0 (low risk) to 1.0 (critical risk).
    #[must_use]
    pub fn risk_score(&self) -> f64 {
        match self.exclusion_type.as_deref() {
            Some("Ineligible (Proceedings Completed)") => 1.0,
            Some("Prohibition/Restriction") => 0.9,
            Some("Ineligible (Proceedings Pending)") => 0.7,
            Some("Voluntary Exclusion") => 0.5,
            _ => 0.3,
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
    fn test_exclusion_classification_query_param() {
        assert_eq!(
            ExclusionClassification::Individual.as_query_param(),
            "Individual"
        );
        assert_eq!(ExclusionClassification::Firm.as_query_param(), "Firm");
        assert_eq!(ExclusionClassification::Vessel.as_query_param(), "Vessel");
    }

    #[test]
    fn test_query_builder() {
        let query = ExclusionQuery::new()
            .classification(ExclusionClassification::Firm)
            .state("CA")
            .page(2)
            .size(5);

        let params = query.build_params("test-key");

        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "api_key" && v == "test-key")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "classification" && v == "Firm")
        );
        assert!(
            params
                .iter()
                .any(|(k, v)| *k == "stateProvince" && v == "CA")
        );
        assert!(params.iter().any(|(k, v)| *k == "page" && v == "2"));
        assert!(params.iter().any(|(k, v)| *k == "size" && v == "5"));
    }

    #[test]
    fn test_query_size_capped() {
        let query = ExclusionQuery::new().size(100);
        assert_eq!(query.size, 10); // Max is 10
    }

    #[test]
    fn test_exclusion_risk_score() {
        let exclusion = Exclusion {
            exclusion_type: Some("Ineligible (Proceedings Completed)".to_string()),
            ..Default::default()
        };
        assert!((exclusion.risk_score() - 1.0).abs() < f64::EPSILON);

        let exclusion = Exclusion {
            exclusion_type: Some("Voluntary Exclusion".to_string()),
            ..Default::default()
        };
        assert!((exclusion.risk_score() - 0.5).abs() < f64::EPSILON);
    }
}

impl Default for Exclusion {
    fn default() -> Self {
        Self {
            classification: None,
            name: None,
            uei_sam: None,
            cage_code: None,
            npi: None,
            exclusion_type: None,
            exclusion_program: None,
            excluding_agency_code: None,
            excluding_agency_name: None,
            activation_date: None,
            termination_date: None,
            creation_date: None,
            update_date: None,
            address: None,
        }
    }
}
