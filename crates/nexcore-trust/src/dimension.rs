/// Multi-dimensional trust engine based on Mayer, Davis & Schoorman (1995).
///
/// Decomposes trust into three orthogonal dimensions:
/// - **Ability**: Competence, skills, domain expertise
/// - **Benevolence**: Goodwill, genuine concern for the trustor
/// - **Integrity**: Adherence to principles, honesty, consistency
///
/// Each dimension has its own Beta engine with dimension-specific asymmetry
/// factors based on Kim et al. (2004, 2006) trust repair research:
/// - Ability violations recover faster (asymmetry = 1.5)
/// - Benevolence violations recover moderately (asymmetry = 2.5)
/// - Integrity violations are hardest to recover from (asymmetry = 4.0)
///
/// Fixes: Fallacy #1 (False Dichotomy), Fallacy #14 (Conjunction Fallacy),
/// Gap #1 (Multi-dimensional trust), Gap #4 (Violation-specific recovery).
///
/// Tier: T3 (grounds to State + Boundary + Quantity + Causality + Mapping)
use crate::engine::{TrustConfig, TrustEngine, TrustSnapshot};
use crate::evidence::Evidence;
use crate::level::TrustLevel;

/// The three orthogonal dimensions of trust (Mayer et al. 1995).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustDimension {
    /// Can they do it? Skills, competence, domain expertise.
    /// Violations: honest mistakes, lack of skill, poor judgment.
    /// Recovery: demonstrate improved capability (faster recovery).
    Ability,
    /// Do they care about my interests? Goodwill, attachment, loyalty.
    /// Violations: neglect, self-serving behavior, indifference.
    /// Recovery: demonstrate care and investment (moderate recovery).
    Benevolence,
    /// Are they honest and principled? Consistency, fairness, reliability.
    /// Violations: lies, broken promises, hidden agendas.
    /// Recovery: sustained principled behavior over time (slowest recovery).
    Integrity,
}

impl TrustDimension {
    /// All three dimensions.
    pub const ALL: [TrustDimension; 3] = [
        TrustDimension::Ability,
        TrustDimension::Benevolence,
        TrustDimension::Integrity,
    ];

    /// Default asymmetry factor for this dimension (Kim et al. 2004, 2006).
    pub fn default_asymmetry(self) -> f64 {
        match self {
            Self::Ability => 1.5,
            Self::Benevolence => 2.5,
            Self::Integrity => 4.0,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ability => "Ability",
            Self::Benevolence => "Benevolence",
            Self::Integrity => "Integrity",
        }
    }
}

impl core::fmt::Display for TrustDimension {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Weights for combining dimension scores into a composite.
#[derive(Debug, Clone, Copy)]
pub struct DimensionWeights {
    /// Weight for Ability dimension. Default: 0.35
    pub ability: f64,
    /// Weight for Benevolence dimension. Default: 0.25
    pub benevolence: f64,
    /// Weight for Integrity dimension. Default: 0.40
    pub integrity: f64,
}

impl DimensionWeights {
    /// Get weight for a specific dimension.
    pub fn get(self, dim: TrustDimension) -> f64 {
        match dim {
            TrustDimension::Ability => self.ability,
            TrustDimension::Benevolence => self.benevolence,
            TrustDimension::Integrity => self.integrity,
        }
    }

    /// Normalize weights so they sum to 1.0.
    pub fn normalized(self) -> Self {
        let total = self.ability + self.benevolence + self.integrity;
        if total <= 0.0 {
            return Self::default();
        }
        Self {
            ability: self.ability / total,
            benevolence: self.benevolence / total,
            integrity: self.integrity / total,
        }
    }
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            ability: 0.35,
            benevolence: 0.25,
            integrity: 0.40,
        }
    }
}

/// Multi-dimensional trust engine composing three independent Beta engines.
///
/// Each dimension tracks trust independently with dimension-specific
/// asymmetry factors. Composite scores are computed via weighted average
/// or minimum (weakest-link).
#[derive(Debug, Clone)]
pub struct MultiTrustEngine {
    ability: TrustEngine,
    benevolence: TrustEngine,
    integrity: TrustEngine,
    weights: DimensionWeights,
}

impl MultiTrustEngine {
    /// Create a new multi-dimensional trust engine with default settings.
    ///
    /// Uses Kim et al. asymmetry factors:
    /// - Ability: 1.5 (mistakes are more forgivable)
    /// - Benevolence: 2.5 (neglect is concerning)
    /// - Integrity: 4.0 (dishonesty is hardest to forgive)
    pub fn new() -> Self {
        Self::with_weights(DimensionWeights::default())
    }

    /// Create with custom dimension weights.
    pub fn with_weights(weights: DimensionWeights) -> Self {
        let weights = weights.normalized();
        Self {
            ability: TrustEngine::with_config(TrustConfig {
                asymmetry_factor: TrustDimension::Ability.default_asymmetry(),
                ..TrustConfig::default()
            }),
            benevolence: TrustEngine::with_config(TrustConfig {
                asymmetry_factor: TrustDimension::Benevolence.default_asymmetry(),
                ..TrustConfig::default()
            }),
            integrity: TrustEngine::with_config(TrustConfig {
                asymmetry_factor: TrustDimension::Integrity.default_asymmetry(),
                ..TrustConfig::default()
            }),
            weights,
        }
    }

    /// Create with fully custom per-dimension configs.
    pub fn with_configs(
        ability_config: TrustConfig,
        benevolence_config: TrustConfig,
        integrity_config: TrustConfig,
        weights: DimensionWeights,
    ) -> Self {
        Self {
            ability: TrustEngine::with_config(ability_config),
            benevolence: TrustEngine::with_config(benevolence_config),
            integrity: TrustEngine::with_config(integrity_config),
            weights: weights.normalized(),
        }
    }

    /// Record evidence for a specific trust dimension.
    pub fn record(&mut self, dimension: TrustDimension, evidence: Evidence) {
        self.engine_mut(dimension).record(evidence);
    }

    /// Record evidence that affects all dimensions equally.
    ///
    /// Use for holistic trust signals that aren't dimension-specific.
    pub fn record_all(&mut self, evidence: Evidence) {
        self.ability.record(evidence);
        self.benevolence.record(evidence);
        self.integrity.record(evidence);
    }

    /// Advance time for all dimensions.
    pub fn advance_time(&mut self, dt: f64) {
        self.ability.advance_time(dt);
        self.benevolence.advance_time(dt);
        self.integrity.advance_time(dt);
    }

    /// Weighted composite trust score.
    ///
    /// Returns the weighted average of dimension scores.
    pub fn composite_score(&self) -> f64 {
        self.weights.ability * self.ability.score()
            + self.weights.benevolence * self.benevolence.score()
            + self.weights.integrity * self.integrity.score()
    }

    /// Composite trust level from the weighted score.
    pub fn composite_level(&self) -> TrustLevel {
        TrustLevel::from_score(self.composite_score())
    }

    /// Minimum dimension score (weakest-link trust).
    ///
    /// You are only as trusted as your weakest dimension.
    /// Addresses the Conjunction Fallacy: "Highly Trusted" requires
    /// ALL dimensions to be strong, not just the average.
    pub fn minimum_score(&self) -> f64 {
        self.ability
            .score()
            .min(self.benevolence.score())
            .min(self.integrity.score())
    }

    /// Trust level based on weakest dimension.
    pub fn minimum_level(&self) -> TrustLevel {
        TrustLevel::from_score(self.minimum_score())
    }

    /// Identify the weakest trust dimension.
    pub fn weakest_dimension(&self) -> (TrustDimension, f64) {
        let scores = [
            (TrustDimension::Ability, self.ability.score()),
            (TrustDimension::Benevolence, self.benevolence.score()),
            (TrustDimension::Integrity, self.integrity.score()),
        ];
        let mut weakest = scores[0];
        for &s in &scores[1..] {
            if s.1 < weakest.1 {
                weakest = s;
            }
        }
        weakest
    }

    /// Score for a specific dimension.
    pub fn dimension_score(&self, dimension: TrustDimension) -> f64 {
        self.engine(dimension).score()
    }

    /// Get a reference to a dimension's engine.
    pub fn engine(&self, dimension: TrustDimension) -> &TrustEngine {
        match dimension {
            TrustDimension::Ability => &self.ability,
            TrustDimension::Benevolence => &self.benevolence,
            TrustDimension::Integrity => &self.integrity,
        }
    }

    /// Get a mutable reference to a dimension's engine.
    fn engine_mut(&mut self, dimension: TrustDimension) -> &mut TrustEngine {
        match dimension {
            TrustDimension::Ability => &mut self.ability,
            TrustDimension::Benevolence => &mut self.benevolence,
            TrustDimension::Integrity => &mut self.integrity,
        }
    }

    /// Snapshot of all three dimensions plus composite scores.
    pub fn snapshot(&self) -> MultiTrustSnapshot {
        MultiTrustSnapshot {
            ability: self.ability.snapshot(),
            benevolence: self.benevolence.snapshot(),
            integrity: self.integrity.snapshot(),
            composite_score: self.composite_score(),
            composite_level: self.composite_level(),
            minimum_score: self.minimum_score(),
            minimum_level: self.minimum_level(),
            weakest: self.weakest_dimension(),
        }
    }

    /// Reset all dimensions to prior state.
    pub fn reset(&mut self) {
        self.ability.reset();
        self.benevolence.reset();
        self.integrity.reset();
    }
}

impl Default for MultiTrustEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for MultiTrustEngine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MultiTrust[A={:.3} B={:.3} I={:.3} | composite={:.3} ({}) | weakest={}:{:.3}]",
            self.ability.score(),
            self.benevolence.score(),
            self.integrity.score(),
            self.composite_score(),
            self.composite_level(),
            self.weakest_dimension().0,
            self.weakest_dimension().1,
        )
    }
}

/// Immutable snapshot of multi-dimensional trust state.
#[derive(Debug, Clone)]
pub struct MultiTrustSnapshot {
    /// Ability dimension snapshot
    pub ability: TrustSnapshot,
    /// Benevolence dimension snapshot
    pub benevolence: TrustSnapshot,
    /// Integrity dimension snapshot
    pub integrity: TrustSnapshot,
    /// Weighted composite score
    pub composite_score: f64,
    /// Composite trust level
    pub composite_level: TrustLevel,
    /// Minimum dimension score (weakest-link)
    pub minimum_score: f64,
    /// Trust level based on weakest dimension
    pub minimum_level: TrustLevel,
    /// Weakest dimension and its score
    pub weakest: (TrustDimension, f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-6;

    #[test]
    fn initial_state_is_neutral_all_dimensions() {
        let engine = MultiTrustEngine::new();
        for dim in TrustDimension::ALL {
            assert!(
                (engine.dimension_score(dim) - 0.5).abs() < EPS,
                "{dim} should start neutral"
            );
        }
        assert!((engine.composite_score() - 0.5).abs() < EPS);
    }

    #[test]
    fn dimension_evidence_is_independent() {
        let mut engine = MultiTrustEngine::new();

        // Only record positive ability evidence
        for _ in 0..10 {
            engine.record(TrustDimension::Ability, Evidence::positive());
        }

        assert!(engine.dimension_score(TrustDimension::Ability) > 0.8);
        assert!((engine.dimension_score(TrustDimension::Benevolence) - 0.5).abs() < EPS);
        assert!((engine.dimension_score(TrustDimension::Integrity) - 0.5).abs() < EPS);
    }

    #[test]
    fn integrity_violations_hit_hardest() {
        let mut ability_engine = MultiTrustEngine::new();
        ability_engine.record(TrustDimension::Ability, Evidence::negative());
        let ability_drop = 0.5 - ability_engine.dimension_score(TrustDimension::Ability);

        let mut integrity_engine = MultiTrustEngine::new();
        integrity_engine.record(TrustDimension::Integrity, Evidence::negative());
        let integrity_drop = 0.5 - integrity_engine.dimension_score(TrustDimension::Integrity);

        assert!(
            integrity_drop > ability_drop,
            "integrity violation drop ({integrity_drop:.4}) should exceed ability ({ability_drop:.4})"
        );
    }

    #[test]
    fn minimum_score_reflects_weakest_link() {
        let mut engine = MultiTrustEngine::new();

        // Strong ability and benevolence
        for _ in 0..20 {
            engine.record(TrustDimension::Ability, Evidence::positive());
            engine.record(TrustDimension::Benevolence, Evidence::positive());
        }
        // Weak integrity
        for _ in 0..10 {
            engine.record(TrustDimension::Integrity, Evidence::negative());
        }

        // Composite might look OK (weighted average)
        // But minimum reveals the integrity problem
        assert!(engine.minimum_score() < 0.2);
        assert_eq!(engine.weakest_dimension().0, TrustDimension::Integrity);
    }

    #[test]
    fn record_all_affects_all_dimensions() {
        let mut engine = MultiTrustEngine::new();
        engine.record_all(Evidence::positive());

        for dim in TrustDimension::ALL {
            assert!(
                engine.dimension_score(dim) > 0.5,
                "{dim} should increase with record_all"
            );
        }
    }

    #[test]
    fn advance_time_decays_all() {
        let mut engine = MultiTrustEngine::new();

        for dim in TrustDimension::ALL {
            for _ in 0..20 {
                engine.record(dim, Evidence::positive());
            }
        }

        engine.advance_time(500.0);

        for dim in TrustDimension::ALL {
            assert!(
                engine.dimension_score(dim) < 0.6,
                "{dim} should decay toward neutral"
            );
        }
    }

    #[test]
    fn competent_liar_scenario() {
        let mut engine = MultiTrustEngine::new();

        // Highly competent
        for _ in 0..20 {
            engine.record(TrustDimension::Ability, Evidence::positive());
        }
        // But dishonest
        for _ in 0..10 {
            engine.record(TrustDimension::Integrity, Evidence::negative());
        }

        // Composite might mask the problem...
        let composite = engine.composite_level();
        // ...but minimum reveals truth
        let minimum = engine.minimum_level();

        assert!(
            minimum < composite,
            "weakest-link ({minimum}) should be lower than composite ({composite})"
        );
        assert!(
            engine.minimum_score() < 0.2,
            "liar should be untrusted at minimum"
        );
    }

    #[test]
    fn snapshot_captures_all_dimensions() {
        let mut engine = MultiTrustEngine::new();
        engine.record(TrustDimension::Ability, Evidence::positive());
        engine.record(TrustDimension::Integrity, Evidence::negative());

        let snap = engine.snapshot();
        assert!(snap.ability.score > 0.5);
        assert!(snap.integrity.score < 0.5);
        assert!((snap.benevolence.score - 0.5).abs() < EPS);
    }

    #[test]
    fn display_includes_all_dimensions() {
        let engine = MultiTrustEngine::new();
        let display = format!("{engine}");
        assert!(display.contains("MultiTrust"));
        assert!(display.contains("composite"));
    }

    #[test]
    fn reset_returns_all_to_neutral() {
        let mut engine = MultiTrustEngine::new();
        engine.record_all(Evidence::positive());
        engine.record_all(Evidence::positive());
        engine.reset();
        assert!((engine.composite_score() - 0.5).abs() < EPS);
    }

    #[test]
    fn weights_are_normalized() {
        let weights = DimensionWeights {
            ability: 10.0,
            benevolence: 10.0,
            integrity: 10.0,
        };
        let norm = weights.normalized();
        let sum = norm.ability + norm.benevolence + norm.integrity;
        assert!((sum - 1.0).abs() < EPS);
    }
}
