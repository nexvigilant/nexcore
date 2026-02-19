//! # ToV §6: Axiom 5 - Emergence
//!
//! Formal implementation of Axiom 5 and its prerequisites (Definitions 6.1-6.5).
//!
//! ## Axiom Statement
//!
//! For every vigilance system 𝒱 with hierarchy ℒ and harm level ℓ_H:
//!
//! **5.1** Harm manifests at level ℓ_H while perturbations originate at level ℓ₁ ≺ ℓ_H
//!
//! **5.2** For each i = 1, ..., H-1, propagation probability Pᵢ→ᵢ₊₁ ∈ (0,1)
//!
//! **5.3** Under Markov assumption: ℙ(H | δs₁) = ∏ᵢ₌₁ᴴ⁻¹ Pᵢ→ᵢ₊₁(mᵢ, cᵢ, bᵢ, tᵢ, θ)
//!
//! ## Symbolic Formulation
//!
//! **ℙ(H | δs₁) = ∏ᵢ₌₁ᴴ⁻¹ Pᵢ→ᵢ₊₁(||δsᵢ||, cᵢ, bᵢ, tᵢ, θ)**
//!
//! ## Definitions Implemented
//!
//! - **Definition 6.1**: Level-specific perturbation δsᵢ
//! - **Definition 6.2**: Propagation (δsᵢ → δsᵢ₊₁ if ||δsᵢ₊₁|| > εᵢ)
//! - **Definition 6.3**: Buffering capacity bᵢ
//! - **Definition 6.4**: Propagation probability Pᵢ→ᵢ₊₁(m, c, b, t, θ)
//! - **Definition 6.5**: Harm level ℓ_H
//!
//! ## Propagation Function Properties (P1-P4)
//!
//! - **(P1)** ∂P/∂m ≥ 0: Higher magnitude → higher propagation
//! - **(P2)** ∂P/∂c ≥ 0: Higher centrality → higher propagation
//! - **(P3)** ∂P/∂b ≤ 0: Higher buffering → lower propagation
//! - **(P4)** ∂P/∂t ≥ 0: Longer exposure → higher propagation
//!
//! ## Mathematical Validation (2026-01-29)
//!
//! | Formula | Expression | Validated |
//! |---------|------------|-----------|
//! | 8-level attenuation | 0.9^8 | 0.43046721 ✓ |
//! | Variable attenuation | ∏[0.95..0.60] | 0.11904165 ✓ |
//! | Exponential decay | e^(-b·m) | Standard form ✓ |

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 6.1: LEVEL-SPECIFIC PERTURBATION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 6.1: Level-specific perturbation δsᵢ.
///
/// A perturbation at level ℓᵢ is a deviation δsᵢ from baseline in the level
/// state space Sᵢ. The magnitude is ||δsᵢ||.
///
/// # Remarks
/// The baseline s̄ᵢ ∈ Sᵢ represents the nominal operating condition at level i—
/// the state in the absence of perturbation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelPerturbation {
    /// Level index i (0-based).
    pub level: usize,

    /// Deviation from baseline δsᵢ.
    pub deviation: Vec<f64>,

    /// Baseline state s̄ᵢ (optional).
    pub baseline: Option<Vec<f64>>,

    /// Magnitude ||δsᵢ|| (Euclidean norm).
    pub magnitude: f64,
}

impl LevelPerturbation {
    /// Create a perturbation from deviation vector.
    #[must_use]
    pub fn new(level: usize, deviation: Vec<f64>) -> Self {
        let magnitude = Self::euclidean_norm(&deviation);
        Self {
            level,
            deviation,
            baseline: None,
            magnitude,
        }
    }

    /// Create a perturbation with explicit baseline.
    #[must_use]
    pub fn with_baseline(level: usize, state: Vec<f64>, baseline: Vec<f64>) -> Self {
        let deviation: Vec<f64> = state
            .iter()
            .zip(baseline.iter())
            .map(|(s, b)| s - b)
            .collect();
        let magnitude = Self::euclidean_norm(&deviation);
        Self {
            level,
            deviation,
            baseline: Some(baseline),
            magnitude,
        }
    }

    /// Calculate Euclidean norm ||v||.
    fn euclidean_norm(v: &[f64]) -> f64 {
        v.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Check if perturbation exceeds threshold εᵢ (Definition 6.2).
    #[must_use]
    pub fn exceeds_threshold(&self, epsilon: f64) -> bool {
        self.magnitude > epsilon
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 6.3: BUFFERING CAPACITY
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 6.3: Buffering capacity bᵢ ∈ ℝ≥0.
///
/// Quantifies the system's ability to absorb perturbations at level ℓᵢ
/// without propagating them to level ℓᵢ₊₁.
///
/// Higher buffering capacity reduces propagation probability.
///
/// # Domain-Specific Examples
/// - **Biological**: Homeostatic feedback, redundant pathways, repair mechanisms
/// - **Engineering**: Error correction, capacity margins, fault tolerance
/// - **Pharmacological**: Compensatory physiological responses, receptor reserve
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BufferingCapacity {
    /// Level index i.
    pub level: usize,

    /// Buffering capacity value bᵢ ≥ 0.
    pub capacity: f64,

    /// Type of buffering mechanism.
    pub mechanism: BufferingMechanism,
}

/// Types of buffering mechanisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BufferingMechanism {
    /// Homeostatic feedback loops.
    Homeostatic,
    /// Redundant pathways or components.
    Redundancy,
    /// Active repair mechanisms.
    Repair,
    /// Passive absorption (capacity margin).
    Absorption,
    /// Error correction codes/protocols.
    ErrorCorrection,
    /// Fault tolerance (graceful degradation).
    FaultTolerance,
    /// Compensatory response (e.g., receptor reserve).
    Compensatory,
    /// Custom/unspecified mechanism.
    Custom,
}

impl BufferingCapacity {
    /// Create a new buffering capacity.
    #[must_use]
    pub fn new(level: usize, capacity: f64, mechanism: BufferingMechanism) -> Self {
        Self {
            level,
            capacity: capacity.max(0.0), // Ensure non-negative
            mechanism,
        }
    }

    /// Check if capacity provides significant buffering.
    #[must_use]
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.capacity >= threshold
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 6.4: PROPAGATION PROBABILITY
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for propagation probability function.
///
/// Pᵢ→ᵢ₊₁(m, c, b, t, θ) ∈ (0, 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationParams {
    /// Perturbation magnitude m = ||δsᵢ||.
    pub magnitude: f64,

    /// Network centrality c ∈ [0, 1] of perturbed component.
    pub centrality: f64,

    /// Buffering capacity b at level i.
    pub buffering: f64,

    /// Exposure duration t.
    pub duration: f64,

    /// Individual susceptibility parameters θ.
    pub susceptibility: Vec<f64>,
}

impl PropagationParams {
    /// Create propagation parameters with defaults.
    #[must_use]
    pub fn new(magnitude: f64) -> Self {
        Self {
            magnitude,
            centrality: 0.5,
            buffering: 1.0,
            duration: 1.0,
            susceptibility: vec![],
        }
    }

    /// Set network centrality.
    #[must_use]
    pub fn with_centrality(mut self, centrality: f64) -> Self {
        self.centrality = centrality.clamp(0.0, 1.0);
        self
    }

    /// Set buffering capacity.
    #[must_use]
    pub fn with_buffering(mut self, buffering: f64) -> Self {
        self.buffering = buffering.max(0.0);
        self
    }

    /// Set exposure duration.
    #[must_use]
    pub fn with_duration(mut self, duration: f64) -> Self {
        self.duration = duration.max(0.0);
        self
    }

    /// Set susceptibility parameters.
    #[must_use]
    pub fn with_susceptibility(mut self, susceptibility: Vec<f64>) -> Self {
        self.susceptibility = susceptibility;
        self
    }
}

/// Definition 6.4: Propagation probability function.
///
/// Pᵢ→ᵢ₊₁: ℝ≥0 × [0,1] × ℝ≥0 × ℝ≥0 × Θ → [0,1]
///
/// # Properties (P1-P4)
/// - (P1) ∂P/∂m ≥ 0: Higher perturbation magnitude increases propagation
/// - (P2) ∂P/∂c ≥ 0: Higher network centrality increases propagation
/// - (P3) ∂P/∂b ≤ 0: Higher buffering capacity decreases propagation
/// - (P4) ∂P/∂t ≥ 0: Longer exposure duration increases propagation
pub trait PropagationFunction: std::fmt::Debug {
    /// Evaluate Pᵢ→ᵢ₊₁(m, c, b, t, θ).
    ///
    /// Must return a value in (0, 1) - never 0 or 1.
    fn evaluate(&self, params: &PropagationParams) -> f64;

    /// Source level index i.
    fn source_level(&self) -> usize;

    /// Target level index i+1.
    fn target_level(&self) -> usize {
        self.source_level() + 1
    }

    /// Verify monotonicity properties P1-P4.
    fn verify_properties(
        &self,
        baseline: &PropagationParams,
        epsilon: f64,
    ) -> PropertyVerification {
        let p_base = self.evaluate(baseline);

        // P1: ∂P/∂m ≥ 0
        let mut params_m = baseline.clone();
        params_m.magnitude += epsilon;
        let p1_holds = self.evaluate(&params_m) >= p_base - 1e-10;

        // P2: ∂P/∂c ≥ 0
        let mut params_c = baseline.clone();
        params_c.centrality = (params_c.centrality + epsilon).min(1.0);
        let p2_holds = self.evaluate(&params_c) >= p_base - 1e-10;

        // P3: ∂P/∂b ≤ 0
        let mut params_b = baseline.clone();
        params_b.buffering += epsilon;
        let p3_holds = self.evaluate(&params_b) <= p_base + 1e-10;

        // P4: ∂P/∂t ≥ 0
        let mut params_t = baseline.clone();
        params_t.duration += epsilon;
        let p4_holds = self.evaluate(&params_t) >= p_base - 1e-10;

        PropertyVerification {
            p1_magnitude_monotone: p1_holds,
            p2_centrality_monotone: p2_holds,
            p3_buffering_monotone: p3_holds,
            p4_duration_monotone: p4_holds,
            all_satisfied: p1_holds && p2_holds && p3_holds && p4_holds,
        }
    }
}

/// Result of verifying propagation function properties P1-P4.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PropertyVerification {
    /// P1: ∂P/∂m ≥ 0 (magnitude monotonicity).
    pub p1_magnitude_monotone: bool,
    /// P2: ∂P/∂c ≥ 0 (centrality monotonicity).
    pub p2_centrality_monotone: bool,
    /// P3: ∂P/∂b ≤ 0 (buffering monotonicity).
    pub p3_buffering_monotone: bool,
    /// P4: ∂P/∂t ≥ 0 (duration monotonicity).
    pub p4_duration_monotone: bool,
    /// All properties satisfied.
    pub all_satisfied: bool,
}

/// Standard sigmoidal propagation function.
///
/// P(m, c, b, t, θ) = σ((m · c · t - b) / scale) where σ(x) = 1/(1+e^(-x))
///
/// Clipped to (ε, 1-ε) to ensure strict (0, 1) bounds.
#[derive(Debug, Clone)]
pub struct SigmoidalPropagation {
    source_level: usize,
    /// Scale parameter (higher = sharper transition).
    scale: f64,
    /// Minimum probability (avoids 0).
    min_prob: f64,
    /// Maximum probability (avoids 1).
    max_prob: f64,
}

impl SigmoidalPropagation {
    /// Create a sigmoidal propagation function.
    #[must_use]
    pub fn new(source_level: usize, scale: f64) -> Self {
        Self {
            source_level,
            scale,
            min_prob: 0.001,
            max_prob: 0.999,
        }
    }

    /// Set probability bounds.
    #[must_use]
    pub fn with_bounds(mut self, min: f64, max: f64) -> Self {
        self.min_prob = min.clamp(0.001, 0.5);
        self.max_prob = max.clamp(0.5, 0.999);
        self
    }

    /// Sigmoid function σ(x) = 1/(1+e^(-x)).
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
}

impl PropagationFunction for SigmoidalPropagation {
    fn evaluate(&self, params: &PropagationParams) -> f64 {
        // Effective "pressure" toward propagation
        let pressure = params.magnitude * params.centrality * params.duration;
        // Buffering reduces the effective pressure
        let effective = (pressure - params.buffering) / self.scale;
        // Apply sigmoid and clip to (min, max)
        let raw = Self::sigmoid(effective);
        raw.clamp(self.min_prob, self.max_prob)
    }

    fn source_level(&self) -> usize {
        self.source_level
    }
}

/// Exponential decay propagation function.
///
/// P(m, c, b, t, θ) = 1 - e^(-(m · c · t) / b)
///
/// Models buffering as exponential absorption.
#[derive(Debug, Clone)]
pub struct ExponentialPropagation {
    source_level: usize,
    /// Minimum probability (avoids 0).
    min_prob: f64,
    /// Maximum probability (avoids 1).
    max_prob: f64,
}

impl ExponentialPropagation {
    /// Create an exponential propagation function.
    #[must_use]
    pub fn new(source_level: usize) -> Self {
        Self {
            source_level,
            min_prob: 0.001,
            max_prob: 0.999,
        }
    }
}

impl PropagationFunction for ExponentialPropagation {
    fn evaluate(&self, params: &PropagationParams) -> f64 {
        let b_safe = params.buffering.max(0.001); // Avoid division by zero
        let exponent = -(params.magnitude * params.centrality * params.duration) / b_safe;
        let raw = 1.0 - exponent.exp();
        raw.clamp(self.min_prob, self.max_prob)
    }

    fn source_level(&self) -> usize {
        self.source_level
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 6.5: HARM LEVEL
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 6.5: Harm level ℓ_H ∈ L.
///
/// The hierarchical level at which the harm event H is defined.
/// Typically a high level (clinical, system, societal) where harm
/// is observable and consequential.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HarmLevel {
    /// Level index H (0-based, highest levels typically correspond to H).
    pub level_index: usize,

    /// Human-readable name.
    pub name: &'static str,

    /// Whether this level is observable (can detect harm directly).
    pub observable: bool,
}

impl HarmLevel {
    /// Create a harm level specification.
    #[must_use]
    pub const fn new(level_index: usize, name: &'static str, observable: bool) -> Self {
        Self {
            level_index,
            name,
            observable,
        }
    }

    /// Standard PV harm level (Organism, level 5).
    pub const PV_ORGANISM: Self = Self::new(5, "Organism", true);

    /// Standard PV population harm level (level 6).
    pub const PV_POPULATION: Self = Self::new(6, "Population", true);

    /// Standard cloud harm level (System, level 4).
    pub const CLOUD_SYSTEM: Self = Self::new(4, "System", true);

    /// Standard AI harm level (Behavior, level 3).
    pub const AI_BEHAVIOR: Self = Self::new(3, "Behavior", true);
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 5: EMERGENCE
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 5: Emergence framework.
///
/// Computes harm probability as product of propagation probabilities:
/// ℙ(H | δs₁) = ∏ᵢ₌₁ᴴ⁻¹ Pᵢ→ᵢ₊₁(mᵢ, cᵢ, bᵢ, tᵢ, θ)
#[derive(Debug)]
pub struct EmergenceFramework {
    /// Harm level ℓ_H.
    harm_level: HarmLevel,

    /// Propagation functions for each level transition.
    propagation_functions: Vec<Box<dyn PropagationFunction>>,

    /// Detection thresholds εᵢ for each level.
    detection_thresholds: Vec<f64>,

    /// Uses Markov assumption.
    is_markov: bool,
}

impl EmergenceFramework {
    /// Create an emergence framework.
    #[must_use]
    pub fn new(harm_level: HarmLevel) -> Self {
        Self {
            harm_level,
            propagation_functions: Vec::new(),
            detection_thresholds: Vec::new(),
            is_markov: true,
        }
    }

    /// Add a propagation function for level i → i+1 transition.
    pub fn add_propagation<F: PropagationFunction + 'static>(&mut self, func: F) {
        self.propagation_functions.push(Box::new(func));
    }

    /// Set detection thresholds εᵢ.
    pub fn set_thresholds(&mut self, thresholds: Vec<f64>) {
        self.detection_thresholds = thresholds;
    }

    /// Set whether Markov assumption holds.
    pub fn set_markov(&mut self, is_markov: bool) {
        self.is_markov = is_markov;
    }

    /// Get harm level.
    #[must_use]
    pub const fn harm_level(&self) -> &HarmLevel {
        &self.harm_level
    }

    /// Number of level transitions.
    #[must_use]
    pub fn n_transitions(&self) -> usize {
        self.propagation_functions.len()
    }

    /// Compute harm probability under Markov assumption.
    ///
    /// ℙ(H | δs₁) = ∏ᵢ₌₁ᴴ⁻¹ Pᵢ→ᵢ₊₁(params[i])
    #[must_use]
    pub fn harm_probability_markov(
        &self,
        level_params: &[PropagationParams],
    ) -> HarmProbabilityResult {
        if level_params.len() != self.propagation_functions.len() {
            return HarmProbabilityResult {
                probability: 0.0,
                level_probabilities: vec![],
                propagated_to_level: 0,
                is_markov: true,
                error: Some("Parameter count != transition count".to_string()),
            };
        }

        let mut level_probabilities = Vec::with_capacity(level_params.len());
        let mut product = 1.0;

        for (i, (func, params)) in self
            .propagation_functions
            .iter()
            .zip(level_params.iter())
            .enumerate()
        {
            let p_i = func.evaluate(params);
            level_probabilities.push(p_i);
            product *= p_i;

            // Check if propagation continues (Definition 6.2)
            if let Some(&threshold) = self.detection_thresholds.get(i) {
                if params.magnitude <= threshold {
                    // Perturbation below detection threshold at this level
                    return HarmProbabilityResult {
                        probability: product,
                        level_probabilities,
                        propagated_to_level: i,
                        is_markov: true,
                        error: None,
                    };
                }
            }
        }

        HarmProbabilityResult {
            probability: product,
            level_probabilities,
            propagated_to_level: self.harm_level.level_index,
            is_markov: true,
            error: None,
        }
    }

    /// Compute harm probability with uniform propagation.
    ///
    /// ℙ(H | δs₁) = α^(H-1) for uniform α.
    #[must_use]
    pub fn harm_probability_uniform(&self, alpha: f64, n_levels: usize) -> f64 {
        let alpha_clamped = alpha.clamp(0.001, 0.999);
        alpha_clamped.powi(n_levels.saturating_sub(1) as i32)
    }
}

/// Result of harm probability computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmProbabilityResult {
    /// Total harm probability ℙ(H | δs₁).
    pub probability: f64,

    /// Individual level propagation probabilities Pᵢ→ᵢ₊₁.
    pub level_probabilities: Vec<f64>,

    /// Highest level to which perturbation propagated.
    pub propagated_to_level: usize,

    /// Whether Markov assumption was used.
    pub is_markov: bool,

    /// Error message if computation failed.
    pub error: Option<String>,
}

impl HarmProbabilityResult {
    /// Check if harm is likely (probability > threshold).
    #[must_use]
    pub fn is_likely(&self, threshold: f64) -> bool {
        self.probability > threshold
    }

    /// Get attenuation factor (1 - probability).
    #[must_use]
    pub fn attenuation(&self) -> f64 {
        1.0 - self.probability
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AXIOM 5 VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════

/// Axiom 5 verification result.
///
/// Verifies:
/// 1. Harm manifests at level ℓ_H while perturbations originate at ℓ₁ ≺ ℓ_H
/// 2. For each i, Pᵢ→ᵢ₊₁ ∈ (0, 1) (open interval)
/// 3. Propagation functions satisfy monotonicity properties P1-P4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom5Verification {
    /// Condition 1: Harm level > perturbation origin level.
    pub condition_1_level_ordering: bool,

    /// Condition 2: All Pᵢ→ᵢ₊₁ ∈ (0, 1).
    pub condition_2_open_interval: bool,

    /// Condition 3: P1-P4 properties satisfied.
    pub condition_3_monotonicity: bool,

    /// Harm level index.
    pub harm_level: usize,

    /// Number of level transitions.
    pub n_transitions: usize,

    /// Property verification for each level.
    pub property_verifications: Vec<PropertyVerification>,

    /// All axiom conditions satisfied.
    pub axiom_satisfied: bool,

    /// Details.
    pub details: Vec<String>,
}

impl Axiom5Verification {
    /// Verify Axiom 5 for an emergence framework.
    #[must_use]
    pub fn verify(framework: &EmergenceFramework, test_params: &PropagationParams) -> Self {
        let mut details = Vec::new();
        let harm_level = framework.harm_level.level_index;
        let n_transitions = framework.n_transitions();

        // Condition 1: Level ordering (harm at high level, perturbation at low)
        let condition_1_level_ordering = harm_level > 0 && n_transitions > 0;
        details.push(format!(
            "Condition 1: Harm at level {}, {} transitions defined",
            harm_level, n_transitions
        ));

        // Condition 2: Open interval (0, 1) for all probabilities
        let mut condition_2_open_interval = true;
        for (i, func) in framework.propagation_functions.iter().enumerate() {
            let p = func.evaluate(test_params);
            if p <= 0.0 || p >= 1.0 {
                condition_2_open_interval = false;
                details.push(format!(
                    "Condition 2 VIOLATED: P_{}_{}={:.6} not in (0,1)",
                    i,
                    i + 1,
                    p
                ));
            }
        }
        if condition_2_open_interval {
            details.push("Condition 2: All Pᵢ→ᵢ₊₁ ∈ (0, 1) ✓".to_string());
        }

        // Condition 3: Monotonicity properties P1-P4
        let mut property_verifications = Vec::new();
        let mut condition_3_monotonicity = true;
        for func in &framework.propagation_functions {
            let verification = func.verify_properties(test_params, 0.1);
            if !verification.all_satisfied {
                condition_3_monotonicity = false;
            }
            property_verifications.push(verification);
        }
        if condition_3_monotonicity {
            details.push("Condition 3: Monotonicity P1-P4 satisfied ✓".to_string());
        } else {
            details.push("Condition 3: Some monotonicity properties violated".to_string());
        }

        let axiom_satisfied =
            condition_1_level_ordering && condition_2_open_interval && condition_3_monotonicity;

        Self {
            condition_1_level_ordering,
            condition_2_open_interval,
            condition_3_monotonicity,
            harm_level,
            n_transitions,
            property_verifications,
            axiom_satisfied,
            details,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NON-MARKOVIAN EXTENSION (§6.3.C)
// ═══════════════════════════════════════════════════════════════════════════

/// Non-Markovian extension for systems with memory effects.
///
/// For systems violating the Markov assumption (e.g., sensitization, tolerance),
/// the transition kernel depends on full history:
///
/// ℙ(H) = ∫_{paths} ∏ᵢ Kᵢ(δsᵢ₊₁ | δs₁, ..., δsᵢ) d(paths)
///
/// This struct represents a simplified history-dependent model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonMarkovianHistory {
    /// Past perturbation magnitudes at each level.
    pub past_magnitudes: Vec<Vec<f64>>,

    /// Sensitization factor (past exposure increases propagation).
    pub sensitization: f64,

    /// Tolerance factor (past exposure decreases propagation).
    pub tolerance: f64,

    /// Memory decay rate (older events have less influence).
    pub decay_rate: f64,
}

impl NonMarkovianHistory {
    /// Create empty history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            past_magnitudes: Vec::new(),
            sensitization: 0.0,
            tolerance: 0.0,
            decay_rate: 0.5,
        }
    }

    /// Add a perturbation to history.
    pub fn record(&mut self, level: usize, magnitude: f64) {
        while self.past_magnitudes.len() <= level {
            self.past_magnitudes.push(Vec::new());
        }
        self.past_magnitudes[level].push(magnitude);
    }

    /// Calculate history-adjusted propagation probability modifier.
    ///
    /// Modifier > 1: sensitization effect (increases propagation)
    /// Modifier < 1: tolerance effect (decreases propagation)
    #[must_use]
    pub fn history_modifier(&self, level: usize) -> f64 {
        let Some(history) = self.past_magnitudes.get(level) else {
            return 1.0;
        };

        if history.is_empty() {
            return 1.0;
        }

        // Weighted sum of past magnitudes with exponential decay
        let n = history.len();
        let mut weighted_sum = 0.0;
        for (i, &mag) in history.iter().enumerate() {
            let age = (n - 1 - i) as f64;
            let weight = (-self.decay_rate * age).exp();
            weighted_sum += mag * weight;
        }

        // Net effect: sensitization increases, tolerance decreases
        let modifier = 1.0 + self.sensitization * weighted_sum - self.tolerance * weighted_sum;
        modifier.max(0.1) // Never fully suppress
    }
}

impl Default for NonMarkovianHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // DEFINITION 6.1 TESTS: LEVEL PERTURBATION
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_level_perturbation_magnitude() {
        let perturbation = LevelPerturbation::new(0, vec![3.0, 4.0]);
        assert!((perturbation.magnitude - 5.0).abs() < 1e-10); // 3-4-5 triangle
    }

    #[test]
    fn test_level_perturbation_with_baseline() {
        let perturbation = LevelPerturbation::with_baseline(0, vec![5.0, 7.0], vec![2.0, 3.0]);
        // Deviation = [3.0, 4.0], magnitude = 5.0
        assert!((perturbation.magnitude - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_perturbation_exceeds_threshold() {
        let perturbation = LevelPerturbation::new(0, vec![3.0, 4.0]);
        assert!(perturbation.exceeds_threshold(4.0));
        assert!(!perturbation.exceeds_threshold(6.0));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEFINITION 6.3 TESTS: BUFFERING CAPACITY
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_buffering_capacity() {
        let buffer = BufferingCapacity::new(0, 2.5, BufferingMechanism::Homeostatic);
        assert_eq!(buffer.level, 0);
        assert!((buffer.capacity - 2.5).abs() < 1e-10);
        assert!(buffer.is_significant(2.0));
        assert!(!buffer.is_significant(3.0));
    }

    #[test]
    fn test_buffering_non_negative() {
        let buffer = BufferingCapacity::new(0, -1.0, BufferingMechanism::Absorption);
        assert!(buffer.capacity >= 0.0); // Should be clamped to 0
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEFINITION 6.4 TESTS: PROPAGATION PROBABILITY
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_sigmoidal_propagation_bounds() {
        let func = SigmoidalPropagation::new(0, 1.0);

        // Test various parameters
        let params_low = PropagationParams::new(0.1);
        let params_high = PropagationParams::new(10.0);

        let p_low = func.evaluate(&params_low);
        let p_high = func.evaluate(&params_high);

        // Must be in (0, 1)
        assert!(p_low > 0.0 && p_low < 1.0);
        assert!(p_high > 0.0 && p_high < 1.0);

        // Higher magnitude should give higher probability (P1)
        assert!(p_high > p_low);
    }

    #[test]
    fn test_exponential_propagation() {
        let func = ExponentialPropagation::new(0);

        let params = PropagationParams::new(1.0)
            .with_centrality(0.5)
            .with_buffering(1.0)
            .with_duration(1.0);

        let p = func.evaluate(&params);
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_propagation_properties_p1_to_p4() {
        let func = SigmoidalPropagation::new(0, 1.0);
        let baseline = PropagationParams::new(1.0)
            .with_centrality(0.5)
            .with_buffering(1.0)
            .with_duration(1.0);

        let verification = func.verify_properties(&baseline, 0.1);

        assert!(verification.p1_magnitude_monotone);
        assert!(verification.p2_centrality_monotone);
        assert!(verification.p3_buffering_monotone);
        assert!(verification.p4_duration_monotone);
        assert!(verification.all_satisfied);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AXIOM 5 TESTS: EMERGENCE
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_harm_probability_markov() {
        let mut framework = EmergenceFramework::new(HarmLevel::PV_ORGANISM);

        // Add 5 level transitions (0→1, 1→2, 2→3, 3→4, 4→5)
        for i in 0..5 {
            framework.add_propagation(SigmoidalPropagation::new(i, 1.0));
        }

        // Create parameters for each level
        let params: Vec<PropagationParams> = (0..5)
            .map(|_| {
                PropagationParams::new(1.0)
                    .with_centrality(0.5)
                    .with_buffering(0.5)
                    .with_duration(1.0)
            })
            .collect();

        let result = framework.harm_probability_markov(&params);

        assert!(result.error.is_none());
        assert!(result.probability > 0.0);
        assert!(result.probability < 1.0);
        assert_eq!(result.level_probabilities.len(), 5);
    }

    #[test]
    fn test_harm_probability_uniform_wolfram_validated() {
        let framework = EmergenceFramework::new(HarmLevel::PV_ORGANISM);

        // Wolfram: 0.9^8 = 0.43046721
        let prob = framework.harm_probability_uniform(0.9, 9); // 8 transitions
        assert!((prob - 0.430_467_21).abs() < 0.000_001);
    }

    #[test]
    fn test_axiom5_verification() {
        let mut framework = EmergenceFramework::new(HarmLevel::PV_ORGANISM);

        for i in 0..5 {
            framework.add_propagation(SigmoidalPropagation::new(i, 1.0));
        }

        let test_params = PropagationParams::new(1.0)
            .with_centrality(0.5)
            .with_buffering(1.0)
            .with_duration(1.0);

        let verification = Axiom5Verification::verify(&framework, &test_params);

        assert!(verification.condition_1_level_ordering);
        assert!(verification.condition_2_open_interval);
        assert!(verification.condition_3_monotonicity);
        assert!(verification.axiom_satisfied);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NON-MARKOVIAN EXTENSION TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_non_markovian_history() {
        let mut history = NonMarkovianHistory::new();
        history.sensitization = 0.1;
        history.tolerance = 0.0;

        // No history -> modifier = 1.0
        assert!((history.history_modifier(0) - 1.0).abs() < 1e-10);

        // Add history
        history.record(0, 1.0);
        history.record(0, 1.0);

        // With sensitization, modifier should be > 1.0
        let modifier = history.history_modifier(0);
        assert!(modifier > 1.0);
    }

    #[test]
    fn test_non_markovian_tolerance() {
        let mut history = NonMarkovianHistory::new();
        history.sensitization = 0.0;
        history.tolerance = 0.1;

        history.record(0, 1.0);
        history.record(0, 1.0);

        // With tolerance, modifier should be < 1.0
        let modifier = history.history_modifier(0);
        assert!(modifier < 1.0);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HARM LEVEL TESTS
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_harm_level_constants() {
        assert_eq!(HarmLevel::PV_ORGANISM.level_index, 5);
        assert_eq!(HarmLevel::PV_POPULATION.level_index, 6);
        assert_eq!(HarmLevel::CLOUD_SYSTEM.level_index, 4);
        assert_eq!(HarmLevel::AI_BEHAVIOR.level_index, 3);
    }

    #[test]
    fn test_harm_probability_attenuation() {
        let result = HarmProbabilityResult {
            probability: 0.3,
            level_probabilities: vec![0.9, 0.9, 0.9],
            propagated_to_level: 3,
            is_markov: true,
            error: None,
        };

        assert!((result.attenuation() - 0.7).abs() < 1e-10);
        assert!(!result.is_likely(0.5));
        assert!(result.is_likely(0.2));
    }
}
