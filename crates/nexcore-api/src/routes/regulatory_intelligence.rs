//! Regulatory Intelligence endpoints
//!
//! Provides HTTP access to FDA Guidance Documents (2,794+) and the ICH/CIOMS
//! pharmacovigilance glossary (904 terms) that are already exposed as MCP tools
//! but now also available as dedicated REST routes.
//!
//! ## Endpoints
//!
//! ### FDA Guidance Documents
//! - `POST /guidance/search` — Full-text + filter search over 2,794+ guidance docs
//! - `GET  /guidance/:id`    — Retrieve a single doc by slug, prefix, or title fragment
//!
//! ### ICH Glossary
//! - `GET  /ich/glossary/search?q=&limit=&category=` — Search 904 ICH/CIOMS terms
//! - `GET  /ich/glossary/:term`                      — O(1) lookup for a single term

use axum::{
    Json, Router,
    extract::{Path, Query},
    routing::{get, post},
};
use nexcore_fda_guidance::{FdaGuidanceDoc, index as fda_index};
use nexcore_pv_core::regulatory::ich_glossary::{
    self as ich,
    types::{IchCategory, MatchType, TermQuery},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

// ── FDA Guidance: request / response types ───────────────────────────────────

/// Request body for guidance document search.
#[derive(Debug, Deserialize, ToSchema)]
pub struct GuidanceSearchRequest {
    /// Full-text search query (matched against title, topics, products, centers)
    pub query: String,
    /// Filter by FDA center abbreviation (e.g. `"CDER"`, `"CBER"`, `"CDRH"`)
    #[serde(default)]
    pub center: Option<String>,
    /// Filter by product area keyword (e.g. `"Drugs"`, `"Biologics"`)
    #[serde(default)]
    pub product: Option<String>,
    /// Filter by status: `"Final"` or `"Draft"`
    #[serde(default)]
    pub status: Option<String>,
    /// Maximum number of results to return (default: 25, max: 100)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Response for a guidance document search.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuidanceSearchResponse {
    /// Matched guidance documents, ordered by relevance score descending
    pub results: Vec<GuidanceDocSummary>,
    /// Number of results returned
    pub count: usize,
    /// Query used for the search
    pub query: String,
}

/// Condensed representation of an FDA guidance document for search results.
///
/// Contains all fields from [`FdaGuidanceDoc`] re-exported so the OpenAPI schema
/// is fully self-contained within this route file.
#[derive(Debug, Serialize, ToSchema)]
pub struct GuidanceDocSummary {
    /// URL slug used as the document's unique identifier
    pub slug: String,
    /// Plain-text document title
    pub title: String,
    /// Full URL on fda.gov
    pub url: String,
    /// PDF download URL (absent for ~20% of older documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_url: Option<String>,
    /// PDF file size string (e.g., `"291.05 KB"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_size: Option<String>,
    /// Issue date in ISO 8601 format (`YYYY-MM-DD`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<String>,
    /// `"Final"` or `"Draft"`
    pub status: String,
    /// FDA centers (e.g., `["CDER", "CBER"]`)
    pub centers: Vec<String>,
    /// Topic taxonomy (e.g., `["ICH-Quality", "Biosimilars"]`)
    pub topics: Vec<String>,
    /// Regulated product areas (e.g., `["Drugs", "Biologics"]`)
    pub products: Vec<String>,
    /// Whether the document is currently open for public comment
    pub open_for_comment: bool,
    /// Comment period closing date in ISO 8601 (`YYYY-MM-DD`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_close_date: Option<String>,
    /// Document type (e.g., `"Guidance Document"`)
    pub document_type: String,
    /// Last modified timestamp in ISO 8601
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// Federal Register docket number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docket_number: Option<String>,
}

impl From<FdaGuidanceDoc> for GuidanceDocSummary {
    fn from(doc: FdaGuidanceDoc) -> Self {
        Self {
            slug: doc.slug,
            title: doc.title,
            url: doc.url,
            pdf_url: doc.pdf_url,
            pdf_size: doc.pdf_size,
            issue_date: doc.issue_date,
            status: doc.status,
            centers: doc.centers,
            topics: doc.topics,
            products: doc.products,
            open_for_comment: doc.open_for_comment,
            comment_close_date: doc.comment_close_date,
            document_type: doc.document_type,
            last_modified: doc.last_modified,
            docket_number: doc.docket_number,
        }
    }
}

// ── ICH Glossary: request / response types ───────────────────────────────────

/// Query parameters for ICH glossary search.
#[derive(Debug, Deserialize, ToSchema)]
pub struct IchGlossarySearchQuery {
    /// Search text (matched against term names, abbreviations, and optionally definitions)
    pub q: String,
    /// Maximum number of results (default: 20, max: 200)
    pub limit: Option<usize>,
    /// ICH category filter: `"Q"` (Quality), `"S"` (Safety), `"E"` (Efficacy),
    /// `"M"` (Multidisciplinary)
    pub category: Option<String>,
    /// When `true`, also searches within definition text (default: false)
    #[serde(default)]
    pub search_definitions: Option<bool>,
}

/// Response for an ICH glossary search.
#[derive(Debug, Serialize, ToSchema)]
pub struct IchGlossarySearchResponse {
    /// Matched terms, ordered by relevance score (highest first)
    pub results: Vec<IchTermResult>,
    /// Number of terms returned
    pub count: usize,
    /// Query used for the search
    pub query: String,
}

/// A single ICH/CIOMS glossary term with its definition and source metadata.
#[derive(Debug, Serialize, ToSchema)]
pub struct IchTermResult {
    /// Canonical term name (e.g., `"Adverse Drug Reaction"`)
    pub name: String,
    /// Normalized lookup key (e.g., `"adverse_drug_reaction"`)
    pub key: String,
    /// Primary definition text
    pub definition: String,
    /// Abbreviation if applicable (e.g., `"ADR"`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,
    /// CIOMS clarification text (from curly-bracket annotations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clarification: Option<String>,
    /// Whether this is a new entry in the current glossary version
    pub is_new: bool,
    /// Source ICH guideline information
    pub source: IchSourceInfo,
    /// Related terms ("see also" references)
    pub see_also: Vec<String>,
    /// Match type from the search algorithm
    pub match_type: String,
    /// Relevance score (0.0–1.0)
    pub score: f64,
}

/// Source reference for an ICH term definition.
#[derive(Debug, Serialize, ToSchema)]
pub struct IchSourceInfo {
    /// ICH guideline ID (e.g., `"E2A"`, `"Q9(R1)"`)
    pub guideline_id: String,
    /// Full guideline title
    pub guideline_title: String,
    /// ICH category (`"Quality"`, `"Safety"`, `"Efficacy"`, `"Multidisciplinary"`)
    pub category: Option<String>,
    /// Guideline status (e.g., `"Step 4 (final)"`)
    pub status: String,
    /// Publication date
    pub date: String,
    /// Section reference within the guideline
    pub section: String,
    /// URL to the guideline PDF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Response when looking up a single ICH term by name.
#[derive(Debug, Serialize, ToSchema)]
pub struct IchTermLookupResponse {
    /// The resolved term, or `null` if not found
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<IchTermResult>,
    /// Whether a match was found
    pub found: bool,
}

// ── Helper: convert ICH search result to API response type ───────────────────

fn match_type_str(mt: &MatchType) -> &'static str {
    match mt {
        MatchType::Exact => "exact",
        MatchType::Prefix => "prefix",
        MatchType::Contains => "contains",
        MatchType::DefinitionMatch => "definition_match",
        MatchType::Fuzzy => "fuzzy",
    }
}

fn ich_category_str(cat: IchCategory) -> &'static str {
    match cat {
        IchCategory::Quality => "Quality",
        IchCategory::Safety => "Safety",
        IchCategory::Efficacy => "Efficacy",
        IchCategory::Multidisciplinary => "Multidisciplinary",
    }
}

fn parse_ich_category(s: &str) -> Option<IchCategory> {
    match s.to_uppercase().as_str() {
        "Q" | "QUALITY" => Some(IchCategory::Quality),
        "S" | "SAFETY" => Some(IchCategory::Safety),
        "E" | "EFFICACY" => Some(IchCategory::Efficacy),
        "M" | "MULTIDISCIPLINARY" => Some(IchCategory::Multidisciplinary),
        _ => None,
    }
}

fn search_result_to_term_result(
    sr: nexcore_pv_core::regulatory::ich_glossary::types::SearchResult,
) -> IchTermResult {
    let source = IchSourceInfo {
        guideline_id: sr.term.source.guideline_id.to_string(),
        guideline_title: sr.term.source.guideline_title.to_string(),
        category: sr
            .term
            .source
            .category()
            .map(ich_category_str)
            .map(str::to_string),
        status: sr.term.source.status.to_string(),
        date: sr.term.source.date.to_string(),
        section: sr.term.source.section.to_string(),
        url: sr.term.source.url.map(str::to_string),
    };

    IchTermResult {
        name: sr.term.name.to_string(),
        key: sr.term.key.to_string(),
        definition: sr.term.definition.to_string(),
        abbreviation: sr.term.abbreviation.map(str::to_string),
        clarification: sr.term.clarification.map(str::to_string),
        is_new: sr.term.is_new,
        source,
        see_also: sr.term.see_also.iter().map(|s| s.to_string()).collect(),
        match_type: match_type_str(&sr.match_type).to_string(),
        score: sr.score,
    }
}

// ── Router ───────────────────────────────────────────────────────────────────

/// Regulatory Intelligence router. Nested under `/api/v1/regulatory`.
pub fn router() -> Router<crate::ApiState> {
    Router::new()
        // FDA Guidance Documents
        .route("/guidance/search", post(guidance_search))
        .route("/guidance/{id}", get(guidance_get))
        // ICH Glossary — note: /search must be registered BEFORE /:term so the
        // literal segment takes precedence over the wildcard capture.
        .route("/ich/glossary/search", get(ich_glossary_search))
        .route("/ich/glossary/{term}", get(ich_glossary_lookup))
}

// ── Handlers: FDA Guidance ───────────────────────────────────────────────────

/// Search FDA guidance documents.
///
/// Performs scored full-text search across 2,794+ FDA guidance documents
/// embedded at compile time. Results are ordered by relevance score descending.
/// Optional filters narrow by center, product area, and status.
#[utoipa::path(
    post,
    path = "/api/v1/regulatory/guidance/search",
    tag = "regulatory-intelligence",
    request_body = GuidanceSearchRequest,
    responses(
        (status = 200, description = "Guidance document search results", body = GuidanceSearchResponse),
        (status = 400, description = "Validation error — query is empty", body = ApiError),
        (status = 500, description = "Internal index error", body = ApiError),
    )
)]
pub async fn guidance_search(
    Json(body): Json<GuidanceSearchRequest>,
) -> ApiResult<GuidanceSearchResponse> {
    if body.query.trim().is_empty() {
        return Err(ApiError::new("VALIDATION_ERROR", "query must not be empty"));
    }

    let limit = body.limit.unwrap_or(25).min(100);

    let docs = fda_index::search(
        body.query.trim(),
        body.center.as_deref(),
        body.product.as_deref(),
        body.status.as_deref(),
        limit,
    )
    .map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    let query = body.query.trim().to_string();
    let count = docs.len();
    let results = docs.into_iter().map(GuidanceDocSummary::from).collect();

    Ok(Json(GuidanceSearchResponse {
        results,
        count,
        query,
    }))
}

/// Retrieve a single FDA guidance document by identifier.
///
/// Resolution order:
/// 1. Exact slug match
/// 2. Prefix match on slug
/// 3. Case-insensitive partial title match
///
/// Returns `404` when no match is found.
#[utoipa::path(
    get,
    path = "/api/v1/regulatory/guidance/{id}",
    tag = "regulatory-intelligence",
    params(
        ("id" = String, Path, description = "Document slug, slug prefix, or title fragment")
    ),
    responses(
        (status = 200, description = "Guidance document found", body = GuidanceDocSummary),
        (status = 404, description = "Document not found", body = ApiError),
        (status = 500, description = "Internal index error", body = ApiError),
    )
)]
pub async fn guidance_get(Path(id): Path<String>) -> ApiResult<GuidanceDocSummary> {
    if id.trim().is_empty() {
        return Err(ApiError::new("VALIDATION_ERROR", "id must not be empty"));
    }

    let doc =
        fda_index::get(id.trim()).map_err(|e| ApiError::new("INTERNAL_ERROR", e.to_string()))?;

    match doc {
        Some(d) => Ok(Json(GuidanceDocSummary::from(d))),
        None => Err(ApiError::new(
            "NOT_FOUND",
            format!("No FDA guidance document found for identifier '{id}'"),
        )),
    }
}

// ── Handlers: ICH Glossary ───────────────────────────────────────────────────

/// Search ICH/CIOMS pharmacovigilance glossary terms.
///
/// Searches 904 ICH/CIOMS terms using multi-strategy matching (exact, prefix,
/// contains, definition, and fuzzy). Results are ordered by relevance score.
///
/// The `category` parameter accepts single-letter prefixes (`Q`, `S`, `E`, `M`)
/// or full names (`Quality`, `Safety`, `Efficacy`, `Multidisciplinary`).
#[utoipa::path(
    get,
    path = "/api/v1/regulatory/ich/glossary/search",
    tag = "regulatory-intelligence",
    params(
        ("q" = String, Query, description = "Search text"),
        ("limit" = Option<usize>, Query, description = "Max results (default 20, max 200)"),
        ("category" = Option<String>, Query, description = "ICH category filter: Q, S, E, or M"),
        ("search_definitions" = Option<bool>, Query, description = "Also search within definition text (default false)")
    ),
    responses(
        (status = 200, description = "Matching ICH glossary terms", body = IchGlossarySearchResponse),
        (status = 400, description = "Validation error — q is empty or category is invalid", body = ApiError),
    )
)]
pub async fn ich_glossary_search(
    Query(params): Query<IchGlossarySearchQuery>,
) -> ApiResult<IchGlossarySearchResponse> {
    if params.q.trim().is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "q (search query) must not be empty",
        ));
    }

    let limit = params.limit.unwrap_or(20).min(200);

    // Resolve optional category filter
    let category = if let Some(cat_str) = &params.category {
        match parse_ich_category(cat_str) {
            Some(c) => Some(c),
            None => {
                return Err(ApiError::new(
                    "VALIDATION_ERROR",
                    format!(
                        "Invalid category '{}'. Use Q, S, E, or M (or full names Quality/Safety/Efficacy/Multidisciplinary)",
                        cat_str
                    ),
                ));
            }
        }
    } else {
        None
    };

    let search_defs = params.search_definitions.unwrap_or(false);

    let mut query = TermQuery::new(params.q.trim()).limit(limit);
    if let Some(cat) = category {
        query = query.with_category(cat);
    }
    if search_defs {
        query = query.search_definitions();
    }

    let raw_results = ich::search_with_query(query);
    let query_str = params.q.trim().to_string();
    let count = raw_results.len();
    let results = raw_results
        .into_iter()
        .map(search_result_to_term_result)
        .collect();

    Ok(Json(IchGlossarySearchResponse {
        results,
        count,
        query: query_str,
    }))
}

/// Look up a single ICH/CIOMS glossary term by name.
///
/// Uses an O(1) perfect-hash lookup on the normalised term key (lowercase,
/// spaces replaced with underscores). Falls back to a prefix/fuzzy search
/// when an exact key match is not found, returning the best match.
///
/// Returns `found: false` with `term: null` when the term cannot be resolved
/// by either exact lookup or the fallback scored search.
#[utoipa::path(
    get,
    path = "/api/v1/regulatory/ich/glossary/{term}",
    tag = "regulatory-intelligence",
    params(
        ("term" = String, Path, description = "ICH term name (e.g. 'adverse drug reaction' or 'ADR')")
    ),
    responses(
        (status = 200, description = "Term lookup result (found may be false if not in glossary)", body = IchTermLookupResponse),
        (status = 400, description = "Validation error — term is empty", body = ApiError),
    )
)]
pub async fn ich_glossary_lookup(Path(term): Path<String>) -> ApiResult<IchTermLookupResponse> {
    let term_trimmed = term.trim();
    if term_trimmed.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "term path segment must not be empty",
        ));
    }

    // O(1) direct lookup first
    if let Some(t) = ich::lookup_term(term_trimmed) {
        let sr = nexcore_pv_core::regulatory::ich_glossary::types::SearchResult {
            term: t.clone(),
            score: 1.0,
            match_type: MatchType::Exact,
        };
        return Ok(Json(IchTermLookupResponse {
            term: Some(search_result_to_term_result(sr)),
            found: true,
        }));
    }

    // Fall back to a scored search for partial/abbreviation input — take the best match.
    let mut results = ich::search_terms(term_trimmed);
    if results.is_empty() {
        return Ok(Json(IchTermLookupResponse {
            term: None,
            found: false,
        }));
    }

    // search_terms already returns results sorted by score; take the top result.
    let best = results.swap_remove(0);
    Ok(Json(IchTermLookupResponse {
        term: Some(search_result_to_term_result(best)),
        found: true,
    }))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ich_category_single_letter() {
        assert_eq!(parse_ich_category("Q"), Some(IchCategory::Quality));
        assert_eq!(parse_ich_category("q"), Some(IchCategory::Quality));
        assert_eq!(parse_ich_category("S"), Some(IchCategory::Safety));
        assert_eq!(parse_ich_category("E"), Some(IchCategory::Efficacy));
        assert_eq!(
            parse_ich_category("M"),
            Some(IchCategory::Multidisciplinary)
        );
    }

    #[test]
    fn test_parse_ich_category_full_name() {
        assert_eq!(parse_ich_category("Quality"), Some(IchCategory::Quality));
        assert_eq!(parse_ich_category("SAFETY"), Some(IchCategory::Safety));
        assert_eq!(parse_ich_category("efficacy"), Some(IchCategory::Efficacy));
        assert_eq!(
            parse_ich_category("Multidisciplinary"),
            Some(IchCategory::Multidisciplinary)
        );
    }

    #[test]
    fn test_parse_ich_category_invalid() {
        assert_eq!(parse_ich_category("X"), None);
        assert_eq!(parse_ich_category(""), None);
        assert_eq!(parse_ich_category("unknown"), None);
    }

    #[test]
    fn test_match_type_str() {
        assert_eq!(match_type_str(&MatchType::Exact), "exact");
        assert_eq!(match_type_str(&MatchType::Prefix), "prefix");
        assert_eq!(match_type_str(&MatchType::Contains), "contains");
        assert_eq!(
            match_type_str(&MatchType::DefinitionMatch),
            "definition_match"
        );
        assert_eq!(match_type_str(&MatchType::Fuzzy), "fuzzy");
    }

    #[test]
    fn test_ich_category_str() {
        assert_eq!(ich_category_str(IchCategory::Quality), "Quality");
        assert_eq!(ich_category_str(IchCategory::Safety), "Safety");
        assert_eq!(ich_category_str(IchCategory::Efficacy), "Efficacy");
        assert_eq!(
            ich_category_str(IchCategory::Multidisciplinary),
            "Multidisciplinary"
        );
    }

    #[tokio::test]
    async fn test_guidance_search_rejects_empty_query() {
        let body = GuidanceSearchRequest {
            query: "  ".to_string(),
            center: None,
            product: None,
            status: None,
            limit: None,
        };
        match guidance_search(Json(body)).await {
            Err(e) => assert_eq!(e.code, "VALIDATION_ERROR"),
            Ok(_) => panic!("Expected VALIDATION_ERROR for empty query"),
        }
    }

    #[tokio::test]
    async fn test_guidance_search_returns_results() {
        let body = GuidanceSearchRequest {
            query: "pharmacovigilance".to_string(),
            center: None,
            product: None,
            status: None,
            limit: Some(5),
        };
        match guidance_search(Json(body)).await {
            Ok(Json(resp)) => {
                assert!(
                    !resp.results.is_empty(),
                    "Expected results for 'pharmacovigilance'"
                );
                assert!(resp.count <= 5);
                assert_eq!(resp.query, "pharmacovigilance");
            }
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_guidance_search_with_center_filter() {
        let body = GuidanceSearchRequest {
            query: "safety".to_string(),
            center: Some("CDER".to_string()),
            product: None,
            status: None,
            limit: Some(10),
        };
        match guidance_search(Json(body)).await {
            Ok(Json(resp)) => {
                for doc in &resp.results {
                    assert!(
                        doc.centers.iter().any(|c| c == "CDER"),
                        "Expected CDER center in: {:?}",
                        doc.centers
                    );
                }
            }
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_guidance_search_limit_capped_at_100() {
        let body = GuidanceSearchRequest {
            query: "guidance".to_string(),
            center: None,
            product: None,
            status: None,
            limit: Some(9999),
        };
        match guidance_search(Json(body)).await {
            Ok(Json(resp)) => assert!(resp.count <= 100),
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_guidance_get_not_found() {
        match guidance_get(Path("zzz-no-such-slug-exists-xyz".to_string())).await {
            Err(e) => assert_eq!(e.code, "NOT_FOUND"),
            Ok(_) => panic!("Expected NOT_FOUND for nonexistent slug"),
        }
    }

    #[tokio::test]
    async fn test_guidance_get_empty_id() {
        match guidance_get(Path("  ".to_string())).await {
            Err(e) => assert_eq!(e.code, "VALIDATION_ERROR"),
            Ok(_) => panic!("Expected VALIDATION_ERROR for empty id"),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_search_rejects_empty_q() {
        let params = IchGlossarySearchQuery {
            q: "".to_string(),
            limit: None,
            category: None,
            search_definitions: None,
        };
        match ich_glossary_search(Query(params)).await {
            Err(e) => assert_eq!(e.code, "VALIDATION_ERROR"),
            Ok(_) => panic!("Expected VALIDATION_ERROR for empty q"),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_search_returns_results() {
        let params = IchGlossarySearchQuery {
            q: "adverse event".to_string(),
            limit: Some(5),
            category: None,
            search_definitions: None,
        };
        match ich_glossary_search(Query(params)).await {
            Ok(Json(resp)) => {
                assert!(
                    !resp.results.is_empty(),
                    "Expected results for 'adverse event'"
                );
                assert!(resp.count <= 5);
            }
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_search_invalid_category() {
        let params = IchGlossarySearchQuery {
            q: "clinical".to_string(),
            limit: None,
            category: Some("X".to_string()),
            search_definitions: None,
        };
        match ich_glossary_search(Query(params)).await {
            Err(e) => assert_eq!(e.code, "VALIDATION_ERROR"),
            Ok(_) => panic!("Expected VALIDATION_ERROR for invalid category 'X'"),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_search_category_filter() {
        let params = IchGlossarySearchQuery {
            q: "safety".to_string(),
            limit: Some(20),
            category: Some("E".to_string()),
            search_definitions: None,
        };
        match ich_glossary_search(Query(params)).await {
            Ok(Json(resp)) => {
                for term in &resp.results {
                    assert_eq!(
                        term.source.category.as_deref(),
                        Some("Efficacy"),
                        "Expected Efficacy category for term '{}'",
                        term.name
                    );
                }
            }
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_lookup_known_term() {
        match ich_glossary_lookup(Path("adverse drug reaction".to_string())).await {
            Ok(Json(resp)) => {
                assert!(resp.found, "Expected 'adverse drug reaction' to be found");
                match resp.term {
                    Some(t) => assert!(!t.definition.is_empty(), "Definition must not be empty"),
                    None => panic!("Expected term to be Some when found=true"),
                }
            }
            Err(e) => panic!("Unexpected error: {}", e.message),
        }
    }

    /// The lookup handler returns `found: false` only when both the O(1) perfect-hash
    /// lookup and the fallback scored search produce no hits. The fuzzy scorer fires
    /// when Jaccard similarity > 0.5, so the test input must use characters with
    /// near-zero overlap with any ICH term name. A string of repeated 'W' characters
    /// achieves this because 'W' is essentially absent from ICH terminology.
    #[tokio::test]
    async fn test_ich_glossary_lookup_unknown_term() {
        match ich_glossary_lookup(Path("WWWWWWWWWWWWWWWWWWWW".to_string())).await {
            Ok(Json(resp)) => {
                assert!(!resp.found, "Expected no fuzzy match for repeated-W string");
                assert!(resp.term.is_none(), "term must be None when found=false");
            }
            Err(e) => panic!("Unexpected error for unknown term: {}", e.message),
        }
    }

    #[tokio::test]
    async fn test_ich_glossary_lookup_empty_term() {
        match ich_glossary_lookup(Path("  ".to_string())).await {
            Err(e) => assert_eq!(e.code, "VALIDATION_ERROR"),
            Ok(_) => panic!("Expected VALIDATION_ERROR for empty term"),
        }
    }

    #[tokio::test]
    async fn test_guidance_doc_summary_from_conversion() {
        use nexcore_fda_guidance::FdaGuidanceDoc;
        let doc = FdaGuidanceDoc {
            slug: "test-slug".to_string(),
            title: "Test Title".to_string(),
            url: "https://fda.gov/test".to_string(),
            pdf_url: Some("https://fda.gov/test.pdf".to_string()),
            pdf_size: Some("100 KB".to_string()),
            issue_date: Some("2024-01-01".to_string()),
            status: "Final".to_string(),
            centers: vec!["CDER".to_string()],
            topics: vec!["Safety".to_string()],
            products: vec!["Drugs".to_string()],
            docket_number: None,
            docket_url: None,
            open_for_comment: false,
            comment_close_date: None,
            document_type: "Guidance Document".to_string(),
            last_modified: None,
        };
        let summary = GuidanceDocSummary::from(doc);
        assert_eq!(summary.slug, "test-slug");
        assert_eq!(summary.title, "Test Title");
        assert_eq!(summary.status, "Final");
        assert!(summary.centers.contains(&"CDER".to_string()));
    }
}
