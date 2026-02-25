//! Type system for nexcore-measure.
//!
//! All domain values wrapped per Primitive Codex — no naked `f64` in public API.
//!
//! ## Tier Classification
//!
//! | Type | Tier | Description |
//! |------|------|-------------|
//! | `Entropy` | T2-P | Shannon entropy in bits |
//! | `Probability` | T2-P | Clamped [0,1] |
//! | `Density` | T2-P | Graph density [0,1] |
//! | `TestDensity` | T2-P | Tests per KLOC |
//! | `Centrality` | T2-P | Node centrality [0,1] |
//! | `CouplingRatio` | T2-P | Dependency balance [0,1] |
//! | `CodeDensityIndex` | T2-P | Semantic ops per token [0,1] |
//! | `CrateId` | T2-C | Crate identity |
//! | `MeasureTimestamp` | T2-C | Unix epoch seconds |
//! | `HealthScore` | T2-P | Composite [0,10] |
//! | `CrateMeasurement` | T3 | Per-crate snapshot |
//! | `WorkspaceMeasurement` | T3 | Workspace snapshot |
//! | `CrateHealth` | T3 | Composite score + rating |
//! | `WorkspaceHealth` | T3 | Aggregate health |
//! | `DriftResult` | T3 | Welch t-test result |

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// T2-P: Newtypes over f64
// ---------------------------------------------------------------------------

/// Tier: T2-P — Shannon entropy in bits (non-negative).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Entropy(f64);

impl Entropy {
    /// Create entropy value, clamping to >= 0.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Raw entropy value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Entropy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} bits", self.0)
    }
}

/// Tier: T2-P — Probability clamped to [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Probability(f64);

impl Probability {
    /// Create, clamping to [0,1].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw probability value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Probability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Tier: T2-P — Graph density [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Density(f64);

impl Density {
    /// Create, clamping to [0,1].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw density value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Density {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Tier: T2-P — Test density (tests per KLOC, non-negative).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TestDensity(f64);

impl TestDensity {
    /// Create, clamping to >= 0.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Raw value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for TestDensity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} tests/KLOC", self.0)
    }
}

/// Tier: T2-P — Node centrality [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Centrality(f64);

impl Centrality {
    /// Create, clamping to [0,1].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw centrality value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Centrality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Tier: T2-P — Coupling ratio [0, 1] (fan_out / (fan_in + fan_out)).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CouplingRatio(f64);

impl CouplingRatio {
    /// Create, clamping to [0,1].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw ratio.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for CouplingRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Tier: T2-P — Code Density Index [0, 1] (semantic ops / total tokens).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CodeDensityIndex(f64);

impl CodeDensityIndex {
    /// Create, clamping to [0,1].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw index value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for CodeDensityIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} ops/token", self.0)
    }
}

/// Tier: T2-P — Composite health score [0, 10].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HealthScore(f64);

impl HealthScore {
    /// Create, clamping to [0,10].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 10.0))
    }

    /// Raw score.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Convert to health rating.
    #[must_use]
    pub fn rating(self) -> HealthRating {
        HealthRating::from_score(self)
    }
}

impl fmt::Display for HealthScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}/10 ({})", self.0, self.rating())
    }
}

// ---------------------------------------------------------------------------
// T2-C: Composed newtypes
// ---------------------------------------------------------------------------

/// Tier: T2-C — Crate identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrateId(pub String);

impl CrateId {
    /// Create from string.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the crate name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CrateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tier: T2-C — Timestamp as Unix epoch seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MeasureTimestamp(i64);

impl MeasureTimestamp {
    /// Create from Unix epoch seconds.
    #[must_use]
    pub const fn new(epoch_secs: i64) -> Self {
        Self(epoch_secs)
    }

    /// Current time.
    #[must_use]
    pub fn now() -> Self {
        Self(nexcore_chrono::DateTime::now().timestamp())
    }

    /// Raw epoch seconds.
    #[must_use]
    pub const fn epoch_secs(self) -> i64 {
        self.0
    }
}

impl fmt::Display for MeasureTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Enums
// ---------------------------------------------------------------------------

/// Health rating categories (parallels SqiRating in nexcore-vigilance).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthRating {
    /// Score 0-2: Severe quality issues.
    Critical,
    /// Score 2-4: Significant weaknesses.
    Weak,
    /// Score 4-6: Meets minimum bar.
    Adequate,
    /// Score 6-8: Solid engineering.
    Good,
    /// Score 8-10: Exemplary quality.
    Excellent,
}

impl HealthRating {
    /// Derive rating from score.
    #[must_use]
    pub fn from_score(score: HealthScore) -> Self {
        let v = score.value();
        if v < 2.0 {
            Self::Critical
        } else if v < 4.0 {
            Self::Weak
        } else if v < 6.0 {
            Self::Adequate
        } else if v < 8.0 {
            Self::Good
        } else {
            Self::Excellent
        }
    }
}

impl fmt::Display for HealthRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::Weak => write!(f, "Weak"),
            Self::Adequate => write!(f, "Adequate"),
            Self::Good => write!(f, "Good"),
            Self::Excellent => write!(f, "Excellent"),
        }
    }
}

// ---------------------------------------------------------------------------
// T3: Domain aggregates
// ---------------------------------------------------------------------------

/// Tier: T3 — Per-crate measurement snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateMeasurement {
    /// Which crate was measured.
    pub crate_id: CrateId,
    /// When the measurement was taken.
    pub timestamp: MeasureTimestamp,
    /// Lines of code (excluding blanks/comments).
    pub loc: usize,
    /// Number of `#[test]` functions found.
    pub test_count: usize,
    /// Number of source modules (`.rs` files).
    pub module_count: usize,
    /// Lines-of-code distribution across modules.
    pub module_loc_distribution: Vec<usize>,
    /// Shannon entropy of the module distribution.
    pub entropy: Entropy,
    /// Redundancy (1 - H/H_max).
    pub redundancy: Probability,
    /// Test density (tests/KLOC).
    pub test_density: TestDensity,
    /// Fan-in (incoming dependencies).
    pub fan_in: usize,
    /// Fan-out (outgoing dependencies).
    pub fan_out: usize,
    /// Coupling ratio.
    pub coupling_ratio: CouplingRatio,
    /// Code Density Index (semantic ops / tokens).
    pub cdi: CodeDensityIndex,
}

/// Tier: T3 — Workspace-level measurement snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMeasurement {
    /// When the measurement was taken.
    pub timestamp: MeasureTimestamp,
    /// Total crates in workspace.
    pub crate_count: usize,
    /// Total lines of code.
    pub total_loc: usize,
    /// Total tests.
    pub total_tests: usize,
    /// Graph density of dependency graph.
    pub graph_density: Density,
    /// Longest dependency chain.
    pub max_depth: usize,
    /// Number of strongly connected components (cycles).
    pub scc_count: usize,
    /// Individual crate measurements.
    pub crates: Vec<CrateMeasurement>,
}

/// Tier: T3 — Composite crate health assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateHealth {
    /// Which crate.
    pub crate_id: CrateId,
    /// Overall health score.
    pub score: HealthScore,
    /// Rating derived from score.
    pub rating: HealthRating,
    /// Component scores (0.0-1.0 normalized).
    pub components: HealthComponents,
}

/// Normalized component scores for health computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthComponents {
    /// Entropy balance: optimal at H/H_max in [0.6, 0.9].
    pub entropy_norm: f64,
    /// Test density sigmoid, centered at 10 tests/KLOC.
    pub test_density_norm: f64,
    /// Coupling balance: optimal at ~0.3.
    pub coupling_norm: f64,
    /// Freshness half-life decay from last modification.
    pub freshness_norm: f64,
    /// Code Density Index normalized score.
    pub cdi_norm: f64,
}

/// Tier: T3 — Workspace aggregate health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceHealth {
    /// Timestamp.
    pub timestamp: MeasureTimestamp,
    /// Mean health across all crates.
    pub mean_score: HealthScore,
    /// Mean rating.
    pub mean_rating: HealthRating,
    /// Distribution of ratings.
    pub rating_distribution: RatingDistribution,
    /// Per-crate health.
    pub crate_healths: Vec<CrateHealth>,
}

/// Count of crates in each health tier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RatingDistribution {
    pub critical: usize,
    pub weak: usize,
    pub adequate: usize,
    pub good: usize,
    pub excellent: usize,
}

impl RatingDistribution {
    /// Increment the appropriate bucket.
    pub fn add(&mut self, rating: HealthRating) {
        match rating {
            HealthRating::Critical => {
                self.critical += 1;
            }
            HealthRating::Weak => {
                self.weak += 1;
            }
            HealthRating::Adequate => {
                self.adequate += 1;
            }
            HealthRating::Good => {
                self.good += 1;
            }
            HealthRating::Excellent => {
                self.excellent += 1;
            }
        }
    }
}

/// Tier: T3 — Welch t-test drift result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResult {
    /// Metric name that drifted.
    pub metric: String,
    /// Welch t-statistic.
    pub t_statistic: f64,
    /// Degrees of freedom.
    pub dof: f64,
    /// Two-tailed p-value.
    pub p_value: f64,
    /// Whether drift is statistically significant (p < 0.05).
    pub significant: bool,
    /// Direction of drift.
    pub direction: DriftDirection,
}

/// Direction of detected drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftDirection {
    /// Metric improved.
    Improving,
    /// Metric degraded.
    Degrading,
    /// No significant change.
    Stable,
}

impl fmt::Display for DriftDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Improving => write!(f, "↑ Improving"),
            Self::Degrading => write!(f, "↓ Degrading"),
            Self::Stable => write!(f, "→ Stable"),
        }
    }
}

/// Regression result from OLS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegressionResult {
    /// Slope of the regression line.
    pub slope: f64,
    /// Y-intercept.
    pub intercept: f64,
    /// Coefficient of determination.
    pub r_squared: f64,
    /// P-value for slope significance.
    pub p_value: f64,
}

/// Welch t-test result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WelchResult {
    /// T-statistic.
    pub t_statistic: f64,
    /// Welch-Satterthwaite degrees of freedom.
    pub dof: f64,
    /// Two-tailed p-value.
    pub p_value: f64,
}

/// Poisson confidence interval.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PoissonCi {
    /// Point estimate (count / exposure).
    pub rate: f64,
    /// Lower bound of CI.
    pub lower: f64,
    /// Upper bound of CI.
    pub upper: f64,
    /// Confidence level (e.g. 0.95).
    pub alpha: f64,
}

/// Bayesian posterior result (Gamma-Poisson conjugate).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BayesianPosterior {
    /// Posterior mean (alpha_post / beta_post).
    pub mean: f64,
    /// Posterior variance.
    pub variance: f64,
    /// Posterior alpha (shape).
    pub alpha_post: f64,
    /// Posterior beta (rate).
    pub beta_post: f64,
}

// ---------------------------------------------------------------------------
// Graph types
// ---------------------------------------------------------------------------

/// A node in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Crate identifier.
    pub crate_id: CrateId,
    /// Fan-in (other crates depending on this).
    pub fan_in: usize,
    /// Fan-out (crates this depends on).
    pub fan_out: usize,
    /// Coupling ratio.
    pub coupling_ratio: CouplingRatio,
    /// Degree centrality.
    pub degree_centrality: Centrality,
    /// Betweenness centrality.
    pub betweenness_centrality: Centrality,
}

/// Workspace dependency graph analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphAnalysis {
    /// Total nodes (crates).
    pub node_count: usize,
    /// Total edges (dependencies).
    pub edge_count: usize,
    /// Graph density.
    pub density: Density,
    /// Maximum topological depth.
    pub max_depth: usize,
    /// Number of SCCs with more than 1 node (cycles).
    pub cycle_count: usize,
    /// Strongly connected components with >1 node.
    pub cycles: Vec<Vec<CrateId>>,
    /// Per-node analysis.
    pub nodes: Vec<GraphNode>,
}

// ---------------------------------------------------------------------------
// Bridge types
// ---------------------------------------------------------------------------

/// Chemistry bridge mapping result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemistryMapping {
    /// Source metric name.
    pub source_metric: String,
    /// Target chemistry analog.
    pub chemistry_analog: String,
    /// Mapped value.
    pub mapped_value: f64,
    /// Transfer confidence.
    pub confidence: f64,
    /// Reasoning for the mapping.
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_clamps_negative() {
        let e = Entropy::new(-1.0);
        assert!((e.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn probability_clamps_range() {
        assert!((Probability::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Probability::new(-0.5).value() - 0.0).abs() < f64::EPSILON);
        assert!((Probability::new(0.7).value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn health_score_clamps_to_0_10() {
        assert!((HealthScore::new(15.0).value() - 10.0).abs() < f64::EPSILON);
        assert!((HealthScore::new(-5.0).value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn health_rating_boundaries() {
        assert_eq!(
            HealthRating::from_score(HealthScore::new(0.0)),
            HealthRating::Critical
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(1.9)),
            HealthRating::Critical
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(2.0)),
            HealthRating::Weak
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(4.0)),
            HealthRating::Adequate
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(6.0)),
            HealthRating::Good
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(8.0)),
            HealthRating::Excellent
        );
        assert_eq!(
            HealthRating::from_score(HealthScore::new(10.0)),
            HealthRating::Excellent
        );
    }

    #[test]
    fn rating_distribution_add() {
        let mut dist = RatingDistribution::default();
        dist.add(HealthRating::Good);
        dist.add(HealthRating::Good);
        dist.add(HealthRating::Excellent);
        assert_eq!(dist.good, 2);
        assert_eq!(dist.excellent, 1);
        assert_eq!(dist.critical, 0);
    }

    #[test]
    fn crate_id_display() {
        let id = CrateId::new("nexcore-vigilance");
        assert_eq!(format!("{id}"), "nexcore-vigilance");
    }

    #[test]
    fn timestamp_now_is_positive() {
        let ts = MeasureTimestamp::now();
        assert!(ts.epoch_secs() > 0);
    }

    #[test]
    fn drift_direction_display() {
        assert_eq!(format!("{}", DriftDirection::Improving), "↑ Improving");
        assert_eq!(format!("{}", DriftDirection::Degrading), "↓ Degrading");
        assert_eq!(format!("{}", DriftDirection::Stable), "→ Stable");
    }

    #[test]
    fn serde_round_trip_entropy() {
        let e = Entropy::new(3.14);
        let json = serde_json::to_string(&e).unwrap();
        let back: Entropy = serde_json::from_str(&json).unwrap();
        assert!((e.value() - back.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn serde_round_trip_health_rating() {
        let r = HealthRating::Good;
        let json = serde_json::to_string(&r).unwrap();
        let back: HealthRating = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
