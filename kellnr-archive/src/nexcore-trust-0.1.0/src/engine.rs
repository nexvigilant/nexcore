use crate::evidence::Evidence;
use crate::level::{LevelThresholds, TrustLevel};

/// Configuration for the trust engine's behavior.
///
/// Tier: T2-C (State + Boundary + Quantity)
#[derive(Debug, Clone, Copy)]
pub struct TrustConfig {
    /// Prior alpha (positive evidence baseline). Default: 1.0
    pub prior_alpha: f64,
    /// Prior beta (negative evidence baseline). Default: 1.0
    pub prior_beta: f64,
    /// Multiplier for negative evidence impact. Default: 2.5
    /// Values > 1.0 make trust harder to gain than to lose.
    pub asymmetry_factor: f64,
    /// Rate at which evidence decays toward prior. Default: 0.01
    /// Higher values = faster forgetting.
    pub decay_rate: f64,
    /// Minimum total evidence (alpha + beta) before score is meaningful.
    /// Below this threshold, the engine reports high uncertainty. Default: 5.0
    pub significance_threshold: f64,
    /// Configurable trust level thresholds. Default: 0.2/0.4/0.6/0.8
    pub thresholds: LevelThresholds,
    /// Hard floor for trust score output. Score cannot drop below this.
    /// Use to prevent "unrecoverable" trust states. Default: 0.0 (no floor).
    /// Fixes: Gap #10 (Trust Floor/Ceiling).
    pub score_floor: f64,
    /// Hard ceiling for trust score output. Score cannot exceed this.
    /// Use to prevent overconfidence in limited-observation contexts.
    /// Default: 1.0 (no ceiling).
    pub score_ceiling: f64,
    /// Diminishing returns factor for accumulated evidence.
    /// At 0.0, each evidence unit has equal impact (linear).
    /// At higher values, the Nth evidence of the same sign has less impact:
    /// `effective_weight = weight / (1 + diminishing_factor * accumulated)`.
    /// Fixes: Gap #12 (Diminishing Returns), Fallacy #7 (Sunk Cost).
    /// Default: 0.0 (disabled).
    pub diminishing_factor: f64,
    /// Cement factor: how much accumulated evidence slows temporal decay.
    /// At 0.0, decay rate is constant regardless of evidence volume.
    /// At higher values, well-established trust decays slower:
    /// `effective_rate = base_rate / (1 + cement_factor * evidence_volume)`.
    /// Fixes: Gap #7 (Adaptive Decay).
    /// Default: 0.0 (disabled).
    pub cement_factor: f64,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            prior_alpha: 1.0,
            prior_beta: 1.0,
            asymmetry_factor: 2.5,
            decay_rate: 0.01,
            significance_threshold: 5.0,
            thresholds: LevelThresholds::default(),
            score_floor: 0.0,
            score_ceiling: 1.0,
            diminishing_factor: 0.0,
            cement_factor: 0.0,
        }
    }
}

/// Bayesian trust engine using Beta distribution model.
///
/// Trust is modeled as Beta(alpha, beta) where:
/// - alpha accumulates positive evidence
/// - beta accumulates negative evidence (asymmetrically weighted)
/// - Score = E[Beta(alpha, beta)] = alpha / (alpha + beta)
/// - Temporal decay pulls alpha, beta toward priors (forgetting)
///
/// # Key Properties
///
/// - **Asymmetric**: Negative evidence has `asymmetry_factor` times the impact
/// - **Bayesian**: Naturally bounded [0, 1] with uncertainty quantification
/// - **Temporal**: Evidence decays via exponential forgetting
/// - **Composable**: Multiple engines can be merged (alpha/beta are additive)
///
/// # Algorithm
///
/// ```text
/// score = alpha / (alpha + beta)
/// uncertainty = alpha * beta / ((alpha + beta)^2 * (alpha + beta + 1))
/// decay(dt) = prior + (current - prior) * exp(-lambda * dt)
/// ```
///
/// Tier: T3 (grounds to State + Boundary + Quantity + Causality + Frequency + Irreversibility + Comparison + Sequence)
#[derive(Debug, Clone)]
pub struct TrustEngine {
    /// Positive evidence accumulator
    alpha: f64,
    /// Negative evidence accumulator
    beta: f64,
    /// Total interactions recorded
    interaction_count: u64,
    /// Time units since last non-neutral evidence (survivorship bias guard)
    time_since_last_evidence: f64,
    /// Configuration
    config: TrustConfig,
}

impl TrustEngine {
    /// Create a new trust engine with default configuration.
    /// Starts at Neutral trust (0.5) with uniform prior Beta(1, 1).
    pub fn new() -> Self {
        Self::with_config(TrustConfig::default())
    }

    /// Create a trust engine with custom configuration.
    pub fn with_config(config: TrustConfig) -> Self {
        Self {
            alpha: config.prior_alpha,
            beta: config.prior_beta,
            interaction_count: 0,
            time_since_last_evidence: 0.0,
            config,
        }
    }

    /// Create a trust engine pre-loaded with a known state.
    /// Useful for restoring from persistence.
    pub fn from_state(alpha: f64, beta: f64, interactions: u64, config: TrustConfig) -> Self {
        Self {
            alpha: alpha.max(config.prior_alpha),
            beta: beta.max(config.prior_beta),
            interaction_count: interactions,
            time_since_last_evidence: 0.0,
            config,
        }
    }

    /// Record a single piece of evidence and update trust state.
    ///
    /// - `Positive(w)` adds `w` to alpha (with diminishing returns if configured)
    /// - `Negative(w)` adds `w * asymmetry_factor` to beta (with diminishing returns)
    /// - `Neutral` increments interaction count only
    ///
    /// When `diminishing_factor > 0`, each additional unit of same-sign evidence
    /// has decreasing marginal impact: `effective = w / (1 + factor * accumulated)`.
    /// This prevents the Sunk Cost fallacy where ancient accumulated evidence
    /// dominates despite recent contrary signals.
    pub fn record(&mut self, evidence: Evidence) {
        match evidence {
            Evidence::Positive(weight) => {
                let effective = if self.config.diminishing_factor > 0.0 {
                    weight / (1.0 + self.config.diminishing_factor * self.positive_evidence())
                } else {
                    weight
                };
                self.alpha += effective;
                self.interaction_count += 1;
                self.time_since_last_evidence = 0.0;
            }
            Evidence::Negative(weight) => {
                let effective = if self.config.diminishing_factor > 0.0 {
                    weight / (1.0 + self.config.diminishing_factor * self.negative_evidence())
                } else {
                    weight
                };
                self.beta += effective * self.config.asymmetry_factor;
                self.interaction_count += 1;
                self.time_since_last_evidence = 0.0;
            }
            Evidence::Neutral => {
                self.interaction_count += 1;
            }
        }
    }

    /// Record multiple pieces of evidence in sequence.
    pub fn record_batch(&mut self, evidence: &[Evidence]) {
        for &ev in evidence {
            self.record(ev);
        }
    }

    /// Advance time by `dt` abstract time units, applying exponential decay.
    ///
    /// Both alpha and beta decay toward their prior values:
    /// `param = prior + (param - prior) * exp(-effective_rate * dt)`
    ///
    /// When `cement_factor > 0`, the effective decay rate is reduced by
    /// accumulated evidence volume: well-established trust (lots of evidence)
    /// decays slower than fragile trust (little evidence).
    /// `effective_rate = base_rate / (1 + cement_factor * evidence_volume)`
    ///
    /// This models "forgetting" — trust that isn't reinforced erodes
    /// back toward the neutral prior over time, but cemented trust
    /// is more resistant.
    pub fn advance_time(&mut self, dt: f64) {
        if dt <= 0.0 {
            return;
        }
        self.time_since_last_evidence += dt;

        let effective_rate = if self.config.cement_factor > 0.0 {
            let evidence_volume = (self.alpha - self.config.prior_alpha).max(0.0)
                + (self.beta - self.config.prior_beta).max(0.0);
            self.config.decay_rate / (1.0 + self.config.cement_factor * evidence_volume)
        } else {
            self.config.decay_rate
        };

        let factor = (-effective_rate * dt).exp();
        self.alpha = self.config.prior_alpha + (self.alpha - self.config.prior_alpha) * factor;
        self.beta = self.config.prior_beta + (self.beta - self.config.prior_beta) * factor;
    }

    /// Current trust score, clamped to [floor, ceiling].
    ///
    /// This is the expected value of Beta(alpha, beta) = alpha / (alpha + beta),
    /// then clamped to the configured score bounds.
    /// A score of 0.5 indicates maximum uncertainty (uniform prior).
    pub fn score(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            return 0.5;
        }
        let raw = self.alpha / total;
        raw.clamp(self.config.score_floor, self.config.score_ceiling)
    }

    /// Raw unclamped score (ignores floor/ceiling).
    ///
    /// Use this for statistical analysis where the true Beta distribution
    /// expected value is needed, not the policy-clamped output.
    pub fn raw_score(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            return 0.5;
        }
        self.alpha / total
    }

    /// Current trust level (discrete classification using configured thresholds).
    pub fn level(&self) -> TrustLevel {
        TrustLevel::from_score_with_thresholds(self.score(), &self.config.thresholds)
    }

    /// Uncertainty in the trust estimate.
    ///
    /// Returns the variance of Beta(alpha, beta):
    /// `Var = alpha * beta / ((alpha + beta)^2 * (alpha + beta + 1))`
    ///
    /// Range: [0, 0.25]. Lower values indicate more confident estimates.
    /// Maximum uncertainty (0.25) occurs at Beta(0, 0) — undefined.
    pub fn uncertainty(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            return 0.25;
        }
        (self.alpha * self.beta) / (total * total * (total + 1.0))
    }

    /// Whether sufficient evidence has been collected for a meaningful score.
    ///
    /// Returns true when `alpha + beta >= significance_threshold`.
    pub fn is_significant(&self) -> bool {
        (self.alpha + self.beta) >= self.config.significance_threshold
    }

    /// Total positive evidence accumulated above the prior.
    pub fn positive_evidence(&self) -> f64 {
        (self.alpha - self.config.prior_alpha).max(0.0)
    }

    /// Total negative evidence accumulated above the prior (raw, before asymmetry).
    pub fn negative_evidence(&self) -> f64 {
        (self.beta - self.config.prior_beta).max(0.0)
    }

    /// Number of interactions recorded.
    pub fn interaction_count(&self) -> u64 {
        self.interaction_count
    }

    /// Raw alpha parameter (positive accumulator).
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Raw beta parameter (negative accumulator).
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// Reference to the current configuration.
    pub fn config(&self) -> &TrustConfig {
        &self.config
    }

    /// Reset to prior state (complete trust amnesia).
    pub fn reset(&mut self) {
        self.alpha = self.config.prior_alpha;
        self.beta = self.config.prior_beta;
        self.interaction_count = 0;
        self.time_since_last_evidence = 0.0;
    }

    /// Whether the trust score is stale (no evidence for longer than threshold).
    ///
    /// Stale scores decayed toward neutral without verification. This guards
    /// against survivorship bias: avoided entities shouldn't silently rehabilitate.
    pub fn is_stale(&self, staleness_threshold: f64) -> bool {
        self.interaction_count > 0 && self.time_since_last_evidence > staleness_threshold
    }

    /// Time units since last non-neutral evidence was recorded.
    pub fn time_since_last_evidence(&self) -> f64 {
        self.time_since_last_evidence
    }

    /// Merge another engine's evidence into this one.
    ///
    /// Adds the excess evidence (above prior) from `other` into `self`.
    /// Useful for combining trust signals from multiple independent sources.
    pub fn merge(&mut self, other: &TrustEngine) {
        self.alpha += (other.alpha - other.config.prior_alpha).max(0.0);
        self.beta += (other.beta - other.config.prior_beta).max(0.0);
        self.interaction_count += other.interaction_count;
    }

    /// Create an immutable snapshot of the current trust state.
    pub fn snapshot(&self) -> TrustSnapshot {
        TrustSnapshot {
            score: self.score(),
            level: self.level(),
            uncertainty: self.uncertainty(),
            significant: self.is_significant(),
            alpha: self.alpha,
            beta: self.beta,
            interactions: self.interaction_count,
            time_since_evidence: self.time_since_last_evidence,
        }
    }
}

impl Default for TrustEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for TrustEngine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let snap = self.snapshot();
        write!(
            f,
            "Trust[{}: {:.3} +/-{:.4} | a={:.2} b={:.2} | n={}{}]",
            snap.level,
            snap.score,
            snap.uncertainty,
            snap.alpha,
            snap.beta,
            snap.interactions,
            if snap.significant {
                ""
            } else {
                " (insufficient data)"
            },
        )
    }
}

/// Immutable snapshot of trust state at a point in time.
///
/// Tier: T2-C (State + Quantity + Persistence)
#[derive(Debug, Clone, Copy)]
pub struct TrustSnapshot {
    /// Trust score [0.0, 1.0]
    pub score: f64,
    /// Discrete trust level
    pub level: TrustLevel,
    /// Variance of the Beta distribution (lower = more certain)
    pub uncertainty: f64,
    /// Whether enough evidence exists for meaningful score
    pub significant: bool,
    /// Alpha parameter
    pub alpha: f64,
    /// Beta parameter
    pub beta: f64,
    /// Total interactions
    pub interactions: u64,
    /// Time since last non-neutral evidence
    pub time_since_evidence: f64,
}

impl core::fmt::Display for TrustSnapshot {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}: {:.3} (uncertainty: {:.4}, n={})",
            self.level, self.score, self.uncertainty, self.interactions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // --- Initial State ---

    #[test]
    fn initial_score_is_neutral() {
        let engine = TrustEngine::new();
        assert!((engine.score() - 0.5).abs() < EPS);
        assert_eq!(engine.level(), TrustLevel::Neutral);
    }

    #[test]
    fn initial_interaction_count_is_zero() {
        let engine = TrustEngine::new();
        assert_eq!(engine.interaction_count(), 0);
    }

    #[test]
    fn initial_state_not_significant() {
        let engine = TrustEngine::new();
        // alpha + beta = 2.0 < 5.0 threshold
        assert!(!engine.is_significant());
    }

    // --- Positive Evidence ---

    #[test]
    fn positive_evidence_increases_score() {
        let mut engine = TrustEngine::new();
        let before = engine.score();
        engine.record(Evidence::positive());
        assert!(engine.score() > before);
    }

    #[test]
    fn many_positives_approach_one() {
        let mut engine = TrustEngine::new();
        for _ in 0..100 {
            engine.record(Evidence::positive());
        }
        assert!(engine.score() > 0.95);
    }

    // --- Negative Evidence ---

    #[test]
    fn negative_evidence_decreases_score() {
        let mut engine = TrustEngine::new();
        let before = engine.score();
        engine.record(Evidence::negative());
        assert!(engine.score() < before);
    }

    #[test]
    fn many_negatives_approach_zero() {
        let mut engine = TrustEngine::new();
        for _ in 0..100 {
            engine.record(Evidence::negative());
        }
        assert!(engine.score() < 0.05);
    }

    // --- Asymmetry (the core trust insight) ---

    #[test]
    fn asymmetry_negative_hits_harder_than_positive() {
        let mut pos_engine = TrustEngine::new();
        pos_engine.record(Evidence::positive());
        let pos_delta = (pos_engine.score() - 0.5).abs();

        let mut neg_engine = TrustEngine::new();
        neg_engine.record(Evidence::negative());
        let neg_delta = (neg_engine.score() - 0.5).abs();

        // With asymmetry_factor = 2.5, one negative should move score
        // more than one positive
        assert!(
            neg_delta > pos_delta,
            "negative delta {neg_delta:.4} should exceed positive delta {pos_delta:.4}"
        );
    }

    #[test]
    fn trust_hard_to_rebuild_after_betrayal() {
        let mut engine = TrustEngine::new();

        // Build trust with 10 positive interactions
        for _ in 0..10 {
            engine.record(Evidence::positive());
        }
        let trusted_score = engine.score();
        assert!(trusted_score > 0.7);

        // Single betrayal
        engine.record(Evidence::negative());
        let after_betrayal = engine.score();

        // Count how many positives needed to recover
        let mut recovery_count = 0u32;
        while engine.score() < trusted_score && recovery_count < 1000 {
            engine.record(Evidence::positive());
            recovery_count += 1;
        }

        // Should take more than 1 positive to recover from 1 negative
        assert!(
            recovery_count > 1,
            "recovery took {recovery_count} positives (betrayal dropped {trusted_score:.3} -> {after_betrayal:.3})"
        );
    }

    // --- Temporal Decay ---

    #[test]
    fn decay_moves_score_toward_neutral() {
        let mut engine = TrustEngine::new();

        // Build high trust
        for _ in 0..20 {
            engine.record(Evidence::positive());
        }
        assert!(engine.score() > 0.8);

        // Long time passes
        engine.advance_time(500.0);

        // Score decays toward 0.5 (prior)
        assert!(
            engine.score() < 0.6,
            "score should decay toward neutral, got {:.3}",
            engine.score()
        );
    }

    #[test]
    fn decay_from_distrust_also_returns_to_neutral() {
        let mut engine = TrustEngine::new();

        // Build distrust
        for _ in 0..20 {
            engine.record(Evidence::negative());
        }
        assert!(engine.score() < 0.2);

        // Long time passes
        engine.advance_time(500.0);

        // Score rises toward 0.5
        assert!(
            engine.score() > 0.4,
            "score should decay toward neutral, got {:.3}",
            engine.score()
        );
    }

    #[test]
    fn zero_time_advance_is_noop() {
        let mut engine = TrustEngine::new();
        engine.record(Evidence::positive());
        let before = engine.score();
        engine.advance_time(0.0);
        assert!((engine.score() - before).abs() < EPS);
    }

    #[test]
    fn negative_time_advance_is_noop() {
        let mut engine = TrustEngine::new();
        engine.record(Evidence::positive());
        let before = engine.score();
        engine.advance_time(-10.0);
        assert!((engine.score() - before).abs() < EPS);
    }

    // --- Uncertainty ---

    #[test]
    fn uncertainty_decreases_with_evidence() {
        let engine_fresh = TrustEngine::new();
        let mut engine_experienced = TrustEngine::new();
        for _ in 0..50 {
            engine_experienced.record(Evidence::positive());
        }

        assert!(
            engine_experienced.uncertainty() < engine_fresh.uncertainty(),
            "experienced uncertainty {:.4} should be less than fresh {:.4}",
            engine_experienced.uncertainty(),
            engine_fresh.uncertainty()
        );
    }

    // --- Significance ---

    #[test]
    fn becomes_significant_after_enough_evidence() {
        let mut engine = TrustEngine::new();
        // Need alpha + beta >= 5.0, start at 2.0, so need 3 more
        assert!(!engine.is_significant());
        engine.record(Evidence::positive());
        engine.record(Evidence::positive());
        engine.record(Evidence::positive());
        // alpha = 4.0, beta = 1.0, total = 5.0
        assert!(engine.is_significant());
    }

    // --- Neutral Evidence ---

    #[test]
    fn neutral_evidence_does_not_change_score() {
        let mut engine = TrustEngine::new();
        engine.record(Evidence::positive());
        let before = engine.score();
        engine.record(Evidence::Neutral);
        assert!((engine.score() - before).abs() < EPS);
        assert_eq!(engine.interaction_count(), 2);
    }

    // --- Batch Recording ---

    #[test]
    fn batch_equivalent_to_sequential() {
        let mut seq = TrustEngine::new();
        seq.record(Evidence::positive());
        seq.record(Evidence::negative());
        seq.record(Evidence::positive());

        let mut batch = TrustEngine::new();
        batch.record_batch(&[
            Evidence::positive(),
            Evidence::negative(),
            Evidence::positive(),
        ]);

        assert!((seq.score() - batch.score()).abs() < EPS);
        assert_eq!(seq.interaction_count(), batch.interaction_count());
    }

    // --- Merge ---

    #[test]
    fn merge_combines_evidence() {
        let mut source_a = TrustEngine::new();
        source_a.record(Evidence::positive());
        source_a.record(Evidence::positive());

        let mut source_b = TrustEngine::new();
        source_b.record(Evidence::negative());

        let mut combined = TrustEngine::new();
        combined.merge(&source_a);
        combined.merge(&source_b);

        // Combined should have alpha = 1 + 2 = 3, beta = 1 + 2.5 = 3.5
        assert!((combined.alpha() - 3.0).abs() < EPS);
        assert!((combined.beta() - 3.5).abs() < EPS);
    }

    // --- Reset ---

    #[test]
    fn reset_returns_to_prior() {
        let mut engine = TrustEngine::new();
        for _ in 0..20 {
            engine.record(Evidence::positive());
        }
        assert!(engine.score() > 0.8);
        engine.reset();
        assert!((engine.score() - 0.5).abs() < EPS);
        assert_eq!(engine.interaction_count(), 0);
    }

    // --- Custom Config ---

    #[test]
    fn high_asymmetry_amplifies_negative_impact() {
        let config = TrustConfig {
            asymmetry_factor: 10.0,
            ..TrustConfig::default()
        };
        let mut engine = TrustEngine::with_config(config);
        engine.record(Evidence::negative());

        // With 10x asymmetry, beta = 1.0 + 10.0 = 11.0, alpha = 1.0
        // score = 1/12 ~ 0.083
        assert!(engine.score() < 0.1);
    }

    #[test]
    fn zero_asymmetry_makes_symmetric() {
        let config = TrustConfig {
            asymmetry_factor: 1.0,
            ..TrustConfig::default()
        };
        let mut pos = TrustEngine::with_config(config);
        pos.record(Evidence::positive());

        let mut neg = TrustEngine::with_config(config);
        neg.record(Evidence::negative());

        let pos_delta = (pos.score() - 0.5).abs();
        let neg_delta = (neg.score() - 0.5).abs();
        assert!((pos_delta - neg_delta).abs() < EPS);
    }

    // --- from_state ---

    #[test]
    fn from_state_restores_correctly() {
        let engine = TrustEngine::from_state(5.0, 2.0, 6, TrustConfig::default());
        assert!((engine.alpha() - 5.0).abs() < EPS);
        assert!((engine.beta() - 2.0).abs() < EPS);
        assert_eq!(engine.interaction_count(), 6);
        assert!((engine.score() - 5.0 / 7.0).abs() < EPS);
    }

    // --- Snapshot ---

    #[test]
    fn snapshot_captures_current_state() {
        let mut engine = TrustEngine::new();
        engine.record(Evidence::positive());
        engine.record(Evidence::positive());
        engine.record(Evidence::positive());

        let snap = engine.snapshot();
        assert!((snap.score - engine.score()).abs() < EPS);
        assert_eq!(snap.level, engine.level());
        assert!((snap.uncertainty - engine.uncertainty()).abs() < EPS);
        assert_eq!(snap.significant, engine.is_significant());
        assert_eq!(snap.interactions, 3);
    }

    // --- Display ---

    #[test]
    fn display_includes_level_and_score() {
        let engine = TrustEngine::new();
        let display = format!("{engine}");
        assert!(display.contains("Neutral"));
        assert!(display.contains("0.500"));
    }

    // --- Weighted Evidence ---

    #[test]
    fn heavy_weight_has_larger_effect() {
        let mut light = TrustEngine::new();
        light.record(Evidence::positive_weighted(0.1));

        let mut heavy = TrustEngine::new();
        heavy.record(Evidence::positive_weighted(5.0));

        assert!(heavy.score() > light.score());
    }

    // --- Edge: Zero-Weight Evidence ---

    #[test]
    fn zero_weight_evidence_is_effectively_neutral() {
        let mut engine = TrustEngine::new();
        let before = engine.score();
        engine.record(Evidence::positive_weighted(0.0));
        assert!((engine.score() - before).abs() < EPS);
        // But interaction count still increments
        assert_eq!(engine.interaction_count(), 1);
    }

    // --- Score Floor/Ceiling (Gap #10) ---

    #[test]
    fn score_floor_prevents_total_distrust() {
        let config = TrustConfig {
            score_floor: 0.15,
            ..TrustConfig::default()
        };
        let mut engine = TrustEngine::with_config(config);
        for _ in 0..100 {
            engine.record(Evidence::negative());
        }
        // Raw score approaches 0, but output is clamped to floor
        assert!(
            engine.score() >= 0.15,
            "score should not drop below floor, got {:.4}",
            engine.score()
        );
        assert!(engine.raw_score() < 0.05, "raw score should be near zero");
    }

    #[test]
    fn score_ceiling_prevents_overconfidence() {
        let config = TrustConfig {
            score_ceiling: 0.90,
            ..TrustConfig::default()
        };
        let mut engine = TrustEngine::with_config(config);
        for _ in 0..100 {
            engine.record(Evidence::positive());
        }
        assert!(
            engine.score() <= 0.90,
            "score should not exceed ceiling, got {:.4}",
            engine.score()
        );
        assert!(engine.raw_score() > 0.95, "raw score should be near one");
    }

    // --- Diminishing Returns (Gap #12, Fallacy #7) ---

    #[test]
    fn diminishing_returns_caps_evidence_growth() {
        let config_linear = TrustConfig::default();
        let config_diminish = TrustConfig {
            diminishing_factor: 0.1,
            ..TrustConfig::default()
        };

        let mut linear = TrustEngine::with_config(config_linear);
        let mut diminished = TrustEngine::with_config(config_diminish);

        for _ in 0..50 {
            linear.record(Evidence::positive());
            diminished.record(Evidence::positive());
        }

        // Both should have high scores, but diminished should be lower
        assert!(
            diminished.score() < linear.score(),
            "diminished ({:.4}) should be lower than linear ({:.4})",
            diminished.score(),
            linear.score()
        );
    }

    #[test]
    fn first_evidence_unaffected_by_diminishing() {
        let config = TrustConfig {
            diminishing_factor: 0.5,
            ..TrustConfig::default()
        };
        let mut engine = TrustEngine::with_config(config);
        let before = engine.alpha();

        engine.record(Evidence::positive());

        // First evidence has accumulated = 0, so divisor = 1.0
        assert!(
            (engine.alpha() - before - 1.0).abs() < EPS,
            "first evidence should have full weight"
        );
    }

    // --- Cement Factor / Adaptive Decay (Gap #7) ---

    #[test]
    fn cemented_trust_decays_slower() {
        let config = TrustConfig {
            cement_factor: 0.1,
            ..TrustConfig::default()
        };

        let mut fresh = TrustEngine::with_config(config);
        fresh.record(Evidence::positive());
        fresh.record(Evidence::positive());
        let fresh_before = fresh.score();
        fresh.advance_time(100.0);
        let fresh_decay = fresh_before - fresh.score();

        let mut cemented = TrustEngine::with_config(config);
        for _ in 0..50 {
            cemented.record(Evidence::positive());
        }
        let cemented_before = cemented.score();
        cemented.advance_time(100.0);
        let cemented_decay = cemented_before - cemented.score();

        // Cemented trust should decay less in the same time period
        assert!(
            cemented_decay < fresh_decay,
            "cemented decay ({cemented_decay:.4}) should be less than fresh ({fresh_decay:.4})"
        );
    }

    #[test]
    fn zero_cement_matches_original_behavior() {
        let config = TrustConfig {
            cement_factor: 0.0,
            ..TrustConfig::default()
        };
        let mut a = TrustEngine::with_config(config);
        let mut b = TrustEngine::new(); // default has cement_factor = 0.0

        for _ in 0..10 {
            a.record(Evidence::positive());
            b.record(Evidence::positive());
        }
        a.advance_time(50.0);
        b.advance_time(50.0);

        assert!(
            (a.score() - b.score()).abs() < EPS,
            "zero cement should match original behavior"
        );
    }

    // --- Scenario: Real-World Trust Lifecycle ---

    #[test]
    fn trust_lifecycle_scenario() {
        let mut engine = TrustEngine::new();

        // Phase 1: New relationship — neutral
        assert_eq!(engine.level(), TrustLevel::Neutral);

        // Phase 2: Consistent good behavior builds trust
        for _ in 0..15 {
            engine.record(Evidence::positive());
        }
        assert!(engine.level() >= TrustLevel::Trusted);

        // Phase 3: A significant betrayal
        engine.record(Evidence::negative_weighted(3.0));
        let post_betrayal = engine.level();

        // Trust should have dropped meaningfully
        assert!(
            post_betrayal < TrustLevel::HighlyTrusted,
            "betrayal should reduce trust, got {post_betrayal}"
        );

        // Phase 4: Slow recovery through consistent behavior
        for _ in 0..10 {
            engine.record(Evidence::positive());
        }
        assert!(engine.level() >= TrustLevel::Trusted);

        // Phase 5: Time without interaction erodes trust
        engine.advance_time(200.0);
        let post_decay = engine.score();
        assert!(
            post_decay < 0.7,
            "trust should decay without reinforcement, got {post_decay:.3}"
        );
    }
}
