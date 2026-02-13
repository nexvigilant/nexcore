//! Foundation algorithm endpoints

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::{ApiError, ApiResult};

/// Levenshtein distance request
#[derive(Debug, Deserialize, ToSchema)]
pub struct LevenshteinRequest {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
}

/// Levenshtein distance response
#[derive(Debug, Serialize, ToSchema)]
pub struct LevenshteinResponse {
    /// Edit distance
    pub distance: usize,
    /// Similarity ratio (0.0 - 1.0)
    pub similarity: f64,
}

/// Fuzzy search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct FuzzySearchRequest {
    /// Query string
    pub query: String,
    /// Candidates to search
    pub candidates: Vec<String>,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Minimum similarity threshold (0.0 - 1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_limit() -> usize {
    10
}
fn default_threshold() -> f64 {
    0.6
}

/// Fuzzy search response
#[derive(Debug, Serialize, ToSchema)]
pub struct FuzzySearchResponse {
    /// Matched results with scores
    pub matches: Vec<FuzzyMatch>,
}

/// A fuzzy match result
#[derive(Debug, Serialize, ToSchema)]
pub struct FuzzyMatch {
    /// Matched string
    pub text: String,
    /// Similarity score
    pub score: f64,
    /// Index in original candidates
    pub index: usize,
}

/// Bounded Levenshtein distance request
#[derive(Debug, Deserialize, ToSchema)]
pub struct LevenshteinBoundedRequest {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
    /// Maximum distance before early termination
    pub max_distance: usize,
}

/// Bounded Levenshtein distance response
#[derive(Debug, Serialize, ToSchema)]
pub struct LevenshteinBoundedResponse {
    /// Edit distance (null if exceeded)
    pub distance: Option<usize>,
    /// Whether max_distance was exceeded
    pub exceeded: bool,
    /// Similarity ratio (null if exceeded)
    pub similarity: Option<f64>,
}

/// SHA-256 hash request
#[derive(Debug, Deserialize, ToSchema)]
pub struct Sha256Request {
    /// Input to hash
    pub input: String,
}

/// SHA-256 hash response
#[derive(Debug, Serialize, ToSchema)]
pub struct Sha256Response {
    /// Hex-encoded hash
    pub hash: String,
}

/// YAML parse request
#[derive(Debug, Deserialize, ToSchema)]
pub struct YamlParseRequest {
    /// YAML content to parse
    pub content: String,
}

/// Graph topological sort request
#[derive(Debug, Deserialize, ToSchema)]
pub struct GraphTopsortRequest {
    /// Edges as (from, to) pairs
    pub edges: Vec<(String, String)>,
}

/// Graph levels request
#[derive(Debug, Deserialize, ToSchema)]
pub struct GraphLevelsRequest {
    /// Edges as (from, to) pairs
    pub edges: Vec<(String, String)>,
}

/// FSRS review request
#[derive(Debug, Deserialize, ToSchema)]
pub struct FsrsReviewRequest {
    /// Current stability
    pub stability: f64,
    /// Current difficulty
    pub difficulty: f64,
    /// Days since last review
    pub elapsed_days: u32,
    /// Rating (1=Again, 2=Hard, 3=Good, 4=Easy)
    pub rating: u8,
}

/// FSRS review response
#[derive(Debug, Serialize, ToSchema)]
pub struct FsrsReviewResponse {
    /// New stability
    pub stability: f64,
    /// New difficulty
    pub difficulty: f64,
    /// Next review interval in days
    pub interval: u32,
}

/// Foundation router
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/levenshtein", post(levenshtein_handler))
        .route("/levenshtein-bounded", post(levenshtein_bounded_handler))
        .route("/fuzzy-search", post(fuzzy_search_handler))
        .route("/sha256", post(sha256_handler))
        .route("/yaml/parse", post(yaml_parse_handler))
        .route("/graph/topsort", post(graph_topsort_handler))
        .route("/graph/levels", post(graph_levels_handler))
        .route("/fsrs/review", post(fsrs_review_handler))
}

/// Calculate Levenshtein edit distance between two strings
#[utoipa::path(
    post,
    path = "/api/v1/foundation/levenshtein",
    tag = "foundation",
    request_body = LevenshteinRequest,
    responses(
        (status = 200, description = "Distance calculated", body = LevenshteinResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn levenshtein_handler(
    Json(req): Json<LevenshteinRequest>,
) -> ApiResult<LevenshteinResponse> {
    let result = nexcore_vigilance::foundation::levenshtein(&req.source, &req.target);
    Ok(Json(LevenshteinResponse {
        distance: result.distance,
        similarity: result.similarity,
    }))
}

/// Calculate bounded Levenshtein distance with early termination
#[utoipa::path(
    post,
    path = "/api/v1/foundation/levenshtein-bounded",
    tag = "foundation",
    request_body = LevenshteinBoundedRequest,
    responses(
        (status = 200, description = "Bounded distance calculated", body = LevenshteinBoundedResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn levenshtein_bounded_handler(
    Json(req): Json<LevenshteinBoundedRequest>,
) -> ApiResult<LevenshteinBoundedResponse> {
    match nexcore_vigilance::foundation::levenshtein_bounded(
        &req.source,
        &req.target,
        req.max_distance,
    ) {
        Some(distance) => {
            let max_len = req.source.chars().count().max(req.target.chars().count());
            let similarity = if max_len == 0 {
                1.0
            } else {
                ((1.0 - (distance as f64 / max_len as f64)) * 10000.0).round() / 10000.0
            };
            Ok(Json(LevenshteinBoundedResponse {
                distance: Some(distance),
                exceeded: false,
                similarity: Some(similarity),
            }))
        }
        None => Ok(Json(LevenshteinBoundedResponse {
            distance: None,
            exceeded: true,
            similarity: None,
        })),
    }
}

/// Fuzzy search for best matches in a list of candidates
#[utoipa::path(
    post,
    path = "/api/v1/foundation/fuzzy-search",
    tag = "foundation",
    request_body = FuzzySearchRequest,
    responses(
        (status = 200, description = "Search results", body = FuzzySearchResponse),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn fuzzy_search_handler(
    Json(req): Json<FuzzySearchRequest>,
) -> ApiResult<FuzzySearchResponse> {
    let query_len = req.query.chars().count();
    // Use bounded Levenshtein with threshold-derived max_distance to pre-prune
    // candidates that cannot meet the similarity threshold.
    // similarity >= threshold ⟹ distance <= max_len × (1 - threshold)
    let mut matches: Vec<FuzzyMatch> = req
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(idx, candidate)| {
            let c_len = candidate.chars().count();
            let max_len = query_len.max(c_len);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let max_distance = (max_len as f64 * (1.0 - req.threshold)).floor() as usize;

            let distance = nexcore_vigilance::foundation::levenshtein_bounded(
                &req.query,
                candidate,
                max_distance,
            )?;

            let similarity = if max_len == 0 {
                1.0
            } else {
                ((1.0 - (distance as f64 / max_len as f64)) * 10000.0).round() / 10000.0
            };

            if similarity >= req.threshold {
                Some(FuzzyMatch {
                    text: candidate.clone(),
                    score: similarity,
                    index: idx,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by score descending
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(req.limit);

    Ok(Json(FuzzySearchResponse { matches }))
}

/// Calculate SHA-256 hash of input
#[utoipa::path(
    post,
    path = "/api/v1/foundation/sha256",
    tag = "foundation",
    request_body = Sha256Request,
    responses(
        (status = 200, description = "Hash calculated", body = Sha256Response),
        (status = 400, description = "Invalid request", body = super::common::ApiError)
    )
)]
pub async fn sha256_handler(Json(req): Json<Sha256Request>) -> ApiResult<Sha256Response> {
    let result = nexcore_vigilance::foundation::sha256_hash(&req.input);
    Ok(Json(Sha256Response { hash: result.hex }))
}

/// Parse YAML content to JSON
#[utoipa::path(
    post,
    path = "/api/v1/foundation/yaml/parse",
    tag = "foundation",
    request_body = YamlParseRequest,
    responses(
        (status = 200, description = "Parsed JSON", body = serde_json::Value),
        (status = 400, description = "Invalid YAML", body = super::common::ApiError)
    )
)]
pub async fn yaml_parse_handler(
    Json(req): Json<YamlParseRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    nexcore_vigilance::foundation::parse_yaml(&req.content)
        .map(|r| Json(r.data))
        .map_err(|e| ApiError::new("VALIDATION_ERROR", e.to_string()))
}

/// Topologically sort a directed acyclic graph
#[utoipa::path(
    post,
    path = "/api/v1/foundation/graph/topsort",
    tag = "foundation",
    request_body = GraphTopsortRequest,
    responses(
        (status = 200, description = "Sorted nodes", body = Vec<String>),
        (status = 400, description = "Cycle detected", body = super::common::ApiError)
    )
)]
pub async fn graph_topsort_handler(
    Json(req): Json<GraphTopsortRequest>,
) -> Result<Json<Vec<String>>, ApiError> {
    use nexcore_vigilance::foundation::{SkillGraph, SkillNode};

    let mut graph = SkillGraph::new();

    // Collect all unique nodes
    let mut nodes: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (from, to) in &req.edges {
        nodes.insert(from.as_str());
        nodes.insert(to.as_str());
    }

    // Build dependency map
    let mut deps: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for (from, to) in &req.edges {
        deps.entry(to.as_str()).or_default().push(from.as_str());
    }

    // Add nodes to graph
    for node in nodes {
        let node_deps = deps.get(node).cloned().unwrap_or_default();
        graph.add_node(SkillNode::simple(node, node_deps));
    }

    graph
        .topological_sort()
        .map(Json)
        .map_err(|cycle| ApiError::new("VALIDATION_ERROR", format!("Cycle detected: {:?}", cycle)))
}

/// Assign levels to nodes in a DAG
#[utoipa::path(
    post,
    path = "/api/v1/foundation/graph/levels",
    tag = "foundation",
    request_body = GraphLevelsRequest,
    responses(
        (status = 200, description = "Node levels", body = std::collections::HashMap<String, usize>),
        (status = 400, description = "Cycle detected", body = super::common::ApiError)
    )
)]
pub async fn graph_levels_handler(
    Json(req): Json<GraphLevelsRequest>,
) -> Result<Json<std::collections::HashMap<String, usize>>, ApiError> {
    use nexcore_vigilance::foundation::{SkillGraph, SkillNode};

    let mut graph = SkillGraph::new();

    // Collect all unique nodes
    let mut nodes: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (from, to) in &req.edges {
        nodes.insert(from.as_str());
        nodes.insert(to.as_str());
    }

    // Build dependency map
    let mut deps: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for (from, to) in &req.edges {
        deps.entry(to.as_str()).or_default().push(from.as_str());
    }

    // Add nodes to graph
    for node in nodes {
        let node_deps = deps.get(node).cloned().unwrap_or_default();
        graph.add_node(SkillNode::simple(node, node_deps));
    }

    graph
        .level_parallelization()
        .map(|levels| {
            let mut result = std::collections::HashMap::new();
            for (level_idx, level_nodes) in levels.iter().enumerate() {
                for node in level_nodes {
                    result.insert(node.clone(), level_idx);
                }
            }
            Json(result)
        })
        .map_err(|cycle| ApiError::new("VALIDATION_ERROR", format!("Cycle detected: {:?}", cycle)))
}

/// Calculate next review interval using FSRS algorithm
#[utoipa::path(
    post,
    path = "/api/v1/foundation/fsrs/review",
    tag = "foundation",
    request_body = FsrsReviewRequest,
    responses(
        (status = 200, description = "Next review parameters", body = FsrsReviewResponse),
        (status = 400, description = "Invalid parameters", body = super::common::ApiError)
    )
)]
pub async fn fsrs_review_handler(
    Json(req): Json<FsrsReviewRequest>,
) -> ApiResult<FsrsReviewResponse> {
    use nexcore_vigilance::foundation::{Card, CardState, FsrsScheduler, Rating};

    let scheduler = FsrsScheduler::new(None, 0.9);

    let rating = match req.rating {
        1 => Rating::Again,
        2 => Rating::Hard,
        3 => Rating::Good,
        _ => Rating::Easy,
    };

    let card = Card {
        stability: req.stability,
        difficulty: req.difficulty,
        elapsed_days: u64::from(req.elapsed_days),
        scheduled_days: 0,
        reps: 1,
        lapses: 0,
        state: CardState::Review,
    };

    let result = scheduler.review(&card, rating, u64::from(req.elapsed_days));

    Ok(Json(FsrsReviewResponse {
        stability: result.card.stability,
        difficulty: result.card.difficulty,
        interval: result.scheduled_days as u32,
    }))
}
