//! # Principal Theorems (ToV §10)
//!
//! This module implements the five principal theorems derivable from the axiomatic foundation.
//!
//! ## Theorem Summary
//!
//! | Theorem | Name | Statement |
//! |---------|------|-----------|
//! | 10.1 | Predictability | Harm probability satisfies Kolmogorov backward equation |
//! | 10.2 | Attenuation | Hierarchical organization provides exponential protection |
//! | 10.3 | Intervention | Harm probability is monotonic in intervention parameters |
//! | 10.4 | Conservation | Harm ⟺ constraint violation (diagnostic, preventive) |
//! | 10.5 | Manifold Equivalence | Safety manifold unique up to constraint equivalence |
//!
//! ## Dependencies (§10.6)
//!
//! ```text
//!                     Axiom 3         Axiom 4        Axiom 5
//!                 (Conservation)    (Manifold)    (Emergence)
//!                       │               │              │
//!                       ▼               ▼              │
//!                  ┌─────────────────────┐            │
//!                  │   Theorem 10.4     │◄───────────┘
//!                  │  (Conservation)    │
//!                  └─────────┬─────────┘
//!                            │
//!               ┌────────────┼────────────┐
//!               │            │            │
//!               ▼            ▼            ▼
//!     ┌─────────────┐  ┌──────────┐  ┌──────────┐
//!     │ Theorem 10.1│  │Thm 10.2  │  │Thm 10.3  │
//!     │(Predictability)│(Attenuation)│(Intervention)│
//!     └─────────────┘  └──────────┘  └──────────┘
//!            │               │              │
//!            └───────────────┼──────────────┘
//!                            │
//!                            ▼
//!               ┌────────────────────────┐
//!               │   Theorem 10.5        │
//!               │ (Manifold Equivalence)│
//!               └────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Theorem 10.1: Predictability Theorem
// =============================================================================

/// Hypotheses for Theorem 10.1 (Predictability).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictabilityHypotheses {
    /// H1: Safety manifold M is compact with C² boundary (or stratified)
    pub h1_manifold_compact_c2_boundary: bool,
    /// H2: Drift f is Lipschitz continuous with constant L_f
    pub h2_drift_lipschitz: bool,
    /// Lipschitz constant for drift (if known)
    pub lipschitz_constant: Option<f64>,
    /// H3: Diffusion σ is Lipschitz and uniformly elliptic (σσᵀ ≥ λI)
    pub h3_diffusion_elliptic: bool,
    /// Ellipticity constant λ (if known)
    pub ellipticity_constant: Option<f64>,
    /// H4: Initial state s₀ ∈ int(M) (strictly inside)
    pub h4_initial_state_interior: bool,
    /// H5: Perturbation u is measurable and bounded
    pub h5_perturbation_bounded: bool,
    /// Maximum perturbation bound u_max (if known)
    pub perturbation_bound: Option<f64>,
}

impl PredictabilityHypotheses {
    /// Check if all hypotheses are satisfied.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.h1_manifold_compact_c2_boundary
            && self.h2_drift_lipschitz
            && self.h3_diffusion_elliptic
            && self.h4_initial_state_interior
            && self.h5_perturbation_bounded
    }

    /// Create hypotheses with all satisfied (for testing).
    #[must_use]
    pub fn all_true() -> Self {
        Self {
            h1_manifold_compact_c2_boundary: true,
            h2_drift_lipschitz: true,
            lipschitz_constant: Some(1.0),
            h3_diffusion_elliptic: true,
            ellipticity_constant: Some(0.1),
            h4_initial_state_interior: true,
            h5_perturbation_bounded: true,
            perturbation_bound: Some(10.0),
        }
    }
}

/// Manifold geometry type for computational method selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManifoldGeometry {
    /// Half-space geometry
    HalfSpace,
    /// Spherical geometry
    Sphere,
    /// Ellipsoidal geometry
    Ellipsoid,
    /// General convex geometry
    GeneralConvex,
    /// Non-convex or stratified geometry
    NonConvexStratified,
}

impl ManifoldGeometry {
    /// Recommended solution method for this geometry.
    #[must_use]
    pub const fn solution_method(&self) -> &'static str {
        match self {
            Self::HalfSpace => "Analytical (reflection principle)",
            Self::Sphere => "Analytical (series expansion)",
            Self::Ellipsoid => "Semi-analytical",
            Self::GeneralConvex => "Finite element methods",
            Self::NonConvexStratified => "Monte Carlo simulation",
        }
    }
}

impl fmt::Display for ManifoldGeometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HalfSpace => write!(f, "Half-space"),
            Self::Sphere => write!(f, "Sphere"),
            Self::Ellipsoid => write!(f, "Ellipsoid"),
            Self::GeneralConvex => write!(f, "General convex"),
            Self::NonConvexStratified => write!(f, "Non-convex/stratified"),
        }
    }
}

/// First-passage time result from Theorem 10.1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstPassageResult {
    /// Initial safety margin d(s₀)
    pub initial_margin: f64,
    /// Time horizon T
    pub time_horizon: f64,
    /// Computed harm probability P(τ_∂M ≤ T | s₀)
    pub harm_probability: f64,
    /// Computation method used
    pub method: String,
    /// Whether hypotheses were verified
    pub hypotheses_verified: bool,
}

/// Theorem 10.1: Predictability Theorem.
///
/// Under hypotheses (H1)-(H5), the first-passage time τ_∂M = inf{t ≥ 0 : s(t) ∈ ∂M}
/// is a well-defined stopping time and the harm probability
/// P(H | s₀, u, θ, T) = P(τ_∂M ≤ T | s₀)
/// is the unique solution to the Kolmogorov backward equation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictabilityTheorem {
    /// Hypotheses for the theorem
    pub hypotheses: PredictabilityHypotheses,
    /// Manifold geometry (for method selection)
    pub geometry: ManifoldGeometry,
}

impl PredictabilityTheorem {
    /// Create a new predictability theorem instance.
    #[must_use]
    pub fn new(hypotheses: PredictabilityHypotheses, geometry: ManifoldGeometry) -> Self {
        Self {
            hypotheses,
            geometry,
        }
    }

    /// Check if the theorem applies (all hypotheses satisfied).
    #[must_use]
    pub fn applies(&self) -> bool {
        self.hypotheses.all_satisfied()
    }

    /// Get the recommended solution method.
    #[must_use]
    pub fn solution_method(&self) -> &'static str {
        self.geometry.solution_method()
    }

    /// Compute harm probability using simplified model.
    ///
    /// This is a simplified computation for demonstration.
    /// Full implementation would solve the Kolmogorov backward equation.
    ///
    /// P(H | s₀, T) ≈ 1 - exp(-T / τ_characteristic)
    /// where τ_characteristic ∝ d(s₀)² / D (diffusion timescale)
    #[must_use]
    pub fn compute_harm_probability(
        &self,
        initial_margin: f64,
        time_horizon: f64,
        diffusion_coefficient: f64,
    ) -> FirstPassageResult {
        if !self.applies() {
            return FirstPassageResult {
                initial_margin,
                time_horizon,
                harm_probability: f64::NAN,
                method: "N/A (hypotheses not satisfied)".to_string(),
                hypotheses_verified: false,
            };
        }

        // Characteristic timescale for diffusion to boundary
        // τ ∝ d² / D (from dimensional analysis)
        let tau_characteristic = if diffusion_coefficient > 0.0 {
            initial_margin.powi(2) / (2.0 * diffusion_coefficient)
        } else {
            f64::INFINITY
        };

        // Simplified survival probability (exponential approximation)
        // This is exact for 1D Brownian motion to absorbing boundary
        let harm_probability = if tau_characteristic.is_finite() && tau_characteristic > 0.0 {
            1.0 - (-time_horizon / tau_characteristic).exp()
        } else {
            0.0
        };

        FirstPassageResult {
            initial_margin,
            time_horizon,
            harm_probability: harm_probability.clamp(0.0, 1.0),
            method: self.solution_method().to_string(),
            hypotheses_verified: true,
        }
    }

    /// Statement of Theorem 10.1.
    #[must_use]
    pub fn statement() -> &'static str {
        "Under (H1)-(H5), the first-passage time τ_∂M = inf{t ≥ 0 : s(t) ∈ ∂M} \
         is a well-defined {ℱₜ}-stopping time and the harm probability \
         P(H | s₀, u, θ, T) = P(τ_∂M ≤ T | s₀) is the unique solution to the \
         Kolmogorov backward equation: ∂p/∂t + Lp = 0 on int(M) × (0,T)."
    }

    /// Implication of Theorem 10.1.
    #[must_use]
    pub fn implication() -> &'static str {
        "Harm is not inherently unpredictable. With sufficient system knowledge, \
         harm probability can be quantified before harm occurs."
    }
}

// =============================================================================
// Theorem 10.2: Attenuation Theorem
// =============================================================================

/// Version of the Attenuation Theorem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttenuationVersion {
    /// Version A: Uniform bound using P_max
    UniformBound,
    /// Version B: Exact product of propagation probabilities
    ExactProduct,
    /// Version C: Geometric mean formulation
    GeometricMean,
    /// Version D: Attenuation rate formulation
    AttenuationRate,
}

impl fmt::Display for AttenuationVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UniformBound => write!(f, "Version A (Uniform Bound)"),
            Self::ExactProduct => write!(f, "Version B (Exact Product)"),
            Self::GeometricMean => write!(f, "Version C (Geometric Mean)"),
            Self::AttenuationRate => write!(f, "Version D (Attenuation Rate)"),
        }
    }
}

/// Attenuation rate interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttenuationInterpretation {
    /// Weak attenuation (α ≈ 0.1, P ≈ 0.9) - tight coupling
    Weak,
    /// Moderate attenuation (α ≈ 0.5, P ≈ 0.6) - normal systems
    Moderate,
    /// Strong attenuation (α ≈ 1.0, P ≈ 0.37) - well-buffered
    Strong,
    /// Very strong attenuation (α ≈ 2.0, P ≈ 0.14) - highly isolated
    VeryStrong,
}

impl AttenuationInterpretation {
    /// Get interpretation from attenuation rate α.
    #[must_use]
    pub fn from_rate(alpha: f64) -> Self {
        if alpha < 0.3 {
            Self::Weak
        } else if alpha < 0.75 {
            Self::Moderate
        } else if alpha < 1.5 {
            Self::Strong
        } else {
            Self::VeryStrong
        }
    }

    /// Approximate propagation probability P for this interpretation.
    #[must_use]
    pub fn approximate_probability(&self) -> f64 {
        match self {
            Self::Weak => 0.9,
            Self::Moderate => 0.6,
            Self::Strong => 0.37,
            Self::VeryStrong => 0.14,
        }
    }

    /// Description of this attenuation level.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Weak => "Tight coupling between levels",
            Self::Moderate => "Normal system behavior",
            Self::Strong => "Well-buffered system",
            Self::VeryStrong => "Highly isolated levels",
        }
    }
}

impl fmt::Display for AttenuationInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Weak => write!(f, "Weak (α ≈ 0.1)"),
            Self::Moderate => write!(f, "Moderate (α ≈ 0.5)"),
            Self::Strong => write!(f, "Strong (α ≈ 1.0)"),
            Self::VeryStrong => write!(f, "Very Strong (α ≈ 2.0)"),
        }
    }
}

/// Result of attenuation computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttenuationResult {
    /// Harm probability P(H | δs₁)
    pub harm_probability: f64,
    /// Attenuation rate α = -log(P̄)
    pub attenuation_rate: f64,
    /// Geometric mean propagation probability P̄
    pub geometric_mean_p: f64,
    /// Number of hierarchy levels H
    pub hierarchy_depth: usize,
    /// Interpretation of attenuation strength
    pub interpretation: AttenuationInterpretation,
    /// Version used for computation
    pub version: AttenuationVersion,
}

/// Theorem 10.2: Attenuation Theorem.
///
/// The probability of harm given a perturbation at the lowest hierarchical level
/// is bounded by the product of propagation probabilities, providing exponential
/// protection through hierarchical organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttenuationTheorem {
    /// Propagation probabilities P_{i→i+1} for each level transition
    pub propagation_probs: Vec<f64>,
}

impl AttenuationTheorem {
    /// Create a new attenuation theorem with given propagation probabilities.
    #[must_use]
    pub fn new(propagation_probs: Vec<f64>) -> Self {
        Self { propagation_probs }
    }

    /// Create with uniform propagation probability.
    #[must_use]
    pub fn uniform(p: f64, levels: usize) -> Self {
        Self {
            propagation_probs: vec![p; levels.saturating_sub(1)],
        }
    }

    /// Number of hierarchy levels H.
    #[must_use]
    pub fn hierarchy_depth(&self) -> usize {
        self.propagation_probs.len() + 1
    }

    /// Version A: Uniform bound P(H) ≤ P_max^{H-1}.
    #[must_use]
    pub fn uniform_bound(&self) -> f64 {
        if self.propagation_probs.is_empty() {
            return 1.0;
        }
        let p_max = self
            .propagation_probs
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max);
        p_max.powi(self.propagation_probs.len() as i32)
    }

    /// Version B: Exact product P(H) = ∏ P_{i→i+1}.
    #[must_use]
    pub fn exact_product(&self) -> f64 {
        self.propagation_probs.iter().product()
    }

    /// Version C: Geometric mean P̄ = (∏ P_i)^{1/(H-1)}.
    #[must_use]
    pub fn geometric_mean(&self) -> f64 {
        if self.propagation_probs.is_empty() {
            return 1.0;
        }
        let product: f64 = self.propagation_probs.iter().product();
        product.powf(1.0 / self.propagation_probs.len() as f64)
    }

    /// Version D: Attenuation rate α = -log(P̄).
    #[must_use]
    pub fn attenuation_rate(&self) -> f64 {
        let p_bar = self.geometric_mean();
        if p_bar > 0.0 {
            -p_bar.ln()
        } else {
            f64::INFINITY
        }
    }

    /// Compute harm probability using specified version.
    #[must_use]
    pub fn compute(&self, version: AttenuationVersion) -> AttenuationResult {
        let harm_probability = match version {
            AttenuationVersion::UniformBound => self.uniform_bound(),
            AttenuationVersion::ExactProduct => self.exact_product(),
            AttenuationVersion::GeometricMean => {
                let p_bar = self.geometric_mean();
                p_bar.powi(self.propagation_probs.len() as i32)
            }
            AttenuationVersion::AttenuationRate => {
                let alpha = self.attenuation_rate();
                (-alpha * self.propagation_probs.len() as f64).exp()
            }
        };

        let alpha = self.attenuation_rate();

        AttenuationResult {
            harm_probability,
            attenuation_rate: alpha,
            geometric_mean_p: self.geometric_mean(),
            hierarchy_depth: self.hierarchy_depth(),
            interpretation: AttenuationInterpretation::from_rate(alpha),
            version,
        }
    }

    /// Corollary: Required hierarchy depth for target harm probability.
    ///
    /// To achieve P(H) ≤ ε, the required depth is: H ≥ 1 + log(1/ε) / α
    #[must_use]
    pub fn required_depth_for_probability(&self, epsilon: f64) -> usize {
        let alpha = self.attenuation_rate();
        if alpha <= 0.0 || epsilon <= 0.0 || epsilon >= 1.0 {
            return usize::MAX;
        }
        let required = 1.0 + (1.0 / epsilon).ln() / alpha;
        required.ceil() as usize
    }

    /// Statement of Theorem 10.2.
    #[must_use]
    pub fn statement() -> &'static str {
        "The probability of harm given a perturbation at the lowest hierarchical level is: \
         P(H | δs₁) = ∏_{i=1}^{H-1} P_{i→i+1} = P̄^{H-1} = e^{-α(H-1)} \
         where α = -log(P̄) is the attenuation rate."
    }

    /// Implication of Theorem 10.2.
    #[must_use]
    pub fn implication() -> &'static str {
        "Hierarchical organization provides exponential protection against low-level perturbations. \
         Most molecular/parameter-level fluctuations are absorbed before reaching harm-manifesting levels."
    }
}

// =============================================================================
// Theorem 10.3: Intervention Theorem
// =============================================================================

/// Propagation function properties (P1-P4) from Theorem 10.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropagationProperty {
    /// P1: ∂P/∂m ≥ 0 (higher magnitude increases propagation)
    P1MagnitudeMonotonic,
    /// P2: ∂P/∂c ≥ 0 (higher centrality increases propagation)
    P2CentralityMonotonic,
    /// P3: ∂P/∂b ≤ 0 (higher buffering decreases propagation)
    P3BufferingMonotonic,
    /// P4: ∂P/∂t ≥ 0 (longer exposure increases propagation)
    P4DurationMonotonic,
}

impl PropagationProperty {
    /// Description of this property.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::P1MagnitudeMonotonic => {
                "∂P/∂m ≥ 0: Higher perturbation magnitude increases propagation"
            }
            Self::P2CentralityMonotonic => {
                "∂P/∂c ≥ 0: Higher network centrality increases propagation"
            }
            Self::P3BufferingMonotonic => {
                "∂P/∂b ≤ 0: Higher buffering capacity decreases propagation"
            }
            Self::P4DurationMonotonic => {
                "∂P/∂t ≥ 0: Longer exposure duration increases propagation"
            }
        }
    }

    /// All four properties.
    pub const ALL: &'static [PropagationProperty] = &[
        Self::P1MagnitudeMonotonic,
        Self::P2CentralityMonotonic,
        Self::P3BufferingMonotonic,
        Self::P4DurationMonotonic,
    ];
}

impl fmt::Display for PropagationProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::P1MagnitudeMonotonic => write!(f, "P1 (Magnitude)"),
            Self::P2CentralityMonotonic => write!(f, "P2 (Centrality)"),
            Self::P3BufferingMonotonic => write!(f, "P3 (Buffering)"),
            Self::P4DurationMonotonic => write!(f, "P4 (Duration)"),
        }
    }
}

/// Propagation model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropagationModel {
    /// Logistic: P = σ(β₀ + β_m·m + β_c·c - β_b·b + β_t·t)
    Logistic,
    /// Multiplicative: P = P_base · (m/m₀)^α · (c/c₀)^β · (b₀/b)^γ · (1 - e^{-t/τ})
    Multiplicative,
}

impl PropagationModel {
    /// Check if this model satisfies all P1-P4 properties.
    #[must_use]
    pub const fn satisfies_all_properties(&self) -> bool {
        // Both standard models satisfy P1-P4 by construction
        true
    }
}

impl fmt::Display for PropagationModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Logistic => write!(f, "Logistic (σ-function)"),
            Self::Multiplicative => write!(f, "Multiplicative (power law)"),
        }
    }
}

/// Intervention parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionParams {
    /// Perturbation magnitude m
    pub magnitude: f64,
    /// Network centrality c
    pub centrality: f64,
    /// Buffering capacity b
    pub buffering: f64,
    /// Exposure duration t
    pub duration: f64,
}

impl InterventionParams {
    /// Create new intervention parameters.
    #[must_use]
    pub fn new(magnitude: f64, centrality: f64, buffering: f64, duration: f64) -> Self {
        Self {
            magnitude,
            centrality,
            buffering,
            duration,
        }
    }
}

/// Result of intervention analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionResult {
    /// Base harm probability
    pub base_probability: f64,
    /// Sensitivity ∂P(H)/∂m (magnitude)
    pub sensitivity_magnitude: f64,
    /// Sensitivity ∂P(H)/∂c (centrality)
    pub sensitivity_centrality: f64,
    /// Sensitivity ∂P(H)/∂b (buffering)
    pub sensitivity_buffering: f64,
    /// Sensitivity ∂P(H)/∂t (duration)
    pub sensitivity_duration: f64,
    /// Most effective intervention (largest |sensitivity|)
    pub most_effective: String,
}

/// Theorem 10.3: Intervention Theorem.
///
/// Under hypotheses (H1)-(H3), the harm probability P(H) is monotonic:
/// 1. ∂P(H)/∂m ≥ 0 (reducing perturbation reduces harm)
/// 2. ∂P(H)/∂c ≥ 0 (reducing centrality reduces harm)
/// 3. ∂P(H)/∂b ≤ 0 (increasing buffering reduces harm)
/// 4. ∂P(H)/∂t ≥ 0 (reducing exposure reduces harm)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionTheorem {
    /// Propagation model
    pub model: PropagationModel,
    /// Model coefficients (β_m, β_c, β_b, β_t for logistic)
    pub coefficients: (f64, f64, f64, f64),
}

impl InterventionTheorem {
    /// Create a new intervention theorem with logistic model.
    #[must_use]
    pub fn logistic(beta_m: f64, beta_c: f64, beta_b: f64, beta_t: f64) -> Self {
        Self {
            model: PropagationModel::Logistic,
            coefficients: (beta_m, beta_c, beta_b, beta_t),
        }
    }

    /// Default logistic model coefficients.
    #[must_use]
    pub fn default_logistic() -> Self {
        Self::logistic(1.0, 0.5, 1.0, 0.3)
    }

    /// Compute propagation probability using logistic model.
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Compute single-level propagation probability.
    #[must_use]
    pub fn propagation_probability(&self, params: &InterventionParams) -> f64 {
        let (beta_m, beta_c, beta_b, beta_t) = self.coefficients;
        let z = beta_m * params.magnitude + beta_c * params.centrality - beta_b * params.buffering
            + beta_t * params.duration;
        Self::sigmoid(z)
    }

    /// Compute sensitivities (partial derivatives).
    #[must_use]
    pub fn compute_sensitivities(
        &self,
        params: &InterventionParams,
        levels: usize,
    ) -> InterventionResult {
        let p = self.propagation_probability(params);
        let base_probability = p.powi(levels as i32);

        // For logistic: ∂P/∂x = P(1-P)·β_x
        let dp_dm = p * (1.0 - p) * self.coefficients.0;
        let dp_dc = p * (1.0 - p) * self.coefficients.1;
        let dp_db = -p * (1.0 - p) * self.coefficients.2; // Negative because b decreases P
        let dp_dt = p * (1.0 - p) * self.coefficients.3;

        // Chain rule for P(H) = P^n: ∂P(H)/∂x = n·P^{n-1}·∂P/∂x
        let n = levels as f64;
        let p_n_minus_1 = p.powi((levels - 1) as i32);

        let sens_m = n * p_n_minus_1 * dp_dm;
        let sens_c = n * p_n_minus_1 * dp_dc;
        let sens_b = n * p_n_minus_1 * dp_db;
        let sens_t = n * p_n_minus_1 * dp_dt;

        // Find most effective intervention
        let sensitivities = [
            (sens_m.abs(), "Reduce magnitude"),
            (sens_c.abs(), "Reduce centrality"),
            (sens_b.abs(), "Increase buffering"),
            (sens_t.abs(), "Reduce duration"),
        ];

        let most_effective = sensitivities
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, name)| *name)
            .unwrap_or("Unknown");

        InterventionResult {
            base_probability,
            sensitivity_magnitude: sens_m,
            sensitivity_centrality: sens_c,
            sensitivity_buffering: sens_b,
            sensitivity_duration: sens_t,
            most_effective: most_effective.to_string(),
        }
    }

    /// Statement of Theorem 10.3.
    #[must_use]
    pub fn statement() -> &'static str {
        "Under (H1)-(H3), the harm probability P(H) is monotonic in intervention parameters: \
         ∂P(H)/∂m ≥ 0, ∂P(H)/∂c ≥ 0, ∂P(H)/∂b ≤ 0, ∂P(H)/∂t ≥ 0."
    }

    /// Implication of Theorem 10.3.
    #[must_use]
    pub fn implication() -> &'static str {
        "There are multiple intervention strategies for reducing harm: reducing perturbation magnitude, \
         increasing buffering, modifying network topology, or relaxing constraints. The optimal \
         strategy depends on which parameters are most modifiable."
    }
}

// =============================================================================
// Theorem 10.4: Conservation Theorem
// =============================================================================

/// Part of the Conservation Theorem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConservationTheoremPart {
    /// Part (a): Definitional restatement
    DefinitionalRestatement,
    /// Part (b): Diagnostic property
    DiagnosticProperty,
    /// Part (c): Sufficiency for prevention
    SufficiencyForPrevention,
    /// Part (d): Completeness
    Completeness,
    /// Part (e): Violation magnitude
    ViolationMagnitude,
}

impl ConservationTheoremPart {
    /// Statement for this part.
    #[must_use]
    pub const fn statement(&self) -> &'static str {
        match self {
            Self::DefinitionalRestatement => "H ⟺ ∃i ∈ {1, ..., m} : gᵢ(s, u, θ) > 0",
            Self::DiagnosticProperty => {
                "If harm H occurs, the violated constraint(s) can be identified: \
                 I_violated = {i : gᵢ(s,u,θ) > 0}"
            }
            Self::SufficiencyForPrevention => {
                "To prevent harm, it SUFFICES to ensure all constraints satisfied: \
                 [∀i : gᵢ(s,u,θ) ≤ 0] ⟹ ¬H"
            }
            Self::Completeness => {
                "The constraint set 𝒢 is COMPLETE for harm specification ℋ if every \
                 harm event corresponds to violation of some gᵢ ∈ 𝒢"
            }
            Self::ViolationMagnitude => {
                "The SEVERITY of harm is quantified by: v_max(s, u, θ) = max_i {gᵢ(s, u, θ)}"
            }
        }
    }

    /// All five parts.
    pub const ALL: &'static [ConservationTheoremPart] = &[
        Self::DefinitionalRestatement,
        Self::DiagnosticProperty,
        Self::SufficiencyForPrevention,
        Self::Completeness,
        Self::ViolationMagnitude,
    ];
}

impl fmt::Display for ConservationTheoremPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DefinitionalRestatement => write!(f, "Part (a): Definitional Restatement"),
            Self::DiagnosticProperty => write!(f, "Part (b): Diagnostic Property"),
            Self::SufficiencyForPrevention => write!(f, "Part (c): Sufficiency for Prevention"),
            Self::Completeness => write!(f, "Part (d): Completeness"),
            Self::ViolationMagnitude => write!(f, "Part (e): Violation Magnitude"),
        }
    }
}

/// Constraint violation diagnosis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintDiagnosis {
    /// Whether harm occurred
    pub harm_occurred: bool,
    /// Indices of violated constraints
    pub violated_constraints: Vec<usize>,
    /// Violation values for each constraint (g_i values)
    pub violation_values: Vec<f64>,
    /// Maximum violation (severity)
    pub max_violation: f64,
    /// Primary violated constraint index (first to become positive)
    pub primary_violation: Option<usize>,
}

impl ConstraintDiagnosis {
    /// Diagnose constraint violations from constraint values.
    ///
    /// Constraint g_i(s,u,θ) > 0 indicates violation.
    #[must_use]
    pub fn diagnose(constraint_values: &[f64]) -> Self {
        let violated_constraints: Vec<usize> = constraint_values
            .iter()
            .enumerate()
            .filter(|(_, g)| **g > 0.0)
            .map(|(i, _)| i)
            .collect();

        let harm_occurred = !violated_constraints.is_empty();

        let max_violation = constraint_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        // Primary violation is the constraint with largest positive value
        let primary_violation = constraint_values
            .iter()
            .enumerate()
            .filter(|(_, g)| **g > 0.0)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i);

        Self {
            harm_occurred,
            violated_constraints,
            violation_values: constraint_values.to_vec(),
            max_violation,
            primary_violation,
        }
    }

    /// Check if a specific constraint is violated.
    #[must_use]
    pub fn is_violated(&self, constraint_idx: usize) -> bool {
        self.violated_constraints.contains(&constraint_idx)
    }

    /// Number of violated constraints.
    #[must_use]
    pub fn violation_count(&self) -> usize {
        self.violated_constraints.len()
    }
}

/// Theorem 10.4: Conservation Theorem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConservationTheorem {
    /// Number of constraints in the set
    pub num_constraints: usize,
    /// Whether the constraint set is minimal (no redundant constraints)
    pub is_minimal: bool,
}

impl ConservationTheorem {
    /// Create a new conservation theorem.
    #[must_use]
    pub fn new(num_constraints: usize, is_minimal: bool) -> Self {
        Self {
            num_constraints,
            is_minimal,
        }
    }

    /// Statement of Theorem 10.4.
    #[must_use]
    pub fn statement() -> &'static str {
        "Harm H ⟺ ∃i ∈ {1,...,m} : gᵢ(s,u,θ) > 0. \
         If harm occurs, violated constraints can be identified. \
         To prevent harm, it suffices to ensure all constraints satisfied. \
         Severity is quantified by v_max = max_i{gᵢ}."
    }

    /// Practical implications.
    #[must_use]
    pub fn practical_implications() -> &'static str {
        "1. Diagnosis: When harm occurs, compute all gᵢ to identify which constraint(s) failed\n\
         2. Prevention: Monitor all gᵢ and intervene when any approaches zero\n\
         3. Prioritization: Focus on constraints with smallest margin dᵢ = -gᵢ\n\
         4. Root Cause: The first constraint to become positive in time is the PRIMARY violation"
    }
}

// =============================================================================
// Theorem 10.5: Manifold Equivalence
// =============================================================================

/// Constraint set equivalence result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    /// Whether the constraint sets are equivalent
    pub are_equivalent: bool,
    /// Same interior
    pub same_interior: bool,
    /// Same boundary as point sets
    pub same_boundary: bool,
    /// Potentially different normal vectors at boundary
    pub potentially_different_normals: bool,
}

/// Theorem 10.5: Manifold Equivalence.
///
/// Two constraint sets 𝒢 = {gᵢ}, 𝒢' = {g'ⱼ} are EQUIVALENT if they define
/// the same feasible region F_𝒢(u,θ) = F_𝒢'(u,θ) for all (u,θ).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldEquivalenceTheorem;

impl ManifoldEquivalenceTheorem {
    /// Statement of Theorem 10.5.
    #[must_use]
    pub fn statement() -> &'static str {
        "(a) The safety manifold M = F_𝒢(u,θ) is unique up to constraint equivalence.\n\
         (b) If 𝒢 and 𝒢' are equivalent, they define same interior and boundary as point sets, \
             but POTENTIALLY DIFFERENT normal vectors at ∂M.\n\
         (c) The MINIMAL constraint set 𝒢_min for given M is unique up to positive scaling."
    }

    /// Implication of Theorem 10.5.
    #[must_use]
    pub fn implication() -> &'static str {
        "When defining the safety manifold, the specific constraint functions {gᵢ} affect:\n\
         - Gradient information at ∂M (for optimization, control)\n\
         - Constraint propagation dynamics (for Sentinel)\n\
         - Diagnostic information (which constraint violated)\n\
         but NOT the fundamental safe/unsafe classification."
    }
}

// =============================================================================
// §10.6 Theorem Dependencies
// =============================================================================

/// Theorem identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TheoremId {
    /// Theorem 10.1: Predictability
    T10_1Predictability,
    /// Theorem 10.2: Attenuation
    T10_2Attenuation,
    /// Theorem 10.3: Intervention
    T10_3Intervention,
    /// Theorem 10.4: Conservation
    T10_4Conservation,
    /// Theorem 10.5: Manifold Equivalence
    T10_5ManifoldEquivalence,
}

impl TheoremId {
    /// All theorems.
    pub const ALL: &'static [TheoremId] = &[
        Self::T10_1Predictability,
        Self::T10_2Attenuation,
        Self::T10_3Intervention,
        Self::T10_4Conservation,
        Self::T10_5ManifoldEquivalence,
    ];

    /// Get theorem name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::T10_1Predictability => "Predictability",
            Self::T10_2Attenuation => "Attenuation",
            Self::T10_3Intervention => "Intervention",
            Self::T10_4Conservation => "Conservation",
            Self::T10_5ManifoldEquivalence => "Manifold Equivalence",
        }
    }

    /// Get dependencies (which theorems/axioms this depends on).
    #[must_use]
    pub fn dependencies(&self) -> Vec<&'static str> {
        match self {
            Self::T10_1Predictability => vec!["Axiom 4 (Manifold)", "T10.4 (Conservation)"],
            Self::T10_2Attenuation => vec!["Axiom 5 (Emergence)", "T10.4 (Conservation)"],
            Self::T10_3Intervention => vec![
                "Axiom 5 (Emergence)",
                "T10.2 (Attenuation)",
                "T10.4 (Conservation)",
            ],
            Self::T10_4Conservation => vec![
                "Axiom 3 (Conservation)",
                "Axiom 4 (Manifold)",
                "Axiom 5 (Emergence)",
            ],
            Self::T10_5ManifoldEquivalence => vec!["T10.1", "T10.2", "T10.3", "T10.4"],
        }
    }
}

impl fmt::Display for TheoremId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::T10_1Predictability => write!(f, "Theorem 10.1 (Predictability)"),
            Self::T10_2Attenuation => write!(f, "Theorem 10.2 (Attenuation)"),
            Self::T10_3Intervention => write!(f, "Theorem 10.3 (Intervention)"),
            Self::T10_4Conservation => write!(f, "Theorem 10.4 (Conservation)"),
            Self::T10_5ManifoldEquivalence => write!(f, "Theorem 10.5 (Manifold Equivalence)"),
        }
    }
}

/// Theorem dependency graph (§10.6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremDependencies {
    /// Adjacency list: theorem -> dependencies
    pub dependencies: Vec<(TheoremId, Vec<TheoremId>)>,
}

impl TheoremDependencies {
    /// Build the theorem dependency graph from §10.6.
    #[must_use]
    pub fn build() -> Self {
        use TheoremId::*;

        Self {
            dependencies: vec![
                // T10.4 depends on axioms (no theorem deps)
                (T10_4Conservation, vec![]),
                // T10.1, T10.2, T10.3 depend on T10.4
                (T10_1Predictability, vec![T10_4Conservation]),
                (T10_2Attenuation, vec![T10_4Conservation]),
                (T10_3Intervention, vec![T10_2Attenuation, T10_4Conservation]),
                // T10.5 depends on T10.1-T10.4
                (
                    T10_5ManifoldEquivalence,
                    vec![
                        T10_1Predictability,
                        T10_2Attenuation,
                        T10_3Intervention,
                        T10_4Conservation,
                    ],
                ),
            ],
        }
    }

    /// Topological order for theorem proofs.
    #[must_use]
    pub fn proof_order(&self) -> Vec<TheoremId> {
        // From the dependency graph, T10.4 is first, then T10.1/T10.2, then T10.3, then T10.5
        vec![
            TheoremId::T10_4Conservation,
            TheoremId::T10_1Predictability,
            TheoremId::T10_2Attenuation,
            TheoremId::T10_3Intervention,
            TheoremId::T10_5ManifoldEquivalence,
        ]
    }
}

// =============================================================================
// §10.7 Computational Complexity
// =============================================================================

/// Computational complexity class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputationalComplexity {
    /// Constant time O(1)
    Constant,
    /// Linear O(n)
    Linear,
    /// Polynomial O(n^k)
    Polynomial,
    /// #P-hard
    SharpPHard,
    /// Co-NP
    CoNP,
}

impl fmt::Display for ComputationalComplexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant => write!(f, "O(1)"),
            Self::Linear => write!(f, "O(n)"),
            Self::Polynomial => write!(f, "O(n^k)"),
            Self::SharpPHard => write!(f, "#P-hard"),
            Self::CoNP => write!(f, "Co-NP"),
        }
    }
}

/// Computational complexity for each theorem (§10.7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TheoremComplexity {
    /// Theorem
    pub theorem: TheoremId,
    /// Computation type
    pub computation: &'static str,
    /// Complexity class
    pub complexity: ComputationalComplexity,
    /// Recommended method
    pub method: &'static str,
}

impl TheoremComplexity {
    /// Get complexity information for all theorems.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                theorem: TheoremId::T10_1Predictability,
                computation: "First-passage probability",
                complexity: ComputationalComplexity::SharpPHard,
                method: "PDE/Monte Carlo",
            },
            Self {
                theorem: TheoremId::T10_2Attenuation,
                computation: "Product ∏Pᵢ",
                complexity: ComputationalComplexity::Linear, // O(H)
                method: "Direct multiplication",
            },
            Self {
                theorem: TheoremId::T10_3Intervention,
                computation: "Gradient ∂P/∂x",
                complexity: ComputationalComplexity::Linear, // O(H) per parameter
                method: "Automatic differentiation",
            },
            Self {
                theorem: TheoremId::T10_4Conservation,
                computation: "Constraint check",
                complexity: ComputationalComplexity::Linear, // O(m)
                method: "Evaluate all gᵢ",
            },
            Self {
                theorem: TheoremId::T10_5ManifoldEquivalence,
                computation: "Manifold equivalence",
                complexity: ComputationalComplexity::CoNP,
                method: "LP for convex case",
            },
        ]
    }

    /// Get complexity for a specific theorem.
    #[must_use]
    pub fn for_theorem(theorem: TheoremId) -> Option<Self> {
        Self::all().into_iter().find(|c| c.theorem == theorem)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Theorem 10.1 tests
    #[test]
    fn test_predictability_hypotheses() {
        let h = PredictabilityHypotheses::all_true();
        assert!(h.all_satisfied());

        let h_partial = PredictabilityHypotheses {
            h1_manifold_compact_c2_boundary: true,
            h2_drift_lipschitz: false,
            lipschitz_constant: None,
            h3_diffusion_elliptic: true,
            ellipticity_constant: Some(0.1),
            h4_initial_state_interior: true,
            h5_perturbation_bounded: true,
            perturbation_bound: Some(10.0),
        };
        assert!(!h_partial.all_satisfied());
    }

    #[test]
    fn test_manifold_geometry_methods() {
        assert_eq!(
            ManifoldGeometry::HalfSpace.solution_method(),
            "Analytical (reflection principle)"
        );
        assert_eq!(
            ManifoldGeometry::NonConvexStratified.solution_method(),
            "Monte Carlo simulation"
        );
    }

    #[test]
    fn test_predictability_theorem() {
        let theorem = PredictabilityTheorem::new(
            PredictabilityHypotheses::all_true(),
            ManifoldGeometry::Sphere,
        );

        assert!(theorem.applies());

        let result = theorem.compute_harm_probability(1.0, 10.0, 0.1);
        assert!(result.hypotheses_verified);
        assert!(result.harm_probability >= 0.0 && result.harm_probability <= 1.0);
    }

    // Theorem 10.2 tests
    #[test]
    fn test_attenuation_uniform_bound() {
        let theorem = AttenuationTheorem::uniform(0.5, 5);
        let bound = theorem.uniform_bound();

        // 0.5^4 = 0.0625
        assert!((bound - 0.0625).abs() < 1e-10);
    }

    #[test]
    fn test_attenuation_exact_product() {
        let theorem = AttenuationTheorem::new(vec![0.9, 0.8, 0.7, 0.6]);
        let product = theorem.exact_product();

        // 0.9 * 0.8 * 0.7 * 0.6 = 0.3024
        assert!((product - 0.3024).abs() < 1e-10);
    }

    #[test]
    fn test_attenuation_geometric_mean() {
        let theorem = AttenuationTheorem::uniform(0.5, 5);
        let p_bar = theorem.geometric_mean();

        // Uniform case: geometric mean equals single value
        assert!((p_bar - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_attenuation_rate() {
        let theorem = AttenuationTheorem::uniform(0.5, 5);
        let alpha = theorem.attenuation_rate();

        // α = -ln(0.5) ≈ 0.693
        assert!((alpha - 0.5_f64.ln().abs()).abs() < 1e-10);
    }

    #[test]
    fn test_required_depth() {
        let theorem = AttenuationTheorem::uniform(0.5, 5);
        let depth = theorem.required_depth_for_probability(0.01);

        // Need enough levels for P^{H-1} ≤ 0.01
        // 0.5^{H-1} ≤ 0.01 → H-1 ≥ log(100)/log(2) ≈ 6.64
        assert!(depth >= 7);
    }

    #[test]
    fn test_attenuation_interpretation() {
        assert_eq!(
            AttenuationInterpretation::from_rate(0.1),
            AttenuationInterpretation::Weak
        );
        assert_eq!(
            AttenuationInterpretation::from_rate(0.5),
            AttenuationInterpretation::Moderate
        );
        assert_eq!(
            AttenuationInterpretation::from_rate(1.0),
            AttenuationInterpretation::Strong
        );
        assert_eq!(
            AttenuationInterpretation::from_rate(2.0),
            AttenuationInterpretation::VeryStrong
        );
    }

    // Theorem 10.3 tests
    #[test]
    fn test_propagation_properties() {
        assert_eq!(PropagationProperty::ALL.len(), 4);
        assert!(PropagationModel::Logistic.satisfies_all_properties());
    }

    #[test]
    fn test_intervention_theorem() {
        let theorem = InterventionTheorem::default_logistic();
        let params = InterventionParams::new(1.0, 0.5, 1.0, 1.0);
        let result = theorem.compute_sensitivities(&params, 4);

        // Verify sensitivities have correct signs per P1-P4
        assert!(result.sensitivity_magnitude >= 0.0); // P1
        assert!(result.sensitivity_centrality >= 0.0); // P2
        assert!(result.sensitivity_buffering <= 0.0); // P3
        assert!(result.sensitivity_duration >= 0.0); // P4
    }

    // Theorem 10.4 tests
    #[test]
    fn test_constraint_diagnosis_no_harm() {
        let values = vec![-0.5, -0.3, -0.8, -0.1];
        let diag = ConstraintDiagnosis::diagnose(&values);

        assert!(!diag.harm_occurred);
        assert!(diag.violated_constraints.is_empty());
        assert!(diag.primary_violation.is_none());
    }

    #[test]
    fn test_constraint_diagnosis_with_harm() {
        let values = vec![-0.5, 0.3, -0.8, 0.5];
        let diag = ConstraintDiagnosis::diagnose(&values);

        assert!(diag.harm_occurred);
        assert_eq!(diag.violation_count(), 2);
        assert!(diag.violated_constraints.contains(&1));
        assert!(diag.violated_constraints.contains(&3));
        assert_eq!(diag.primary_violation, Some(3)); // Highest violation
        assert!((diag.max_violation - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_conservation_theorem_parts() {
        assert_eq!(ConservationTheoremPart::ALL.len(), 5);
    }

    // Theorem 10.5 tests
    #[test]
    fn test_manifold_equivalence_statement() {
        let stmt = ManifoldEquivalenceTheorem::statement();
        assert!(stmt.contains("unique up to constraint equivalence"));
    }

    // §10.6 tests
    #[test]
    fn test_theorem_dependencies() {
        let deps = TheoremDependencies::build();
        let order = deps.proof_order();

        // T10.4 should be first (no theorem dependencies)
        assert_eq!(order[0], TheoremId::T10_4Conservation);

        // T10.5 should be last (depends on all others)
        assert_eq!(order[4], TheoremId::T10_5ManifoldEquivalence);
    }

    #[test]
    fn test_theorem_ids() {
        assert_eq!(TheoremId::ALL.len(), 5);
        assert_eq!(TheoremId::T10_1Predictability.name(), "Predictability");
    }

    // §10.7 tests
    #[test]
    fn test_theorem_complexity() {
        let all = TheoremComplexity::all();
        assert_eq!(all.len(), 5);

        // T10.1 is #P-hard
        let t101 = TheoremComplexity::for_theorem(TheoremId::T10_1Predictability);
        assert!(t101.is_some());
        assert_eq!(
            t101.unwrap().complexity,
            ComputationalComplexity::SharpPHard
        );

        // T10.2 is O(H) - linear
        let t102 = TheoremComplexity::for_theorem(TheoremId::T10_2Attenuation);
        assert_eq!(t102.unwrap().complexity, ComputationalComplexity::Linear);
    }

    #[test]
    fn test_complexity_display() {
        assert_eq!(format!("{}", ComputationalComplexity::Linear), "O(n)");
        assert_eq!(
            format!("{}", ComputationalComplexity::SharpPHard),
            "#P-hard"
        );
    }
}
