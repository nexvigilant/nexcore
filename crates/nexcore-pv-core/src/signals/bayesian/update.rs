//! # BayesianUpdate Trait + Conjugate Implementations
//!
//! Universal Bayesian update pattern extracted from the 10+ independent Bayesian
//! implementations across the nexcore workspace.
//!
//! ## Implementations
//!
//! - [`ConjugateBetaBinomial`] — Beta-Binomial conjugate pair (BCPNN foundation)
//! - [`GammaPoissonMixture`] — Gamma-Poisson conjugate pair (EBGM foundation)
//!
//! These exist ALONGSIDE existing BCPNN/EBGM code. They prove the trait works.
//! Future refactoring to use the trait is a separate directive.
//!
//! ## The Sequential Update — The Lifecycle Primitive
//!
//! `sequential_update` enables:
//! - QBR updating across PSUR cycles (posterior from Q1 → prior for Q2)
//! - Signal strength accumulation over FAERS quarters
//! - Evidence accumulation: preclinical → clinical → post-market
//!
//! ## Grounding
//!
//! GroundsTo: →(Causality) + ρ(Recursion) + ∂(Boundary) + N(Quantity)
//! - → dominates: evidence → belief update (causal chain)
//! - ρ: sequential update is recursive application
//! - ∂: credible intervals bound the posterior
//! - N: numerical estimates throughout

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// BAYESIAN UPDATE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Universal Bayesian update pattern.
///
/// Implementations define their own prior, likelihood, and posterior types.
/// The trait enforces the Bayesian cycle: prior + evidence → posterior.
pub trait BayesianUpdate {
    /// Prior distribution parameters
    type Prior: Clone;
    /// Observed evidence
    type Evidence;
    /// Posterior distribution parameters (often same type as Prior for conjugates)
    type Posterior;

    /// Construct the default (uninformative) prior.
    fn default_prior() -> Self::Prior;

    /// Update prior with evidence to produce posterior.
    fn update(prior: &Self::Prior, evidence: &Self::Evidence) -> Self::Posterior;

    /// Extract a point estimate with confidence from the posterior.
    fn summarize(posterior: &Self::Posterior) -> Measured<f64>;

    /// Sequential update: fold evidence sequence through the update cycle.
    ///
    /// This is the lifecycle primitive — posterior becomes prior for next evidence.
    fn sequential_update(
        prior: &Self::Prior,
        evidence_sequence: &[Self::Evidence],
    ) -> Option<Self::Posterior>
    where
        Self::Posterior: Into<Self::Prior>,
    {
        if evidence_sequence.is_empty() {
            return None;
        }

        let mut current = Self::update(prior, &evidence_sequence[0]);

        for ev in &evidence_sequence[1..] {
            let next_prior: Self::Prior = current.into();
            current = Self::update(&next_prior, ev);
        }

        Some(current)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONJUGATE BETA-BINOMIAL
// ═══════════════════════════════════════════════════════════════════════════════

/// Beta distribution prior/posterior parameters.
///
/// Beta(α, β) is the conjugate prior for the Binomial likelihood.
/// BCPNN uses this: prior on drug-event probability → posterior after observing cases.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BetaParams {
    /// Shape parameter α (successes + prior)
    pub alpha: f64,
    /// Shape parameter β (failures + prior)
    pub beta: f64,
}

impl BetaParams {
    /// Create new Beta parameters.
    #[must_use]
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self { alpha, beta }
    }

    /// Jeffreys non-informative prior: Beta(0.5, 0.5)
    #[must_use]
    pub fn jeffreys() -> Self {
        Self {
            alpha: 0.5,
            beta: 0.5,
        }
    }

    /// Uniform (flat) prior: Beta(1, 1)
    #[must_use]
    pub fn uniform() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
        }
    }

    /// Mean of the distribution: α / (α + β)
    #[must_use]
    pub fn mean(&self) -> f64 {
        if self.alpha + self.beta > 0.0 {
            self.alpha / (self.alpha + self.beta)
        } else {
            0.5
        }
    }

    /// Variance of the distribution: αβ / ((α+β)²(α+β+1))
    #[must_use]
    pub fn variance(&self) -> f64 {
        let ab = self.alpha + self.beta;
        if ab > 0.0 && (ab + 1.0) > 0.0 {
            (self.alpha * self.beta) / (ab * ab * (ab + 1.0))
        } else {
            0.25
        }
    }
}

/// Binomial evidence: observed successes and failures.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BinomialEvidence {
    /// Number of successes (e.g., drug-event co-occurrences)
    pub successes: u64,
    /// Number of failures (e.g., drug exposures without event)
    pub failures: u64,
}

impl BinomialEvidence {
    /// Create new binomial evidence.
    #[must_use]
    pub fn new(successes: u64, failures: u64) -> Self {
        Self {
            successes,
            failures,
        }
    }
}

/// Conjugate Beta-Binomial Bayesian update.
///
/// Prior: Beta(α, β)
/// Likelihood: Binomial(n, p)
/// Posterior: Beta(α + successes, β + failures)
///
/// Grounding: →(Causality) + N(Quantity) + ∂(Boundary)
pub struct ConjugateBetaBinomial;

impl BayesianUpdate for ConjugateBetaBinomial {
    type Prior = BetaParams;
    type Evidence = BinomialEvidence;
    type Posterior = BetaParams;

    fn default_prior() -> BetaParams {
        BetaParams::jeffreys()
    }

    fn update(prior: &BetaParams, evidence: &BinomialEvidence) -> BetaParams {
        BetaParams {
            alpha: prior.alpha + evidence.successes as f64,
            beta: prior.beta + evidence.failures as f64,
        }
    }

    fn summarize(posterior: &BetaParams) -> Measured<f64> {
        let mean = posterior.mean();
        let variance = posterior.variance();

        // CALIBRATION: Beta posterior precision → Confidence
        // Higher concentration (α + β) → narrower distribution → higher confidence
        // confidence = clamp(1.0 - 4.0 * sqrt(variance), 0.05, 0.99)
        let conf = (1.0 - 4.0 * variance.sqrt()).clamp(0.05, 0.99);

        Measured::new(mean, Confidence::new(conf))
    }
}

// Posterior = Prior for conjugate pairs — Into<Self> is provided by core's blanket impl.

// ═══════════════════════════════════════════════════════════════════════════════
// GAMMA-POISSON MIXTURE
// ═══════════════════════════════════════════════════════════════════════════════

/// Gamma distribution prior/posterior parameters.
///
/// Gamma(α, β) is the conjugate prior for the Poisson likelihood.
/// EBGM uses a two-component mixture of these.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GammaParams {
    /// Shape parameter α (event count + prior)
    pub shape: f64,
    /// Rate parameter β (exposure + prior)
    pub rate: f64,
}

impl GammaParams {
    /// Create new Gamma parameters.
    #[must_use]
    pub fn new(shape: f64, rate: f64) -> Self {
        Self { shape, rate }
    }

    /// Weakly informative prior: Gamma(0.5, 0.1)
    #[must_use]
    pub fn weak() -> Self {
        Self {
            shape: 0.5,
            rate: 0.1,
        }
    }

    /// Mean of the distribution: shape / rate
    #[must_use]
    pub fn mean(&self) -> f64 {
        if self.rate > 0.0 {
            self.shape / self.rate
        } else {
            0.0
        }
    }

    /// Variance: shape / rate²
    #[must_use]
    pub fn variance(&self) -> f64 {
        if self.rate > 0.0 {
            self.shape / (self.rate * self.rate)
        } else {
            f64::INFINITY
        }
    }
}

/// Poisson evidence: observed count and exposure.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoissonEvidence {
    /// Observed event count
    pub count: u64,
    /// Exposure (expected count under null)
    pub exposure: f64,
}

impl PoissonEvidence {
    /// Create new Poisson evidence.
    #[must_use]
    pub fn new(count: u64, exposure: f64) -> Self {
        Self { count, exposure }
    }
}

/// Conjugate Gamma-Poisson Bayesian update.
///
/// Prior: Gamma(α, β)
/// Likelihood: Poisson(λ·E) with exposure E
/// Posterior: Gamma(α + count, β + exposure)
///
/// Grounding: →(Causality) + N(Quantity) + ν(Frequency)
pub struct GammaPoissonMixture;

impl BayesianUpdate for GammaPoissonMixture {
    type Prior = GammaParams;
    type Evidence = PoissonEvidence;
    type Posterior = GammaParams;

    fn default_prior() -> GammaParams {
        GammaParams::weak()
    }

    fn update(prior: &GammaParams, evidence: &PoissonEvidence) -> GammaParams {
        GammaParams {
            shape: prior.shape + evidence.count as f64,
            rate: prior.rate + evidence.exposure,
        }
    }

    fn summarize(posterior: &GammaParams) -> Measured<f64> {
        let mean = posterior.mean();
        let variance = posterior.variance();

        // CALIBRATION: Gamma posterior precision → Confidence
        // Coefficient of variation: CV = sqrt(variance) / mean
        // Low CV → high confidence
        // confidence = clamp(1.0 - CV, 0.05, 0.99)
        let cv = if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            1.0
        };
        let conf = (1.0 - cv).clamp(0.05, 0.99);

        Measured::new(mean, Confidence::new(conf))
    }
}

// Posterior = Prior (conjugate pair)
// Posterior = Prior for conjugate pairs — Into<Self> is provided by core's blanket impl.

// ═══════════════════════════════════════════════════════════════════════════════
// GROUNDING
// ═══════════════════════════════════════════════════════════════════════════════

/// Dominant primitive for BayesianUpdate outputs.
///
/// →(Causality) dominates: evidence → belief update is fundamentally causal.
///
/// Full composition: →(Causality) + ρ(Recursion) + ∂(Boundary) + N(Quantity)
#[must_use]
pub fn bayesian_update_dominant_primitive() -> nexcore_lex_primitiva::primitiva::LexPrimitiva {
    nexcore_lex_primitiva::primitiva::LexPrimitiva::Causality
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // ConjugateBetaBinomial
    // ================================================================

    #[test]
    fn beta_binomial_known_posterior() {
        // Beta(1,1) prior + 10 successes, 5 failures → Beta(11, 6)
        let prior = BetaParams::uniform();
        let evidence = BinomialEvidence::new(10, 5);
        let posterior = ConjugateBetaBinomial::update(&prior, &evidence);

        assert!((posterior.alpha - 11.0).abs() < f64::EPSILON);
        assert!((posterior.beta - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn beta_binomial_jeffreys_prior() {
        // Jeffreys Beta(0.5, 0.5) + 3 successes, 7 failures → Beta(3.5, 7.5)
        let prior = ConjugateBetaBinomial::default_prior();
        assert!((prior.alpha - 0.5).abs() < f64::EPSILON);
        assert!((prior.beta - 0.5).abs() < f64::EPSILON);

        let evidence = BinomialEvidence::new(3, 7);
        let posterior = ConjugateBetaBinomial::update(&prior, &evidence);

        assert!((posterior.alpha - 3.5).abs() < f64::EPSILON);
        assert!((posterior.beta - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn beta_binomial_mean_moves_toward_data() {
        let prior = BetaParams::uniform(); // mean = 0.5
        let evidence = BinomialEvidence::new(90, 10); // observed rate = 0.9
        let posterior = ConjugateBetaBinomial::update(&prior, &evidence);

        assert!(
            posterior.mean() > prior.mean(),
            "Posterior mean should shift toward observed rate"
        );
        assert!(
            posterior.mean() > 0.8,
            "With 100 observations, posterior should be close to 0.9"
        );
    }

    #[test]
    fn beta_binomial_summarize_confidence() {
        let posterior = BetaParams::new(100.0, 100.0); // High precision
        let measured = ConjugateBetaBinomial::summarize(&posterior);

        assert!(
            measured.confidence.value() >= 0.05 && measured.confidence.value() <= 0.99,
            "Confidence in range: {}",
            measured.confidence.value()
        );
        assert!(
            (measured.value - 0.5).abs() < 0.01,
            "Mean of Beta(100,100) ≈ 0.5"
        );
    }

    #[test]
    fn beta_binomial_large_sample_high_confidence() {
        let small = BetaParams::new(2.0, 3.0);
        let large = BetaParams::new(200.0, 300.0);

        let small_conf = ConjugateBetaBinomial::summarize(&small).confidence.value();
        let large_conf = ConjugateBetaBinomial::summarize(&large).confidence.value();

        assert!(
            large_conf > small_conf,
            "Large sample confidence ({large_conf}) > small ({small_conf})"
        );
    }

    // ================================================================
    // Sequential Update — Beta-Binomial
    // ================================================================

    #[test]
    fn beta_binomial_sequential_equals_batch() {
        // Sequential: (10s, 5f) then (20s, 10f) then (5s, 3f)
        // Should equal single batch: (35s, 18f) from Beta(1,1)
        let prior = BetaParams::uniform();

        let evidence = [
            BinomialEvidence::new(10, 5),
            BinomialEvidence::new(20, 10),
            BinomialEvidence::new(5, 3),
        ];

        let sequential = ConjugateBetaBinomial::sequential_update(&prior, &evidence);
        assert!(sequential.is_some());
        let seq = sequential.unwrap_or(BetaParams::uniform());

        // Batch: 35 successes, 18 failures
        let batch = ConjugateBetaBinomial::update(&prior, &BinomialEvidence::new(35, 18));

        assert!(
            (seq.alpha - batch.alpha).abs() < f64::EPSILON,
            "Sequential α ({}) != batch α ({})",
            seq.alpha,
            batch.alpha
        );
        assert!(
            (seq.beta - batch.beta).abs() < f64::EPSILON,
            "Sequential β ({}) != batch β ({})",
            seq.beta,
            batch.beta
        );
    }

    #[test]
    fn beta_binomial_sequential_empty() {
        let prior = BetaParams::uniform();
        let result = ConjugateBetaBinomial::sequential_update(&prior, &[]);
        assert!(result.is_none(), "Empty evidence should return None");
    }

    // ================================================================
    // GammaPoissonMixture
    // ================================================================

    #[test]
    fn gamma_poisson_known_posterior() {
        // Gamma(0.5, 0.1) prior + count=10, exposure=5.0 → Gamma(10.5, 5.1)
        let prior = GammaPoissonMixture::default_prior();
        let evidence = PoissonEvidence::new(10, 5.0);
        let posterior = GammaPoissonMixture::update(&prior, &evidence);

        assert!((posterior.shape - 10.5).abs() < f64::EPSILON);
        assert!((posterior.rate - 5.1).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_poisson_mean_interpretation() {
        // Posterior mean = (prior_shape + count) / (prior_rate + exposure)
        // This is the shrinkage estimator for the Poisson rate
        let prior = GammaParams::new(1.0, 1.0);
        let evidence = PoissonEvidence::new(20, 10.0);
        let posterior = GammaPoissonMixture::update(&prior, &evidence);

        // Mean = 21 / 11 ≈ 1.909 (shrunk from raw 2.0)
        assert!(
            (posterior.mean() - 21.0 / 11.0).abs() < 1e-10,
            "Posterior mean = (α + n) / (β + E)"
        );
        assert!(
            posterior.mean() < 2.0,
            "Shrinkage: posterior mean < raw ratio"
        );
    }

    #[test]
    fn gamma_poisson_summarize_confidence() {
        let posterior = GammaParams::new(100.0, 50.0); // High precision
        let measured = GammaPoissonMixture::summarize(&posterior);

        assert!(
            measured.confidence.value() >= 0.05 && measured.confidence.value() <= 0.99,
            "Confidence in range: {}",
            measured.confidence.value()
        );
        assert!(
            (measured.value - 2.0).abs() < 0.01,
            "Mean of Gamma(100,50) = 2.0"
        );
    }

    #[test]
    fn gamma_poisson_large_sample_high_confidence() {
        let small = GammaParams::new(2.0, 1.0);
        let large = GammaParams::new(200.0, 100.0);

        let small_conf = GammaPoissonMixture::summarize(&small).confidence.value();
        let large_conf = GammaPoissonMixture::summarize(&large).confidence.value();

        assert!(
            large_conf > small_conf,
            "Large sample confidence ({large_conf}) > small ({small_conf})"
        );
    }

    // ================================================================
    // Sequential Update — Gamma-Poisson
    // ================================================================

    #[test]
    fn gamma_poisson_sequential_equals_batch() {
        // Sequential: 3 FAERS quarters
        let prior = GammaParams::new(1.0, 1.0);

        let evidence = [
            PoissonEvidence::new(5, 3.0),  // Q1
            PoissonEvidence::new(8, 4.0),  // Q2
            PoissonEvidence::new(12, 6.0), // Q3
        ];

        let sequential = GammaPoissonMixture::sequential_update(&prior, &evidence);
        assert!(sequential.is_some());
        let seq = sequential.unwrap_or(GammaParams::new(0.0, 0.0));

        // Batch: 25 counts, 13.0 exposure
        let batch = GammaPoissonMixture::update(&prior, &PoissonEvidence::new(25, 13.0));

        assert!(
            (seq.shape - batch.shape).abs() < f64::EPSILON,
            "Sequential shape ({}) != batch shape ({})",
            seq.shape,
            batch.shape
        );
        assert!(
            (seq.rate - batch.rate).abs() < f64::EPSILON,
            "Sequential rate ({}) != batch rate ({})",
            seq.rate,
            batch.rate
        );
    }

    #[test]
    fn gamma_poisson_sequential_empty() {
        let prior = GammaParams::new(1.0, 1.0);
        let result = GammaPoissonMixture::sequential_update(&prior, &[]);
        assert!(result.is_none());
    }

    // ================================================================
    // Grounding
    // ================================================================

    #[test]
    fn dominant_primitive_is_causality() {
        assert_eq!(
            bayesian_update_dominant_primitive(),
            nexcore_lex_primitiva::primitiva::LexPrimitiva::Causality
        );
    }

    // ================================================================
    // Cross-implementation consistency
    // ================================================================

    #[test]
    fn both_conjugates_produce_valid_measured() {
        let beta_post = BetaParams::new(10.0, 5.0);
        let gamma_post = GammaParams::new(10.0, 5.0);

        let beta_m = ConjugateBetaBinomial::summarize(&beta_post);
        let gamma_m = GammaPoissonMixture::summarize(&gamma_post);

        // Both should produce valid confidence
        assert!(beta_m.confidence.value() >= 0.05);
        assert!(gamma_m.confidence.value() >= 0.05);
        assert!(beta_m.confidence.value() <= 0.99);
        assert!(gamma_m.confidence.value() <= 0.99);

        // Both should produce finite values
        assert!(beta_m.value.is_finite());
        assert!(gamma_m.value.is_finite());
    }
}
