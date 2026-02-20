//! STEM (Science, Technology, Engineering, Math) Principles
//! Tier: T1-T3 (Foundation to Domain-specific)
//!
//! Basic physics, math, and cross-domain conservation laws.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for combining two confidence values
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemConfidenceCombineParams {
    /// First confidence value [0.0, 1.0]
    pub a: f64,
    /// Second confidence value [0.0, 1.0]
    pub b: f64,
}

/// Parameters for tier info lookup
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemTierInfoParams {
    /// Tier name: T1, T2-P, T2-C, or T3
    pub tier: String,
}

/// Parameters for chemistry equilibrium balance
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
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemFractionParams {
    /// Fraction value [0.0, 1.0]
    pub value: f64,
}

/// Parameters for F=ma calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysFmaParams {
    /// Force value (Newtons)
    pub force: f64,
    /// Mass value (kg)
    pub mass: f64,
}

/// Parameters for quantity conservation check
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
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysPeriodParams {
    /// Frequency in Hz
    pub frequency: f64,
}

/// Parameters for bounds checking
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathBoundsCheckParams {
    /// Value to check
    pub value: f64,
    /// Lower bound
    #[serde(default)]
    pub lower: Option<f64>,
    /// Upper bound
    #[serde(default)]
    pub upper: Option<f64>,
}

/// Parameters for relation inversion
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathRelationInvertParams {
    /// Relation: LessThan, Equal, GreaterThan, or Incomparable
    pub relation: String,
}

// ============================================================================
// New Chemistry Tools
// ============================================================================

/// Parameters for concentration ratio
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemRatioParams {
    /// Concentration ratio value (substance/volume, ≥ 0.0)
    pub value: f64,
    /// Optional second ratio for comparison
    #[serde(default)]
    pub compare_to: Option<f64>,
}

/// Parameters for rate of change
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemRateParams {
    /// Rate value (≥ 0.0)
    pub value: f64,
    /// Optional second rate for ratio comparison
    #[serde(default)]
    pub compare_to: Option<f64>,
}

/// Parameters for binding affinity
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemChemAffinityParams {
    /// Affinity value [0.0 = none, 1.0 = perfect]
    pub value: f64,
    /// Threshold for "strong binding" classification (default: 0.7)
    #[serde(default)]
    pub strong_threshold: Option<f64>,
}

// ============================================================================
// New Physics Tools
// ============================================================================

/// Parameters for amplitude operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysAmplitudeParams {
    /// First amplitude value (≥ 0.0)
    pub value: f64,
    /// Optional second amplitude for superposition
    #[serde(default)]
    pub superpose_with: Option<f64>,
}

/// Parameters for scale factor operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysScaleParams {
    /// Scale factor value
    pub factor: f64,
    /// Value to apply the scale factor to
    pub apply_to: f64,
}

/// Parameters for inertia/resistance calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemPhysInertiaParams {
    /// Mass value (kg, > 0)
    pub mass: f64,
    /// Proposed change magnitude
    pub proposed_change: f64,
}

// ============================================================================
// New Mathematics Tools
// ============================================================================

/// Parameters for proof construction
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathProofParams {
    /// List of premise statements
    pub premises: Vec<String>,
    /// Conclusion statement
    pub conclusion: String,
    /// Whether the proof is valid (true) or a counterexample (false)
    #[serde(default = "default_true")]
    pub valid: bool,
}

fn default_true() -> bool {
    true
}

/// Parameters for identity element check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemMathIdentityParams {
    /// The candidate identity value
    pub value: f64,
    /// The operation: "add" (identity=0) or "multiply" (identity=1)
    pub operation: String,
    /// Value to test against
    pub test_value: f64,
}

// ============================================================================
// Spatial Math Tools
// ============================================================================

/// Parameters for spatial distance operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemSpatialDistanceParams {
    /// Distance value (≥ 0.0)
    pub value: f64,
    /// Optional second distance for comparison (approximate equality)
    #[serde(default)]
    pub compare_to: Option<f64>,
    /// Tolerance for approximate equality (default: 0.001)
    #[serde(default)]
    pub tolerance: Option<f64>,
}

/// Parameters for triangle inequality check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemSpatialTriangleParams {
    /// Distance from A to B
    pub ab: f64,
    /// Distance from B to C
    pub bc: f64,
    /// Distance from A to C
    pub ac: f64,
}

/// Parameters for neighborhood containment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemSpatialNeighborhoodParams {
    /// Neighborhood radius
    pub radius: f64,
    /// Whether the boundary is open (true) or closed (false, default)
    #[serde(default)]
    pub open: bool,
    /// Distance to test for containment
    pub test_distance: f64,
}

/// Parameters for dimension operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemSpatialDimensionParams {
    /// Number of independent axes
    pub rank: u32,
    /// Optional second dimension for subspace check
    #[serde(default)]
    pub subspace_of: Option<u32>,
}

/// Parameters for orientation operations
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemSpatialOrientationParams {
    /// Orientation: "positive", "negative", or "unoriented"
    pub orientation: String,
    /// Optional second orientation for composition
    #[serde(default)]
    pub compose_with: Option<String>,
}

// ============================================================================
// Core: Transfer Confidence, Integrity, Retry, Determinism
// ============================================================================

/// Parameters for cross-domain transfer confidence scoring
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemTransferConfidenceParams {
    /// Structural similarity [0.0, 1.0] — how well the structure maps
    pub structural: f64,
    /// Functional similarity [0.0, 1.0] — how well the function maps
    pub functional: f64,
    /// Contextual similarity [0.0, 1.0] — how well the context maps
    pub contextual: f64,
}

/// Parameters for integrity gate validation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemIntegrityCheckParams {
    /// Value to validate
    pub value: f64,
    /// Minimum acceptable value
    pub min: f64,
    /// Maximum acceptable value
    pub max: f64,
    /// Label for the validated quantity
    #[serde(default)]
    pub label: Option<String>,
}

/// Parameters for retry budget calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemRetryBudgetParams {
    /// Maximum number of retry attempts allowed
    pub max_attempts: u32,
    /// Current attempt number (1-based)
    pub current_attempt: u32,
}

/// Parameters for determinism score classification
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemDeterminismScoreParams {
    /// Repeatability score [0.0 = stochastic, 1.0 = deterministic]
    pub score: f64,
}

// ============================================================================
// Bio: Behavior and Tone Profiles
// ============================================================================

/// Parameters for behavior profile (no required inputs — reads endocrine state)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemBioBehaviorProfileParams {
    /// Optional stimulus to apply before reading profile: "stress", "reward", "social", "temporal", "urgency"
    #[serde(default)]
    pub stimulus: Option<String>,
    /// Stimulus intensity [0.0, 1.0] (default: 0.5)
    #[serde(default)]
    pub intensity: Option<f64>,
}

/// Parameters for tone profile generation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemBioToneProfileParams {
    /// Optional stimulus to apply before reading profile
    #[serde(default)]
    pub stimulus: Option<String>,
    /// Stimulus intensity [0.0, 1.0] (default: 0.5)
    #[serde(default)]
    pub intensity: Option<f64>,
}

// ============================================================================
// Finance Tools
// ============================================================================

/// Parameters for present value / discount calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceDiscountParams {
    /// Future value amount
    pub future_value: f64,
    /// Discount rate (decimal, e.g., 0.05 = 5%)
    pub rate: f64,
    /// Number of periods
    pub periods: f64,
}

/// Parameters for compound growth calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceCompoundParams {
    /// Principal amount
    pub principal: f64,
    /// Growth rate per period (decimal)
    pub rate: f64,
    /// Number of compounding periods
    pub periods: u32,
    /// If true, use continuous compounding (default: false)
    #[serde(default)]
    pub continuous: bool,
}

/// Parameters for spread calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceSpreadParams {
    /// Bid price (lower)
    pub bid: f64,
    /// Ask price (higher)
    pub ask: f64,
}

/// Parameters for maturity check
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceMaturityParams {
    /// Years to maturity
    pub years: f64,
}

/// Parameters for exposure calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceExposureParams {
    /// Position value (positive = long, negative = short)
    pub value: f64,
}

/// Parameters for arbitrage detection
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceArbitrageParams {
    /// Price from source A
    pub price_a: f64,
    /// Price from source B
    pub price_b: f64,
    /// Transaction cost
    #[serde(default)]
    pub cost: Option<f64>,
}

/// Parameters for diversification calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceDiversifyParams {
    /// List of position values
    pub positions: Vec<f64>,
}

/// Parameters for return calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemFinanceReturnParams {
    /// Initial price/value (P₀)
    pub p0: f64,
    /// Final price/value (P₁)
    pub p1: f64,
    /// Method: "simple" (default) or "log"
    #[serde(default)]
    pub method: Option<String>,
}

// ============================================================================
// Statistics Tools (stem-math statistical inference)
// ============================================================================

/// Parameters for z-test (one-sample)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemStatsZTestParams {
    /// Observed value (sample mean or statistic)
    pub observed: f64,
    /// Hypothesized null value (μ₀)
    pub null_value: f64,
    /// Standard error of the statistic
    pub std_error: f64,
    /// Tail direction: "two" (default), "left", or "right"
    #[serde(default)]
    pub tail: Option<String>,
    /// Confidence level (default: 0.95)
    #[serde(default)]
    pub confidence_level: Option<f64>,
}

/// Parameters for confidence interval construction
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemStatsCiParams {
    /// Point estimate (sample mean)
    pub estimate: f64,
    /// Standard error of the estimate
    pub std_error: f64,
    /// Confidence level (default: 0.95)
    #[serde(default)]
    pub confidence_level: Option<f64>,
    /// CI type: "mean" (default), "proportion", "diff"
    #[serde(default)]
    pub ci_type: Option<String>,
    /// Sample size n (required for proportion CI)
    #[serde(default)]
    pub n: Option<usize>,
    /// Second mean (for difference CI)
    #[serde(default)]
    pub mean2: Option<f64>,
    /// Second standard error (for difference CI)
    #[serde(default)]
    pub se2: Option<f64>,
}

/// Parameters for p-value calculation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemStatsPValueParams {
    /// Z-score (or provide observed + null_value + std_error to compute it)
    #[serde(default)]
    pub z_score: Option<f64>,
    /// Observed value (alternative to z_score)
    #[serde(default)]
    pub observed: Option<f64>,
    /// Null hypothesis value (used with observed)
    #[serde(default)]
    pub null_value: Option<f64>,
    /// Standard error (used with observed)
    #[serde(default)]
    pub std_error: Option<f64>,
    /// Tail direction: "two" (default), "left", or "right"
    #[serde(default)]
    pub tail: Option<String>,
}

/// Parameters for full sample analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct StemStatsAnalyzeParams {
    /// Sample data values
    pub values: Vec<f64>,
    /// Null hypothesis value to test against (default: 0.0)
    #[serde(default)]
    pub null_value: Option<f64>,
    /// Confidence level (default: 0.95)
    #[serde(default)]
    pub confidence_level: Option<f64>,
}
