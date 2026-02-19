//! Foundation Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Basic algorithms, crypto, and utility parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Levenshtein distance calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LevenshteinParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
}

// ============================================================================
// Edit Distance Framework
// ============================================================================

/// Parameters for generic edit distance computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
    /// Algorithm: "levenshtein" (default), "damerau", "lcs"
    pub algorithm: Option<String>,
}

/// Parameters for similarity check with threshold.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceSimilarityParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
    /// Similarity threshold (default: 0.8)
    pub threshold: Option<f64>,
}

/// Parameters for edit distance with traceback (operation sequence).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceTracebackParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
}

/// Parameters for batch edit distance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceBatchParams {
    /// Query string
    pub query: String,
    /// Candidate strings
    pub candidates: Vec<String>,
    /// Maximum results
    pub limit: Option<usize>,
    /// Minimum similarity threshold
    pub min_similarity: Option<f64>,
    /// Algorithm: "levenshtein" (default), "damerau", "lcs"
    pub algorithm: Option<String>,
}

/// Parameters for cross-domain transfer confidence lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceTransferParams {
    /// Source domain
    pub source_domain: String,
    /// Target domain
    pub target_domain: String,
}

/// Parameters for bounded Levenshtein distance calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LevenshteinBoundedParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
    /// Maximum distance before early termination
    pub max_distance: usize,
}

/// Parameters for fuzzy search
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FuzzySearchParams {
    /// Query string to search for
    pub query: String,
    /// Candidate strings to search against
    pub candidates: Vec<String>,
    /// Maximum number of results to return (default: 5)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Parameters for SHA-256 hashing
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct Sha256Params {
    /// Input string to hash
    pub input: String,
}

/// Parameters for YAML parsing
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct YamlParseParams {
    /// YAML content to parse
    pub content: String,
}

/// Parameters for graph topological sort
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphTopsortParams {
    /// Edges as array of [from, to] pairs
    pub edges: Vec<(String, String)>,
}

/// Parameters for graph parallel levels
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphLevelsParams {
    /// Edges as array of [from, to] pairs
    pub edges: Vec<(String, String)>,
}

/// Parameters for 2x2 Nash equilibrium analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GameTheoryNash2x2Params {
    /// Row player payoff matrix (2x2)
    pub row_payoffs: Vec<Vec<f64>>,
    /// Column player payoff matrix (2x2)
    pub col_payoffs: Vec<Vec<f64>>,
}

/// Parameters for N×M payoff matrix analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgePayoffMatrixParams {
    /// Payoff values as flat array, row-major (rows × cols)
    pub values: Vec<f64>,
    /// Number of rows (player actions)
    pub rows: usize,
    /// Number of columns (opponent responses)
    pub cols: usize,
    /// Optional row labels
    pub row_labels: Option<Vec<String>>,
    /// Optional column labels
    pub col_labels: Option<Vec<String>>,
}

/// Parameters for N×M mixed strategy Nash equilibrium via iterated best response
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeNashSolveParams {
    /// Payoff values as flat array, row-major
    pub values: Vec<f64>,
    /// Number of rows
    pub rows: usize,
    /// Number of columns
    pub cols: usize,
    /// Number of iterations for fictitious play (default 1000)
    pub iterations: Option<usize>,
}

/// Parameters for forge quality score computation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeQualityScoreParams {
    /// Primitives collected (0-16)
    pub primitives_collected: usize,
    /// Enemies killed
    pub enemies_killed: usize,
    /// Total enemies seen
    pub enemies_seen: usize,
    /// Actual turns taken
    pub actual_turns: u32,
    /// Ideal turns (floor × 40)
    pub ideal_turns: u32,
    /// Current HP
    pub current_hp: i32,
    /// Maximum HP
    pub max_hp: i32,
}

/// Parameters for forge code generation from collected primitives and defeated enemies
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeCodeGenerateParams {
    /// Primitive indices that have been collected (0-15)
    pub collected_primitives: Vec<usize>,
    /// Safety keys from defeated enemies (unwrap, panic, unsafe, deadlock, clone, leak)
    pub defeated_enemies: Option<Vec<String>>,
}

/// Parameters for signal theory detection (observed vs expected)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryDetectParams {
    /// Observed count or rate
    pub observed: f64,
    /// Expected count or rate (under null hypothesis)
    pub expected: f64,
    /// Detection threshold (default: 2.0)
    pub threshold: Option<f64>,
}

/// Parameters for signal theory decision matrix (SDT 2×2)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryDecisionMatrixParams {
    /// True positives (hits)
    pub hits: u64,
    /// False negatives (misses)
    pub misses: u64,
    /// False positives (false alarms)
    pub false_alarms: u64,
    /// True negatives (correct rejections)
    pub correct_rejections: u64,
}

/// Parameters for conservation law verification
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryConservationCheckParams {
    /// True positives (hits)
    pub hits: u64,
    /// False negatives (misses)
    pub misses: u64,
    /// False positives (false alarms)
    pub false_alarms: u64,
    /// True negatives (correct rejections)
    pub correct_rejections: u64,
    /// Expected total for L1 verification (optional, defaults to sum of cells)
    pub expected_total: Option<u64>,
    /// Maximum d' for L4 information conservation check (optional)
    pub max_dprime: Option<f64>,
}

/// Parameters for signal theory detection pipeline (multi-stage)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryPipelineParams {
    /// Pipeline label
    pub label: String,
    /// Stages: each has a name, threshold, and phase ("screening" or "confirmation")
    pub stages: Vec<PipelineStageSpec>,
    /// Value to evaluate through the pipeline
    pub value: f64,
}

/// A single stage in a detection pipeline
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PipelineStageSpec {
    /// Stage name (e.g., "PRR screening", "Chi² confirmation")
    pub name: String,
    /// Threshold value for this stage
    pub threshold: f64,
    /// Phase: "screening" or "confirmation" (default: "screening")
    #[serde(default = "default_screening")]
    pub phase: String,
}

fn default_screening() -> String {
    "screening".to_string()
}

/// Parameters for signal theory cascaded threshold evaluation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryCascadeParams {
    /// Thresholds in ascending order (each must be >= previous)
    pub thresholds: Vec<f64>,
    /// Labels for each stage
    pub labels: Vec<String>,
    /// Value to evaluate
    pub value: f64,
}

/// Parameters for signal theory parallel detection
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalTheoryParallelParams {
    /// Threshold for detector 1
    pub threshold_1: f64,
    /// Label for detector 1
    pub label_1: String,
    /// Threshold for detector 2
    pub threshold_2: f64,
    /// Label for detector 2
    pub label_2: String,
    /// Value to evaluate
    pub value: f64,
    /// Mode: "both" (AND) or "either" (OR). Default: "both"
    #[serde(default = "default_both")]
    pub mode: String,
}

fn default_both() -> String {
    "both".to_string()
}

/// Parameters for signal fence evaluation (process + port against default-deny policy)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalFenceEvaluateParams {
    /// Process name to evaluate
    pub process: String,
    /// Remote port to evaluate
    pub port: u16,
    /// Optional remote IP address (default: 10.0.0.1)
    pub remote_addr: Option<String>,
    /// Optional local port (default: 54321)
    pub local_port: Option<u16>,
    /// Allow rules to add before evaluation
    #[serde(default)]
    pub allow_rules: Vec<SignalFenceAllowRule>,
}

/// An allow rule for signal fence evaluation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalFenceAllowRule {
    /// Process name to allow (None = any)
    pub process: Option<String>,
    /// Port to allow (None = any)
    pub port: Option<u16>,
}

/// Parameters for FSRS spaced repetition review
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FsrsReviewParams {
    /// Current stability value
    pub stability: f64,
    /// Current difficulty value (0.0-1.0)
    pub difficulty: f64,
    /// Days since last review
    pub elapsed_days: u32,
    /// Rating: 1=Again, 2=Hard, 3=Good, 4=Easy
    pub rating: u8,
}

/// Parameters for concept grep expansion
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ConceptGrepParams {
    /// Concept to expand (e.g., "Signal Detection", "pharmacovigilance")
    pub concept: String,
    /// Include markdown section marker patterns (default: false)
    #[serde(default)]
    pub sections: bool,
}
