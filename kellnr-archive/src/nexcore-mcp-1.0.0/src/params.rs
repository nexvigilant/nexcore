//! Parameter structs for MCP tools
//!
//! All parameter types derive `JsonSchema` for automatic MCP schema generation.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// Foundation Parameters
// ============================================================================

/// Parameters for Levenshtein distance calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LevenshteinParams {
    /// Source string
    pub source: String,
    /// Target string
    pub target: String,
}

/// Parameters for bounded Levenshtein distance calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct Sha256Params {
    /// Input string to hash
    pub input: String,
}

/// Parameters for YAML parsing
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct YamlParseParams {
    /// YAML content to parse
    pub content: String,
}

/// Parameters for graph topological sort
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphTopsortParams {
    /// Edges as array of [from, to] pairs
    pub edges: Vec<(String, String)>,
}

/// Parameters for graph parallel levels
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphLevelsParams {
    /// Edges as array of [from, to] pairs
    pub edges: Vec<(String, String)>,
}

/// Parameters for 2x2 Nash equilibrium analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GameTheoryNash2x2Params {
    /// Row player payoff matrix (2x2)
    pub row_payoffs: Vec<Vec<f64>>,
    /// Column player payoff matrix (2x2)
    pub col_payoffs: Vec<Vec<f64>>,
}

/// Parameters for N×M payoff matrix analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Quantity, Comparison, Mapping) via matrix structure.
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Recursion, Comparison, Quantity) via iterative convergence.
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
/// Tier: T2-C (Quantity + Comparison + Boundary)
/// Grounds to T1 Concepts (N, κ, ∂) via weighted scoring.
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Mapping, Sequence, Existence) via template selection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeCodeGenerateParams {
    /// Primitive indices that have been collected (0-15)
    pub collected_primitives: Vec<usize>,
    /// Safety keys from defeated enemies (unwrap, panic, unsafe, deadlock, clone, leak)
    pub defeated_enemies: Option<Vec<String>>,
}

/// Parameters for signal theory detection (observed vs expected)
/// Tier: T2-P (Boundary + Quantity)
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
/// Tier: T2-C (Boundary + Comparison + Quantity + Void)
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
/// Tier: T2-C (Sum + Quantity + Boundary + Comparison)
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

/// Parameters for signal fence evaluation (process + port against default-deny policy)
/// Tier: T2-C (Boundary + Mapping + Comparison)
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
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
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ConceptGrepParams {
    /// Concept to expand (e.g., "Signal Detection", "pharmacovigilance")
    pub concept: String,
    /// Include markdown section marker patterns (default: false)
    #[serde(default)]
    pub sections: bool,
}

// ============================================================================
// PV Signal Detection Parameters
// ============================================================================

/// Contingency table for signal detection (2x2)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ContingencyTableParams {
    /// Drug + Event count (cell a)
    pub a: u64,
    /// Drug + No Event count (cell b)
    pub b: u64,
    /// No Drug + Event count (cell c)
    pub c: u64,
    /// No Drug + No Event count (cell d)
    pub d: u64,
}

/// Parameters for complete signal analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalCompleteParams {
    /// Contingency table
    #[serde(flatten)]
    pub table: ContingencyTableParams,
    /// PRR threshold (default: 2.0)
    #[serde(default = "default_prr_threshold")]
    pub prr_threshold: f64,
    /// Minimum case count (default: 3)
    #[serde(default = "default_min_n")]
    pub min_n: u32,
}

fn default_prr_threshold() -> f64 {
    2.0
}

fn default_min_n() -> u32 {
    3
}

/// Parameters for individual signal algorithm
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalAlgorithmParams {
    /// Contingency table
    #[serde(flatten)]
    pub table: ContingencyTableParams,
}

/// Parameters for Naranjo causality assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NaranjoParams {
    /// Temporal relationship: 1=yes, 0=unknown, -1=no
    pub temporal: i32,
    /// Improved after withdrawal: 1=yes, 0=unknown, -1=no
    pub dechallenge: i32,
    /// Recurred on re-exposure: 1=yes, 0=unknown, -1=no
    pub rechallenge: i32,
    /// Alternative causes exist: 1=yes, -1=no, 0=unknown
    pub alternatives: i32,
    /// Previously reported: 1=yes, 0=no
    pub previous: i32,
}

/// Parameters for WHO-UMC causality assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WhoUmcParams {
    /// Temporal relationship: 1=yes, 0=unknown, -1=no
    pub temporal: i32,
    /// Improved after withdrawal: 1=yes, 0=unknown, -1=no
    pub dechallenge: i32,
    /// Recurred on re-exposure: 1=yes, 0=unknown, -1=no
    pub rechallenge: i32,
    /// Alternative causes exist: 1=yes, -1=no, 0=unknown
    pub alternatives: i32,
    /// Pharmacological plausibility: 1=yes, 0=unknown, -1=no
    pub plausibility: i32,
}

// ============================================================================
// Vigilance Parameters
// ============================================================================

/// Parameters for safety margin calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SafetyMarginParams {
    /// PRR value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// Information Component 2.5th percentile
    pub ic025: f64,
    /// EBGM 5th percentile
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for risk score calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RiskScoreParams {
    /// Drug name
    pub drug: String,
    /// Adverse event
    pub event: String,
    /// PRR value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// Information Component 2.5th percentile
    pub ic025: f64,
    /// EBGM 5th percentile
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for ToV level mapping
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MapToTovParams {
    /// Safety level: 1-8 (Molecular to Regulatory)
    pub level: u8,
}

// ============================================================================
// Skills Parameters
// ============================================================================

/// Parameters for skill scan
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillScanParams {
    /// Directory to scan for skills
    pub directory: String,
}

/// Parameters for skill get
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillGetParams {
    /// Skill name
    pub name: String,
}

/// Parameters for skill validation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillValidateParams {
    /// Path to skill directory
    pub path: String,
}

/// Parameters for skill search by tag
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillSearchByTagParams {
    /// Tag to search for
    pub tag: String,
}

/// Parameters for nexcore_assist intent-based skill search
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AssistParams {
    /// Natural language intent describing what you want to do
    pub intent: String,
    /// Optional context to narrow search (tag filter)
    pub context: Option<String>,
    /// Maximum number of results (default: 5)
    #[serde(default = "default_assist_limit")]
    pub limit: usize,
}

fn default_assist_limit() -> usize {
    5
}

/// Parameters for listing nested skills
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillListNestedParams {
    /// Parent skill name
    pub parent: String,
}

/// Parameters for skill execution
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillExecuteParams {
    /// Name of the skill to execute
    pub name: String,
    /// Parameters to pass to the skill (JSON object)
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// Timeout in seconds (default: 60)
    #[serde(default = "default_skill_timeout")]
    pub timeout_seconds: u64,
}

fn default_skill_timeout() -> u64 {
    60
}

/// Parameters for getting a skill's input/output schema.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillSchemaParams {
    /// Name of the skill
    pub name: String,
}

/// Parameters for taxonomy query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TaxonomyQueryParams {
    /// Taxonomy type: compliance, smst, category, or node
    pub taxonomy_type: String,
    /// Key to look up
    pub key: String,
}

/// Parameters for taxonomy list
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TaxonomyListParams {
    /// Taxonomy type: compliance, smst, category, or node
    pub taxonomy_type: String,
}

// ============================================================================
// Vocabulary Parameters
// ============================================================================

/// Parameters for vocab skill lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VocabSkillLookupParams {
    /// Vocabulary shorthand (e.g., "build-doctrine", "ctvp-validated")
    pub vocab: String,
}

/// Parameters for primitive skill lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveSkillLookupParams {
    /// Primitive name (e.g., "sequence", "mapping", "state")
    pub primitive: String,
}

/// Parameters for skill chain lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillChainLookupParams {
    /// Chain name or trigger phrase
    pub query: String,
}

/// Parameters for skill orchestration analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillOrchestrationAnalyzeParams {
    /// Skill path or glob pattern (e.g., "~/.claude/skills/forge" or "~/.claude/skills/*")
    pub path_or_pattern: String,
    /// Include recommendations for frontmatter additions
    #[serde(default = "default_include_recommendations")]
    pub include_recommendations: bool,
}

fn default_include_recommendations() -> bool {
    true
}

// ============================================================================
// Skill Compiler Parameters
// ============================================================================

/// Parameters for compiling a compound skill from multiple sub-skills
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillCompileParams {
    /// Skill names to compose
    pub skills: Vec<String>,
    /// Composition strategy: sequential, parallel, feedback_loop
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Name for the compound skill
    pub name: String,
    /// Whether to build the binary (default: false, just generates source)
    #[serde(default)]
    pub build: bool,
}

fn default_strategy() -> String {
    "sequential".into()
}

/// Parameters for checking skill compilation compatibility (dry run)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillCompileCheckParams {
    /// Skill names to check
    pub skills: Vec<String>,
    /// Composition strategy: sequential, parallel, feedback_loop
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

/// Parameters for skill token analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Mapping: path -> metrics)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SkillTokenAnalyzeParams {
    /// Path to skill directory (e.g., "~/.claude/skills/forge")
    pub path: String,
}

// ============================================================================
// Guidelines Parameters
// ============================================================================

/// Parameters for guidelines search
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesSearchParams {
    /// Search query (matches ID, title, keywords, description)
    pub query: String,
    /// Filter by source: "ich", "cioms", or "ema"
    #[serde(default)]
    pub source: Option<String>,
    /// Filter by category within source
    #[serde(default)]
    pub category: Option<String>,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for guidelines get
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesGetParams {
    /// Guideline ID (e.g., "E2B", "CIOMS-I", "GVP-Module-VI")
    pub id: String,
}

/// Parameters for guidelines URL lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuidelinesUrlParams {
    /// Guideline ID
    pub id: String,
}

// ============================================================================
// FAERS Parameters
// ============================================================================

/// Parameters for FAERS search
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSearchParams {
    /// Drug name to search (generic or brand)
    #[serde(default)]
    pub drug_name: Option<String>,
    /// Adverse reaction to search (MedDRA PT)
    #[serde(default)]
    pub reaction: Option<String>,
    /// Filter to serious events only
    #[serde(default)]
    pub serious: Option<bool>,
    /// Max results (1-100)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for FAERS drug events
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersDrugEventsParams {
    /// Drug name to analyze
    pub drug_name: String,
    /// Number of top events to return (default: 20)
    #[serde(default)]
    pub top_n: Option<usize>,
}

/// Parameters for FAERS signal check and disproportionality
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSignalParams {
    /// Drug name
    pub drug_name: String,
    /// Adverse event (MedDRA PT)
    pub event_name: String,
}

/// Parameters for FAERS drug comparison
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersCompareDrugsParams {
    /// First drug name
    pub drug1: String,
    /// Second drug name
    pub drug2: String,
    /// Number of events per drug (default: 15)
    #[serde(default)]
    pub top_n: Option<usize>,
}

// ============================================================================
// GCloud Parameters
// ============================================================================

/// Parameters for gcloud config get
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudConfigGetParams {
    /// Property name (e.g., 'project', 'account', 'compute/region')
    pub property: String,
}

/// Parameters for gcloud config set
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudConfigSetParams {
    /// Property to set (e.g., 'project', 'compute/region')
    pub property: String,
    /// Value to set
    pub value: String,
}

/// Parameters for gcloud project describe
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudProjectParams {
    /// Project ID
    pub project_id: String,
}

/// Parameters with optional project
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudOptionalProjectParams {
    /// Project ID (uses default if not specified)
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for secrets access
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudSecretsAccessParams {
    /// Secret name
    pub secret_name: String,
    /// Version to access (default: 'latest')
    #[serde(default = "default_latest")]
    pub version: String,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

fn default_latest() -> String {
    "latest".to_string()
}

/// Parameters for GCS path operations
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudStoragePathParams {
    /// GCS path (e.g., 'gs://bucket-name/prefix/')
    pub path: String,
}

/// Parameters for storage copy
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudStorageCpParams {
    /// Source path (local or gs://)
    pub source: String,
    /// Destination path (local or gs://)
    pub destination: String,
    /// Copy directories recursively
    #[serde(default)]
    pub recursive: bool,
}

/// Parameters for compute instances list
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudComputeInstancesParams {
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
    /// Zone filter
    #[serde(default)]
    pub zone: Option<String>,
}

/// Parameters for Cloud Run/Functions with region
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudServiceListParams {
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
    /// Region filter
    #[serde(default)]
    pub region: Option<String>,
}

/// Parameters for Cloud Run/Functions describe
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudServiceDescribeParams {
    /// Service/function name
    pub name: String,
    /// Region where deployed
    pub region: String,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for logging read
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudLoggingReadParams {
    /// Log filter expression
    pub filter: String,
    /// Maximum entries to return (default: 50)
    #[serde(default = "default_log_limit")]
    pub limit: u32,
    /// Project ID
    #[serde(default)]
    pub project: Option<String>,
}

fn default_log_limit() -> u32 {
    50
}

/// Parameters for generic gcloud command
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GcloudRunCommandParams {
    /// Command to run (without 'gcloud' prefix)
    pub command: String,
    /// Timeout in seconds (default: 60)
    #[serde(default = "default_gcloud_timeout")]
    pub timeout: u64,
}

fn default_gcloud_timeout() -> u64 {
    60
}

// ============================================================================
// Wolfram Alpha Parameters
// ============================================================================

/// Parameters for Wolfram Alpha full query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryParams {
    /// Natural language query or mathematical expression
    pub query: String,
    /// Preferred unit system for results
    #[serde(default = "default_metric")]
    pub units: String,
    /// Location context for geographic queries (city, country, or coordinates)
    #[serde(default)]
    pub location: Option<String>,
    /// Include images, sources, and available drill-down options
    #[serde(default)]
    pub verbose: bool,
}

fn default_metric() -> String {
    "metric".to_string()
}

/// Parameters for short/spoken answer
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframShortParams {
    /// Question or calculation to answer
    pub query: String,
    /// Preferred unit system
    #[serde(default = "default_metric")]
    pub units: String,
}

/// Parameters for calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframCalculateParams {
    /// Mathematical expression or equation to evaluate
    pub expression: String,
}

/// Parameters for step-by-step solution
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframStepByStepParams {
    /// Math problem to solve with steps shown
    pub problem: String,
}

/// Parameters for plotting
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframPlotParams {
    /// Function or expression to plot
    pub expression: String,
    /// Optional range specification (e.g., 'x from -10 to 10')
    #[serde(default)]
    pub range: Option<String>,
}

/// Parameters for unit conversion
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframConvertParams {
    /// Numeric value to convert
    pub value: f64,
    /// Source unit (e.g., 'miles', 'kg', 'fahrenheit')
    pub from_unit: String,
    /// Target unit (e.g., 'km', 'pounds', 'celsius')
    pub to_unit: String,
}

/// Parameters for chemistry lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframChemistryParams {
    /// Chemical compound name, formula, SMILES, or CAS number
    pub compound: String,
    /// Specific property to look up (e.g., 'boiling point', 'density', 'structure')
    #[serde(default)]
    pub property: Option<String>,
}

/// Parameters for physics query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframPhysicsParams {
    /// Physics question, constant, or calculation
    pub query: String,
}

/// Parameters for astronomy query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframAstronomyParams {
    /// Astronomical query
    pub query: String,
    /// Observer location for rise/set times and visibility
    #[serde(default)]
    pub location: Option<String>,
}

/// Parameters for statistics query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframStatisticsParams {
    /// Statistical query or dataset
    pub query: String,
}

/// Parameters for data lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframDataLookupParams {
    /// Data query
    pub query: String,
}

/// Parameters for query with assumption
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryWithAssumptionParams {
    /// The query to interpret
    pub query: String,
    /// Assumption string from previous query (e.g., '*C.Mercury-_*Planet-')
    pub assumption: String,
}

/// Parameters for filtered query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframQueryFilteredParams {
    /// Query to send
    pub query: String,
    /// Pod IDs to include (only these will be returned)
    #[serde(default)]
    pub include_pods: Option<Vec<String>>,
    /// Pod IDs to exclude from results
    #[serde(default)]
    pub exclude_pods: Option<Vec<String>>,
}

/// Parameters for image result
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframImageParams {
    /// Query to visualize
    pub query: String,
}

/// Parameters for datetime query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframDatetimeParams {
    /// Date/time query or calculation
    pub query: String,
    /// Location for time zone context
    #[serde(default)]
    pub location: Option<String>,
}

/// Parameters for nutrition lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframNutritionParams {
    /// Food item or meal to look up
    pub food: String,
    /// Optional quantity (e.g., '100g', '1 cup', '1 serving')
    #[serde(default)]
    pub amount: Option<String>,
}

/// Parameters for finance query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframFinanceParams {
    /// Financial query or calculation
    pub query: String,
}

/// Parameters for linguistics query
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WolframLinguisticsParams {
    /// Linguistics query
    pub query: String,
}

// ============================================================================
// Principles Parameters
// ============================================================================

/// Parameters for listing available principles
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesListParams {}

/// Parameters for getting a specific principle
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesGetParams {
    /// Name of the principle (e.g., 'dalio-principles', 'kiss', 'first-principles')
    pub name: String,
}

/// Parameters for searching principles
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrinciplesSearchParams {
    /// Search query (keyword or phrase)
    pub query: String,
    /// Maximum results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

// ============================================================================
// Validation Parameters
// ============================================================================

/// Parameters for L1-L5 validation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationRunParams {
    /// Target path to validate (file or directory)
    pub target: String,
    /// Domain type (auto-detected if not specified)
    /// Options: skill, agent, config, architecture, pv_terminology, construct
    #[serde(default)]
    pub domain: Option<String>,
    /// Maximum validation level: L1, L2, L3, L4, or L5 (default: L5)
    #[serde(default)]
    pub max_level: Option<String>,
    /// Stop on first failing level (default: true)
    #[serde(default)]
    pub fail_fast: Option<bool>,
}

/// Parameters for quick check (L1-L2 only)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationCheckParams {
    /// Target path to validate
    pub target: String,
    /// Domain type (auto-detected if not specified)
    #[serde(default)]
    pub domain: Option<String>,
}

/// Parameters for listing validation domains
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationDomainsParams {}

/// Parameters for classifying tests in Rust source
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValidationClassifyTestsParams {
    /// Path to Rust file or directory to analyze
    pub path: String,
}

// ============================================================================
// ICH Glossary Parameters
// ============================================================================

/// Parameters for ICH term lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchLookupParams {
    /// Term name to look up (case-insensitive)
    pub term: String,
}

/// Parameters for ICH term search
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchSearchParams {
    /// Search query (supports fuzzy matching)
    pub query: String,
    /// Maximum number of results (default: 10, max: 50)
    #[serde(default = "default_ten_limit")]
    pub limit: Option<usize>,
}

fn default_ten_limit() -> Option<usize> {
    Some(10)
}

/// Parameters for ICH guideline lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IchGuidelineParams {
    /// Guideline ID (e.g., "E2A", "Q9", "M4")
    pub guideline_id: String,
}

// ============================================================================
// Brain Parameters (Antigravity-style working memory)
// ============================================================================

/// Parameters for creating a new brain session
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionCreateParams {
    /// Project name or path
    #[serde(default)]
    pub project: Option<String>,
    /// Git commit hash
    #[serde(default)]
    pub git_commit: Option<String>,
    /// Session description
    #[serde(default)]
    pub description: Option<String>,
}

/// Parameters for loading a brain session
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionLoadParams {
    /// Session ID (UUID)
    pub session_id: String,
}

/// Parameters for listing brain sessions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionsListParams {
    /// Maximum number of sessions to return (default: 20)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for saving an artifact
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactSaveParams {
    /// Artifact name (e.g., "task.md", "plan.md")
    pub name: String,
    /// Artifact content
    pub content: String,
    /// Artifact type (task, plan, walkthrough, review, research, decision, custom)
    #[serde(default)]
    pub artifact_type: Option<String>,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for resolving an artifact
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactResolveParams {
    /// Artifact name
    pub name: String,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for getting an artifact
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactGetParams {
    /// Artifact name
    pub name: String,
    /// Specific version number (omit for current)
    #[serde(default)]
    pub version: Option<u32>,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for diffing artifact versions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactDiffParams {
    /// Artifact name
    pub name: String,
    /// First version number
    pub v1: u32,
    /// Second version number
    pub v2: u32,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for tracking a file
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerTrackParams {
    /// File path to track
    pub path: String,
    /// Project name (defaults to current directory)
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for checking if a tracked file changed
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerChangedParams {
    /// File path to check
    pub path: String,
    /// Project name
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for getting original file content
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerOriginalParams {
    /// File path
    pub path: String,
    /// Project name
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for getting implicit knowledge
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitGetParams {
    /// Preference key
    pub key: String,
}

/// Parameters for setting implicit knowledge
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitSetParams {
    /// Preference key
    pub key: String,
    /// Preference value (JSON string)
    pub value: String,
}

/// Parameters for finding corrections by fuzzy matching
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitFindCorrectionsParams {
    /// Query string to fuzzy-match against corrections
    pub query: String,
    /// Minimum similarity threshold (0.0-1.0, default 0.3)
    pub threshold: Option<f64>,
}

/// Parameters for listing patterns by T1 grounding
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitPatternsByGroundingParams {
    /// T1 primitive to filter by: "sequence", "mapping", "recursion", "state", or "void"
    pub primitive: String,
}

/// Parameters for brain recovery repair
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainRecoveryRepairParams {
    /// Session ID to repair (defaults to latest)
    pub session_id: Option<String>,
}

/// Parameters for verifying engrams in the brain
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainVerifyParams {
    /// Placeholder for future expansion
    #[serde(default)]
    pub _placeholder: Option<bool>,
}

// ============================================================================
// Belief Parameters (PROJECT GROUNDED)
// ============================================================================

/// Parameters for saving a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefSaveParams {
    /// Unique belief identifier
    pub id: String,
    /// The proposition this belief represents
    pub proposition: String,
    /// Belief category (e.g., "capability", "behavior", "preference")
    pub category: String,
    /// If true, creates as hypothesis with lower initial confidence
    pub is_hypothesis: Option<bool>,
}

/// Parameters for getting a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGetParams {
    /// Belief ID to retrieve
    pub id: String,
}

/// Parameters for adding evidence to a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefAddEvidenceParams {
    /// Belief ID to add evidence to
    pub belief_id: String,
    /// Unique evidence identifier
    pub evidence_id: String,
    /// Evidence type: observation, test_result, user_feedback, inference, authority, prior
    pub evidence_type: String,
    /// Description of what this evidence shows
    pub description: String,
    /// Weight (-1.0 to 1.0): positive supports, negative contradicts
    pub weight: f64,
    /// Source identifier
    pub source: String,
    /// Link to MCP execution that produced this evidence
    pub execution_id: Option<String>,
    /// Link to originating hypothesis
    pub hypothesis_id: Option<String>,
}

/// Parameters for validating a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefValidateParams {
    /// Belief ID to validate
    pub belief_id: String,
    /// Whether validation passed
    pub passed: bool,
}

// ============================================================================
// Trust Parameters (PROJECT GROUNDED)
// ============================================================================

/// Parameters for recording a trust demonstration
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainTrustRecordParams {
    /// Domain to record trust for
    pub domain: String,
    /// Whether this was a success (true) or failure (false)
    pub success: bool,
}

/// Parameters for getting trust score
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainTrustGetParams {
    /// Domain to get trust for
    pub domain: String,
}

// ============================================================================
// Belief Graph Parameters (PROJECT GROUNDED)
// ============================================================================

/// Parameters for adding a belief implication
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGraphAddImplicationParams {
    /// Source belief ID (antecedent)
    pub from: String,
    /// Target belief ID (consequent)
    pub to: String,
    /// Implication strength: "strong", "moderate", "weak"
    pub strength: String,
}

/// Parameters for querying the belief graph
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGraphQueryParams {
    /// Belief ID to query
    pub belief_id: String,
}

// ============================================================================
// Hook Registry Parameters
// ============================================================================

/// Parameters for querying hooks by event type
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HooksForEventParams {
    /// Event type (e.g., "SessionStart", "PreToolUse:Edit|Write")
    pub event: String,
}

/// Parameters for querying hooks by deployment tier
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HooksForTierParams {
    /// Deployment tier: "dev", "review", or "deploy"
    pub tier: String,
}

/// Parameters for listing nested hooks in a compound hook molecule
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HookListNestedParams {
    /// Parent hook name (nucleus of the hook molecule)
    pub parent: String,
}

/// Parameters for hook metrics summary (no parameters needed)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HookMetricsSummaryParams {
    /// Placeholder (no parameters needed, but MCP requires a struct)
    #[serde(default)]
    pub _placeholder: Option<bool>,
}

/// Parameters for hook metrics filtered by event type
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HookMetricsByEventParams {
    /// Event type to filter by (e.g., "PreToolUse", "SessionStart")
    pub event: String,
}

// ============================================================================
// MCP Server Management Parameters
// ============================================================================

/// Parameters for listing configured MCP servers
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServersListParams {
    /// Include project-specific servers (default: false, global only)
    #[serde(default)]
    pub include_projects: bool,
}

/// Parameters for getting a specific MCP server configuration
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerGetParams {
    /// MCP server name
    pub name: String,
}

/// Parameters for adding a new MCP server
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerAddParams {
    /// MCP server name (identifier)
    pub name: String,
    /// Command to execute (path to binary)
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// Parameters for removing an MCP server
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpServerRemoveParams {
    /// MCP server name to remove
    pub name: String,
}

// ============================================================================
// PVDSL (Pharmacovigilance Domain-Specific Language) Parameters
// ============================================================================

/// Parameters for compiling PVDSL source code
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslCompileParams {
    /// PVDSL source code to compile
    pub source: String,
}

/// Parameters for executing compiled PVDSL bytecode
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslExecuteParams {
    /// PVDSL source code to execute
    pub source: String,
    /// Optional variables to set before execution (key-value pairs as JSON)
    #[serde(default)]
    pub variables: std::collections::HashMap<String, serde_json::Value>,
}

/// Parameters for evaluating a PVDSL expression
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvdslEvalParams {
    /// PVDSL expression to evaluate (e.g., "signal::prr(10, 90, 100, 9800)")
    pub expression: String,
}

// ============================================================================
// Guardian (Homeostasis Control Loop) Parameters
// ============================================================================

/// Parameters for running a homeostasis loop tick
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianTickParams {
    // No params needed - runs one iteration of the control loop
}

/// Parameters for evaluating PV risk
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianEvaluatePvParams {
    /// Drug name
    pub drug: String,
    /// Adverse event name
    pub event: String,
    /// PRR (Proportional Reporting Ratio) value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// IC 2.5th percentile (Information Component)
    pub ic025: f64,
    /// EB05 (EBGM 5th percentile)
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
}

/// Parameters for getting homeostasis status
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianStatusParams {
    // No params needed - returns current loop status
}

/// Parameters for resetting homeostasis loop
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianResetParams {
    // No params needed - resets loop state
}

/// Parameters for injecting a test signal into Guardian
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianInjectSignalParams {
    /// Signal source: "external" (PAMP), "internal" (DAMP), or "pv"
    pub source: String,
    /// Signal pattern/type identifier
    pub pattern: String,
    /// Severity score (0.0 - 1.0)
    pub severity: f64,
    /// Optional context message
    #[serde(default)]
    pub context: Option<String>,
}

/// Parameters for listing Guardian sensors
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSensorsListParams {
    // No params - lists all registered sensors
}

/// Parameters for listing Guardian actuators
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianActuatorsListParams {
    // No params - lists all registered actuators
}

/// Parameters for getting Guardian event history
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianHistoryParams {
    /// Maximum number of events to return (default: 50)
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    /// Filter by event type: "signal", "action", "all" (default)
    #[serde(default = "default_history_filter")]
    pub filter: String,
}

fn default_history_limit() -> usize {
    50
}

fn default_history_filter() -> String {
    "all".to_string()
}

/// Parameters for subscribing to Guardian events (for WebSocket/SSE)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSubscribeParams {
    /// Event types to subscribe to: "signals", "actions", "all"
    #[serde(default = "default_subscribe_filter")]
    pub events: String,
}

/// Parameters for setting input for the adversarial prompt sensor
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdversarialSensorInputParams {
    /// Text input to analyze for statistical fingerprints
    pub text: String,
}

fn default_subscribe_filter() -> String {
    "all".to_string()
}

// ============================================================================
// Commandment (15 Human Commandments) Parameters
// ============================================================================

/// Parameters for verifying a single commandment
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentVerifyParams {
    /// Commandment name: TruthInGrounding, Falsifiability, Consensus, etc.
    pub commandment: String,
    /// Whether proof was provided for this commandment
    #[serde(default)]
    pub proof_provided: bool,
}

/// Parameters for getting commandment info
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentInfoParams {
    /// Commandment name or number (1-15)
    pub commandment: String,
}

/// Parameters for listing commandments by category
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentListParams {
    /// Category filter: Epistemic, Authority, Observability, Process, Integrity, or "all"
    #[serde(default = "default_commandment_category")]
    pub category: String,
}

fn default_commandment_category() -> String {
    "all".to_string()
}

/// Parameters for full commandment audit
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentAuditParams {
    /// Proof of grounding provided
    #[serde(default)]
    pub grounding_proof: bool,
    /// Owner identified for action
    #[serde(default)]
    pub owner_identified: bool,
    /// Audit trail exists
    #[serde(default)]
    pub audit_trail_exists: bool,
    /// Sensing is active
    #[serde(default)]
    pub sensing_active: bool,
    /// Correction mechanism exists
    #[serde(default)]
    pub correction_mechanism: bool,
    /// State is public
    #[serde(default)]
    pub state_public: bool,
    /// Persistence is guaranteed
    #[serde(default)]
    pub persistence_guaranteed: bool,
    /// Market is fair (no asymmetry abuse)
    #[serde(default)]
    pub fair_market: bool,
    /// Human override is available
    #[serde(default)]
    pub human_override_available: bool,
    /// Codex compliant
    #[serde(default)]
    pub codex_compliant: bool,
    /// Falsifiability test exists
    #[serde(default)]
    pub has_falsifiability_test: bool,
    /// Provenance chain exists
    #[serde(default)]
    pub has_provenance: bool,
    /// Oracle count - agreeing oracles for consensus
    #[serde(default)]
    pub oracle_agreeing: u32,
    /// Oracle count - total oracles for consensus
    #[serde(default)]
    pub oracle_total: u32,
    /// Precedent hash exists
    #[serde(default)]
    pub has_precedent: bool,
    /// Code compiles successfully
    #[serde(default)]
    pub compiled: bool,
    /// Code passes type-checking
    #[serde(default)]
    pub type_checked: bool,
    /// Effect annotations verified
    #[serde(default)]
    pub effects_verified: bool,
}

// ============================================================================
// End-to-End PV Pipeline Parameters
// ============================================================================

/// Parameters for end-to-end pharmacovigilance pipeline
///
/// Chains: FAERS Query → Signal Detection → Guardian Risk Scoring
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPipelineParams {
    /// Drug name (generic or brand)
    pub drug_name: String,
    /// Adverse event (MedDRA Preferred Term)
    pub event_name: String,
    /// Signal detection threshold preset: "evans" (default), "strict", or "sensitive"
    #[serde(default = "default_threshold_preset")]
    pub threshold_preset: String,
}

fn default_threshold_preset() -> String {
    "evans".to_string()
}

// ============================================================================
// Node Hunter Parameters
// ============================================================================

/// Parameters for real-time node signal scanning
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NodeHuntScanParams {
    /// Behavioral signature to hunt for (e.g., 'LATENCY_SPIKE', 'ANOMALY')
    pub target_pattern: String,
    /// Optional: Filter to specific network partition
    #[serde(default)]
    pub partition: Option<String>,
}

/// Parameters for node isolation
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NodeHuntIsolateParams {
    /// Node ID to isolate
    pub node_id: String,
    /// Reason for isolation
    pub reason: String,
}

// ============================================================================
// Regulatory Primitives Parameters
// ============================================================================

/// Parameters for extracting regulatory primitives from FDA/ICH/CIOMS
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryExtractParams {
    /// Regulatory source identifier (e.g., "fda", "ich", "cioms", "21 CFR 314.80")
    pub source: String,
    /// Raw content to extract from (optional - uses built-in definitions if empty)
    #[serde(default)]
    pub content: String,
    /// Maximum tier to include: 1=T1 only, 2=T1+T2-P, 3=all (default: 3)
    #[serde(default = "default_max_tier")]
    pub max_tier: u8,
}

fn default_max_tier() -> u8 {
    3
}

/// Parameters for auditing FDA vs CIOMS/ICH consistency
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryAuditParams {
    /// FDA term to audit (e.g., "serious", "unexpected", "reasonable possibility")
    pub fda_term: String,
    /// Corresponding CIOMS/ICH term to compare
    pub cioms_term: String,
    /// Include component-level audit (default: true)
    #[serde(default = "default_include_components")]
    pub include_components: Option<bool>,
}

fn default_include_components() -> Option<bool> {
    Some(true)
}

/// Parameters for cross-domain primitive comparison
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryCompareParams {
    /// First domain (e.g., "pharmacovigilance", "pv", "drug safety")
    pub domain1: String,
    /// Second domain (e.g., "cloud", "ai_safety", "finance")
    pub domain2: String,
    /// Minimum transfer confidence threshold (default: 0.5)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
}

fn default_confidence_threshold() -> f64 {
    0.5
}

/// Parameters for FDA effectiveness endpoint assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EffectivenessAssessParams {
    /// Approval pathway: "traditional", "accelerated", "breakthrough", "fast_track", "priority"
    pub pathway: String,
    /// Endpoint tier: "primary", "secondary", "exploratory"
    pub endpoint_tier: String,
    /// Endpoint type: "clinical", "surrogate", "intermediate_clinical", "biomarker", "patient_reported", "digital_health"
    pub endpoint_type: String,
    /// Endpoint name (e.g., "Overall Survival", "Progression-Free Survival")
    pub endpoint_name: String,
    /// P-value from statistical analysis (optional)
    #[serde(default)]
    pub p_value: Option<f64>,
    /// Whether endpoint met success criterion (optional)
    #[serde(default)]
    pub success: Option<bool>,
    /// Alpha level (default: 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Number of comparisons for multiplicity adjustment (default: 1)
    #[serde(default = "default_n_comparisons")]
    pub n_comparisons: usize,
    /// Multiplicity method: "bonferroni", "holm", "hochberg", "fixed_sequence" (default: "bonferroni")
    #[serde(default = "default_multiplicity_method")]
    pub multiplicity_method: String,
}

fn default_alpha() -> f64 {
    0.05
}

fn default_n_comparisons() -> usize {
    1
}

fn default_multiplicity_method() -> String {
    "bonferroni".to_string()
}

/// Parameters for QBRI benefit-risk computation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct QbriComputeParams {
    /// Benefit effect size (e.g., hazard ratio inverse, odds ratio)
    pub benefit_effect: f64,
    /// Benefit p-value
    pub benefit_pvalue: f64,
    /// Unmet medical need [1-10]
    pub unmet_need: f64,
    /// Risk signal strength (PRR, ROR, or EBGM)
    pub risk_signal: f64,
    /// Risk probability (causal likelihood 0-1)
    pub risk_probability: f64,
    /// Severity (Hartwig-Siegel 1-7)
    pub risk_severity: u8,
    /// Is the adverse event reversible?
    #[serde(default = "default_reversible")]
    pub reversible: bool,
}

fn default_reversible() -> bool {
    true
}

/// Parameters for deriving QBRI thresholds from historical data
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct QbriDeriveParams {
    /// Use synthetic test data
    #[serde(default = "default_use_synthetic")]
    pub use_synthetic: bool,
}

fn default_use_synthetic() -> bool {
    true
}

// ============================================================================
// Primitive Validation Parameters (Corpus-Backed)
// ============================================================================

/// Parameters for validating a primitive against external corpus
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveValidateParams {
    /// Primitive term to validate (e.g., "adverse event", "signal detection")
    pub term: String,
    /// Domain context: "pv", "pharmacovigilance", "regulatory", "medical" (default: "pv")
    #[serde(default = "default_pv_domain")]
    pub domain: String,
    /// Minimum confidence tier: 1 (authoritative only), 2 (+peer-reviewed), 3 (+web), 4 (all)
    #[serde(default = "default_min_tier")]
    pub min_tier: u8,
    /// Maximum number of citations to return (default: 5)
    #[serde(default = "default_citation_limit")]
    pub max_citations: usize,
}

fn default_pv_domain() -> String {
    "pv".to_string()
}

fn default_min_tier() -> u8 {
    2
}

fn default_citation_limit() -> usize {
    5
}

/// Parameters for generating a professional citation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveCiteParams {
    /// PubMed ID (PMID) or DOI to cite
    pub identifier: String,
    /// Citation format: "vancouver" (default), "apa", "chicago"
    #[serde(default = "default_citation_format")]
    pub format: String,
}

fn default_citation_format() -> String {
    "vancouver".to_string()
}

/// Parameters for batch primitive validation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveValidateBatchParams {
    /// List of primitive terms to validate
    pub terms: Vec<String>,
    /// Domain context (default: "pv")
    #[serde(default = "default_pv_domain")]
    pub domain: String,
    /// Minimum tier for all validations (default: 2)
    #[serde(default = "default_min_tier")]
    pub min_tier: u8,
}

/// Parameters for re-validating existing primitives
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveRevalidateParams {
    /// Path to primitives file or directory to re-validate
    pub source: String,
    /// Only re-validate if last validation was more than N days ago (default: 90)
    #[serde(default = "default_revalidation_days")]
    pub max_age_days: u32,
}

fn default_revalidation_days() -> u32 {
    90
}

// ============================================================================
// Vigil Orchestrator Parameters
// ============================================================================

/// Parameters for Vigil status check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilStatusParams")]
pub struct VigilStatusParams {}

/// Parameters for Vigil health check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilHealthParams")]
pub struct VigilHealthParams {}

/// Parameters for emitting events to Vigil event bus
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilEmitEventParams")]
pub struct VigilEmitEventParams {
    /// Event source identifier (e.g., "mcp", "user", "system")
    pub source: String,
    /// Event type (e.g., "command", "query", "notification")
    pub event_type: String,
    /// Event payload as JSON
    pub payload: serde_json::Value,
    /// Priority: "Critical", "High", "Normal", "Low" (default: "Normal")
    pub priority: Option<String>,
}

/// Parameters for searching Vigil memory
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilMemorySearchParams")]
pub struct VigilMemorySearchParams {
    /// Search query for semantic matching
    pub query: String,
    /// Maximum results to return (default: 10)
    pub limit: Option<usize>,
}

/// Parameters for Vigil memory statistics
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilMemoryStatsParams")]
pub struct VigilMemoryStatsParams {}

/// Parameters for Vigil LLM usage statistics
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilLlmStatsParams")]
pub struct VigilLlmStatsParams {}

/// Parameters for Vigil event source control
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilSourceControlParams")]
pub struct VigilSourceControlParams {
    /// Source name: "filesystem", "webhook", "voice", "git_monitor"
    pub source: String,
    /// Action: "start", "stop", "status", "list"
    pub action: String,
    /// Optional configuration parameters
    pub config: Option<serde_json::Value>,
}

/// Parameters for Vigil executor control
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilExecutorControlParams")]
pub struct VigilExecutorControlParams {
    /// Action: "set-default", "route-by-complexity", "list", "status"
    pub action: String,
    /// LLM provider: "claude", "gemini", "auto" (for auto-routing)
    pub provider: Option<String>,
    /// Complexity thresholds for routing: low/medium/high
    pub complexity_thresholds: Option<serde_json::Value>,
}

/// Parameters for Vigil authority configuration
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilAuthorityConfigParams")]
pub struct VigilAuthorityConfigParams {
    /// Action: "set-rule", "get-rules", "set-threshold", "list-config"
    pub action: String,
    /// Rule type: "human_required", "ai_allowed", "escalation_threshold", "confirmation_required"
    pub rule_type: Option<String>,
    /// Rule value (varies by type)
    pub value: Option<serde_json::Value>,
}

fn default_nexcore() -> String {
    "nexcore".into()
}
fn default_general() -> String {
    "general".into()
}
fn default_cli() -> String {
    "cli".into()
}
fn default_webhook() -> String {
    "webhook".into()
}
fn default_get() -> String {
    "get".into()
}

/// Parameters for Vigil context assembly
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilContextAssembleParams")]
pub struct VigilContextAssembleParams {
    /// Project name to assemble context for (default: "nexcore")
    #[serde(default = "default_nexcore")]
    pub project: String,
    /// Focus area for context assembly (default: "general")
    #[serde(default = "default_general")]
    pub focus: String,
}

/// Parameters for Vigil authority verification
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilAuthorityVerifyParams")]
pub struct VigilAuthorityVerifyParams {
    /// Command to verify authority for (e.g. "deploy", "search", "delete")
    #[serde(default)]
    pub command: String,
    /// Source of the command (e.g. "cli", "voice", "webhook")
    #[serde(default = "default_cli")]
    pub source: String,
}

/// Parameters for Vigil webhook testing
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilWebhookTestParams")]
pub struct VigilWebhookTestParams {
    /// Webhook source identifier
    #[serde(default)]
    pub source: String,
    /// Webhook payload to validate (JSON object)
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Parameters for Vigil source configuration
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilSourceConfigParams")]
pub struct VigilSourceConfigParams {
    /// Source type: "voice", "webhook", "scheduler", "filesystem"
    #[serde(default = "default_webhook")]
    pub source: String,
    /// Action: "get" or "set"
    #[serde(default = "default_get")]
    pub action: String,
}

/// Evaluate decision quality with confidence intervals
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilDecisionConfidenceParams")]
pub struct VigilDecisionConfidenceParams {}

/// Persist memories to Qdrant with semantic embeddings
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilMemoryPersistParams")]
pub struct VigilMemoryPersistParams {}

/// Performance profiling for different executors
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilExecutorBenchmarkParams")]
pub struct VigilExecutorBenchmarkParams {}

/// Token cost estimation before execution
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilContextCostEstimateParams")]
pub struct VigilContextCostEstimateParams {}

/// Inject test signals for simulation/testing
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
#[schemars(title = "VigilSignalInjectionParams")]
pub struct VigilSignalInjectionParams {}

// ============================================================================
// Chemistry Primitives Parameters
// ============================================================================

/// Parameters for Arrhenius rate calculation (threshold gating)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryThresholdRateParams {
    /// Pre-exponential factor A (sensitivity, same units as rate constant)
    pub pre_exponential: f64,
    /// Activation energy in kJ/mol (threshold)
    pub activation_energy_kj: f64,
    /// Temperature in Kelvin (scaling factor)
    pub temperature_k: f64,
}

/// Parameters for decay remaining calculation (half-life kinetics)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryDecayRemainingParams {
    /// Initial amount
    pub initial: f64,
    /// Half-life (time for 50% decay)
    pub half_life: f64,
    /// Time elapsed
    pub time: f64,
}

/// Parameters for Michaelis-Menten saturation rate
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistrySaturationRateParams {
    /// Substrate/input concentration
    pub substrate: f64,
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km) - load at 50% capacity
    pub k_m: f64,
}

/// Parameters for Gibbs free energy feasibility calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFeasibilityParams {
    /// Enthalpy change ΔH in kJ/mol (direct benefit/cost)
    pub delta_h: f64,
    /// Entropy change ΔS in J/(mol·K) (disorder/complexity)
    pub delta_s: f64,
    /// Temperature in Kelvin (uncertainty scaling)
    pub temperature_k: f64,
}

/// Parameters for rate law dependency calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryDependencyRateParams {
    /// Rate constant k
    pub k: f64,
    /// Reactants as (concentration, order) pairs
    pub reactants: Vec<(f64, f64)>,
}

/// Parameters for buffer capacity calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryBufferCapacityParams {
    /// Total buffer concentration
    pub total_conc: f64,
    /// [A⁻]/[HA] ratio (base to acid)
    pub ratio: f64,
}

/// Parameters for Beer-Lambert absorbance calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistrySignalAbsorbanceParams {
    /// Molar absorptivity ε (L/(mol·cm))
    pub absorptivity: f64,
    /// Path length l (cm)
    pub path_length: f64,
    /// Concentration c (mol/L)
    pub concentration: f64,
}

/// Parameters for equilibrium steady-state fractions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryEquilibriumParams {
    /// Equilibrium constant K
    pub k_eq: f64,
}

/// Parameters for simple threshold exceeded check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryThresholdExceededParams {
    /// Signal value
    pub signal: f64,
    /// Threshold value
    pub threshold: f64,
}

/// Parameters for PV mappings (no args needed)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryPvMappingsParams {}

/// Parameters for Hill equation (cooperative binding)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryHillResponseParams {
    /// Input concentration or signal strength
    pub input: f64,
    /// Half-saturation constant (K₀.₅) - input at 50% response
    pub k_half: f64,
    /// Hill coefficient (nH): >1 positive cooperativity, <1 negative
    pub n_hill: f64,
}

/// Parameters for Nernst equation (dynamic threshold)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryNernstParams {
    /// Standard potential (E⁰) in volts
    pub e_standard: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Number of electrons transferred
    pub n_electrons: f64,
    /// Reaction quotient Q = [products]/[reactants]
    pub q: f64,
}

/// Parameters for competitive inhibition
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryInhibitionParams {
    /// Substrate concentration [S]
    pub substrate: f64,
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km)
    pub k_m: f64,
    /// Inhibitor concentration [I]
    pub inhibitor: f64,
    /// Inhibition constant (Ki)
    pub k_i: f64,
}

/// Parameters for Eyring equation (transition state theory)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryEyringRateParams {
    /// Gibbs free energy of activation (ΔG‡) in J/mol
    pub delta_g: f64,
    /// Temperature in Kelvin
    pub temperature_k: f64,
    /// Transmission coefficient (κ), typically 1.0
    #[serde(default = "default_kappa")]
    pub kappa: f64,
}

fn default_kappa() -> f64 {
    1.0
}

/// Parameters for Langmuir isotherm (resource binding)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryLangmuirParams {
    /// Adsorbate concentration [A]
    pub concentration: f64,
    /// Equilibrium constant K (affinity)
    pub k_eq: f64,
}

/// Parameters for First Law closed system energy balance
/// Tier: T2-C (Cross-domain composite - conservation law)
/// ΔU = Q - W
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFirstLawClosedParams {
    /// Initial internal energy (J)
    pub u_initial: f64,
    /// Heat added to system (J, positive = heat in)
    pub heat_in: f64,
    /// Work done by system (J, positive = work out)
    pub work_out: f64,
}

/// Parameters for First Law open system energy balance
/// Tier: T2-C (Cross-domain composite - conservation law)
/// dE/dt = Q̇ - Ẇ + Σṁh_in - Σṁh_out
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemistryFirstLawOpenParams {
    /// Heat transfer rate (W)
    pub heat_rate: f64,
    /// Power output (W)
    pub power_out: f64,
    /// Inlet mass flow rates (kg/s)
    pub inflow_mass_rates: Vec<f64>,
    /// Inlet specific enthalpies (J/kg)
    pub inflow_enthalpies: Vec<f64>,
    /// Outlet mass flow rates (kg/s)
    pub outflow_mass_rates: Vec<f64>,
    /// Outlet specific enthalpies (J/kg)
    pub outflow_enthalpies: Vec<f64>,
}

// ============================================================================
// Brand Semantics Parameters
// ============================================================================

/// Parameters for getting a brand decomposition
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrandDecompositionGetParams {
    /// Brand name to look up (e.g., "nexvigilant")
    pub name: String,
}

/// Parameters for testing if a term is primitive
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrandPrimitiveTestParams {
    /// Term to test
    pub term: String,
    /// Definition of the term
    pub definition: String,
    /// Domain-specific terms found in the definition (for Test 1)
    #[serde(default)]
    pub domain_terms_in_definition: Option<Vec<String>>,
    /// External concepts the term grounds to (for Test 2)
    #[serde(default)]
    pub external_grounding: Option<Vec<String>>,
    /// Whether the term is merely a synonym (for Test 3)
    #[serde(default)]
    pub is_synonym: Option<bool>,
    /// Analysis of synonym status
    #[serde(default)]
    pub synonym_analysis: Option<String>,
    /// Number of domains the term appears in (for tier classification)
    #[serde(default)]
    pub domain_count: Option<u32>,
}

// ============================================================================
// CEP (Cognitive Evolution Pipeline) Parameters
// Patent: NV-2026-001
// ============================================================================

/// Parameters for executing a CEP stage
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepExecuteStageParams {
    /// Stage to execute: SEE, SPEAK, DECOMPOSE, COMPOSE, TRANSLATE, VALIDATE, DEPLOY, IMPROVE (or 1-8)
    pub stage: String,
    /// Domain being processed
    pub domain: String,
    /// Input data for the stage (JSON)
    #[serde(default)]
    pub input: Option<serde_json::Value>,
}

/// Parameters for CEP pipeline status
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepPipelineStatusParams {
    /// Pipeline execution ID (optional - returns current if not specified)
    #[serde(default)]
    pub execution_id: Option<String>,
}

/// Parameters for validating primitive extraction
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CepValidateExtractionParams {
    /// Coverage score (0.0-1.0): proportion of concepts expressible from primitives
    pub coverage: f64,
    /// Minimality score (0.0-1.0): absence of redundant primitives
    pub minimality: f64,
    /// Independence score (0.0-1.0): absence of implied relationships
    pub independence: f64,
    /// Custom coverage threshold (default: 0.95)
    #[serde(default)]
    pub coverage_threshold: Option<f64>,
    /// Custom minimality threshold (default: 0.90)
    #[serde(default)]
    pub minimality_threshold: Option<f64>,
    /// Custom independence threshold (default: 0.90)
    #[serde(default)]
    pub independence_threshold: Option<f64>,
}

/// Parameters for extracting primitives from a domain
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveExtractParams {
    /// Domain to extract primitives from
    pub domain: String,
    /// Extraction mode: "standard" or "deep" (default: standard)
    #[serde(default)]
    pub mode: Option<String>,
    /// Source documents or corpus (optional)
    #[serde(default)]
    pub sources: Option<Vec<String>>,
}

/// Parameters for domain translation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainTranslateParams {
    /// Source domain
    pub source_domain: String,
    /// Target domain
    pub target_domain: String,
    /// Concept to translate
    pub concept: String,
    /// Concept tier (T1, T2, T3) if known
    #[serde(default)]
    pub tier: Option<String>,
}

/// Parameters for classifying a primitive's tier
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveTierClassifyParams {
    /// Number of domains the concept appears in
    pub domain_count: usize,
}

/// Parameters for getting tier summary
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TierSummaryParams {
    /// Count of T1 Universal primitives
    pub t1_count: usize,
    /// Count of T2 Cross-Domain primitives
    pub t2_count: usize,
    /// Count of T3 Domain-Specific primitives
    pub t3_count: usize,
}

// ============================================================================
// {G, V, R} Framework Parameters (Autonomy-Aware Risk Assessment)
// ============================================================================

/// Parameters for classifying an entity by {G,V,R} capabilities
///
/// G = Goal-selection (selects own objectives)
/// V = Value-evaluation (judges good/bad)
/// R = Refusal-capacity (can decline execution)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianOriginatorClassifyParams {
    /// Entity has Goal-selection capability (selects own objectives)
    pub has_goal_selection: bool,
    /// Entity has Value-evaluation capability (judges good/bad)
    pub has_value_evaluation: bool,
    /// Entity has Refusal-capacity capability (can decline execution)
    pub has_refusal_capacity: bool,
}

/// Parameters for getting autonomy-aware ceiling limits
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianCeilingForOriginatorParams {
    /// Originator type: "tool", "agent_with_r", "agent_with_vr", "agent_with_gr", "agent_with_gvr"
    pub originator_type: String,
}

/// Parameters for PV control loop tick
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvControlLoopTickParams {
    /// Number of cases with drug AND event (cell a)
    pub a: u64,
    /// Number of cases with drug but NOT event (cell b)
    pub b: u64,
    /// Number of cases without drug but WITH event (cell c)
    pub c: u64,
    /// Number of cases without drug and without event (cell d)
    pub d: u64,
}

/// Parameters for FDA data bridge evaluation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaBridgeEvaluateParams {
    /// Number of cases with drug AND event (cell a)
    pub a: u64,
    /// Number of cases with drug but NOT event (cell b)
    pub b: u64,
    /// Number of cases without drug but WITH event (cell c)
    pub c: u64,
    /// Number of cases without drug and without event (cell d)
    pub d: u64,
}

/// Parameters for FDA data bridge batch evaluation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaBridgeBatchParams {
    /// List of contingency tables as [a, b, c, d] arrays
    pub tables: Vec<[u64; 4]>,
}

/// Parameters for computing 3D safety space point
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GuardianSpace3DComputeParams {
    /// PRR (Proportional Reporting Ratio) value
    pub prr: f64,
    /// ROR lower confidence interval
    pub ror_lower: f64,
    /// IC 2.5th percentile (Information Component)
    pub ic025: f64,
    /// EB05 (EBGM 5th percentile)
    pub eb05: f64,
    /// Number of cases
    pub n: u64,
    /// Originator type: "tool", "agent_with_r", "agent_with_vr", "agent_with_gr", "agent_with_gvr"
    #[serde(default = "default_originator")]
    pub originator: String,
    /// Harm type: "acute", "cascade", "population", "cumulative", "off_target", "interaction", "saturation", "idiosyncratic"
    #[serde(default)]
    pub harm_type: Option<String>,
    /// Hierarchy level (1-8, where 1=highest priority)
    #[serde(default = "default_hierarchy_level")]
    pub hierarchy_level: u8,
    /// Number of signal metrics present (for detectability calculation)
    #[serde(default = "default_signal_metrics")]
    pub signal_metrics_present: usize,
}

fn default_originator() -> String {
    "tool".to_string()
}

fn default_hierarchy_level() -> u8 {
    4
}

fn default_signal_metrics() -> usize {
    4
}

// ============================================================================
// MCP Lock Parameters
// ============================================================================

/// Parameters for acquiring an agent lock on the MCP server
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpLockParams {
    /// Agent identifier requesting the lock
    pub agent_id: String,
    /// Resource or state path to lock (e.g. "mcp://session/global")
    pub path: String,
    /// Lock duration in seconds (default: 3600)
    #[serde(default = "default_mcp_lock_ttl")]
    pub ttl_seconds: u64,
}

fn default_mcp_lock_ttl() -> u64 {
    3600
}

/// Parameters for releasing an agent lock on the MCP server
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct McpUnlockParams {
    /// Agent identifier releasing the lock
    pub agent_id: String,
    /// Resource path to unlock
    pub path: String,
}

// ============================================================================
// Browser/Chrome DevTools Parameters (26 tools)
// ============================================================================

// --- Navigation Tools (6) ---

/// Parameters for creating a new browser page
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserNewPageParams {
    /// URL to navigate to
    pub url: String,
    /// Whether to wait for page load (default: true)
    #[serde(default = "default_true")]
    pub wait_for_load: bool,
}

fn default_true() -> bool {
    true
}

/// Parameters for navigating the current page
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserNavigateParams {
    /// Navigation type: "url", "back", "forward", or "reload"
    #[serde(default = "default_nav_type")]
    pub nav_type: String,
    /// Target URL (required when nav_type is "url")
    #[serde(default)]
    pub url: Option<String>,
    /// Whether to ignore cache on reload
    #[serde(default)]
    pub ignore_cache: bool,
    /// Timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_nav_type() -> String {
    "url".to_string()
}

fn default_timeout_ms() -> u64 {
    30000
}

/// Parameters for selecting a page by ID
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserSelectPageParams {
    /// Page ID to select (from list_pages)
    pub page_id: String,
    /// Whether to bring the page to front
    #[serde(default)]
    pub bring_to_front: bool,
}

/// Parameters for closing a page
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserClosePageParams {
    /// Page ID to close (from list_pages)
    pub page_id: String,
}

/// Parameters for listing pages (no params needed)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListPagesParams {}

/// Parameters for waiting for text on page
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserWaitForParams {
    /// Text to wait for on the page
    pub text: String,
    /// Timeout in milliseconds (default: 30000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

// --- Input Tools (8) ---

/// Parameters for clicking an element
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserClickParams {
    /// CSS selector for the element to click
    pub selector: String,
    /// Whether to double-click (default: false)
    #[serde(default)]
    pub double_click: bool,
    /// Click at specific coordinates instead of selector
    #[serde(default)]
    pub x: Option<i32>,
    /// Y coordinate (requires x)
    #[serde(default)]
    pub y: Option<i32>,
}

/// Parameters for hovering over an element
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserHoverParams {
    /// CSS selector for the element to hover
    pub selector: String,
}

/// Parameters for filling an input field
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFillParams {
    /// CSS selector for the input element
    pub selector: String,
    /// Value to fill in
    pub value: String,
}

/// Parameters for filling multiple form fields at once
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFillFormParams {
    /// Array of {selector, value} pairs
    pub fields: Vec<BrowserFormField>,
}

/// A single form field to fill
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserFormField {
    /// CSS selector for the input element
    pub selector: String,
    /// Value to fill in
    pub value: String,
}

/// Parameters for dragging an element
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserDragParams {
    /// CSS selector for the element to drag
    pub from_selector: String,
    /// CSS selector for the drop target
    pub to_selector: String,
}

/// Parameters for pressing a key
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPressKeyParams {
    /// Key or key combination (e.g., "Enter", "Control+A", "Control+Shift+R")
    pub key: String,
}

/// Parameters for uploading a file
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserUploadFileParams {
    /// CSS selector for the file input element
    pub selector: String,
    /// Local file path to upload
    pub file_path: String,
}

/// Parameters for handling browser dialogs
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserHandleDialogParams {
    /// Action: "accept" or "dismiss"
    pub action: String,
    /// Optional prompt text (for prompt dialogs)
    #[serde(default)]
    pub prompt_text: Option<String>,
}

// --- Debugging Tools (5) ---

/// Parameters for taking a screenshot
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserScreenshotParams {
    /// Format: "png", "jpeg", or "webp" (default: "png")
    #[serde(default = "default_png")]
    pub format: String,
    /// Quality for JPEG/WebP (0-100, default: 80)
    #[serde(default = "default_quality")]
    pub quality: u8,
    /// CSS selector for element to screenshot (full page if omitted)
    #[serde(default)]
    pub selector: Option<String>,
    /// Capture full page instead of viewport
    #[serde(default)]
    pub full_page: bool,
}

fn default_png() -> String {
    "png".to_string()
}

fn default_quality() -> u8 {
    80
}

/// Parameters for taking a DOM snapshot
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserSnapshotParams {
    /// Include verbose accessibility tree information
    #[serde(default)]
    pub verbose: bool,
}

/// Parameters for evaluating JavaScript
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserEvaluateParams {
    /// JavaScript expression or function to evaluate
    pub expression: String,
    /// Arguments to pass to the function (as JSON)
    #[serde(default)]
    pub args: Option<serde_json::Value>,
}

/// Parameters for listing console messages
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListConsoleMessagesParams {
    /// Maximum number of messages to return
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by message type: "log", "error", "warn", "info", etc.
    #[serde(default)]
    pub message_type: Option<String>,
}

/// Parameters for getting a specific console message
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserGetConsoleMessageParams {
    /// Message ID (from list_console_messages)
    pub message_id: u32,
}

// --- Network Tools (2) ---

/// Parameters for listing network requests
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserListNetworkRequestsParams {
    /// Maximum number of requests to return
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by resource type: "document", "script", "stylesheet", "image", "xhr", "fetch", etc.
    #[serde(default)]
    pub resource_type: Option<String>,
}

/// Parameters for getting a specific network request
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserGetNetworkRequestParams {
    /// Request ID (from list_network_requests)
    pub request_id: u32,
}

// --- Emulation Tools (2) ---

/// Parameters for emulating device/network conditions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserEmulateParams {
    /// Network condition: "No emulation", "Offline", "Slow 3G", "Fast 3G", "Slow 4G", "Fast 4G"
    #[serde(default)]
    pub network_condition: Option<String>,
    /// CPU throttling rate (1 = no throttle, 2 = 2x slower, etc., max 20)
    #[serde(default)]
    pub cpu_throttle: Option<u8>,
    /// Geolocation latitude (-90 to 90)
    #[serde(default)]
    pub geo_latitude: Option<f64>,
    /// Geolocation longitude (-180 to 180)
    #[serde(default)]
    pub geo_longitude: Option<f64>,
}

/// Parameters for resizing the page
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserResizePageParams {
    /// Page width in pixels
    pub width: u32,
    /// Page height in pixels
    pub height: u32,
}

// --- Performance Tools (3) ---

/// Parameters for starting a performance trace
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfStartTraceParams {
    /// Reload the page after starting trace
    #[serde(default)]
    pub reload: bool,
    /// Automatically stop after ~5 seconds
    #[serde(default)]
    pub auto_stop: bool,
    /// File path to save raw trace data
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for stopping a performance trace
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfStopTraceParams {
    /// File path to save raw trace data
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for analyzing a performance insight
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrowserPerfAnalyzeParams {
    /// Insight set ID from the trace results
    pub insight_set_id: String,
    /// Insight name (e.g., "DocumentLatency", "LCPBreakdown")
    pub insight_name: String,
}

// ============================================================================
// Watchtower Parameters
// ============================================================================

/// Parameters for listing saved sessions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerSessionsListParams {}

/// Parameters for getting active sessions
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerActiveSessionsParams {}

/// Parameters for analyzing a session
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerAnalyzeParams {
    /// Path to session log file (optional - uses latest if not provided)
    #[serde(default)]
    pub session_path: Option<String>,
}

/// Parameters for getting telemetry statistics
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerTelemetryStatsParams {}

/// Parameters for getting recent log entries
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerRecentParams {
    /// Number of entries to return (default: 20)
    #[serde(default = "default_recent_count")]
    pub count: Option<usize>,
    /// Filter by session ID (first 8 chars)
    #[serde(default)]
    pub session_filter: Option<String>,
}

fn default_recent_count() -> Option<usize> {
    Some(20)
}

/// Parameters for symbol audit
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerSymbolAuditParams {
    /// Path to file or directory to audit
    pub path: String,
}

/// Parameters for Gemini telemetry statistics
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerGeminiStatsParams {}

/// Parameters for getting recent Gemini calls
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerGeminiRecentParams {
    /// Number of recent entries to return (default: 20)
    #[serde(default = "default_gemini_recent_count")]
    pub count: usize,
}

fn default_gemini_recent_count() -> usize {
    20
}

/// Parameters for unified Claude + Gemini telemetry view
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct WatchtowerUnifiedParams {
    /// Include Claude Code telemetry (default: true)
    #[serde(default = "default_true_unified")]
    pub include_claude: bool,
    /// Include Gemini telemetry (default: true)
    #[serde(default = "default_true_unified")]
    pub include_gemini: bool,
}

fn default_true_unified() -> bool {
    true
}

/// Parameters for acquiring a file lock
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationAcquireParams {
    /// File path to lock
    pub path: String,
    /// Agent identifier
    pub agent_id: String,
    /// Lock TTL in seconds (default: 300)
    #[serde(default = "default_lock_ttl")]
    pub ttl: u64,
}

fn default_lock_ttl() -> u64 {
    300
}

/// Parameters for releasing a file lock
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationReleaseParams {
    /// File path to unlock
    pub path: String,
    /// Agent identifier
    pub agent_id: String,
}

/// Parameters for checking lock status
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationStatusParams {
    /// File path to check
    pub path: String,
}

// ============================================================================
// Brain Synapse Parameters (Amplitude Growth Learning)
// ============================================================================

/// Parameters for creating/getting a synapse
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseGetOrCreateParams {
    /// Synapse ID (e.g., "pattern:my_pattern", "preference:code_style", "belief:rust_safe")
    pub id: String,
    /// Synapse type: "pattern", "preference", or "belief"
    #[serde(default = "default_synapse_type")]
    pub synapse_type: String,
}

fn default_synapse_type() -> String {
    "pattern".to_string()
}

/// Parameters for observing a learning signal
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseObserveParams {
    /// Synapse ID
    pub id: String,
    /// Confidence in the observation (0.0 to 1.0)
    pub confidence: f64,
    /// Relevance to the learning target (0.0 to 1.0)
    #[serde(default = "default_relevance")]
    pub relevance: f64,
}

fn default_relevance() -> f64 {
    1.0
}

/// Parameters for getting synapse info
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseGetParams {
    /// Synapse ID
    pub id: String,
}

/// Parameters for listing synapses with optional filter
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseListParams {
    /// Filter by type prefix (e.g., "pattern", "preference", "belief")
    #[serde(default)]
    pub filter_type: Option<String>,
    /// Only show consolidated synapses
    #[serde(default)]
    pub consolidated_only: bool,
}

// ============================================================================
// MESH API Parameters
// ============================================================================

/// Parameters for MESH descriptor lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshLookupParams {
    /// Descriptor UI (e.g., "D001241") or name (e.g., "Aspirin")
    pub identifier: String,
    /// Output format: "brief" or "full"
    #[serde(default)]
    pub format: Option<String>,
}

/// Parameters for MESH search
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshSearchParams {
    /// Search query term
    pub query: String,
    /// Maximum results (1-50, default 10)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Include scope notes in results
    #[serde(default)]
    pub include_scope_note: Option<bool>,
}

/// Parameters for MESH tree navigation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshTreeParams {
    /// Descriptor UI to navigate from
    pub descriptor_ui: String,
    /// Direction: "ancestors", "descendants", or "siblings"
    #[serde(default)]
    pub direction: Option<String>,
    /// Depth limit (1-10, default 3)
    #[serde(default)]
    pub depth: Option<u8>,
}

/// Parameters for cross-reference lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshCrossrefParams {
    /// Term to cross-reference
    pub term: String,
    /// Source terminology: "mesh", "meddra", "snomed", "ich"
    pub source: String,
    /// Target terminologies to map to
    pub targets: Vec<String>,
}

/// Parameters for PubMed MESH enrichment
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshEnrichPubmedParams {
    /// PubMed ID
    pub pmid: String,
    /// Include qualifier subheadings
    #[serde(default)]
    pub include_qualifiers: Option<bool>,
}

/// Parameters for terminology consistency check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshConsistencyParams {
    /// Terms to check
    pub terms: Vec<String>,
    /// Corpora to check: "mesh", "meddra", "ich", "snomed"
    pub corpora: Vec<String>,
}

// ============================================================================
// Telemetry Intelligence Parameters
// ============================================================================

/// Parameters for listing telemetry sources
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySourcesListParams {}

/// Parameters for analyzing a telemetry source
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySourceAnalyzeParams {
    /// Path to session file (optional)
    #[serde(default)]
    pub session_path: Option<String>,
    /// Project hash to find latest session (optional)
    #[serde(default)]
    pub project_hash: Option<String>,
}

/// Parameters for governance cross-reference
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryGovernanceCrossrefParams {
    /// Filter by category: "primitives", "governance", "capabilities", "constitutional"
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for snapshot evolution tracking
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySnapshotEvolutionParams {
    /// Session ID to get snapshots for (optional - lists all if not provided)
    #[serde(default)]
    pub session_id: Option<String>,
    /// Maximum number of sessions to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for intelligence report generation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryIntelReportParams {
    /// Maximum recent activity entries (default: 20)
    #[serde(default)]
    pub activity_limit: Option<usize>,
    /// Maximum file patterns to return (default: 50)
    #[serde(default)]
    pub file_limit: Option<usize>,
}

/// Parameters for recent telemetry activity
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryRecentParams {
    /// Number of recent operations to return (default: 20)
    #[serde(default)]
    pub count: Option<usize>,
}

// ============================================================================
// Compliance Parameters (SAM.gov, OSCAL, ICH)
// ============================================================================

/// Parameters for SAM.gov exclusion check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceCheckExclusionParams {
    /// Unique Entity Identifier (UEI) - primary identifier
    #[serde(default)]
    pub uei: Option<String>,
    /// CAGE Code (Commercial and Government Entity)
    #[serde(default)]
    pub cage_code: Option<String>,
    /// Entity name for fuzzy search
    #[serde(default)]
    pub entity_name: Option<String>,
}

/// Control input for compliance assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ControlInput {
    /// Control identifier (e.g., "ICH-E2A-1")
    pub id: String,
    /// Control title
    pub title: String,
    /// Control description
    #[serde(default)]
    pub description: Option<String>,
    /// Source catalog (e.g., "ICH-E2A", "FDA", "CIOMS")
    #[serde(default)]
    pub catalog: Option<String>,
    /// Implementation status: "implemented", "partial", "not_implemented", "na"
    pub status: String,
}

/// Finding input for compliance assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FindingInput {
    /// Related control ID
    pub control_id: String,
    /// Severity: "critical", "high", "medium", "low", "info"
    pub severity: String,
    /// Finding title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Recommended remediation
    #[serde(default)]
    pub remediation: Option<String>,
}

/// Parameters for compliance assessment
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceAssessParams {
    /// Assessment identifier
    pub assessment_id: String,
    /// Controls to assess
    pub controls: Vec<ControlInput>,
    /// Findings to record
    #[serde(default)]
    pub findings: Vec<FindingInput>,
}

/// Parameters for ICH control catalog retrieval
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceCatalogParams {
    /// Optional filter by guideline (e.g., "E2A", "E2B")
    #[serde(default)]
    pub guideline_filter: Option<String>,
}

/// Parameters for SEC EDGAR company filings lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceSecFilingsParams {
    /// Company CIK (Central Index Key) - 10 digits, leading zeros optional
    pub cik: String,
    /// Optional form type filter (e.g., "10-K", "10-Q", "8-K")
    #[serde(default)]
    pub form_filter: Option<String>,
    /// Maximum filings to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for SEC EDGAR pharma company lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ComplianceSecPharmaParams {
    /// Pharma company name: pfizer, jnj, merck, abbvie, bms, lilly, amgen, gilead, regeneron, moderna
    pub company: String,
}

// ============================================================================
// HUD Capability Parameters (CAP-025, CAP-026, CAP-027)
// ============================================================================

/// Parameters for Small Business Act agent allocation (CAP-025)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SbaAllocateAgentParams {
    /// Task description for agent allocation
    pub task_description: String,
    /// Preferred model tier: economy, standard, premium, apex (optional)
    #[serde(default)]
    pub preferred_tier: Option<String>,
}

/// Parameters for Small Business Act chain next step (CAP-025)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SbaChainNextParams {
    /// Chain ID to query
    pub chain_id: String,
    /// Completed step name
    pub completed_step: String,
    /// Whether the completed step had errors
    #[serde(default)]
    pub had_errors: bool,
}

/// Parameters for Social Security Act state persistence (CAP-026)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SsaPersistStateParams {
    /// State identifier (e.g., session ID, artifact name)
    pub state_id: String,
    /// State content to persist
    pub content: String,
    /// Persistence level: ephemeral, session, project, global (optional)
    #[serde(default)]
    pub level: Option<String>,
}

/// Parameters for Social Security Act integrity verification (CAP-026)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SsaVerifyIntegrityParams {
    /// State identifier
    pub state_id: String,
    /// Expected hash (SHA-256 hex string)
    pub expected_hash: String,
    /// Content to verify
    pub content: String,
}

/// Parameters for Federal Reserve Act budget report (CAP-027)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FedBudgetReportParams {
    /// Current token usage (optional, for context)
    #[serde(default)]
    pub current_tokens: Option<u64>,
    /// Budget limit (optional)
    #[serde(default)]
    pub budget_limit: Option<u64>,
}

/// Parameters for Federal Reserve Act model recommendation (CAP-027)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FedRecommendModelParams {
    /// Task complexity: simple, moderate, complex, research (optional)
    #[serde(default)]
    pub task_complexity: Option<String>,
    /// Current budget utilization percentage (0-100)
    #[serde(default)]
    pub budget_utilization: Option<f64>,
    /// Requires high accuracy (optional)
    #[serde(default)]
    pub requires_accuracy: Option<bool>,
}

/// Parameters for Securities Act market audit (CAP-028)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecAuditMarketParams {
    /// Market identifier to audit
    pub market_id: String,
    /// Trade volume for compliance check
    pub trade_volume: u64,
}

/// Parameters for Communications Act protocol recommendation (CAP-029)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommRecommendProtocolParams {
    /// Whether delivery guarantee is required
    #[serde(default)]
    pub needs_guarantee: bool,
    /// Whether low latency is required
    #[serde(default)]
    pub low_latency: bool,
    /// Whether message is broadcast (one-to-many)
    #[serde(default)]
    pub is_broadcast: bool,
}

/// Parameters for Communications Act message routing (CAP-029)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommRouteMessageParams {
    /// Sender agent ID
    pub from: String,
    /// Recipient agent ID
    pub to: String,
    /// Message payload (JSON string)
    pub payload: String,
    /// Protocol: mcp, jsonrpc, event, direct, rest (optional, defaults to mcp)
    #[serde(default)]
    pub protocol: Option<String>,
    /// Time-to-live in seconds (optional)
    #[serde(default)]
    pub ttl: Option<u32>,
}

/// Parameters for Exploration Act mission launch (CAP-030)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreLaunchMissionParams {
    /// Target domain/path to explore
    pub target: String,
    /// Objective description
    pub objective: String,
    /// Scope: quick, medium, thorough (optional)
    #[serde(default)]
    pub scope: Option<String>,
    /// Search patterns (optional)
    #[serde(default)]
    pub patterns: Option<Vec<String>>,
}

/// Parameters for Exploration Act discovery recording (CAP-030)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreRecordDiscoveryParams {
    /// What was found
    pub finding: String,
    /// File location (optional)
    #[serde(default)]
    pub location: Option<String>,
    /// Significance score 0.0-1.0 (optional)
    #[serde(default)]
    pub significance: Option<f64>,
}

/// Parameters for Exploration Act frontier status (CAP-030)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Quantity, Void) via scalar and container types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreGetFrontierParams {}

// ============================================================================
// HUD Capabilities - Batch 2 (CAP-014, CAP-018, CAP-019, CAP-020, CAP-022, CAP-031, CAP-037)
// ============================================================================

/// Parameters for CAP-014 Public Health Act: validate signal efficacy
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HealthValidateSignalParams {
    /// Signal identifier
    pub signal_id: String,
    /// Accuracy of signal against ground truth (0.0 - 1.0)
    pub accuracy: f64,
}

/// Parameters for CAP-014 Public Health Act: measure impact
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HealthMeasureImpactParams {
    /// Total signals analyzed
    pub total_signals: u32,
    /// Number of valid signals
    pub valid_signals: u32,
}

/// Parameters for CAP-018 Treasury Act: convert asymmetry
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TreasuryConvertAsymmetryParams {
    /// Signal identifier providing asymmetry
    pub signal_id: String,
    /// Asymmetry value (informational edge)
    pub asymmetry: f64,
    /// Market odds (implied probability)
    pub market_odds: f64,
}

/// Parameters for CAP-018 Treasury Act: audit treasury
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TreasuryAuditParams {
    /// Compute quota in treasury
    pub compute_quota: u64,
    /// Memory quota in treasury
    pub memory_quota: u64,
}

/// Parameters for CAP-019 Transportation Act: dispatch manifest
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DotDispatchManifestParams {
    /// Origin domain
    pub origin: String,
    /// Destination domain
    pub destination: String,
    /// Number of signals in batch
    pub signal_count: u32,
    /// Transit priority (1-10, higher is faster)
    pub priority: u8,
}

/// Parameters for CAP-019 Transportation Act: verify highway safety
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DotVerifyHighwayParams {
    /// Route identifier to verify
    pub route_id: String,
}

/// Parameters for CAP-020 Homeland Security Act: verify boundary
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DhsVerifyBoundaryParams {
    /// Source identifier
    pub source_id: String,
    /// Payload hash for verification
    pub payload_hash: String,
}

/// Parameters for CAP-022 Education Act: train agent
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduTrainAgentParams {
    /// Subject area of training
    pub subject: String,
    /// Complexity level (1-10)
    pub level: u8,
    /// Current completion progress (0.0 - 1.0)
    pub completion: f64,
}

/// Parameters for CAP-022 Education Act: evaluate comprehension
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduEvaluateParams {
    /// List of comprehension scores
    pub scores: Vec<f64>,
}

/// Parameters for CAP-031 Science Foundation Act: fund research
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NsfFundResearchParams {
    /// Research project title
    pub project: String,
    /// Target capability ID being enhanced
    pub target_cap: String,
}

/// Parameters for CAP-037 General Services Act: procure resource
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsaProcureParams {
    /// Resource identifier
    pub resource_id: String,
    /// Quantity to procure
    pub quantity: u64,
    /// Priority (1-10)
    pub priority: u8,
}

/// Parameters for CAP-037 General Services Act: audit service value
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsaAuditValueParams {
    /// Cost of service
    pub cost: f64,
    /// Benefit of service
    pub benefit: f64,
}

// ========================================================================
// Documentation Generation Params
// ========================================================================

/// Parameters for autonomous CLAUDE.md generation
///
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsGenerateClaudeMdParams {
    /// Path to codebase root (defaults to current directory)
    pub path: Option<String>,
    /// Include architecture section (default: true)
    pub include_architecture: Option<bool>,
    /// Include command reference (default: true)
    pub include_commands: Option<bool>,
    /// Include key directories (default: true)
    pub include_directories: Option<bool>,
}

// ============================================================================
// Hormone (Endocrine System) Parameters
// ============================================================================

/// Parameters for getting a specific hormone level
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HormoneGetParams {
    /// Hormone name: cortisol, dopamine, serotonin, adrenaline, oxytocin, melatonin
    pub hormone: String,
}

/// Parameters for applying a stimulus to the endocrine system
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HormoneStimulusParams {
    /// Stimulus type (e.g., "error", "task_completed", "positive_feedback")
    pub stimulus_type: String,
    /// Intensity (0.0-1.0) for applicable stimuli
    pub intensity: Option<f64>,
    /// Count for count-based stimuli
    pub count: Option<u32>,
    /// Recoverable flag for critical errors
    pub recoverable: Option<bool>,
}

// ============================================================================
// Immunity (Antipattern Detection) Parameters
// ============================================================================

/// Parameters for scanning code content for antipatterns.
/// Tier: T2-C (κ + μ + σ - comparison + mapping + sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityScanParams {
    /// The code content to scan for antipatterns.
    pub content: String,
    /// Optional file path for context-aware pattern matching (e.g., "src/lib.rs", "Cargo.toml").
    #[serde(default)]
    pub file_path: Option<String>,
}

// ============================================================================
// Primitive Scanner Parameters
// ============================================================================

/// Parameters for scanning a domain for primitives.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveScanParams {
    /// Domain name to analyze.
    pub domain: String,
    /// Source file paths or glob patterns.
    #[serde(default)]
    pub sources: Vec<String>,
}

/// A term with definition for batch testing.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TermForTest {
    /// The term being tested.
    pub term: String,
    /// Natural language definition.
    pub definition: String,
    /// Domain terms found in definition (for Test 1).
    #[serde(default)]
    pub domain_terms: Vec<String>,
    /// External grounding concepts (for Test 2).
    #[serde(default)]
    pub external_grounding: Vec<String>,
    /// Number of domains where term appears.
    pub domain_count: Option<usize>,
}

/// Parameters for batch testing terms.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveBatchTestParams {
    /// Terms to test.
    pub terms: Vec<TermForTest>,
}

// ============================================================================
// STEM Primitives Parameters (stem crate system)
// ============================================================================

/// Parameters for combining two confidence values
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemConfidenceCombineParams {
    /// First confidence value [0.0, 1.0]
    pub a: f64,
    /// Second confidence value [0.0, 1.0]
    pub b: f64,
}

/// Parameters for tier info lookup
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (String)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemTierInfoParams {
    /// Tier name: T1, T2-P, T2-C, or T3
    pub tier: String,
}

/// Parameters for chemistry equilibrium balance
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64, Option)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemBalanceParams {
    /// Forward reaction rate
    pub forward_rate: f64,
    /// Reverse reaction rate
    pub reverse_rate: f64,
    /// Tolerance for equilibrium check (default: 0.01)
    #[serde(default)]
    pub tolerance: Option<f64>,
}

/// Parameters for chemistry fraction/saturation check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemFractionParams {
    /// Fraction value [0.0, 1.0]
    pub value: f64,
}

/// Parameters for F=ma calculation
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysFmaParams {
    /// Force value (Newtons)
    pub force: f64,
    /// Mass value (kg, must be > 0)
    pub mass: f64,
}

/// Parameters for quantity conservation check
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysConservationParams {
    /// Quantity before operation
    pub before: f64,
    /// Quantity after operation
    pub after: f64,
    /// Tolerance for conservation check
    pub tolerance: f64,
}

/// Parameters for frequency-to-period conversion
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysPeriodParams {
    /// Frequency in Hz (must be > 0)
    pub frequency: f64,
}

/// Parameters for bounds checking
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (f64, Option)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathBoundsCheckParams {
    /// Value to check
    pub value: f64,
    /// Lower bound (optional)
    #[serde(default)]
    pub lower: Option<f64>,
    /// Upper bound (optional)
    #[serde(default)]
    pub upper: Option<f64>,
}

/// Parameters for relation inversion
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts via field types (String)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathRelationInvertParams {
    /// Relation: LessThan, Equal, GreaterThan, or Incomparable
    pub relation: String,
}

// ============================================================================
// Visualization Parameters (nexcore-viz)
// ============================================================================

/// Parameters for STEM taxonomy sunburst visualization.
/// Returns self-contained SVG string.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizTaxonomyParams {
    /// Title for the diagram (default: "STEM Taxonomy")
    #[serde(default)]
    pub title: Option<String>,
}

/// Parameters for type composition visualization.
/// Shows how a type decomposes to T1 Lex Primitiva.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizCompositionParams {
    /// Type name (e.g., "Machine", "Integrity", "Force")
    pub type_name: String,
    /// Tier classification (T1, T2-P, T2-C, T3)
    pub tier: String,
    /// Comma-separated list of T1 primitives (e.g., "Mapping,Sequence,State")
    pub primitives: String,
    /// Dominant primitive name (optional)
    #[serde(default)]
    pub dominant: Option<String>,
    /// Confidence in grounding (0.0-1.0, default 0.80)
    #[serde(default)]
    pub confidence: Option<f64>,
}

/// Parameters for science loop visualization.
/// Renders a circular flow diagram for any STEM composite.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizLoopParams {
    /// Which loop to render: "science", "chemistry", or "math"
    pub domain: String,
}

/// Parameters for confidence chain visualization.
/// Shows confidence propagation through a derivation chain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizConfidenceParams {
    /// JSON array of claims: [{"text":"...", "confidence":0.95, "proof_type":"analytical", "parent":null}]
    pub claims: String,
    /// Title for the diagram
    #[serde(default)]
    pub title: Option<String>,
}

/// Parameters for bounds visualization.
/// Renders a number line with bounded value.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizBoundsParams {
    /// The value to visualize
    pub value: f64,
    /// Lower bound (optional)
    #[serde(default)]
    pub lower: Option<f64>,
    /// Upper bound (optional)
    #[serde(default)]
    pub upper: Option<f64>,
    /// Label for the value
    #[serde(default)]
    pub label: Option<String>,
}

/// Parameters for DAG topology visualization.
/// Renders a layered directed acyclic graph.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizDagParams {
    /// JSON array of edges: [["from","to"], ...]
    pub edges: String,
    /// Title for the diagram
    #[serde(default)]
    pub title: Option<String>,
}

// ============================================================================
// Signal Pipeline Parameters (signal-stats / signal-core)
// ============================================================================

/// Parameters for single drug-event signal detection via signal-stats pipeline.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalDetectParams {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT preferred)
    pub event: String,
    /// Cell a: drug+ event+
    pub a: u64,
    /// Cell b: drug+ event-
    pub b: u64,
    /// Cell c: drug- event+
    pub c: u64,
    /// Cell d: drug- event-
    pub d: u64,
}

/// Parameters for batch signal detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalBatchParams {
    /// List of drug-event pairs to analyze
    pub items: Vec<SignalDetectParams>,
}

// ============================================================================
// Algovigilance Parameters
// ============================================================================

/// Parameters for comparing two ICSR narratives
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilDedupPairParams {
    /// First ICSR narrative text
    pub narrative_a: String,
    /// Second ICSR narrative text
    pub narrative_b: String,
    /// Similarity threshold (0.0-1.0, default: 0.85)
    #[serde(default = "default_dedup_threshold")]
    pub threshold: f64,
}

fn default_dedup_threshold() -> f64 {
    0.85
}

/// Parameters for batch FAERS deduplication
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilDedupBatchParams {
    /// Drug name to fetch FAERS cases for
    pub drug: String,
    /// Similarity threshold (0.0-1.0, default: 0.85)
    #[serde(default = "default_dedup_threshold")]
    pub threshold: f64,
    /// Maximum cases to fetch (default: 50)
    #[serde(default = "default_batch_limit")]
    pub limit: usize,
}

fn default_batch_limit() -> usize {
    50
}

/// Parameters for signal triage with decay
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageDecayParams {
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// Half-life in days (default: 30.0)
    #[serde(default = "default_half_life")]
    pub half_life_days: f64,
}

fn default_half_life() -> f64 {
    30.0
}

/// Parameters for reinforcing a signal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageReinforceParams {
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// Number of new supporting cases
    pub new_cases: u32,
}

/// Parameters for getting the signal triage queue
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AlgovigilTriageQueueParams {
    /// Drug name to get queue for
    pub drug: String,
    /// Half-life in days (default: 30.0)
    #[serde(default = "default_half_life")]
    pub half_life_days: f64,
    /// Minimum relevance cutoff (default: 0.1)
    #[serde(default = "default_cutoff")]
    pub cutoff: f64,
    /// Maximum signals to return (default: 10)
    #[serde(default = "default_queue_limit")]
    pub limit: usize,
}

fn default_cutoff() -> f64 {
    0.1
}

fn default_queue_limit() -> usize {
    10
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

/// Parameters for batch edit distance: compare query against multiple candidates.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceBatchParams {
    /// Query string to compare against candidates
    pub query: String,
    /// Candidate strings to compare against
    pub candidates: Vec<String>,
    /// Maximum results to return (default: all)
    pub limit: Option<usize>,
    /// Minimum similarity threshold to include (default: 0.0)
    pub min_similarity: Option<f64>,
    /// Algorithm: "levenshtein" (default), "damerau", "lcs"
    pub algorithm: Option<String>,
}

/// Parameters for cross-domain transfer confidence lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EditDistanceTransferParams {
    /// Source domain (e.g., "text/unicode", "bioinformatics/dna")
    pub source_domain: String,
    /// Target domain (e.g., "pharmacovigilance", "spell-checking")
    pub target_domain: String,
}

// ============================================================================
// Telemetry Parameters
// ============================================================================

/// Parameters for telemetry summary query.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives (unit type - no parameters needed)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySummaryParams {
    /// Placeholder - no parameters needed for summary
    #[serde(default)]
    pub _unused: Option<()>,
}

/// Parameters for per-tool telemetry query.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives (String)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetryByToolParams {
    /// Tool name to query (e.g., "pv_signal_prr")
    pub tool_name: String,
}

/// Parameters for slow calls query.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives (u64, Option<usize>)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TelemetrySlowCallsParams {
    /// Duration threshold in milliseconds
    pub threshold_ms: u64,
    /// Maximum number of results to return (default: all)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for audit trail query.
///
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives (String, Option<String>, Option<bool>, Option<usize>)
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AuditTrailParams {
    /// Filter by tool name (exact match, e.g., "pv_signal_prr")
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Only return records after this ISO-8601 timestamp
    #[serde(default)]
    pub since: Option<String>,
    /// Filter by success status (true = only successes, false = only failures)
    #[serde(default)]
    pub success_only: Option<bool>,
    /// Maximum number of results to return (default: 50)
    #[serde(default)]
    pub limit: Option<usize>,
}

// ============================================================================
// Unified Dispatcher Parameters
// ============================================================================

/// Parameters for the unified `nexcore` dispatcher tool.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UnifiedParams {
    /// Command name (e.g. "foundation_levenshtein", "pv_signal_complete").
    /// Use "help" for full catalog.
    pub command: String,
    /// Command-specific parameters as JSON object.
    #[serde(default)]
    pub params: serde_json::Value,
}

// ============================================================================
// Brain Health Metrics Parameters
// ============================================================================

/// Parameters for brain growth rate analysis
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Quantity) via scalar types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainGrowthRateParams {
    /// Number of days to analyze (default: 7)
    #[serde(default = "default_growth_days")]
    pub days: u32,
}

fn default_growth_days() -> u32 {
    7
}

/// Parameters for retrieving largest artifacts
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Quantity) via scalar types.
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainLargestArtifactsParams {
    /// Number of artifacts to return (default: 10)
    #[serde(default = "default_largest_n")]
    pub n: usize,
}

fn default_largest_n() -> usize {
    10
}

// ============================================================================
// Lex Primitiva Parameters (T1 Symbolic Foundation)
// ============================================================================

/// Parameters for listing all 15 Lex Primitiva symbols
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Sum) via collection enumeration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaListParams {
    /// If true, include symbol notation (σ, μ, ς, etc.) in output
    #[serde(default)]
    pub include_symbols: bool,
}

/// Parameters for getting details about a specific Lex Primitiva
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Existence, Mapping) via identity lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaGetParams {
    /// Name of the primitive (e.g., "Sequence", "Mapping", "State")
    pub name: String,
}

/// Parameters for classifying a type's grounding tier
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Comparison, Sum) via tier classification.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaTierParams {
    /// Type name to classify (e.g., "ThresholdGate", "Bdi", "SignalResult")
    pub type_name: String,
}

/// Parameters for computing primitive composition of a grounded type
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Mapping, Sum, Quantity) via composition analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaCompositionParams {
    /// Type name to analyze (e.g., "Bdi", "EcsScore", "SafetyMargin")
    pub type_name: String,
}

/// Parameters for reverse-composing T1 primitives upward through the tier DAG.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Mapping, Comparison, Sequence) via reverse synthesis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaReverseComposeParams {
    /// Primitive names to compose (e.g., ["Boundary", "Comparison"])
    pub primitives: Vec<String>,
    /// Optional target pattern name hint (e.g., "Gatekeeper")
    #[serde(default)]
    pub pattern_hint: Option<String>,
    /// Minimum coherence threshold (0.0-1.0, default 0.0)
    #[serde(default)]
    pub min_coherence: Option<f64>,
}

/// Parameters for reverse-looking up grounded types by their T1 primitives.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Mapping, Comparison) via reverse lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaReverseLookupParams {
    /// Primitive names to search for (e.g., ["Boundary"])
    pub primitives: Vec<String>,
    /// Match mode: "exact", "superset", "subset" (default: "superset")
    #[serde(default)]
    pub match_mode: Option<String>,
}

/// Parameters for computing molecular weight of a word/concept.
///
/// Provide `primitives` (list of primitive names) to compute the Shannon
/// information-theoretic weight of a concept. Algorithm A76.
///
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sum, Quantity, Mapping) via weight calculation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaMolecularWeightParams {
    /// Primitive names composing the word (e.g., ["State", "Comparison", "Boundary"])
    pub primitives: Vec<String>,
    /// Optional concept name for labeling (e.g., "Competency")
    #[serde(default)]
    pub name: Option<String>,
    /// If true, include the full periodic table in the response
    #[serde(default)]
    pub include_periodic_table: bool,
}

/// Get the disambiguated State (ς) mode for a grounded type.
///
/// Returns Mutable, Modal, or Accumulated — or None if the type
/// has no state component.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaStateModeParams {
    /// The type name to query (e.g., "String", "CircuitBreaker", "HashMap")
    pub type_name: String,
}

/// Parameters for self-synthesis of new primitives (Level 5 Evolution)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaSynthParams {
    /// Natural language description of the observed pattern
    pub description: String,
    /// Sample data illustrating the new structure (JSON)
    pub sample_data: serde_json::Value,
}

// ============================================================================
// Decision Tree Parameters (CART Engine)
// ============================================================================

/// Parameters for training a decision tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Mapping, State) via feature matrix and labels.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeTrainParams {
    /// Feature matrix: each row is a sample, each column a feature (numeric).
    pub features: Vec<Vec<f64>>,
    /// Class labels (one per row). Use numeric strings for regression.
    pub labels: Vec<String>,
    /// Splitting criterion: "gini" (default), "entropy", "gain_ratio", "mse"
    #[serde(default)]
    pub criterion: Option<String>,
    /// Maximum tree depth (default: unlimited)
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Minimum samples to attempt a split (default: 2)
    #[serde(default)]
    pub min_samples_split: Option<usize>,
    /// Minimum samples per leaf (default: 1)
    #[serde(default)]
    pub min_samples_leaf: Option<usize>,
    /// Feature names for explainability
    #[serde(default)]
    pub feature_names: Option<Vec<String>>,
}

/// Parameters for predicting with a trained decision tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreePredictParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Feature values for prediction (one per feature)
    pub features: Vec<f64>,
}

/// Parameters for getting feature importance from a trained tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeImportanceParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
}

/// Parameters for pruning a trained decision tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreePruneParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Cost-complexity pruning alpha parameter (higher = more pruning)
    pub alpha: f64,
}

/// Parameters for exporting a trained decision tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeExportParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Export format: "json" (default), "rules", "summary"
    #[serde(default)]
    pub format: Option<String>,
}

/// Parameters for getting info about a trained decision tree.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeInfoParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
}

// ============================================================================
// Sentinel Parameters
// ============================================================================

/// Parameters for checking if an IP is whitelisted by sentinel.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelCheckIpParams {
    /// IP address to check (IPv4 or IPv6)
    pub ip: String,
    /// Optional CIDR ranges to check against (default: 127.0.0.1/8, ::1/128)
    #[serde(default)]
    pub whitelist_cidrs: Vec<String>,
}

/// Parameters for parsing an auth log line.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelParseLineParams {
    /// A syslog auth line (e.g., "Feb  4 14:23:01 host sshd[100]: Failed password for root from 10.0.0.1 port 22 ssh2")
    pub line: String,
}

/// Parameters for getting sentinel config defaults.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SentinelConfigDefaultsParams {
    /// Output format: "json" (default) or "toml"
    #[serde(default)]
    pub format: Option<String>,
}

// =========================================================================
// Measure — Workspace quality measurement
// =========================================================================

/// Parameters for measuring a single crate's health.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureCrateParams {
    /// Crate name (e.g., "nexcore-vigilance")
    pub name: String,
}

/// Parameters for Shannon entropy calculation.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureEntropyParams {
    /// Category counts (e.g., lines per module: [100, 200, 50, 300])
    pub counts: Vec<usize>,
}

/// Parameters for metric drift detection.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureDriftParams {
    /// Window size for drift comparison (default: 5)
    #[serde(default)]
    pub window: Option<usize>,
}

/// Parameters for side-by-side crate comparison.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureCompareParams {
    /// First crate name
    pub crate_a: String,
    /// Second crate name
    pub crate_b: String,
}

/// Parameters for statistical summary.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeasureStatsParams {
    /// Numeric data points (at least 3 required)
    pub counts: Vec<f64>,
}

// ============================================================================
// FAERS ETL Parameters (4 tools — local bulk data pipeline)
// ============================================================================

/// Parameters for running the full FAERS ETL pipeline on local quarterly ASCII data.
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1 Concepts (Sequence, Mapping, State) via file I/O and batch computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlRunParams {
    /// Path to FAERS quarterly ASCII directory (e.g., /home/matthew/data/faers/faers_ascii_2025Q4/ASCII)
    pub faers_dir: String,
    /// Minimum case count to include (default: 3)
    #[serde(default)]
    pub min_cases: Option<i64>,
    /// Include all drug roles, not just suspects (default: false)
    #[serde(default)]
    pub include_all_roles: Option<bool>,
    /// Max results to return (default: 50)
    #[serde(default)]
    pub top_n: Option<usize>,
}

/// Parameters for searching FAERS ETL signals by drug and/or event name.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlSignalsParams {
    /// Path to FAERS quarterly ASCII directory
    pub faers_dir: String,
    /// Filter signals by drug name (case-insensitive, substring match)
    #[serde(default)]
    pub drug: Option<String>,
    /// Filter signals by event name (case-insensitive, substring match)
    #[serde(default)]
    pub event: Option<String>,
    /// Minimum case count (default: 3)
    #[serde(default)]
    pub min_cases: Option<i64>,
    /// Include all drug roles (default: false)
    #[serde(default)]
    pub include_all_roles: Option<bool>,
}

/// Parameters for validating known drug-event pairs against local FAERS data.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlKnownPairsParams {
    /// Path to FAERS quarterly ASCII directory
    pub faers_dir: String,
    /// Known drug-event pairs to validate: [{"drug":"DUPIXENT","event":"CONJUNCTIVITIS"}, ...]
    pub pairs: Vec<FaersEtlDrugEventPair>,
    /// Minimum case count (default: 3)
    #[serde(default)]
    pub min_cases: Option<i64>,
}

/// A single drug-event pair for known-pair validation.
/// Tier: T2-C (Composite of two T2-P newtypes)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlDrugEventPair {
    /// Drug name (matched case-insensitively)
    pub drug: String,
    /// Event/reaction term (substring match)
    pub event: String,
}

/// Parameters for checking status of cached FAERS Parquet output files.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersEtlStatusParams {
    /// Output directory to check (default: ~/nexcore/output)
    #[serde(default)]
    pub output_dir: Option<String>,
}

// ============================================================================
// FAERS Analytics Parameters (A77, A80, A82 — Novel Signal Detection)
// ============================================================================

/// Parameters for A82 Outcome-Conditioned Signal Strength.
/// Computes outcome-severity-weighted PRR adjustments from FAERS reaction outcome data.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersOutcomeConditionedParams {
    /// Array of cases, each with drug, event, and outcome_code ("1"-"6").
    /// Outcome codes: 1=Recovered, 2=Recovering, 3=Not recovered, 4=Recovered with sequelae, 5=Fatal, 6=Unknown
    pub cases: Vec<FaersOutcomeCase>,
    /// Pre-computed standard PRRs for (drug, event) pairs. Each entry: {"drug": "X", "event": "Y", "prr": 2.5}
    #[serde(default)]
    pub standard_prrs: Vec<FaersStandardPrr>,
    /// Minimum cases required (default: 3)
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// PRR threshold for signal detection (default: 2.0)
    #[serde(default)]
    pub prr_threshold: Option<f64>,
}

/// A single case for A82 analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersOutcomeCase {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT)
    pub event: String,
    /// Reaction outcome code: "1"=Recovered, "2"=Recovering, "3"=Not recovered, "4"=Recovered with sequelae, "5"=Fatal, "6"=Unknown
    pub outcome_code: String,
}

/// Pre-computed standard PRR for a drug-event pair.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersStandardPrr {
    /// Drug name
    pub drug: String,
    /// Event name
    pub event: String,
    /// Standard PRR value
    pub prr: f64,
}

/// Parameters for A77 Signal Velocity Detector.
/// Detects emerging signals by measuring temporal acceleration in reporting frequency.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSignalVelocityParams {
    /// Array of temporal cases, each with drug, event, and receipt_date (YYYYMMDD format)
    pub cases: Vec<FaersTemporalCase>,
    /// Minimum months of data required (default: 3)
    #[serde(default)]
    pub min_months: Option<usize>,
    /// Minimum total cases required (default: 3)
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Acceleration threshold for early warning (default: 0.5)
    #[serde(default)]
    pub acceleration_threshold: Option<f64>,
    /// Known PRRs for early warning cross-referencing. Each: {"drug": "X", "event": "Y", "prr": 1.5}
    #[serde(default)]
    pub known_prrs: Vec<FaersStandardPrr>,
}

/// A single temporal case for A77 analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersTemporalCase {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT)
    pub event: String,
    /// Receipt date in YYYYMMDD format (e.g., "20240115")
    pub receipt_date: String,
}

/// Parameters for A80 Seriousness Cascade Detector.
/// Detects signals escalating in severity using all 6 FAERS seriousness flags.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSeriousnessCascadeParams {
    /// Array of cases with drug, event, seriousness flags, and receipt_date
    pub cases: Vec<FaersSeriousnessCase>,
    /// Minimum cases required (default: 3)
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Death rate threshold for immediate human review (default: 0.1 = 10%)
    #[serde(default)]
    pub death_rate_threshold: Option<f64>,
}

/// A single case with seriousness flags for A80 analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersSeriousnessCase {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT)
    pub event: String,
    /// Receipt date in YYYYMMDD format
    pub receipt_date: String,
    /// "1" if death occurred, omit or null otherwise
    #[serde(default)]
    pub seriousness_death: Option<String>,
    /// "1" if hospitalization occurred
    #[serde(default)]
    pub seriousness_hospitalization: Option<String>,
    /// "1" if resulted in disability
    #[serde(default)]
    pub seriousness_disabling: Option<String>,
    /// "1" if caused congenital anomaly
    #[serde(default)]
    pub seriousness_congenital: Option<String>,
    /// "1" if life-threatening
    #[serde(default)]
    pub seriousness_life_threatening: Option<String>,
    /// "1" if other medically important
    #[serde(default)]
    pub seriousness_other: Option<String>,
}

// ============================================================================
// FAERS A78 — Polypharmacy Interaction Signal
// ============================================================================

/// Parameters for Algorithm A78: Polypharmacy Interaction Signal.
/// Tier: T2-C (×+∂+κ+N)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyParams {
    /// Array of cases, each with multiple drugs and one event
    pub cases: Vec<FaersPolypharmacyCase>,
    /// Minimum pair co-occurrence count (default: 3)
    #[serde(default)]
    pub min_pair_count: Option<u32>,
    /// Interaction signal threshold (default: 1.0)
    #[serde(default)]
    pub interaction_threshold: Option<f64>,
}

/// A single case for polypharmacy analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyCase {
    /// Case identifier (safety report ID)
    pub case_id: String,
    /// List of drugs in this case
    pub drugs: Vec<FaersPolypharmacyDrug>,
    /// Event name (MedDRA PT)
    pub event: String,
}

/// A drug entry within a polypharmacy case.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersPolypharmacyDrug {
    /// Drug name
    pub name: String,
    /// Drug characterization code: "1"=Suspect, "2"=Concomitant, "3"=Interacting
    pub characterization: String,
}

// ============================================================================
// FAERS A79 — Reporter-Weighted Disproportionality
// ============================================================================

/// Parameters for Algorithm A79: Reporter-Weighted Disproportionality.
/// Tier: T2-C (∃+κ+N+∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersReporterWeightedParams {
    /// Array of cases with drug, event, and reporter qualification
    pub cases: Vec<FaersReporterCase>,
    /// Minimum raw case count (default: 3)
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Diversity threshold for multi-source confirmation (default: 0.5)
    #[serde(default)]
    pub diversity_threshold: Option<f64>,
}

/// A single case for reporter-weighted analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersReporterCase {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT)
    pub event: String,
    /// Reporter qualification code: "1"=Physician, "2"=Pharmacist, "3"=OtherHP, "4"=Lawyer, "5"=Consumer
    pub qualification_code: String,
}

// ============================================================================
// FAERS A81 — Geographic Signal Divergence
// ============================================================================

/// Parameters for Algorithm A81: Geographic Signal Divergence.
/// Tier: T2-C (λ+κ+ν+∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersGeographicDivergenceParams {
    /// Array of cases with drug, event, and country
    pub cases: Vec<FaersGeographicCase>,
    /// Minimum total cases (default: 5)
    #[serde(default)]
    pub min_cases: Option<u32>,
    /// Minimum countries required (default: 2)
    #[serde(default)]
    pub min_countries: Option<usize>,
    /// Divergence ratio threshold (default: 3.0)
    #[serde(default)]
    pub divergence_threshold: Option<f64>,
    /// P-value threshold for heterogeneity (default: 0.05)
    #[serde(default)]
    pub p_value_threshold: Option<f64>,
    /// Minimum cases per country to include (default: 2)
    #[serde(default)]
    pub min_country_cases: Option<u32>,
}

/// A single case for geographic analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FaersGeographicCase {
    /// Drug name
    pub drug: String,
    /// Event name (MedDRA PT)
    pub event: String,
    /// Occurrence country (ISO 2-letter code, e.g. "US", "JP", "DE")
    pub country: String,
}

// ============================================================================
// Molecular Biology Parameters (Central Dogma, ADME, Codon Translation)
// ============================================================================

/// Parameters for translating a single codon to amino acid.
/// Tier: T2-P (Mapping: codon → amino acid)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularTranslateCodonParams {
    /// RNA codon (3 nucleotides: A, U, G, C). Example: "AUG"
    pub codon: String,
}

/// Parameters for translating mRNA to protein.
/// Tier: T2-C (μ + σ composition)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularTranslateMrnaParams {
    /// mRNA sequence (nucleotides: A, U, G, C). Example: "AUGUUUUAA"
    pub mrna: String,
    /// Start translation from first AUG codon (default: false = start at position 0)
    #[serde(default)]
    pub from_start: Option<bool>,
}

/// Parameters for Central Dogma stage mapping.
/// Tier: T2-P (Biology → Computation mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularCentralDogmaParams {
    /// Stage: "transcription", "translation", "folding", "replication", "proofreading", or "all"
    pub stage: String,
}

/// Parameters for ADME pharmacokinetic phase mapping.
/// Tier: T2-P (Pharmacokinetics → Computation mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MolecularAdmePhaseParams {
    /// Phase: "A"/"absorption", "D"/"distribution", "M"/"metabolism", "E"/"elimination", or "all"
    pub phase: String,
}

// ============================================================================
// Visual Primitives Parameters
// ============================================================================

/// Parameters for classifying a shape by its T1 primitive composition.
/// Tier: T2-P (Visual → Primitive mapping)
/// Grounds to T1: shapes → λ (location) + N (quantity) + σ (sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VisualShapeClassifyParams {
    /// Shape name or symbol: circle/⊙, triangle/△, line/→, rectangle/□, point/λ, polygon
    pub shape: String,
}

/// Parameters for analyzing a color's T1 primitive decomposition.
/// Tier: T2-P (Visual → Quantity mapping)
/// Grounds to T1: Color → N(r) + N(g) + N(b) + N(a)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VisualColorAnalyzeParams {
    /// Color as hex (#ff0000) or named (red, blue, green, white, black)
    pub color: String,
}

/// Parameters for listing all shape primitives.
/// Tier: T1 (Catalog query - no parameters)
#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(crate = "rmcp::serde")]
pub struct VisualShapeListParams {}

/// Parameters for scanning error output for known patterns.
/// Tier: T2-C (κ + μ - pattern matching on compiler errors)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityScanErrorsParams {
    /// The stderr/error output to scan (e.g., from cargo build).
    pub stderr: String,
}

/// Parameters for getting a specific antibody by ID.
/// Tier: T2-P (μ - mapping ID to antibody)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityGetParams {
    /// The antibody ID (e.g., "PANIC-001", "SYNTH-PAT-001").
    pub id: String,
}

/// Parameters for listing antibodies with optional filters.
/// Tier: T2-C (σ + κ - iteration with filtering)
#[derive(Debug, Deserialize, JsonSchema, Default)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityListParams {
    /// Filter by threat type: "PAMP" or "DAMP".
    #[serde(default)]
    pub threat_type: Option<String>,
    /// Filter by minimum severity: "low", "medium", "high", "critical".
    #[serde(default)]
    pub min_severity: Option<String>,
}

/// Parameters for proposing a new antibody from an observed error.
/// Tier: T2-C (π + μ + → - persistence + mapping + causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ImmunityProposeParams {
    /// The error message or pattern that triggered the issue.
    pub error_pattern: String,
    /// The fix that was applied to resolve the issue.
    pub fix_applied: String,
    /// Context about where this occurred (e.g., file, function, task).
    #[serde(default)]
    pub context: Option<String>,
    /// Suggested severity: "low", "medium", "high", "critical".
    #[serde(default)]
    pub severity: Option<String>,
}

// ============================================================================
// Cytokine Parameters
// ============================================================================

/// Parameters for emitting a cytokine signal.
/// Tier: T2-C (→ + π - causality + persistence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineEmitParams {
    /// Cytokine family: "il1", "il2", "il6", "il10", "tnf_alpha", "ifn_gamma", "tgf_beta", "csf"
    pub family: String,
    /// Signal name (e.g., "threat_detected", "rate_limit_exceeded")
    pub name: String,
    /// Severity level: "trace", "low", "medium", "high", "critical"
    #[serde(default)]
    pub severity: Option<String>,
    /// Scope: "autocrine", "paracrine", "endocrine", "systemic"
    #[serde(default)]
    pub scope: Option<String>,
    /// Optional JSON payload data
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Parameters for listing cytokine families.
/// Tier: T2-P (Σ - sum type enumeration)
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineListParams {
    /// Optional filter by family name (substring match)
    #[serde(default)]
    pub family_filter: Option<String>,
}

/// Parameters for querying recent cytokine signals from file-based telemetry.
/// Tier: T2-P (σ - sequence with N - quantity limit)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CytokineRecentParams {
    /// Maximum number of recent cytokines to return (default: 20, max: 100)
    #[serde(default = "default_cytokine_recent_limit")]
    pub limit: u32,
    /// Optional family filter (e.g., "tnf_alpha", "il6")
    #[serde(default)]
    pub family: Option<String>,
}

fn default_cytokine_recent_limit() -> u32 {
    20
}

/// Parameters for computing chemotactic gradient routing.
/// Tier: T2-C (→ + λ — causality + location)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChemotaxisGradientParams {
    /// Array of gradient samples, each with: source, family, concentration (0-1), distance
    pub gradients: Vec<GradientSample>,
}

/// A single gradient sample for chemotaxis computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GradientSample {
    /// Source identifier
    pub source: String,
    /// Cytokine family: "il1", "il6", "tnf_alpha", etc.
    pub family: String,
    /// Signal concentration [0.0, 1.0]
    pub concentration: f64,
    /// Distance from agent (abstract units)
    pub distance: f64,
    /// Tropism: "positive" (attract) or "negative" (repel). Default: positive
    #[serde(default = "default_tropism")]
    pub tropism: String,
}

fn default_tropism() -> String {
    "positive".to_string()
}

/// Parameters for endocytosis pool operations.
/// Tier: T2-C (∂ + ρ — boundary + recursion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EndocytosisInternalizeParams {
    /// Cytokine family to internalize
    pub family: String,
    /// Signal name
    pub name: String,
    /// Severity: "trace", "low", "medium", "high", "critical"
    #[serde(default)]
    pub severity: Option<String>,
    /// Pool capacity (default: 10)
    #[serde(default = "default_pool_capacity")]
    pub pool_capacity: usize,
}

fn default_pool_capacity() -> usize {
    10
}

// ============================================================================
// Value Mining Parameters
// ============================================================================

/// Parameters for listing value signal types.
/// Tier: T2-P (Σ - sum type enumeration)
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueSignalTypesParams {
    /// Optional filter by signal type name
    #[serde(default)]
    pub type_filter: Option<String>,
}

/// Parameters for detecting value signals from numeric inputs.
/// Tier: T2-C (N + κ + → - quantity + comparison + causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueSignalDetectParams {
    /// Signal type: "sentiment", "trend", "engagement", "virality", "controversy"
    pub signal_type: String,
    /// Current observed value (positive rate, trend slope, engagement, etc.)
    pub observed: f64,
    /// Baseline/expected value for comparison
    pub baseline: f64,
    /// Sample size (number of observations)
    pub sample_size: usize,
    /// Entity this signal relates to (e.g., "TSLA", "Bitcoin")
    pub entity: String,
    /// Source identifier (e.g., "wallstreetbets", "cryptocurrency")
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String {
    "unknown".to_string()
}

/// Parameters for creating a baseline.
/// Tier: T2-C (N + ν + π - quantity + frequency + persistence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValueBaselineCreateParams {
    /// Source identifier (e.g., subreddit name)
    pub source: String,
    /// Average positive sentiment rate (0.0 to 1.0)
    #[serde(default = "default_sentiment_rate")]
    pub positive_rate: f64,
    /// Average negative sentiment rate (0.0 to 1.0)
    #[serde(default = "default_sentiment_rate")]
    pub negative_rate: f64,
    /// Average engagement per post
    #[serde(default = "default_engagement")]
    pub avg_engagement: f64,
    /// Average posts per hour
    #[serde(default = "default_posts_per_hour")]
    pub posts_per_hour: f64,
}

fn default_sentiment_rate() -> f64 {
    0.5
}

fn default_engagement() -> f64 {
    100.0
}

fn default_posts_per_hour() -> f64 {
    10.0
}

/// Parameters for getting the PV algorithm mapping.
/// Tier: T2-P (μ - mapping between domains)
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ValuePvMappingParams {
    /// Optional signal type to filter mapping (returns all if not specified)
    #[serde(default)]
    pub signal_type: Option<String>,
}

// ============================================================================
// FDA AI Credibility Assessment Parameters (5 tools)
// ============================================================================

/// Parameters for defining Context of Use (Step 2).
/// Tier: T2-C (λ + μ - location + mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDefineCouParams {
    /// The regulatory question requiring an answer (e.g., "Is drug X safe for population Y?")
    pub question: String,
    /// Input domain for the model (e.g., "Patient demographics + AE reports")
    pub input_domain: String,
    /// Output domain for the model (e.g., "Safety signal scores")
    pub output_domain: String,
    /// Description of what the model transforms (e.g., "PRR/ROR signal detection")
    pub purpose_description: String,
    /// Evidence integration type: "sole", "primary", "contributory", "supplementary"
    pub integration: String,
    /// Sources that confirm or supplement AI outputs
    #[serde(default)]
    pub confirmatory_sources: Option<Vec<String>>,
    /// Regulatory context: "ind", "nda", "bla", "postmarket", "manufacturing"
    pub regulatory_context: String,
}

/// Parameters for assessing AI model risk (Step 3).
/// Tier: T2-C (κ × N - comparison × quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaAssessRiskParams {
    /// Model influence on decision: "high", "medium", "low"
    pub influence: String,
    /// Consequence of incorrect decision: "high", "medium", "low"
    pub consequence: String,
}

/// Parameters for creating a credibility plan (Step 4).
/// Tier: T2-C (σ - sequence of assessment activities)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaCreatePlanParams {
    /// The regulatory question
    pub question: String,
    /// Input domain for the model
    pub input_domain: String,
    /// Output domain for the model
    pub output_domain: String,
    /// Model influence: "high", "medium", "low"
    pub influence: String,
    /// Decision consequence: "high", "medium", "low"
    pub consequence: String,
    /// Regulatory context: "ind", "nda", "bla", "postmarket", "manufacturing"
    pub regulatory_context: String,
}

/// Parameters for validating evidence (Steps 5-6).
/// Tier: T2-C (∃ + κ - existence + comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaValidateEvidenceParams {
    /// Evidence type: "validation_metrics", "code_review", "documentation", "testing"
    pub evidence_type: String,
    /// Evidence quality: "high", "medium", "low"
    pub quality: String,
    /// Description of the evidence (e.g., "ROC AUC = 0.92 (95% CI: 0.89-0.95)")
    pub description: String,
    /// Is the evidence relevant to the COU?
    pub relevant: bool,
    /// Is the evidence reliable (accurate, complete, traceable)?
    pub reliable: bool,
    /// Does the evidence represent the target population?
    pub representative: bool,
}

/// Parameters for determining adequacy (Step 7).
/// Tier: T2-C (κ - comparison against criteria)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDecideAdequacyParams {
    /// Risk level: "high", "medium", "low"
    pub risk_level: String,
    /// Count of high-quality evidence items collected
    pub high_quality_evidence_count: usize,
    /// Did the data pass fit-for-use assessment?
    pub fit_for_use_passed: bool,
    /// Was critical drift detected requiring revalidation?
    pub critical_drift_detected: bool,
}

// ============================================================================
// FDA Credibility Metrics
// ============================================================================

/// Parameters for calculating credibility score.
/// T1 Grounding: N (Weighted linear combination)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaCalculateScoreParams {
    /// Evidence quality score (0.0 - 1.0)
    pub evidence_quality: f64,
    /// Fit for use score (0.0 - 1.0)
    pub fit_for_use: f64,
    /// Risk mitigation score (0.0 - 1.0)
    pub risk_mitigation: f64,
    /// Documentation score (0.0 - 1.0)
    pub documentation: f64,
}

/// Parameters for assessment metrics summary.
/// T1 Grounding: Σ (Aggregation)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaMetricsSummaryParams {
    /// Number of assessments started
    pub started: usize,
    /// Number of assessments completed
    pub completed: usize,
    /// Number of assessments approved
    pub approved: usize,
    /// Number of assessments rejected
    pub rejected: usize,
    /// Number of assessments needing revision
    pub revision: usize,
    /// Number of drift alerts
    pub drift_alerts: usize,
}

/// Parameters for evidence distribution analysis.
/// T1 Grounding: N (Quantity) + ν (Frequency)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaEvidenceDistributionParams {
    /// List of (evidence_type, quality) pairs
    /// Types: "metrics", "test", "data", "architecture", "bias", "explain", "literature", "precedent"
    /// Quality: "high", "medium", "low"
    pub evidence_items: Vec<(String, String)>,
}

/// Parameters for risk distribution analysis.
/// T1 Grounding: N (Quantity) + κ (Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaRiskDistributionParams {
    /// List of risk levels: "high", "medium", "low"
    pub risk_levels: Vec<String>,
}

/// Parameters for drift trend analysis.
/// T1 Grounding: ν (Frequency) + σ (Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDriftTrendParams {
    /// Measurements: (timestamp, drift_percent, severity, model_id)
    pub measurements: Vec<(u64, f64, String, String)>,
    /// Threshold for worsening trend detection
    pub trend_threshold: f64,
}

// ============================================================================
// Prima Language Parameters
// ============================================================================

/// Parameters for parsing Prima source code.
/// T1 Grounding: μ (Mapping) + σ (Sequence) + → (Causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaParseParams {
    /// Prima source code to parse
    pub source: String,
}

/// Parameters for evaluating Prima expressions.
/// T1 Grounding: μ (Mapping) + → (Causality) + N (Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaEvalParams {
    /// Prima source code to evaluate
    pub source: String,
}

/// Parameters for Prima code generation.
/// T1 Grounding: μ (Mapping) + → (Causality) + Σ (Sum/target selection)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaCodegenParams {
    /// Prima source code to compile
    pub source: String,
    /// Target language: rust, python, typescript, go, c (default: rust)
    pub target: Option<String>,
}

/// Parameters for Prima primitive analysis.
/// T1 Grounding: σ (Sequence) + κ (Comparison) + N (Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimaPrimitivesParams {
    /// Prima source code to analyze
    pub source: String,
}

// ============================================================================
// Aggregate Parameters (Σ + ρ + κ)
// ============================================================================

/// Parameters for fold_all aggregation over numeric values.
/// T1 Grounding: Σ (Sum) + σ (Sequence) + N (Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateFoldParams {
    /// Numeric values to aggregate.
    pub values: Vec<f64>,
}

/// Parameters for recursive tree fold.
/// T1 Grounding: ρ (Recursion) + Σ (Sum) + κ (Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateTreeFoldParams {
    /// Tree as JSON: {"id": "root", "value": 1.0, "children": [...]}
    pub tree: serde_json::Value,
    /// Combine function: "sum", "max", or "mean"
    #[serde(default = "default_combine_fn")]
    pub combine: String,
}

fn default_combine_fn() -> String {
    "sum".to_string()
}

/// Parameters for ranking named values.
/// T1 Grounding: κ (Comparison) + N (Quantity) + σ (Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateRankParams {
    /// List of [name, value] pairs to rank.
    pub items: Vec<(String, f64)>,
    /// Number of top entries to return (0 = all).
    #[serde(default)]
    pub top_n: usize,
}

/// Parameters for percentile computation.
/// T1 Grounding: κ (Comparison) + ∝ (Proportion) + N (Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregatePercentileParams {
    /// Numeric values.
    pub values: Vec<f64>,
    /// Percentile to compute (0.0 to 1.0).
    pub percentile: f64,
}

/// Parameters for outlier detection.
/// T1 Grounding: κ (Comparison) + ∂ (Boundary) + N (Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AggregateOutliersParams {
    /// List of [name, value] pairs to check.
    pub items: Vec<(String, f64)>,
}

// ============================================================================
// Compound Growth Parameters (Lex Primitiva)
// ============================================================================

/// Parameters for compound growth projection.
/// T1 Grounding: N (Quantity) + ∂ (Divergence/Derivative) + ∝ (Proportion)
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundGrowthParams {
    /// Tier to add primitives to: "T1", "T2-P", "T2-C", "T3"
    #[serde(default)]
    pub add_tier: Option<String>,
    /// Number of primitives to add to the specified tier
    #[serde(default)]
    pub add_count: Option<u32>,
}

// ============================================================================
// Compound Growth Detector Parameters (Phase + Bottleneck Detection)
// ============================================================================

/// Parameters for compound growth phase and bottleneck detection.
/// Accepts a time-series of basis snapshots for analysis.
/// Tier: T2-C (sigma + kappa + proportional -- Sequence + Comparison + Proportion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundDetectorParams {
    /// Array of basis snapshots in chronological order.
    /// Each snapshot captures the primitive basis at a point in time.
    pub snapshots: Vec<CompoundDetectorSnapshot>,
}

/// A single basis snapshot for compound growth detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundDetectorSnapshot {
    /// Session identifier (e.g. "session-001")
    pub session: String,
    /// T1 primitive count
    pub t1_count: u32,
    /// T2-P primitive count
    pub t2_p_count: u32,
    /// T2-C primitive count
    pub t2_c_count: u32,
    /// T3 primitive count
    pub t3_count: u32,
    /// Primitives reused from existing basis
    pub reused: u32,
    /// Total primitives needed for this session
    pub total_needed: u32,
}

// ============================================================================
// Claude Care Process (CCP) Parameters — Pharmacokinetic Engine
// ============================================================================

/// Parameters for starting a new care episode.
/// Tier: T2-C (σ + ς — sequence start + state initialization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpEpisodeStartParams {
    /// Unique episode identifier
    pub episode_id: String,
    /// Epoch hours when episode starts (default: 0.0)
    #[serde(default)]
    pub started_at: Option<f64>,
}

/// Parameters for computing a recommended dose.
/// Tier: T2-C (∝ + κ — proportionality + comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpDoseComputeParams {
    /// Dosing strategy: "subtherapeutic", "therapeutic", "loading", "maintenance"
    pub strategy: String,
    /// Target plasma level [0, 1]
    pub target_level: f64,
    /// Current plasma level (for titration strategies)
    #[serde(default)]
    pub current_level: Option<f64>,
    /// Bioavailability (0, 1] (default: 0.8)
    #[serde(default)]
    pub bioavailability: Option<f64>,
    /// Half-life in hours (default: 24.0)
    #[serde(default)]
    pub half_life: Option<f64>,
}

/// Parameters for advancing an episode (dose + decay + phase transition).
/// Tier: T2-C (σ + ∝ + ∂ — sequence + proportionality + boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpEpisodeAdvanceParams {
    /// Episode identifier
    pub episode_id: String,
    /// Current phase: "collect", "assess", "plan", "implement", "followup"
    pub current_phase: String,
    /// Current plasma level
    #[serde(default)]
    pub current_plasma: Option<f64>,
    /// Dose to administer [0, 1]
    #[serde(default)]
    pub dose: Option<f64>,
    /// Bioavailability (0, 1] (default: 0.8)
    #[serde(default)]
    pub bioavailability: Option<f64>,
    /// Half-life in hours (default: 24.0)
    #[serde(default)]
    pub half_life: Option<f64>,
    /// Dosing strategy
    #[serde(default)]
    pub strategy: Option<String>,
    /// Hours of decay to apply
    #[serde(default)]
    pub decay_hours: Option<f64>,
    /// Target phase to transition to
    #[serde(default)]
    pub target_phase: Option<String>,
    /// Reason for phase transition
    #[serde(default)]
    pub reason: Option<String>,
    /// Timestamp (epoch hours)
    #[serde(default)]
    pub timestamp: Option<f64>,
}

/// Parameters for checking interaction effects.
/// Tier: T2-C (∝ + κ — proportionality + comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpInteractionCheckParams {
    /// Plasma level of first intervention
    pub level_a: f64,
    /// Plasma level of second intervention
    pub level_b: f64,
    /// Interaction type: "synergistic", "antagonistic", "additive", "potentiating"
    pub interaction_type: String,
}

/// Parameters for scoring episode quality.
/// Tier: T2-C (κ + ∂ — comparison + boundary normalization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpQualityScoreParams {
    /// Current plasma level [0, 1]
    pub plasma_level: f64,
    /// Average bioavailability of interventions (default: 0.8)
    #[serde(default)]
    pub avg_bioavailability: Option<f64>,
    /// Average half-life in hours (default: 24.0)
    #[serde(default)]
    pub avg_half_life: Option<f64>,
    /// Representative dose value (default: 0.5)
    #[serde(default)]
    pub dose: Option<f64>,
}

/// Parameters for validating/executing a phase transition.
/// Tier: T2-C (σ + ∂ — sequence + boundary guard)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CcpPhaseTransitionParams {
    /// Source phase: "collect", "assess", "plan", "implement", "followup"
    pub from: String,
    /// Target phase
    pub to: String,
    /// Reason for transition
    #[serde(default)]
    pub reason: Option<String>,
    /// Timestamp (epoch hours)
    #[serde(default)]
    pub timestamp: Option<f64>,
}

// ============================================================================
// Energy Parameters (Token-as-ATP/ADP biochemistry)
// ============================================================================

/// Parameters for computing energy charge and full state snapshot.
/// Tier: T2-C (N + κ + ∝ — Quantity + Comparison + Proportion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyChargeParams {
    /// Total token budget (all tokens start as tATP)
    pub budget: u64,
    /// Tokens spent on productive work (tATP → tADP)
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted (tATP → tAMP)
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Total value produced (for coupling efficiency calculation)
    #[serde(default)]
    pub total_value: Option<f64>,
}

/// Parameters for deciding the optimal strategy for a specific operation.
/// Tier: T2-C (N + κ + ∝ — Quantity + Comparison + Proportion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EnergyDecideParams {
    /// Total token budget
    pub budget: u64,
    /// Tokens spent on productive work
    #[serde(default)]
    pub productive_spent: Option<u64>,
    /// Tokens wasted
    #[serde(default)]
    pub wasted: Option<u64>,
    /// Label for the operation being considered
    pub operation_label: String,
    /// Estimated token cost of the operation
    pub estimated_cost: u64,
    /// Estimated value of the operation (arbitrary units)
    pub estimated_value: f64,
    /// Whether a cached result might exist
    #[serde(default)]
    pub cache_possible: Option<bool>,
}

// ─── Reverse Transcriptase (Schema Inference + Data Generation) ─────────────

/// Full transcriptase pipeline: infer + merge + violations + fidelity.
/// Tier: T2-C (κ + σ + μ + ∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseProcessParams {
    /// JSON string to process (single record or array of records)
    pub json: String,
    /// Synthesize boundary violations (default: true)
    #[serde(default)]
    pub violations: Option<bool>,
    /// Verify round-trip fidelity (default: false)
    #[serde(default)]
    pub verify: Option<bool>,
}

/// Schema inference only — no violations or fidelity.
/// Tier: T2-P (κ + σ)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseInferParams {
    /// JSON string to infer schema from
    pub json: String,
}

/// Synthesize boundary violations from observed data.
/// Tier: T2-C (κ + σ + ∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseViolationsParams {
    /// JSON string to analyze for boundary violations
    pub json: String,
}

/// Generate synthetic data from observed JSON schema.
/// Tier: T2-C (κ + σ + μ + ∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TranscriptaseGenerateParams {
    /// JSON string to observe (schema will be inferred from this)
    pub json: String,
    /// Number of synthetic records to generate (default: 1)
    #[serde(default)]
    pub count: Option<usize>,
}

// ─── Ribosome (Schema Contract Registry + Drift Detection) ─────────────────

/// Store a baseline contract from JSON data.
/// Tier: T2-C (κ + σ + μ)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeStoreParams {
    /// Unique contract identifier
    pub contract_id: String,
    /// JSON string — schema will be inferred from this data
    pub json: String,
}

/// Validate data against a stored contract (drift detection).
/// Tier: T2-C (κ + σ + ∂ + N)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeValidateParams {
    /// Contract ID to validate against
    pub contract_id: String,
    /// JSON string to validate
    pub json: String,
}

/// Generate synthetic data from a stored contract.
/// Tier: T2-C (κ + σ + μ)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeGenerateParams {
    /// Contract ID to generate from
    pub contract_id: String,
    /// Number of synthetic records (default: 1)
    #[serde(default)]
    pub count: Option<usize>,
}

/// Batch drift detection across contracts.
/// Tier: T2-C (κ + σ + μ + ∂ + N)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RibosomeDriftParams {
    /// Map of contract_id → JSON string to validate
    pub data: std::collections::HashMap<String, String>,
}

// ============================================================================
// Domain Primitives (Tier Taxonomy + Transfer Confidence)
// ============================================================================

/// List primitives from a domain taxonomy, optionally filtered by tier.
/// Tier: T2-C (σ + κ + ρ — Sequence + Comparison + Recursion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesListParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Filter by tier: "T1", "T2-P", "T2-C", "T3" (omit for all)
    #[serde(default)]
    pub tier: Option<String>,
}

/// Compute cross-domain transfer confidence for primitives.
/// Tier: T2-C (κ + ∝ — Comparison + Proportionality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTransferParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Filter by primitive name (omit for all)
    #[serde(default)]
    pub primitive_name: Option<String>,
    /// Filter by target domain (omit for all)
    #[serde(default)]
    pub target_domain: Option<String>,
}

/// Decompose a primitive into its T1 foundation tree.
/// Tier: T2-C (ρ + σ — Recursion + Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesDecomposeParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Name of the primitive to decompose
    pub primitive_name: String,
}

/// Bottleneck analysis: primitives ranked by transitive fan-out.
/// Tier: T2-C (κ + N + σ — Comparison + Quantity + Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesBottlenecksParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Max results to return (default: 10)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Compare two domain taxonomies to find shared and unique primitives.
/// Tier: T2-C (κ + N — Comparison + Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesCompareParams {
    /// First taxonomy name (default: golden-dome)
    #[serde(default)]
    pub taxonomy_a: Option<String>,
    /// Second taxonomy name (default: pharmacovigilance)
    #[serde(default)]
    pub taxonomy_b: Option<String>,
}

/// Topological ordering of primitives (dependencies before dependents).
/// Tier: T2-P (σ + → — Sequence + Causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTopoSortParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
}

/// Critical paths from T1 roots to T3 leaves (longest dependency chains).
/// Tier: T2-C (σ + → + ρ — Sequence + Causality + Recursion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesCriticalPathsParams {
    /// Taxonomy: "golden-dome" or "pharmacovigilance" (default: golden-dome)
    #[serde(default)]
    pub taxonomy: Option<String>,
    /// Max paths to return (default: 5)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// List all registered domain taxonomies.
/// Tier: T1 (∃ — Existence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesRegistryParams {}

/// Save a taxonomy to a JSON file.
/// Tier: T1 (π — Persistence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesSaveParams {
    /// Taxonomy name to save (e.g., "golden-dome", "pharmacovigilance")
    pub taxonomy: String,
    /// File path to save to (JSON format)
    pub path: String,
}

/// Load a taxonomy from a JSON file into the registry.
/// Tier: T1 (∃ — Existence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesLoadParams {
    /// File path to load from (JSON format)
    pub path: String,
}

/// Parameters for computing the cross-domain transfer matrix.
/// Tier: T2-C (μ + → + N — mapping + causality + quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainPrimitivesTransferMatrixParams {
    /// Max bridges to return (default: 10)
    pub limit: Option<usize>,
}

// ============================================================================
// Bonding (Hook-Skill Molecular Bonding) Parameters
// ============================================================================

/// Parameters for analyzing molecular stability.
/// Tier: T2-C (N + κ + ρ — quantity + comparison + recursion)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BondingAnalyzeParams {
    /// Molecule name or JSON/YAML content
    pub molecule: String,
}

/// Parameters for evolving a molecule through double-loop reflection.
/// Tier: T2-C (ρ + ς + → — recursion + state + causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BondingEvolveParams {
    /// Original molecule name
    pub molecule: String,
    /// Reflection results or reason for evolution
    pub reflection: String,
}

// ============================================================================
// Forge (Primitive-First Technology Construction) Parameters
// ============================================================================

/// Initialize a new Forge session.
/// Tier: T1 (ς — State initialization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeInitParams {
    /// Optional session ID (auto-generated if not provided)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Get primitive reference card.
/// Tier: T1 (∃ — Existence query)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeReferenceParams {}

/// Mine primitives from a concept.
/// Tier: T2-P (μ + → — Mapping + Causality)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeMineParams {
    /// Concept name to decompose
    pub concept: String,
    /// T1/T2 primitive symbols (e.g., ["σ", "μ", "→"])
    pub primitives: Vec<String>,
    /// Decomposition rationale
    pub decomposition: String,
}

/// Generate forge prompt for a task.
/// Tier: T2-C (→ + ρ + μ — Causality + Recursion + Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgePromptParams {
    /// Task name
    pub name: String,
    /// Task description
    pub description: String,
    /// Domain (default: "general")
    #[serde(default)]
    pub domain: Option<String>,
    /// Target tier: T1, T2-P, T2-C, or T3
    #[serde(default)]
    pub target_tier: Option<String>,
}

/// Get session summary.
/// Tier: T1 (Σ — Sum/aggregation)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeSummaryParams {}

/// Classify tier from primitive count.
/// Tier: T1 (κ — Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeTierParams {
    /// Number of primitives
    pub count: usize,
}

/// Get Gemini system prompt for Forge mode.
/// Tier: T1 (π — Persistence/retrieval)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ForgeSystemPromptParams {}

// ============================================================================
// State Operating System (SOS) Parameters
// ============================================================================

/// State specification for SOS machine creation.
/// Tier: T2-P (ς + μ — State + Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosStateSpec {
    /// State name
    pub name: String,
    /// State kind: "initial", "normal", "terminal", or "error"
    pub kind: String,
}

/// Transition specification for SOS machine creation.
/// Tier: T2-P (→ + μ — Causality + Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosTransitionSpec {
    /// Source state name
    pub from: String,
    /// Target state name
    pub to: String,
    /// Event name triggering the transition
    pub event: String,
}

/// Parameters for creating a new state machine.
/// Tier: T3 (ς + → + μ + N — Full state machine specification)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosCreateParams {
    /// Machine name
    pub name: String,
    /// List of state specifications
    pub states: Vec<SosStateSpec>,
    /// List of transition specifications
    pub transitions: Vec<SosTransitionSpec>,
}

/// Parameters for executing a state transition.
/// Tier: T2-P (→ + ς — Causality + State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosTransitionParams {
    /// Machine ID
    pub machine_id: u64,
    /// Event name to trigger
    pub event: String,
}

/// Parameters for querying machine state.
/// Tier: T1 (ς — State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosStateParams {
    /// Machine ID
    pub machine_id: u64,
}

/// Parameters for querying machine transition history.
/// Tier: T2-P (σ + ς — Sequence + State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosHistoryParams {
    /// Machine ID
    pub machine_id: u64,
    /// Maximum number of entries to return (default: 50)
    #[serde(default = "default_sos_history_limit")]
    pub limit: usize,
}

fn default_sos_history_limit() -> usize {
    50
}

/// Parameters for validating a machine specification.
/// Tier: T3 (ς + → + μ + ∃ — State, Causality, Mapping, Existence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosValidateParams {
    /// Machine name
    pub name: String,
    /// List of state specifications
    pub states: Vec<SosStateSpec>,
    /// List of transition specifications
    pub transitions: Vec<SosTransitionSpec>,
}

/// Parameters for listing active machines.
/// Tier: T2-P (Σ + μ — Sum + Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosListParams {
    /// Filter pattern (optional, matches machine name)
    #[serde(default)]
    pub filter: Option<String>,
}

/// Parameters for cycle detection in machine transitions.
/// Tier: T2-P (ρ + ς — Recursion + State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosCyclesParams {
    /// Machine ID
    pub machine_id: u64,
    /// Include self-loops in results (default: true)
    #[serde(default = "default_sos_include_self_loops")]
    pub include_self_loops: bool,
}

fn default_sos_include_self_loops() -> bool {
    true
}

/// Parameters for irreversibility audit trail query.
/// Tier: T2-P (∝ + ς — Irreversibility + State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosAuditParams {
    /// Machine ID
    pub machine_id: u64,
    /// Maximum number of audit entries (default: 100)
    #[serde(default = "default_sos_audit_limit")]
    pub limit: usize,
}

fn default_sos_audit_limit() -> usize {
    100
}

/// Parameters for temporal scheduling operations.
/// Tier: T2-P (ν + ς — Frequency + State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosScheduleParams {
    /// Machine ID
    pub machine_id: u64,
    /// Event to schedule
    pub event: String,
    /// Delay in ticks before firing
    pub delay_ticks: u64,
}

/// Parameters for location-based routing.
/// Tier: T2-P (λ + μ — Location + Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SosRouteParams {
    /// Machine ID to route
    pub machine_id: u64,
    /// Target location ID (optional, auto-routes if omitted)
    #[serde(default)]
    pub location_id: Option<u64>,
}

// ============================================================================
// SQI (Skill Quality Index) Parameters
// ============================================================================

/// Parameters for scoring a single skill's SQI.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SqiScoreParams {
    /// SKILL.md content to score (full file text)
    pub content: String,
}

/// Parameters for ecosystem-level SQI scoring.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SqiEcosystemParams {
    /// Tool counts per skill/server (e.g., [262, 26, 9, 4, 2])
    pub tool_counts: Vec<usize>,
    /// Optional SKILL.md contents to score individually
    #[serde(default)]
    pub skill_contents: Vec<String>,
}

// ─── Cortex (Local LLM Inference) ───────────────────────────────────────────

/// Parameters for downloading a model from HuggingFace Hub.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexDownloadParams {
    /// HuggingFace repo ID (e.g. "QuantFactory/SmolLM2-135M-Instruct-GGUF")
    pub repo_id: String,
    /// Filename within the repo (e.g. "SmolLM2-135M-Instruct-Q4_K_M.gguf")
    pub filename: String,
}

/// Parameters for text generation with a local model.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexGenerateParams {
    /// The prompt to generate from
    pub prompt: String,
    /// HuggingFace repo ID of the model to use
    pub repo_id: String,
    /// Maximum tokens to generate (default: 512)
    #[serde(default = "default_cortex_max_tokens")]
    pub max_tokens: usize,
    /// Sampling temperature 0.0-1.0 (default: 0.7)
    #[serde(default = "default_cortex_temperature")]
    pub temperature: f64,
}

fn default_cortex_max_tokens() -> usize {
    512
}

fn default_cortex_temperature() -> f64 {
    0.7
}

/// Parameters for listing cached models.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexListModelsParams {
    /// Optional filter by repo ID substring
    #[serde(default)]
    pub filter: Option<String>,
}

/// Parameters for getting model info.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexModelInfoParams {
    /// HuggingFace repo ID
    pub repo_id: String,
    /// Filename within the repo
    pub filename: String,
}

/// Parameters for generating text embeddings.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexEmbedParams {
    /// Text to embed
    pub text: String,
    /// HuggingFace repo ID of the model to use
    pub repo_id: String,
}

/// Parameters for checking fine-tune job status.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CortexFineTuneStatusParams {
    /// Job ID to check
    pub job_id: String,
}

// ============================================================================
// Monitoring Parameters
// ============================================================================

/// Parameters for monitoring alerts query.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringAlertsParams {
    /// Filter by severity level (CRITICAL, HIGH, WARN, INFO). If omitted, returns all.
    #[serde(default)]
    pub severity_filter: Option<String>,
    /// Maximum number of alerts to return (default: 20, max: 100).
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for monitoring hook health query.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringHookHealthParams {
    /// Specific hook name to analyze. If omitted, returns all hooks.
    #[serde(default)]
    pub hook_name: Option<String>,
}

/// Parameters for monitoring signal digest query.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MonitoringSignalDigestParams {
    /// Time window in minutes (default: 60).
    #[serde(default)]
    pub window_minutes: Option<u64>,
}

// ============================================================================
// Security Classification Parameters (5-level clearance system)
// ============================================================================

/// Parameters for evaluating a gate operation (access, write, or external call).
/// Tier: T3 (∂ Boundary + κ Comparison + ς State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceEvaluateParams {
    /// Target kind: "project", "crate", "file", "skill", "mcp_tool", "region", "data_category"
    pub target_kind: String,
    /// Target name/path
    pub target_name: String,
    /// Classification level: "Public", "Internal", "Confidential", "Secret", "TopSecret"
    pub level: String,
    /// Operation type: "access", "write", "external_call"
    #[serde(default = "default_clearance_op")]
    pub operation: String,
    /// External tool name (only for "external_call" operation)
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Actor identity
    #[serde(default = "default_clearance_actor")]
    pub actor: String,
}

fn default_clearance_op() -> String {
    "access".to_string()
}

fn default_clearance_actor() -> String {
    "claude".to_string()
}

/// Parameters for looking up the policy for a specific classification level.
/// Tier: T2-P (ς State + ∂ Boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearancePolicyForParams {
    /// Classification level: "Public", "Internal", "Confidential", "Secret", "TopSecret"
    pub level: String,
}

/// Parameters for validating a classification change (upgrade/downgrade).
/// Tier: T2-P (∂ Boundary + κ Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceValidateChangeParams {
    /// Current classification level
    pub from_level: String,
    /// Target classification level
    pub to_level: String,
    /// Current access mode: "Unrestricted", "Aware", "Guarded", "Enforced", "Lockdown"
    #[serde(default = "default_clearance_mode")]
    pub mode: String,
    /// Whether downgrade is explicitly permitted
    #[serde(default)]
    pub downgrade_permitted: bool,
}

fn default_clearance_mode() -> String {
    "Guarded".to_string()
}

/// Parameters for querying classification level metadata.
/// Tier: T1 (ς State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClearanceLevelInfoParams {
    /// Classification level: "Public", "Internal", "Confidential", "Secret", "TopSecret"
    pub level: String,
}

// ============================================================================
// Secure Boot params (3 tools)
// ============================================================================

/// Parameters for querying secure boot chain status.
/// Tier: T2-C (σ + → + ∂ — boot chain state)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootStatusParams {
    /// Boot policy: "Strict", "Degraded", "Permissive" (default: Permissive)
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

fn default_boot_policy() -> String {
    "Permissive".to_string()
}

/// Parameters for verifying a boot stage measurement.
/// Tier: T2-P (κ Comparison — expected vs actual)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootVerifyParams {
    /// Boot stage: "Firmware", "Bootloader", "Kernel", "NexCoreOs", "Init", "Services", "Shell", "Apps"
    pub stage: String,
    /// Data to measure (as UTF-8 string — hashed via SHA-256)
    pub data: String,
    /// Optional expected hash (hex string). If omitted, measurement is recorded without verification.
    #[serde(default)]
    pub expected_hex: Option<String>,
    /// Boot policy: "Strict", "Degraded", "Permissive" (default: Permissive)
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

/// Parameters for generating a boot quote (PCR summary).
/// Tier: T2-C (Σ + ∝ — aggregated irreversible measurements)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootQuoteParams {
    /// Boot stages to measure, as JSON array of {stage, data} objects.
    /// Example: [{"stage":"Firmware","data":"uefi-v2.0"},{"stage":"Kernel","data":"vmlinuz-6.17"}]
    pub stages: Vec<SecureBootStageInput>,
    /// Boot policy: "Strict", "Degraded", "Permissive" (default: Permissive)
    #[serde(default = "default_boot_policy")]
    pub policy: String,
}

/// A single stage input for boot quote generation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecureBootStageInput {
    /// Boot stage name
    pub stage: String,
    /// Data to measure (hashed via SHA-256)
    pub data: String,
    /// Description of what was measured
    #[serde(default)]
    pub description: Option<String>,
}

// ============================================================================
// User Management params (7 tools)
// ============================================================================

/// Parameters for creating a new user account.
/// Tier: T2-C (∃ + ∂ + μ — identity creation with boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserCreateParams {
    /// Username (will be normalized to lowercase).
    pub username: String,
    /// Display name for the user.
    pub display_name: String,
    /// Password (min 8 chars, must contain upper + lower + digit).
    pub password: String,
    /// Role: "Guest", "User", "Admin", "Owner" (default: User)
    #[serde(default = "default_user_role")]
    pub role: String,
}

fn default_user_role() -> String {
    "User".to_string()
}

/// Parameters for user login (authentication).
/// Tier: T2-P (κ Comparison — credential verification)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLoginParams {
    /// Username to authenticate.
    pub username: String,
    /// Password to verify.
    pub password: String,
}

/// Parameters for user logout (session invalidation).
/// Tier: T2-P (ς State — session lifecycle)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLogoutParams {
    /// Session token to invalidate.
    pub token: String,
}

/// Parameters for locking a user account.
/// Tier: T2-P (ς State — account state transition)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserLockParams {
    /// Username to lock.
    pub username: String,
}

/// Parameters for unlocking a user account.
/// Tier: T2-P (ς State — account state transition)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserUnlockParams {
    /// Username to unlock.
    pub username: String,
}

/// Parameters for changing a user's password.
/// Tier: T2-P (∝ Irreversibility — credential rotation)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct UserChangePasswordParams {
    /// Username whose password to change.
    pub username: String,
    /// Current password (for verification).
    pub old_password: String,
    /// New password (min 8 chars, upper + lower + digit).
    pub new_password: String,
}

// ============================================================================
// Claude FS params (9 tools)
// ============================================================================

/// Parameters for claude_fs_list.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsListParams {
    /// Relative path under ~/.claude/ to list. Use "." for root.
    #[serde(default = "default_dot")]
    pub path: String,
}

/// Parameters for claude_fs_read.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsReadParams {
    /// Relative path under ~/.claude/ to read.
    pub path: String,
}

/// Parameters for claude_fs_write.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsWriteParams {
    /// Relative path under ~/.claude/ to write.
    pub path: String,
    /// Content to write.
    pub content: String,
    /// Whether to create parent directories. Default: true.
    pub create_dirs: Option<bool>,
}

/// Parameters for claude_fs_delete.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsDeleteParams {
    /// Relative path under ~/.claude/ to delete.
    pub path: String,
}

/// Parameters for claude_fs_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsSearchParams {
    /// Substring to search for in file contents.
    pub query: String,
    /// Root directory to search under (relative to ~/.claude/). Default: ".".
    pub root: Option<String>,
    /// Maximum results to return. Default: 200.
    pub max_results: Option<usize>,
}

/// Parameters for claude_fs_tail.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsTailParams {
    /// Relative path under ~/.claude/ to tail.
    pub path: String,
    /// Number of lines from the end. Default: 100.
    pub lines: Option<usize>,
}

/// Parameters for claude_fs_diff.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsDiffParams {
    /// First file path (relative to ~/.claude/).
    pub path_a: String,
    /// Second file path (relative to ~/.claude/).
    pub path_b: String,
}

/// Parameters for claude_fs_stat.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeFsStatParams {
    /// Relative path under ~/.claude/ to stat.
    pub path: String,
}

fn default_dot() -> String {
    ".".to_string()
}

// ============================================================================
// Compendious params (5 tools)
// ============================================================================

/// Parameters for compendious score_text.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousScoreParams {
    /// Text to score for information density.
    pub text: String,
    /// Required elements that must be present (for completeness calculation).
    pub required_elements: Option<Vec<String>>,
}

/// Parameters for compendious compress_text.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousCompressParams {
    /// Text to compress using BLUFF method.
    pub text: String,
    /// Terms to preserve (skip patterns that contain these words).
    pub preserve: Option<Vec<String>>,
}

/// Parameters for compendious compare_texts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousCompareParams {
    /// Original (uncompressed) text.
    pub original: String,
    /// Optimized (compressed) text.
    pub optimized: String,
}

/// Parameters for compendious analyze_patterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousAnalyzeParams {
    /// Text to analyze for verbose patterns.
    pub text: String,
}

/// Parameters for compendious get_domain_target.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompendiousDomainTargetParams {
    /// Domain (e.g., "technical", "business", "academic", "pharmacovigilance").
    pub domain: String,
    /// Content type (e.g., "api_reference", "readme", "signal_report").
    pub content_type: String,
}

// ============================================================================
// Docs Claude params (4 tools — index has no params)
// ============================================================================

/// Parameters for docs_claude_list_pages.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeListPagesParams {
    /// Optional category filter (e.g., "hooks", "mcp", "settings").
    pub category: Option<String>,
}

/// Parameters for docs_claude_get_page.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeGetPageParams {
    /// Page path (e.g., "hooks", "mcp", "settings").
    pub page: String,
}

/// Parameters for docs_claude_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeSearchParams {
    /// Search query.
    pub query: String,
    /// Max results. Default: 5.
    pub limit: Option<usize>,
}

// ============================================================================
// Google Sheets params (7 tools)
// ============================================================================

/// Parameters for gsheets tools that only need a spreadsheet ID.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsSpreadsheetIdParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
}

/// Parameters for gsheets_read_range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsReadRangeParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range (e.g., "Sheet1!A1:C10").
    pub range: String,
}

/// Parameters for gsheets_batch_read.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsBatchReadParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// List of A1 notation ranges to read.
    pub ranges: Vec<String>,
}

/// Parameters for gsheets_write_range.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsWriteRangeParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range to write to.
    pub range: String,
    /// 2D array of values to write.
    pub values: Vec<Vec<serde_json::Value>>,
}

/// Parameters for gsheets_append.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsAppendParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// A1 notation range to append to.
    pub range: String,
    /// 2D array of values to append.
    pub values: Vec<Vec<serde_json::Value>>,
}

/// Parameters for gsheets_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsheetsSearchParams {
    /// Google Sheets spreadsheet ID.
    pub spreadsheet_id: String,
    /// Substring to search for (case-insensitive).
    pub query: String,
    /// Optional range to search within (default: first sheet).
    pub range: Option<String>,
}

// ============================================================================
// Reddit params (7 tools — status/authenticate have no params)
// ============================================================================

/// Parameters for reddit_hot_posts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditHotPostsParams {
    /// Subreddit name (without /r/ prefix).
    pub subreddit: String,
    /// Max posts to fetch (capped at 100). Default: 25.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_new_posts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditNewPostsParams {
    /// Subreddit name (without /r/ prefix).
    pub subreddit: String,
    /// Max posts to fetch (capped at 100). Default: 25.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_subreddit_info.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditSubredditInfoParams {
    /// Subreddit name (without /r/ prefix).
    pub subreddit: String,
}

/// Parameters for reddit_detect_signals.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditDetectSignalsParams {
    /// Subreddit name (without /r/ prefix).
    pub subreddit: String,
    /// Entity name to detect signals for (e.g., company/drug name).
    pub entity: String,
    /// Max posts to analyze (capped at 100). Default: 25.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

/// Parameters for reddit_search_entity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RedditSearchEntityParams {
    /// Subreddit name (without /r/ prefix).
    pub subreddit: String,
    /// Search query string.
    pub query: String,
    /// Max posts to search (capped at 100). Default: 25.
    #[serde(default = "default_reddit_limit")]
    pub limit: u32,
}

fn default_reddit_limit() -> u32 {
    25
}

// ============================================================================
// Trust params (8 tools)
// ============================================================================

/// Parameters for trust_score.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustScoreParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Use patient-safety-optimized config. Default: false.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_record.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustRecordParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Evidence type: "positive", "negative", or "neutral".
    pub evidence_type: String,
    /// Evidence weight (default: 1.0).
    pub weight: Option<f64>,
    /// Advance time by this many units after recording.
    pub time_delta: Option<f64>,
    /// Use patient-safety-optimized config. Default: false.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_snapshot.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustSnapshotParams {
    /// Unique entity identifier.
    pub entity_id: String,
}

/// Parameters for trust_decide.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustDecideParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Policy preset: "default", "strict", or "permissive".
    pub policy: Option<String>,
    /// Use patient-safety-optimized config. Default: false.
    pub safety_mode: Option<bool>,
}

/// Parameters for trust_harm_weight.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustHarmWeightParams {
    /// ICH E2A severity: non-serious, disability, hospitalization, life-threatening, death, etc.
    pub severity: String,
    /// WHO-UMC causality term (certain, probable, possible, unlikely, unassessable) or Naranjo score (-4..13).
    pub causality: String,
    /// Base evidence weight. Default: 1.0.
    pub base_weight: Option<f64>,
    /// If provided, record the harm evidence into this entity's engine.
    pub entity_id: Option<String>,
}

/// Parameters for trust_velocity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustVelocityParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// Threshold for anomaly detection. Default: 0.05.
    pub anomaly_threshold: Option<f64>,
}

/// Parameters for trust_multi_score.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustMultiScoreParams {
    /// Unique entity identifier.
    pub entity_id: String,
    /// If provided, record evidence: "positive", "negative", or "neutral".
    pub evidence_type: Option<String>,
    /// Dimension to record to: "ability", "benevolence", "integrity", or "all".
    pub dimension: Option<String>,
    /// Evidence weight (default: 1.0).
    pub weight: Option<f64>,
}

/// Parameters for trust_network_chain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TrustNetworkChainParams {
    /// Array of pairwise trust scores [A->B, B->C, C->D, ...].
    pub scores: Vec<f64>,
    /// Per-hop damping factor (0,1]. Default: 0.8.
    pub damping: Option<f64>,
}

// ============================================================================
// Molecular Weight params (4 tools — periodic_table has no params)
// ============================================================================

/// Parameters for mw_compute.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwComputeParams {
    /// Name of the concept/word (e.g., "Competency", "Signal Detection").
    pub name: Option<String>,
    /// Array of T1 primitive names or symbols (e.g., ["state", "boundary"] or ["ς", "∂"]).
    pub primitives: Vec<String>,
}

/// Parameters for mw_compare.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwCompareParams {
    /// Name of concept A.
    pub name_a: Option<String>,
    /// Primitives for concept A (names or symbols).
    pub primitives_a: Vec<String>,
    /// Name of concept B.
    pub name_b: Option<String>,
    /// Primitives for concept B (names or symbols).
    pub primitives_b: Vec<String>,
}

/// Parameters for mw_predict_transfer.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MwPredictTransferParams {
    /// Primitives to compute transfer confidence for (names or symbols).
    pub primitives: Vec<String>,
}

// ============================================================================
// Laboratory params (4 tools)
// ============================================================================

/// Parameters for lab_experiment — run a single word/concept experiment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabExperimentParams {
    /// Name of the concept/word (e.g., "Vigilance", "Guardian").
    pub name: Option<String>,
    /// Array of T1 primitive names or symbols (e.g., ["state", "boundary"] or ["ς", "∂"]).
    pub primitives: Vec<String>,
}

/// Parameters for lab_compare — compare two concepts side-by-side.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabCompareParams {
    /// Name of concept A.
    pub name_a: Option<String>,
    /// Primitives for concept A (names or symbols).
    pub primitives_a: Vec<String>,
    /// Name of concept B.
    pub name_b: Option<String>,
    /// Primitives for concept B (names or symbols).
    pub primitives_b: Vec<String>,
}

/// Parameters for lab_react — "react" two concepts by combining primitives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabReactParams {
    /// Name of concept A.
    pub name_a: Option<String>,
    /// Primitives for concept A (names or symbols).
    pub primitives_a: Vec<String>,
    /// Name of concept B.
    pub name_b: Option<String>,
    /// Primitives for concept B (names or symbols).
    pub primitives_b: Vec<String>,
}

/// Parameters for lab_batch — run experiments on multiple specimens.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabBatchParams {
    /// Array of specimens to experiment on.
    pub specimens: Vec<LabBatchSpecimen>,
}

/// A single specimen in a batch experiment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LabBatchSpecimen {
    /// Name of the concept/word.
    pub name: Option<String>,
    /// Array of T1 primitive names or symbols.
    pub primitives: Vec<String>,
}

// ============================================================================
// Integrity Assessment params (3 tools)
// ============================================================================

/// Parameters for integrity_analyze: full AI text detection with optional Bloom/domain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityAnalyzeParams {
    /// Text to analyze for AI-generation indicators.
    pub text: String,
    /// Bloom taxonomy level (1-7). Default: 3 (Apply).
    pub bloom_level: Option<u8>,
    /// PV domain ID for calibration (D02, D03, D04, D08, D10, D12).
    pub domain_id: Option<String>,
    /// Custom classification threshold (0.0-1.0). Overrides Bloom mapping.
    pub threshold: Option<f64>,
    /// Use strict threshold preset. Default: false.
    #[serde(default)]
    pub strict_mode: bool,
}

/// Parameters for integrity_assess_ksb: convenience endpoint for KSB response assessment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityAssessKsbParams {
    /// KSB response text to assess.
    pub text: String,
    /// Bloom taxonomy level for this KSB (1-7, required).
    pub bloom_level: u8,
    /// PV domain ID (optional, defaults to D08).
    pub domain_id: Option<String>,
}

/// Parameters for integrity_calibration: get domain calibration profile.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityCalibrationParams {
    /// Domain ID to retrieve calibration for (D02, D03, D04, D08, D10, D12).
    pub domain_id: String,
}

// ============================================================================
// Primitive Trace params (1 tool)
// ============================================================================

/// Parameters for primitive_trace.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PrimitiveTraceParams {
    /// Name of the concept being traced (e.g., "Guard", "Signal Detection").
    pub concept: Option<String>,
    /// Array of T1 primitive names or symbols (e.g., ["state", "boundary"] or ["ς", "∂"]).
    pub primitives: Vec<String>,
}

// ============================================================================
// Text Transform params (5 tools)
// ============================================================================

/// Parameters for transform_get_profile.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformGetProfileParams {
    /// Profile name (e.g., "pharmacovigilance", "software-architecture").
    pub name: String,
}

/// Parameters for transform_segment.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformSegmentParams {
    /// Title of the source document.
    pub title: String,
    /// Raw text to segment into paragraphs.
    pub text: String,
}

/// Parameters for transform_compile_plan.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformCompilePlanParams {
    /// Title of the source document.
    pub title: String,
    /// Raw text to transform.
    pub text: String,
    /// Source domain name (e.g., "political-philosophy").
    pub source_domain: String,
    /// Target profile name (e.g., "pharmacovigilance").
    pub target_profile: String,
}

/// Parameters for transform_score_fidelity.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TransformScoreFidelityParams {
    /// Title of the source document (to recompile plan).
    pub title: String,
    /// Raw source text (to recompile plan).
    pub text: String,
    /// Source domain name.
    pub source_domain: String,
    /// Target profile name.
    pub target_profile: String,
    /// Number of paragraphs in the transformation output.
    pub output_paragraph_count: usize,
    /// Per-paragraph concept hit counts (optional; empty array if unknown).
    #[serde(default)]
    pub concept_hits: Vec<usize>,
}

// ============================================================================
// Counter-Awareness (detection/counter-detection matrix)
// ============================================================================

/// Single-sensor detection probability against active countermeasures.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaDetectParams {
    /// Sensor name from catalog (e.g. "eo_camera", "surveillance_radar").
    pub sensor: String,
    /// Active counter-primitive names (e.g. ["absorption", "thermal_equilibrium"]).
    #[serde(default)]
    pub counters: Vec<String>,
    /// Range to target in meters.
    pub range_m: f64,
    /// Raw target signature strength [0.0, 1.0].
    pub raw_signature: f64,
}

/// Multi-sensor fused detection probability.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaFusionParams {
    /// Sensor names from catalog.
    pub sensors: Vec<String>,
    /// Active counter-primitive names.
    #[serde(default)]
    pub counters: Vec<String>,
    /// Range to target in meters.
    pub range_m: f64,
    /// Raw target signature strength [0.0, 1.0].
    pub raw_signature: f64,
    /// Detection threshold (default 0.5).
    pub threshold: Option<f64>,
}

/// Optimal countermeasure loadout selection under weight budget.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaOptimizeParams {
    /// Threat sensor names from catalog.
    pub sensors: Vec<String>,
    /// Available countermeasure names from catalog.
    pub countermeasures: Vec<String>,
    /// Maximum weight budget in kg.
    pub weight_budget_kg: f64,
    /// Engagement range in meters.
    pub range_m: f64,
    /// Raw target signature strength [0.0, 1.0].
    pub raw_signature: f64,
}

/// Query the 8×8 sensing/counter-primitive effectiveness matrix.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaMatrixParams {
    /// Optional sensing primitive name (row filter).
    pub sensing: Option<String>,
    /// Optional counter-primitive name (column filter).
    pub counter: Option<String>,
}

/// List available sensors and countermeasures from the catalog.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaCatalogParams {
    /// Filter: "sensors", "countermeasures", or omit for all.
    pub category: Option<String>,
}

// ====================================================================
// Antitransformer — AI text detection via statistical fingerprints
// T1 Grounding: σ (Sequence) + κ (Comparison) + ∂ (Boundary)
// ====================================================================

/// Analyze a single text for AI generation markers.
/// Tier: T3 (Domain-specific — AI text detection)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerAnalyzeParams {
    /// Text to analyze for AI generation
    pub text: String,
    /// Decision threshold (0.0-1.0, default: 0.5)
    #[serde(default)]
    pub threshold: Option<f64>,
    /// Entropy window size (default: 50)
    #[serde(default)]
    pub window_size: Option<usize>,
}

/// A single text item in a batch request.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerBatchItem {
    /// Optional identifier for this text
    #[serde(default)]
    pub id: Option<String>,
    /// Text to analyze
    pub text: String,
}

/// Analyze a batch of texts for AI generation markers.
/// Tier: T3 (Domain-specific — AI text detection)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AntitransformerBatchParams {
    /// Array of texts to analyze
    pub texts: Vec<AntitransformerBatchItem>,
    /// Decision threshold (0.0-1.0, default: 0.5)
    #[serde(default)]
    pub threshold: Option<f64>,
    /// Entropy window size (default: 50)
    #[serde(default)]
    pub window_size: Option<usize>,
}

// ─── Insight Engine (Pattern Detection + Novelty + Connection + Compression) ──

/// A single observation to ingest into the InsightEngine.
/// Tier: T2-P (∃ + σ — existence + sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightObservationInput {
    /// Unique key identifying this observation (e.g., "drug_a", "headache")
    pub key: String,
    /// String value (for non-numeric observations)
    #[serde(default)]
    pub value: Option<String>,
    /// Numeric value (for threshold/suddenness detection)
    #[serde(default)]
    pub numeric_value: Option<f64>,
    /// Optional tags for grouping
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Ingest observations into the InsightEngine and get all produced events.
/// Tier: T3 (INSIGHT ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂⟩)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightIngestParams {
    /// Observations to ingest (processed sequentially through the 6-stage pipeline)
    pub observations: Vec<InsightObservationInput>,
    /// Minimum co-occurrences to form a pattern (default: 2)
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
    /// Confidence threshold for pattern confirmation (default: 0.6)
    #[serde(default)]
    pub pattern_confidence_threshold: Option<f64>,
    /// Whether to enable suddenness detection (default: true)
    #[serde(default)]
    pub enable_suddenness: Option<bool>,
    /// Threshold for suddenness detection (default: 2.0)
    #[serde(default)]
    pub suddenness_threshold: Option<f64>,
    /// Whether to enable recursive learning / ρ generator (default: true).
    /// When ON: patterns feed back into recognition (Chomsky Type-0).
    /// When OFF: every observation treated as novel (Chomsky Type-1).
    #[serde(default)]
    pub enable_recursive_learning: Option<bool>,
    /// Connection strength threshold for significance (default: 0.5)
    #[serde(default)]
    pub connection_strength_threshold: Option<f64>,
    /// Minimum compression ratio to be considered meaningful (default: 2.0)
    #[serde(default)]
    pub compression_min_ratio: Option<f64>,
}

/// Get engine status after optional observation replay.
/// Tier: T2-C (ς + N)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightStatusParams {
    /// Optional observations to replay before reporting status
    #[serde(default)]
    pub observations: Option<Vec<InsightObservationInput>>,
}

/// View or update the persistent InsightEngine configuration.
/// Only provided fields are changed — omitted fields retain their saved values.
/// Tier: T2-P (∂ + ς — boundary configuration + state persistence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightConfigParams {
    /// Minimum co-occurrences to form a pattern
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
    /// Confidence threshold for pattern confirmation
    #[serde(default)]
    pub pattern_confidence_threshold: Option<f64>,
    /// Connection strength threshold for significance
    #[serde(default)]
    pub connection_strength_threshold: Option<f64>,
    /// Minimum compression ratio to be considered meaningful
    #[serde(default)]
    pub compression_min_ratio: Option<f64>,
    /// Whether to enable suddenness detection
    #[serde(default)]
    pub enable_suddenness: Option<bool>,
    /// Threshold for suddenness detection
    #[serde(default)]
    pub suddenness_threshold: Option<f64>,
    /// Whether to enable recursive learning / ρ generator
    #[serde(default)]
    pub enable_recursive_learning: Option<bool>,
}

/// Establish a connection between two observation keys.
/// Tier: T2-C (μ + κ + ς)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightConnectParams {
    /// Source key
    pub from: String,
    /// Target key
    pub to: String,
    /// Relationship type (e.g., "causes", "correlates", "inhibits")
    pub relation: String,
    /// Connection strength (0.0-1.0)
    pub strength: f64,
}

/// Compress observation keys into a unifying principle.
/// Tier: T2-C (N + μ + κ)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightCompressParams {
    /// Keys to compress
    pub keys: Vec<String>,
    /// The unifying principle (e.g., "headaches are a common adverse event")
    pub principle: String,
}

/// Get detected patterns from observations.
/// Tier: T2-C (σ + κ + μ)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightPatternsParams {
    /// Observations to process for pattern detection
    pub observations: Vec<InsightObservationInput>,
    /// Minimum co-occurrences to form a pattern (default: 2)
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
}

/// Auto-compress observations using tag-based and prefix-based clustering.
/// Tier: T2-C (N + μ + κ — automatic quantity reduction)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightCompressAutoParams {
    /// Observations to ingest before auto-compression
    pub observations: Vec<InsightObservationInput>,
    /// Minimum co-occurrences to form a pattern (default: 2)
    #[serde(default)]
    pub pattern_min_occurrences: Option<u64>,
}

/// Parameters for resetting the persistent InsightEngine state.
/// Tier: T1 (∅ — Void / clearing state)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightResetParams {
    /// Confirmation flag (must be true to reset)
    #[serde(default)]
    pub confirm: Option<bool>,
}

// ============================================================================
// NexCoreInsight System-Level Parameters (4 tools — multi-domain compositor)
// ============================================================================

/// Parameters for the system-level insight status query.
/// Shows registered domains, per-domain counts, and aggregate summary.
///
/// Tier: T2-C (ς + N + Σ — state query + quantity + aggregation)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemStatusParams {
    /// Unused — present for schema compatibility.
    #[serde(default)]
    pub _placeholder: Option<bool>,
}

/// Parameters for ingesting an observation into the system-level compositor.
/// Observations are auto-tagged with the domain name for cross-domain detection.
///
/// Tier: T3 (full pipeline through unified engine)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemIngestParams {
    /// Domain name (e.g., "guardian", "brain", "pv", "faers").
    /// Auto-registered if not already known.
    pub domain: String,

    /// Observations to ingest into the system-level engine.
    pub observations: Vec<InsightObservationInput>,
}

/// Parameters for registering a domain in the system-level compositor.
///
/// Tier: T2-P (λ + ∃ — location + existence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemRegisterParams {
    /// Domain name to register.
    pub name: String,

    /// Description of what this domain contributes.
    #[serde(default)]
    pub description: Option<String>,
}

/// Parameters for resetting the system-level compositor state.
///
/// Tier: T1 (∅ — Void)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightSystemResetParams {
    /// Confirmation flag (must be true to reset).
    #[serde(default)]
    pub confirm: Option<bool>,
}

/// Parameters for querying insight engine observations and patterns.
///
/// Tier: T2-C (κ + μ + σ — comparison-filtered query)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightQueryParams {
    /// Filter by observation key prefix (e.g., "aspirin" matches "aspirin:bleeding").
    #[serde(default)]
    pub key_prefix: Option<String>,
    /// Filter by tag (e.g., "signal", "pv", "faers").
    #[serde(default)]
    pub tag: Option<String>,
    /// Filter by domain (for system-level queries).
    #[serde(default)]
    pub domain: Option<String>,
    /// Maximum results to return (default: 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for listing detected novelties.
///
/// Tier: T2-C (∅ + ∃ + σ — void/existence in temporal order)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct InsightNoveltiesParams {
    /// Minimum novelty score threshold (0.0-1.0, default: 0.0 = all).
    #[serde(default)]
    pub min_score: Option<f64>,
    /// Maximum results to return (default: 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

// ============================================================================
// Anatomy Parameters (4 tools — workspace structural analysis)
// ============================================================================

/// Parameters for blast radius query.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyBlastRadiusParams {
    /// Crate name (e.g., "nexcore-lex-primitiva")
    pub crate_name: String,
}

/// Parameters for Chomsky classification query.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AnatomyChomskyParams {
    /// Crate name to classify (omit for all crates)
    #[serde(default)]
    pub crate_name: Option<String>,
}

// ============================================================================
// Mesh Network Tools — Runtime mesh networking (discovery, gossip, resilience)
// T1 Grounding: λ(addresses) μ(routing) σ(hop paths) ∂(TTL) ρ(relay) ν(heartbeats)
// ============================================================================

/// Parameters for mesh network simulation.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkSimulateParams {
    /// Number of nodes to create (2-20)
    pub node_count: usize,
    /// Topology: "ring", "star", "full", "line", "random"
    #[serde(default = "default_mesh_topology")]
    pub topology: String,
    /// Simulation duration in milliseconds (10-5000)
    #[serde(default = "default_mesh_duration_ms")]
    pub duration_ms: u64,
}

fn default_mesh_topology() -> String {
    "ring".to_string()
}

fn default_mesh_duration_ms() -> u64 {
    100
}

/// Parameters for mesh route quality computation.
/// Tier: T2-C (Cross-domain composite parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkRouteQualityParams {
    /// Latency in milliseconds (0.0-10000.0)
    pub latency_ms: f64,
    /// Reliability ratio (0.0-1.0)
    pub reliability: f64,
    /// Number of hops (1-255)
    pub hop_count: u8,
}

/// Parameters for mesh network node info query.
/// Tier: T2-C (Cross-domain composite parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkNodeInfoParams {
    /// Node address segments (e.g., ["mesh", "region1", "node1"])
    pub address: Vec<String>,
}

// ============================================================================
// DNA — DNA-based Computation (encode/decode, eval, tile, voxel, PV signal)
// ============================================================================

/// Parameters for encoding text as a DNA strand.
/// Tier: T2-P (σ Sequence + μ Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaEncodeParams {
    /// Text to encode as a DNA nucleotide sequence.
    pub text: String,
}

/// Parameters for decoding a DNA strand back to text.
/// Tier: T2-P (σ Sequence + μ Mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaDecodeParams {
    /// DNA strand string (A/T/G/C characters) to decode.
    pub strand: String,
}

/// Parameters for evaluating an expression on the Codon VM.
/// Tier: T2-C (σ Sequence + μ Mapping + ∂ Boundary + ς State)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaEvalParams {
    /// Source expression to compile and run on the 64-instruction Codon VM.
    pub expr: String,
}

/// Parameters for generating an 8×8 pixel tile from a source expression.
/// Tier: T2-C (σ Sequence + μ Mapping + ∂ Boundary + λ Location)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaTileParams {
    /// Source expression to compile into an 8×8 pixel tile visualization.
    pub expr: String,
}

/// Parameters for generating a 4×4×4 voxel cube from a source expression.
/// Tier: T3 (σ + μ + ∂ + N + λ + κ + → + ∃)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaVoxelParams {
    /// Source expression to compile into a 4×4×4 voxel cube.
    pub expr: String,
}

/// Parameters for detecting a PV signal between a drug and event via DNA math.
/// Tier: T2-C (→ Causality + κ Comparison + N Quantity + ν Frequency)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaPvSignalParams {
    /// Drug name to profile.
    pub drug: String,
    /// Adverse event name.
    pub event: String,
}

/// Parameters for profiling a word as a drug compound via DNA properties.
/// Tier: T3 (→ + κ + N + ν + σ + μ + ∂)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaProfileDrugParams {
    /// Drug name/word to profile.
    pub name: String,
}

/// Parameters for compiling source to assembly text.
/// Tier: T2-C (σ Sequence + μ Mapping + ∂ Boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DnaCompileAsmParams {
    /// Source expression to compile into assembly text.
    pub source: String,
}

// ============================================================================
// Perplexity AI Search Tools
// ============================================================================

/// Parameters for Perplexity AI search query.
///
/// Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary + κ Comparison)
/// Dominant: μ — query maps to search-grounded response.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexitySearchParams {
    /// Search query or question
    pub query: String,
    /// Model: "sonar" (fast, default), "sonar-pro" (advanced), "sonar-deep-research" (multi-step)
    #[serde(default)]
    pub model: Option<String>,
    /// Recency filter: "hour", "day", "week", "month"
    #[serde(default)]
    pub recency: Option<String>,
    /// Domain filter: restrict results to these domains (e.g., ["fda.gov", "ema.europa.eu"])
    #[serde(default)]
    pub domains: Option<Vec<String>>,
}

/// Parameters for high-level Perplexity research.
///
/// Tier: T2-C (μ + κ + ∂)
/// Routes to specialized research functions by use case.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityResearchParams {
    /// Research query or question
    pub query: String,
    /// Use case: "general" (web research), "competitive" (market intel), "regulatory" (FDA/EMA/ICH/WHO)
    pub use_case: String,
}

/// Parameters for Perplexity competitive intelligence.
///
/// Tier: T2-C (μ + ∂ + κ)
/// Domain-filtered search for competitor analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityCompetitiveParams {
    /// Competitive intelligence query
    pub query: String,
    /// Competitor domain names to focus search on (e.g., ["competitor1.com", "competitor2.io"])
    pub competitors: Vec<String>,
}

/// Parameters for Perplexity regulatory search.
///
/// Tier: T2-C (μ + ∂ + ν)
/// Pre-filtered to regulatory domains (FDA, EMA, ICH, WHO).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PerplexityRegulatoryParams {
    /// Regulatory query (e.g., "ICH E2E pharmacovigilance planning")
    pub query: String,
    /// Recency filter: "hour", "day", "week", "month" (default: "month")
    #[serde(default)]
    pub recency: Option<String>,
}

// ============================================================================
// Education Machine Parameters (Bayesian mastery, 5-phase FSM, spaced repetition)
// T1 Grounding: σ (sequence) + μ (mapping) + ρ (recursion) + ς (state) + N (quantity) + κ (comparison)
// ============================================================================

/// Parameters for creating a subject.
/// Tier: T2-C (σ + μ — curriculum sequence + concept mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduSubjectCreateParams {
    /// Subject name
    pub name: String,
    /// Subject description
    pub description: String,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Parameters for creating a lesson.
/// Tier: T2-C (μ + σ — content mapping + step sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLessonCreateParams {
    /// Subject this lesson belongs to
    pub subject_id: String,
    /// Lesson title
    pub title: String,
    /// Lesson description
    #[serde(default)]
    pub description: Option<String>,
    /// Difficulty level (0.0-1.0, default: 0.5)
    #[serde(default = "default_edu_difficulty")]
    pub difficulty: f64,
}

fn default_edu_difficulty() -> f64 {
    0.5
}

/// Parameters for adding a step to a lesson.
/// Tier: T2-C (σ + μ — sequence position + content type mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLessonAddStepParams {
    /// Lesson identifier
    pub lesson_id: String,
    /// Step type: "text", "exercise", or "decomposition"
    pub step_type: String,
    /// Step title
    pub title: String,
    /// Text body (for "text" type)
    #[serde(default)]
    pub body: Option<String>,
    /// Exercise prompt (for "exercise" type)
    #[serde(default)]
    pub prompt: Option<String>,
    /// Exercise solution (for "exercise" type)
    #[serde(default)]
    pub solution: Option<String>,
    /// Concept to decompose (for "decomposition" type)
    #[serde(default)]
    pub concept: Option<String>,
}

/// Parameters for creating a learner.
/// Tier: T2-P (ς — learner state initialization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduLearnerCreateParams {
    /// Unique learner identifier
    pub learner_id: String,
    /// Learner display name
    pub name: String,
}

/// Parameters for enrolling a learner in a subject.
/// Tier: T2-P (ς + μ — state change + subject mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduEnrollParams {
    /// Learner identifier
    pub learner_id: String,
    /// Subject identifier
    pub subject_id: String,
}

/// A single assessment result item.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduAssessItem {
    /// Whether the answer was correct
    pub correct: bool,
    /// Question difficulty (0.0-1.0)
    pub difficulty: f64,
}

/// Parameters for running a Bayesian assessment.
/// Tier: T2-C (κ + N + ∂ — comparison + quantity + threshold boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduAssessParams {
    /// Subject being assessed
    pub subject_id: String,
    /// Array of assessment results
    pub results: Vec<EduAssessItem>,
    /// Starting alpha (default: 1.0)
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Starting beta (default: 1.0)
    #[serde(default)]
    pub beta: Option<f64>,
}

/// Parameters for querying mastery verdict from a probability.
/// Tier: T2-P (κ + ∂ — comparison against threshold)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduMasteryParams {
    /// Mastery probability value (0.0-1.0)
    pub mastery_value: f64,
}

/// Parameters for executing a learning phase transition.
/// Tier: T2-C (σ + ∂ + ς — sequence + boundary guard + state change)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPhaseTransitionParams {
    /// Source phase: "discover", "extract", "practice", "assess", "master"
    pub from: String,
    /// Target phase
    pub to: String,
    /// Reason for transition
    #[serde(default)]
    pub reason: Option<String>,
}

/// Parameters for querying phase state information.
/// Tier: T2-P (ς — state query)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPhaseInfoParams {
    /// Current phase: "discover", "extract", "practice", "assess", "master"
    pub phase: String,
}

/// Parameters for creating a spaced repetition review item.
/// Tier: T2-P (ρ + ν — recursion + frequency initialization)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewCreateParams {
    /// Item identifier (lesson or concept ID)
    pub item_id: String,
    /// Current time (epoch seconds, default: 0.0)
    #[serde(default)]
    pub current_time: Option<f64>,
}

/// Parameters for grading and rescheduling a review.
/// Tier: T2-C (ρ + ν + N + ς — recursion + frequency + quantity + state update)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewScheduleParams {
    /// Item identifier
    pub item_id: String,
    /// Grade: "again", "hard", "good", "easy"
    pub grade: String,
    /// Current time (epoch seconds)
    pub current_time: f64,
    /// Current stability in hours (from previous state)
    #[serde(default)]
    pub stability: Option<f64>,
    /// Last review time (epoch seconds)
    #[serde(default)]
    pub last_review: Option<f64>,
    /// Current interval in hours
    #[serde(default)]
    pub interval_hours: Option<f64>,
    /// Current review count
    #[serde(default)]
    pub review_count: Option<u32>,
}

/// Parameters for checking review status.
/// Tier: T2-C (ρ + ν + N — retrievability computation)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduReviewStatusParams {
    /// Item identifier
    pub item_id: String,
    /// Current stability in hours
    pub stability: f64,
    /// Last review time (epoch seconds)
    pub last_review: f64,
    /// Current interval in hours
    pub interval_hours: f64,
    /// Current time (epoch seconds)
    pub current_time: f64,
}

/// Parameters for updating a Bayesian prior.
/// Tier: T2-P (N + κ — quantity update + comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduBayesianUpdateParams {
    /// Current alpha
    pub alpha: f64,
    /// Current beta
    pub beta: f64,
    /// Whether the answer was correct
    pub correct: bool,
    /// Question difficulty (0.0-1.0)
    pub difficulty: f64,
}

/// Parameters for mapping a concept to primitives.
/// Tier: T2-P (μ — concept-to-primitive mapping)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduPrimitiveMapParams {
    /// Domain concept name
    pub concept: String,
    /// Tier classification: "T1", "T2-P", "T2-C", "T3"
    pub tier: String,
    /// Primitive symbols involved (e.g., ["σ", "μ", "ρ"])
    pub primitives: Vec<String>,
    /// Dominant primitive symbol
    pub dominant: String,
}

// ============================================================================
// Declension System Tools — Latin-inspired architectural primitives
// T1 Grounding: ∂(boundary) ς(state) μ(mapping) ∅(void) ×(product)
// ============================================================================

/// Parameters for classifying a crate into its declension and case.
/// Tier: T3 (nexcore-declension × MCP integration)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionClassifyParams {
    /// Crate name to classify (e.g., "nexcore-vigilance").
    pub crate_name: String,
}

/// Parameters for analyzing tool family inflection.
/// Tier: T3 (nexcore-declension × MCP integration)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionInflectParams {
    /// List of tool names to analyze for family groupings.
    pub tool_names: Vec<String>,
}

/// Parameters for checking agreement between two components.
/// Tier: T3 (nexcore-declension × MCP integration)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionAgreeParams {
    /// Name of the "from" (dependent) crate.
    pub from_crate: String,
    /// Name of the "to" (dependency) crate.
    pub to_crate: String,
}

/// Parameters for analyzing pro-drop potential.
/// Tier: T3 (nexcore-declension × MCP integration)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DeclensionProdropParams {
    /// Tool name to analyze pro-drop potential for.
    pub tool_name: String,
    /// Parameter names the tool accepts.
    pub param_names: Vec<String>,
    /// Optional current working directory (for path inference).
    #[serde(default)]
    pub cwd: Option<String>,
    /// Optional last tool invoked (for context inference).
    #[serde(default)]
    pub last_tool: Option<String>,
}

// ============================================================================
// Caesura Detection Tools — structural seam detection in codebases
// T1 Grounding: ∂(boundary) ς(state) ∝(irreversibility) ν(frequency)
// ============================================================================

/// Parameters for scanning a directory for caesuras.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraScanParams {
    /// Directory path to scan for structural seams.
    pub path: String,
    /// Optional strata to scan: "style", "architecture", "dependency".
    /// If omitted, all strata are scanned.
    #[serde(default)]
    pub strata: Option<Vec<String>>,
    /// Sensitivity (sigma threshold). Lower = more sensitive. Default: 2.0.
    #[serde(default)]
    pub sensitivity: Option<f64>,
}

/// Parameters for computing caesura metrics on a single file.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraMetricsParams {
    /// Path to a single file to analyze.
    pub file_path: String,
}

/// Parameters for generating a caesura report from a directory scan.
/// Tier: T3 (Domain-specific MCP tool parameters)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CaesuraReportParams {
    /// Directory path to scan and report on.
    pub path: String,
    /// Sensitivity (sigma threshold). Default: 2.0.
    #[serde(default)]
    pub sensitivity: Option<f64>,
}

// ====================================================================
// Vigil System Tools — π(∂·ν)|∝ Vigilance Engine
// ====================================================================

/// Parameters for starting the vigilance daemon.
/// T1 Grounding: π (Persistence) + ∂ (Boundary) + ν (Frequency) + ∝ (Irreversibility)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysStartParams {
    /// Path to TOML config file (optional, uses defaults if omitted)
    #[serde(default)]
    pub config_path: Option<String>,
}

/// Parameters for adding a boundary specification.
/// T1 Grounding: ∂ (Boundary) + κ (Comparison) + ν (Frequency)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysAddBoundaryParams {
    /// Name for the boundary
    pub name: String,
    /// Threshold type: "always", "severity", "count"
    pub threshold_type: String,
    /// Source filter (optional — only events from this source)
    #[serde(default)]
    pub source_filter: Option<String>,
    /// Severity level for "severity" threshold: "info", "low", "medium", "high", "critical"
    #[serde(default)]
    pub severity: Option<String>,
    /// Count for "count" threshold
    #[serde(default)]
    pub count: Option<u64>,
    /// Window in milliseconds for "count" threshold
    #[serde(default)]
    pub window_ms: Option<u64>,
    /// Cooldown in milliseconds between violations (default: 5000)
    #[serde(default)]
    pub cooldown_ms: Option<u64>,
}

/// Parameters for querying ledger entries.
/// T1 Grounding: π (Persistence) + σ (Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VigilSysLedgerQueryParams {
    /// Filter by entry type (optional)
    #[serde(default)]
    pub entry_type: Option<String>,
    /// Only entries after this timestamp (ms since epoch)
    #[serde(default)]
    pub since: Option<u64>,
    /// Maximum entries to return
    #[serde(default)]
    pub limit: Option<u64>,
}

// ============================================================================
// Formula-Derived Parameters (KU extraction → MCP tools)
// ============================================================================

/// Parameters for Signal Strength composite (F-011: S = U × R × T)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1: N (Quantity) — pure numeric product
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalStrengthParams {
    /// Unexpectedness factor (0.0 to 1.0) — how surprising is this signal?
    pub unexpectedness: f64,
    /// Robustness factor (0.0 to 1.0) — how consistent across methods/data sources?
    pub robustness: f64,
    /// Therapeutic importance factor (0.0 to 1.0) — clinical significance of the drug/condition
    pub therapeutic_importance: f64,
}

/// Parameters for Domain Distance calculation (F-004)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1: κ (Comparison) + Σ (Sum)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DomainDistanceParams {
    /// Primitives in domain A (e.g. ["Sequence", "Mapping", "State"])
    pub primitives_a: Vec<String>,
    /// Primitives in domain B (e.g. ["Sequence", "Boundary", "Causality"])
    pub primitives_b: Vec<String>,
    /// Weight for T1 overlap (default: 0.2)
    #[serde(default = "default_w1")]
    pub w1: f64,
    /// Weight for T2 overlap (default: 0.3)
    #[serde(default = "default_w2")]
    pub w2: f64,
    /// Weight for T3 overlap (default: 0.5)
    #[serde(default = "default_w3")]
    pub w3: f64,
}

fn default_w1() -> f64 {
    0.2
}
fn default_w2() -> f64 {
    0.3
}
fn default_w3() -> f64 {
    0.5
}

/// Parameters for Flywheel Velocity calculation (F-007)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1: ν (Frequency) — inverse of average cycle time
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FlywheelVelocityParams {
    /// Timestamps when failures were logged (ms since epoch)
    pub failure_timestamps: Vec<u64>,
    /// Timestamps when fixes were deployed (ms since epoch, paired with failures)
    pub fix_timestamps: Vec<u64>,
}

/// Parameters for Token Ratio calculation (F-008)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1: N (Quantity) — ratio of counts
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TokenRatioParams {
    /// Number of LLM tokens consumed
    pub token_count: u64,
    /// Number of semantic operations produced
    pub operation_count: u64,
}

/// Parameters for Spectral Overlap (cosine similarity) (F-015)
/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to T1: κ (Comparison) + Σ (Sum)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SpectralOverlapParams {
    /// First spectrum (autocorrelation vector or feature vector)
    pub spectrum_a: Vec<f64>,
    /// Second spectrum (same dimensionality as spectrum_a)
    pub spectrum_b: Vec<f64>,
}

// ============================================================================
// Lessons Learned Parameters (6 tools — primitives_summary has no params)
// ============================================================================

/// Parameters for lesson_add.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonAddParams {
    /// Lesson title.
    pub title: String,
    /// Lesson content.
    pub content: String,
    /// Context (e.g., "hooks", "skills", "mcp").
    pub context: String,
    /// Optional tags for categorization.
    pub tags: Option<Vec<String>>,
    /// Optional source of the lesson.
    pub source: Option<String>,
}

/// Parameters for lesson_get.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonGetParams {
    /// Lesson ID to retrieve.
    pub id: u64,
}

/// Parameters for lesson_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonSearchParams {
    /// Query string to search in title and content.
    pub query: String,
}

/// Parameters for lesson_by_context.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonByContextParams {
    /// Context to filter by (e.g., "hooks", "skills", "mcp").
    pub context: String,
}

/// Parameters for lesson_by_tag.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LessonByTagParams {
    /// Tag to filter by (case-insensitive).
    pub tag: String,
}

// ============================================================================
// Claude REPL Parameters (1 tool)
// ============================================================================

/// Parameters for claude_repl.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ClaudeReplParams {
    /// The prompt to send to Claude Code CLI.
    pub prompt: String,
    /// Optional model to use (e.g., "sonnet", "opus").
    pub model: Option<String>,
    /// Optional session ID to resume.
    pub session_id: Option<String>,
    /// Optional path to settings file.
    pub settings_path: Option<String>,
    /// Optional path to MCP config file.
    pub mcp_config_path: Option<String>,
    /// Whether to strictly validate MCP config.
    pub strict_mcp_config: Option<bool>,
    /// Permission mode (e.g., "plan", "default").
    pub permission_mode: Option<String>,
    /// Optional list of allowed tools.
    pub allowed_tools: Option<Vec<String>>,
    /// Output format (default: "text").
    pub output_format: Option<String>,
    /// Optional system prompt override.
    pub system_prompt: Option<String>,
    /// Optional system prompt to append.
    pub append_system_prompt: Option<String>,
    /// Whether to persist the session.
    pub persist_session: Option<bool>,
    /// Optional timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    /// Maximum output bytes (default: 1MB).
    pub max_output_bytes: Option<usize>,
}

// ============================================================================
// Adventure HUD Parameters (6 tools — adventure_status has no params)
// ============================================================================

/// Parameters for adventure_start.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureStartParams {
    /// Name for the adventure.
    pub name: String,
}

/// Parameters for adventure_task.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureTaskParams {
    /// Task ID.
    pub id: String,
    /// Task subject/description.
    pub subject: String,
    /// Task status (pending, in_progress, completed).
    pub status: String,
}

/// Parameters for adventure_skill.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureSkillParams {
    /// Skill name to log.
    pub skill: String,
}

/// Parameters for adventure_measure.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureMeasureParams {
    /// Metric name.
    pub name: String,
    /// Metric value.
    pub value: f64,
}

/// Parameters for adventure_milestone.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct AdventureMilestoneParams {
    /// Milestone description.
    pub milestone: String,
}

// ============================================================================
// Borrow Miner Parameters (4 tools — mine/drop_ore/get_state have no params)
// ============================================================================

/// Parameters for signal_check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SignalCheckParams {
    /// Drug name to check.
    pub drug: String,
    /// Adverse event to check.
    pub event: String,
}

// ============================================================================
// Brain Database Parameters
// ============================================================================

/// Parameters for brain_db_handoffs — filter handoffs by project and limit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainDbHandoffsParams {
    /// Optional project name filter (e.g., "nexcore")
    pub project: Option<String>,
    /// Maximum number of handoffs to return (default: 20)
    pub limit: Option<u32>,
}

// ============================================================================
// Reproductive Parameters
// ============================================================================

/// Parameters for checking lethal mutation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveGuardMutationParams {
    /// List of T1 primitives present (e.g. ['Persistence', 'Boundary'])
    pub primitives: Vec<String>,
    /// Whether the change uses unsafe code
    #[serde(default)]
    pub uses_unsafe: bool,
}

/// Parameters for somatic tissue specialization
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveSpecializeAgentParams {
    /// Tissue type: 'Nervous', 'Immune', 'Muscular', 'Germ'
    pub phenotype: String,
}

/// Parameters for mitotic repair
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ReproductiveStartMitosisParams {
    /// Name of the failing crate
    pub crate_name: String,
    /// Type of failure encountered
    pub error_type: String,
    /// Failure severity (0.0 - 1.0)
    #[serde(default = "default_reproductive_severity")]
    pub severity: f64,
}

fn default_reproductive_severity() -> f64 {
    0.5
}

// ============================================================================
// Proof of Meaning (Chemistry-Inspired Semantic Equivalence)
// ============================================================================

/// Parameters for distilling a regulatory expression into atoms by volatility.
/// Tier: T2-C (σ + κ + ∂ — Sequence + Comparison + Boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomDistillParams {
    /// The regulatory expression to distill
    pub expression: String,
}

/// Parameters for classifying atoms into hierarchy positions via chromatography.
/// Tier: T2-C (σ + μ + κ — Sequence + Mapping + Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomChromatographParams {
    /// The expression to separate into hierarchy-bound atoms
    pub expression: String,
}

/// Parameters for titrating an expression against canonical standards.
/// Tier: T2-C (σ + κ + N — Sequence + Comparison + Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomTitrateParams {
    /// The expression to titrate against canonical atoms
    pub expression: String,
}

/// Parameters for proving semantic equivalence between two expressions.
/// Tier: T3 (σ + κ + ∂ + → + N — full proof chain)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomProveEquivalenceParams {
    /// First expression
    pub expression_a: String,
    /// Second expression
    pub expression_b: String,
}

/// Parameters for registry statistics (no-params tool, but typed for dispatch).
/// Tier: T2-P (Σ + N — Sum + Quantity)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PomRegistryStatsParams {}

// ── PV Axioms Database ──────────────────────────────────────────────────────

/// Parameters for KSB lookup in pv-axioms.db.
/// Tier: T2-C (κ Comparison + λ Location + σ Sequence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsKsbLookupParams {
    /// Exact KSB ID (e.g. "KSB-D08-K0001")
    pub ksb_id: Option<String>,
    /// Filter by domain (e.g. "D08")
    pub domain_id: Option<String>,
    /// Filter by type: Knowledge, Skill, Behavior, AI_Integration
    pub ksb_type: Option<String>,
    /// LIKE search on item_name and description
    pub keyword: Option<String>,
    /// Max results (default 50)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for regulation search in pv-axioms.db.
/// Tier: T2-C (∂ Boundary + ∃ Existence + λ Location)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsRegulationSearchParams {
    /// LIKE search on title and summary
    pub query: Option<String>,
    /// Filter by jurisdiction (e.g. "FDA", "EMA", "ICH")
    pub jurisdiction: Option<String>,
    /// Filter by domain (joins regulation_domains)
    pub domain_id: Option<String>,
    /// Max results (default 50)
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Parameters for traceability chain query in pv-axioms.db.
/// Tier: T2-C (→ Causality + σ Sequence + ∃ Existence)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsTraceabilityParams {
    /// Filter by axiom ID (e.g. "E2A-01")
    pub axiom_id: Option<String>,
    /// Filter by source guideline (e.g. "ICH E2A")
    pub source_guideline: Option<String>,
    /// Filter by primitive symbol (e.g. "∃")
    pub primitive: Option<String>,
}

/// Parameters for domain dashboard in pv-axioms.db.
/// Tier: T2-C (Σ Sum + N Quantity + κ Comparison)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsDomainDashboardParams {
    /// Specific domain ID (e.g. "D08"), or omit for all 15
    pub domain_id: Option<String>,
}

/// Parameters for raw SQL query against pv-axioms.db (read-only).
/// Tier: T2-C (μ Mapping + σ Sequence + ∂ Boundary)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvAxiomsQueryParams {
    /// SQL SELECT query (read-only, max 100 rows)
    pub sql: String,
}
