//! # ToV §1: Preliminary Definitions
//!
//! Formal implementation of the primitive concepts from Theory of Vigilance §1.
//! These definitions are domain-independent and apply to any system under vigilance.
//!
//! ## Definitions Implemented
//!
//! | Definition | Name | Description |
//! |------------|------|-------------|
//! | 1.1 | Vigilance System | Tuple 𝒱 = (S, U, 𝒰, ℳ, Θ, H_spec) |
//! | 1.2 | Harm Event | Measurable event H ∈ ℱ |
//! | 1.3 | State Space | Topological space (S, τ) with Borel σ-algebra |
//! | 1.4 | Perturbation | Measurable function u: [0,T] → U |
//! | 1.5 | Observable | Measurable function o: S → O |
//! | 1.5a | Monitoring Apparatus | Tuple ℳ = (O, 𝒪, ρ, D) |
//! | 1.6 | Vigilance Isomorphism | Bijection φ: X ≅ Y |
//! | 1.7 | System Dynamics | ODE, SDE, or discrete-time evolution |
//! | 1.8 | Vigilance Loop | Operational cycle S → O → D → Response |
//!
//! ## Proposition 1.1 (Harm Specification Equivalence)
//!
//! The bijection between harm specifications and constraint sets:
//! - Harm→Constraint: g_H(s,u,θ) = H_spec(s,u,θ) - 0.5
//! - H_spec=0 (safe) → g_H = -0.5 ≤ 0 (satisfied)
//! - H_spec=1 (harm) → g_H = 0.5 > 0 (violated)

use serde::{Deserialize, Serialize};

use crate::manifold::GeometricSafetyManifold;

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.1: VIGILANCE SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.1: Vigilance System (ToV §1)
///
/// A vigilance system is a tuple 𝒱 = (S, U, 𝒰, ℳ, Θ, H_spec) where:
/// - S is a state space (dimension `state_dim`)
/// - U is a perturbation value space (dimension via `perturbation_class.dimension`)
/// - 𝒰 is a perturbation function class (`perturbation_class`)
/// - ℳ is a monitoring apparatus (`manifold`)
/// - Θ is a parameter space (`parameters`)
/// - H_spec is a harm indicator (via `manifold.signed_distance()`)
///
/// The dynamical system 𝒮 = (S, f, σ) is derived, where f is drift and σ is diffusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilanceSystem {
    /// Dimension of state space S (typically 4 for signal space)
    pub state_dim: usize,

    /// Perturbation class 𝒰 ⊆ {u: [0,T] → U | u measurable}
    pub perturbation_class: PerturbationClass,

    /// Parameter space Θ = Θ_known × Θ_uncertain
    pub parameters: ParameterSpace,

    /// Monitoring apparatus ℳ with detection function D
    pub manifold: GeometricSafetyManifold,

    /// Optional system identifier
    pub system_id: Option<String>,
}

impl VigilanceSystem {
    /// Create a new vigilance system with default signal space (4D state, 1D perturbation).
    #[must_use]
    pub fn new_signal_space() -> Self {
        Self {
            state_dim: 4, // PRR, ROR, IC, EBGM
            perturbation_class: PerturbationClass::new(1),
            parameters: ParameterSpace::default(),
            manifold: GeometricSafetyManifold::default(),
            system_id: None,
        }
    }

    /// Create with custom dimensions.
    #[must_use]
    pub fn with_dimensions(state_dim: usize, perturbation_dim: usize) -> Self {
        Self {
            state_dim,
            perturbation_class: PerturbationClass::new(perturbation_dim),
            parameters: ParameterSpace::default(),
            manifold: GeometricSafetyManifold::default(),
            system_id: None,
        }
    }

    /// Set the perturbation class 𝒰.
    #[must_use]
    pub fn with_perturbation_class(mut self, perturbation_class: PerturbationClass) -> Self {
        self.perturbation_class = perturbation_class;
        self
    }

    /// Set the monitoring apparatus (safety manifold).
    #[must_use]
    pub fn with_manifold(mut self, manifold: GeometricSafetyManifold) -> Self {
        self.manifold = manifold;
        self
    }

    /// Set the parameter space.
    #[must_use]
    pub fn with_parameters(mut self, parameters: ParameterSpace) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set system identifier.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.system_id = Some(id.into());
        self
    }

    /// Check if a perturbation is admissible for this system (u ∈ 𝒰).
    #[must_use]
    pub fn is_perturbation_admissible(&self, perturbation: &Perturbation) -> bool {
        self.perturbation_class.is_admissible(perturbation)
    }

    /// Dimension of perturbation space U.
    #[must_use]
    pub fn perturbation_dim(&self) -> usize {
        self.perturbation_class.dimension
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.4: PERTURBATION
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.4: Perturbation (ToV §1)
///
/// A perturbation is a measurable function u: [0,T] → U where:
/// - U is a perturbation value space (measurable space)
/// - T ∈ ℝ≥0 ∪ {∞} is the perturbation duration
///
/// The perturbation represents an external input applied to the system
/// (e.g., drug dose, workload, prompt).
///
/// For discrete representation, we store sampled values at time points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perturbation {
    /// Perturbation identifier
    pub id: String,

    /// Time points t ∈ [0, T]
    pub time_points: Vec<f64>,

    /// Values u(t) ∈ U at each time point (flattened if multi-dimensional)
    pub values: Vec<Vec<f64>>,

    /// Dimension of perturbation space U
    pub dimension: usize,

    /// Duration T (None for infinite/ongoing)
    pub duration: Option<f64>,

    /// Type of perturbation
    pub perturbation_type: PerturbationType,
}

/// Types of perturbation functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerturbationType {
    /// Constant: u(t) = u₀ for all t
    Constant,
    /// Step: u(t) = u₀ for t < t₁, u₁ for t ≥ t₁
    Step,
    /// Impulse: u(t) = δ(t - t₀) (Dirac delta approximation)
    Impulse,
    /// Ramp: u(t) = u₀ + αt (linear increase)
    Ramp,
    /// Periodic: u(t) = A·sin(ωt + φ)
    Periodic,
    /// Arbitrary: general discrete samples
    Arbitrary,
}

impl Perturbation {
    /// Create a constant perturbation u(t) = u₀.
    #[must_use]
    pub fn constant(id: impl Into<String>, value: Vec<f64>, duration: Option<f64>) -> Self {
        let dimension = value.len();
        Self {
            id: id.into(),
            time_points: vec![0.0],
            values: vec![value],
            dimension,
            duration,
            perturbation_type: PerturbationType::Constant,
        }
    }

    /// Create an impulse perturbation at time t₀.
    #[must_use]
    pub fn impulse(id: impl Into<String>, t0: f64, magnitude: Vec<f64>) -> Self {
        let dimension = magnitude.len();
        Self {
            id: id.into(),
            time_points: vec![t0],
            values: vec![magnitude],
            dimension,
            duration: Some(t0 + f64::EPSILON),
            perturbation_type: PerturbationType::Impulse,
        }
    }

    /// Create a step perturbation: u₀ before t₁, u₁ after.
    #[must_use]
    pub fn step(
        id: impl Into<String>,
        t1: f64,
        before: Vec<f64>,
        after: Vec<f64>,
        duration: Option<f64>,
    ) -> Self {
        let dimension = before.len();
        Self {
            id: id.into(),
            time_points: vec![0.0, t1],
            values: vec![before, after],
            dimension,
            duration,
            perturbation_type: PerturbationType::Step,
        }
    }

    /// Evaluate perturbation at time t (piecewise constant interpolation).
    #[must_use]
    pub fn evaluate(&self, t: f64) -> Option<&Vec<f64>> {
        if self.time_points.is_empty() {
            return None;
        }

        // Find the largest time point ≤ t
        let idx = self
            .time_points
            .iter()
            .rposition(|&tp| tp <= t)
            .unwrap_or(0);

        self.values.get(idx)
    }

    /// Check if perturbation is active at time t.
    #[must_use]
    pub fn is_active(&self, t: f64) -> bool {
        t >= 0.0 && self.duration.is_none_or(|d| t <= d)
    }
}

/// Definition 1.1 (part): Perturbation Class 𝒰 (ToV §1)
///
/// The perturbation function class is the set of admissible perturbations:
/// 𝒰 ⊆ {u: [0,T] → U | u measurable}
///
/// This struct defines constraints on what perturbations are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationClass {
    /// Dimension of perturbation space U
    pub dimension: usize,

    /// Allowed perturbation types
    pub allowed_types: Vec<PerturbationType>,

    /// Bounds on perturbation values: (min, max) per dimension
    pub value_bounds: Vec<(f64, f64)>,

    /// Maximum duration T (None for unbounded)
    pub max_duration: Option<f64>,

    /// Maximum rate of change |du/dt| per dimension (None for unbounded)
    pub max_rate: Option<Vec<f64>>,
}

impl Default for PerturbationClass {
    fn default() -> Self {
        Self {
            dimension: 1,
            allowed_types: vec![
                PerturbationType::Constant,
                PerturbationType::Step,
                PerturbationType::Impulse,
                PerturbationType::Ramp,
                PerturbationType::Periodic,
                PerturbationType::Arbitrary,
            ],
            value_bounds: vec![(f64::NEG_INFINITY, f64::INFINITY)],
            max_duration: None,
            max_rate: None,
        }
    }
}

impl PerturbationClass {
    /// Create a perturbation class with given dimension.
    #[must_use]
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            value_bounds: vec![(f64::NEG_INFINITY, f64::INFINITY); dimension],
            ..Default::default()
        }
    }

    /// Set value bounds for all dimensions.
    #[must_use]
    pub fn with_bounds(mut self, min: f64, max: f64) -> Self {
        self.value_bounds = vec![(min, max); self.dimension];
        self
    }

    /// Set per-dimension bounds.
    #[must_use]
    pub fn with_bounds_per_dim(mut self, bounds: Vec<(f64, f64)>) -> Self {
        self.value_bounds = bounds;
        self
    }

    /// Restrict to specific perturbation types.
    #[must_use]
    pub fn with_allowed_types(mut self, types: Vec<PerturbationType>) -> Self {
        self.allowed_types = types;
        self
    }

    /// Set maximum duration.
    #[must_use]
    pub fn with_max_duration(mut self, duration: f64) -> Self {
        self.max_duration = Some(duration);
        self
    }

    /// Check if a perturbation is admissible (u ∈ 𝒰).
    #[must_use]
    pub fn is_admissible(&self, perturbation: &Perturbation) -> bool {
        // Check dimension
        if perturbation.dimension != self.dimension {
            return false;
        }

        // Check type
        if !self.allowed_types.contains(&perturbation.perturbation_type) {
            return false;
        }

        // Check duration
        if let (Some(max_d), Some(p_d)) = (self.max_duration, perturbation.duration) {
            if p_d > max_d {
                return false;
            }
        }

        // Check value bounds
        for values in &perturbation.values {
            for (i, &v) in values.iter().enumerate() {
                if let Some(&(min, max)) = self.value_bounds.get(i) {
                    if v < min || v > max {
                        return false;
                    }
                }
            }
        }

        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.1 (PART): PARAMETER SPACE
// ═══════════════════════════════════════════════════════════════════════════

/// Parameter space Θ = Θ_known × Θ_uncertain (ToV §1 Definition 1.1)
///
/// Parameters are partitioned into:
/// - **Known (Θ_known)**: Fixed values with high confidence
/// - **Uncertain (Θ_uncertain)**: Ranges representing epistemic uncertainty
///
/// This partition is critical for:
/// - Axiom 5 (Emergence): θ ∈ Θ_uncertain captures individual susceptibility
/// - Harm Types E, H: θ-space phenomena (not conservation law violations)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParameterSpace {
    /// Known parameters with fixed values
    pub theta_known: Vec<KnownParameter>,

    /// Uncertain parameters with bounds
    pub theta_uncertain: Vec<UncertainParameter>,
}

impl ParameterSpace {
    /// Create empty parameter space.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a known parameter.
    #[must_use]
    pub fn with_known(mut self, name: impl Into<String>, value: f64) -> Self {
        self.theta_known.push(KnownParameter {
            name: name.into(),
            value,
        });
        self
    }

    /// Add an uncertain parameter with bounds.
    #[must_use]
    pub fn with_uncertain(mut self, name: impl Into<String>, lower: f64, upper: f64) -> Self {
        self.theta_uncertain.push(UncertainParameter {
            name: name.into(),
            lower_bound: lower,
            upper_bound: upper,
        });
        self
    }

    /// Total dimension of parameter space.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.theta_known.len() + self.theta_uncertain.len()
    }

    /// Check if parameter space has uncertainty.
    #[must_use]
    pub fn has_uncertainty(&self) -> bool {
        !self.theta_uncertain.is_empty()
    }
}

/// A known parameter with fixed value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownParameter {
    /// Parameter name
    pub name: String,
    /// Fixed value
    pub value: f64,
}

/// An uncertain parameter with bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertainParameter {
    /// Parameter name
    pub name: String,
    /// Lower bound of uncertainty interval
    pub lower_bound: f64,
    /// Upper bound of uncertainty interval
    pub upper_bound: f64,
}

impl UncertainParameter {
    /// Width of uncertainty interval.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper_bound - self.lower_bound
    }

    /// Midpoint of uncertainty interval.
    #[must_use]
    pub fn midpoint(&self) -> f64 {
        f64::midpoint(self.lower_bound, self.upper_bound)
    }

    /// Check if a value is within bounds.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower_bound && value <= self.upper_bound
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.2: HARM EVENT
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.2: Harm Event (ToV §1)
///
/// A harm event is a measurable event H ∈ ℱ in the underlying probability space
/// representing damage, injury, malfunction, or undesired outcome.
///
/// This struct captures a specific harm occurrence with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmEvent {
    /// Unique event identifier
    pub event_id: String,

    /// State at which harm occurred
    pub state: Vec<f64>,

    /// Harm type classification (A-H)
    pub harm_type: crate::tov::HarmType,

    /// Severity score (0.0 to 1.0)
    pub severity: f64,

    /// Timestamp (Unix epoch milliseconds)
    pub timestamp_ms: u64,

    /// Additional context
    pub context: Option<String>,
}

impl HarmEvent {
    /// Create a new harm event.
    #[must_use]
    pub fn new(
        event_id: impl Into<String>,
        state: Vec<f64>,
        harm_type: crate::tov::HarmType,
        severity: f64,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            state,
            harm_type,
            severity: severity.clamp(0.0, 1.0),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            context: None,
        }
    }

    /// Add context to the event.
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.5a: MONITORING APPARATUS
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.5a: Monitoring Apparatus (ToV §1)
///
/// A monitoring apparatus is a tuple ℳ = (O, 𝒪, ρ, D) where:
/// - O is an observation space
/// - 𝒪 = {o₁, ..., oₖ} is a finite set of observables
/// - ρ: O^k → Ŝ is a state reconstruction map
/// - D: O^k × [0,T] → {SIGNAL, NO_SIGNAL} is a detection function
///
/// This struct wraps a `GeometricSafetyManifold` with explicit observable definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringApparatus {
    /// Names of observables in 𝒪
    pub observable_names: Vec<String>,

    /// Units for each observable
    pub observable_units: Vec<String>,

    /// The safety manifold providing detection function D
    pub manifold: GeometricSafetyManifold,

    /// Observability status
    pub observability: Observability,
}

/// Observability condition (ToV §1 Definition 1.5a)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Observability {
    /// π_ℳ is injective (full state reconstruction possible)
    FullyObservable,
    /// ker(π_ℳ) ≠ {0} (some state information lost)
    PartiallyObservable,
}

impl Default for MonitoringApparatus {
    fn default() -> Self {
        Self {
            observable_names: vec![
                "PRR".to_string(),
                "ROR".to_string(),
                "IC".to_string(),
                "EBGM".to_string(),
            ],
            observable_units: vec![
                "ratio".to_string(),
                "ratio".to_string(),
                "bits".to_string(),
                "ratio".to_string(),
            ],
            manifold: GeometricSafetyManifold::default(),
            observability: Observability::FullyObservable,
        }
    }
}

impl MonitoringApparatus {
    /// Create monitoring apparatus for standard 4D signal space.
    #[must_use]
    pub fn signal_space() -> Self {
        Self::default()
    }

    /// Number of observables k = |𝒪|.
    #[must_use]
    pub fn num_observables(&self) -> usize {
        self.observable_names.len()
    }

    /// Detection function D: maps observations to {SIGNAL, NO_SIGNAL}.
    #[must_use]
    pub fn detect(&self, observations: &crate::manifold::SignalPoint) -> DetectionResult {
        let dist = self.manifold.signed_distance(observations);
        if dist.is_harmful() {
            DetectionResult::Signal {
                distance: dist.value,
                critical_observable: self
                    .observable_names
                    .get(dist.critical_dimension)
                    .cloned()
                    .unwrap_or_default(),
            }
        } else {
            DetectionResult::NoSignal { margin: dist.value }
        }
    }
}

/// Detection result from monitoring apparatus D function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionResult {
    /// Signal detected (harm region)
    Signal {
        /// Signed distance (negative = in harm region)
        distance: f64,
        /// Name of the observable that triggered detection
        critical_observable: String,
    },
    /// No signal (safe region)
    NoSignal {
        /// Safety margin (positive distance to boundary)
        margin: f64,
    },
}

impl DetectionResult {
    /// Check if a signal was detected.
    #[must_use]
    pub fn is_signal(&self) -> bool {
        matches!(self, Self::Signal { .. })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.7: SYSTEM DYNAMICS
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.7: System Dynamics (ToV §1)
///
/// System dynamics specify how state s(t) evolves over time.
/// Three forms are supported:
/// 1. Deterministic continuous-time: ds/dt = f(s, u, t)
/// 2. Stochastic continuous-time: ds = f(s,u,θ)dt + σ(s)dW
/// 3. Discrete-time: s_{k+1} = f(s_k, u_k, k)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicsType {
    /// Deterministic ODE: ds/dt = f(s, u, t)
    DeterministicContinuous,
    /// Stochastic SDE (Itô form): ds = f(s,u,θ)dt + σ(s)dW
    StochasticContinuous,
    /// Discrete-time: s_{k+1} = f(s_k, u_k, k)
    DiscreteTime,
}

// ═══════════════════════════════════════════════════════════════════════════
// DEFINITION 1.8: VIGILANCE LOOP
// ═══════════════════════════════════════════════════════════════════════════

/// Definition 1.8: Vigilance Loop (ToV §1)
///
/// The operational cycle connecting observation to response:
/// ```text
/// S ──(dynamics)──► S' ──(observation)──► O^k ──(detection)──► {SIGNAL, NO_SIGNAL}
/// ▲                                                                    │
/// └────────────────────────(response)──────────────────────────────────┘
/// ```
///
/// Components:
/// 1. Dynamics: State evolves under f(s, u, θ) + noise
/// 2. Observation: ℳ projects state to observables
/// 3. Detection: D identifies potential harm signals
/// 4. Response: Perturbation u is adjusted based on detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigilanceLoop {
    /// Current state in the loop
    pub current_stage: LoopStage,

    /// Number of complete cycles executed
    pub cycle_count: u64,

    /// Last detection result
    pub last_detection: Option<DetectionResult>,

    /// Dynamics type being used
    pub dynamics_type: DynamicsType,
}

/// Stage in the vigilance loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopStage {
    /// State evolution under dynamics
    Dynamics,
    /// Observation via monitoring apparatus
    Observation,
    /// Detection function evaluation
    Detection,
    /// Response/perturbation adjustment
    Response,
}

impl Default for VigilanceLoop {
    fn default() -> Self {
        Self {
            current_stage: LoopStage::Dynamics,
            cycle_count: 0,
            last_detection: None,
            dynamics_type: DynamicsType::DeterministicContinuous,
        }
    }
}

impl VigilanceLoop {
    /// Create a new vigilance loop.
    #[must_use]
    pub fn new(dynamics_type: DynamicsType) -> Self {
        Self {
            dynamics_type,
            ..Default::default()
        }
    }

    /// Advance to next stage in the loop.
    pub fn advance(&mut self) {
        self.current_stage = match self.current_stage {
            LoopStage::Dynamics => LoopStage::Observation,
            LoopStage::Observation => LoopStage::Detection,
            LoopStage::Detection => LoopStage::Response,
            LoopStage::Response => {
                self.cycle_count += 1;
                LoopStage::Dynamics
            }
        };
    }

    /// Record detection result.
    pub fn record_detection(&mut self, result: DetectionResult) {
        self.last_detection = Some(result);
    }

    /// Check if a signal was detected in the last cycle.
    #[must_use]
    pub fn signal_detected(&self) -> bool {
        self.last_detection
            .as_ref()
            .is_some_and(DetectionResult::is_signal)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PROPOSITION 1.1: HARM SPECIFICATION EQUIVALENCE
// ═══════════════════════════════════════════════════════════════════════════

/// Proposition 1.1: Convert harm indicator to constraint value.
///
/// g_H(s, u, θ) = H_spec(s, u, θ) - 0.5
///
/// - H_spec = 0 (safe) → g_H = -0.5 ≤ 0 (constraint satisfied)
/// - H_spec = 1 (harm) → g_H = 0.5 > 0 (constraint violated)
#[must_use]
pub fn harm_to_constraint(h_spec: bool) -> f64 {
    if h_spec { 0.5 } else { -0.5 }
}

/// Proposition 1.1: Convert constraint value to harm indicator.
///
/// H_spec = 1 if g_H > 0, else 0
#[must_use]
pub fn constraint_to_harm(g_h: f64) -> bool {
    g_h > 0.0
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vigilance_system_creation() {
        let system = VigilanceSystem::new_signal_space()
            .with_id("test-system")
            .with_parameters(
                ParameterSpace::new()
                    .with_known("threshold", 2.0)
                    .with_uncertain("susceptibility", 0.0, 1.0),
            );

        assert_eq!(system.state_dim, 4);
        assert_eq!(system.parameters.dimension(), 2);
        assert!(system.parameters.has_uncertainty());
    }

    #[test]
    fn test_parameter_space() {
        let params = ParameterSpace::new()
            .with_known("alpha", 0.5)
            .with_uncertain("theta", 0.1, 0.9);

        assert_eq!(params.theta_known.len(), 1);
        assert_eq!(params.theta_uncertain.len(), 1);
        assert!(params.theta_uncertain[0].contains(0.5));
        assert!(!params.theta_uncertain[0].contains(0.05));
    }

    #[test]
    fn test_uncertain_parameter_methods() {
        let param = UncertainParameter {
            name: "theta".to_string(),
            lower_bound: 0.2,
            upper_bound: 0.8,
        };

        assert!((param.width() - 0.6).abs() < 1e-10);
        assert!((param.midpoint() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_monitoring_apparatus_detection() {
        let apparatus = MonitoringApparatus::signal_space();

        // Safe point
        let safe = crate::manifold::SignalPoint::new(1.0, 1.0, -0.5, 1.0);
        let result = apparatus.detect(&safe);
        assert!(!result.is_signal());

        // Harmful point
        let harm = crate::manifold::SignalPoint::new(3.0, 3.0, 1.0, 3.0);
        let result = apparatus.detect(&harm);
        assert!(result.is_signal());
    }

    #[test]
    fn test_vigilance_loop_advance() {
        let mut loop_ = VigilanceLoop::new(DynamicsType::DeterministicContinuous);

        assert_eq!(loop_.current_stage, LoopStage::Dynamics);
        assert_eq!(loop_.cycle_count, 0);

        loop_.advance(); // → Observation
        loop_.advance(); // → Detection
        loop_.advance(); // → Response
        loop_.advance(); // → Dynamics (cycle complete)

        assert_eq!(loop_.current_stage, LoopStage::Dynamics);
        assert_eq!(loop_.cycle_count, 1);
    }

    #[test]
    fn test_harm_constraint_bijection_wolfram_validated() {
        // Wolfram: 1 - 0.5 = 0.5
        assert!((harm_to_constraint(true) - 0.5).abs() < 1e-10);
        // Wolfram: 0 - 0.5 = -0.5
        assert!((harm_to_constraint(false) - (-0.5)).abs() < 1e-10);

        // Round-trip
        assert!(constraint_to_harm(0.5));
        assert!(!constraint_to_harm(-0.5));
    }

    #[test]
    fn test_harm_event_creation() {
        let event = HarmEvent::new(
            "EVT-001",
            vec![3.0, 2.5, 0.5, 2.8],
            crate::tov::HarmType::Acute,
            0.75,
        )
        .with_context("Test harm event");

        assert_eq!(event.harm_type.letter(), 'A');
        assert!((event.severity - 0.75).abs() < 1e-10);
        assert!(event.context.is_some());
    }

    #[test]
    fn test_perturbation_constant() {
        let p = Perturbation::constant("dose", vec![10.0], Some(24.0));

        assert_eq!(p.perturbation_type, PerturbationType::Constant);
        assert_eq!(p.dimension, 1);
        assert!(p.is_active(0.0));
        assert!(p.is_active(12.0));
        assert!(!p.is_active(25.0));

        // Evaluate at any time should return the constant value
        assert_eq!(p.evaluate(5.0), Some(&vec![10.0]));
        assert_eq!(p.evaluate(20.0), Some(&vec![10.0]));
    }

    #[test]
    fn test_perturbation_step() {
        let p = Perturbation::step("workload", 5.0, vec![0.0], vec![100.0], Some(10.0));

        assert_eq!(p.perturbation_type, PerturbationType::Step);

        // Before step
        assert_eq!(p.evaluate(2.0), Some(&vec![0.0]));
        // After step
        assert_eq!(p.evaluate(7.0), Some(&vec![100.0]));
    }

    #[test]
    fn test_perturbation_impulse() {
        let p = Perturbation::impulse("shock", 1.0, vec![1000.0]);

        assert_eq!(p.perturbation_type, PerturbationType::Impulse);
        assert_eq!(p.time_points[0], 1.0);
    }

    #[test]
    fn test_perturbation_class_admissibility() {
        let class = PerturbationClass::new(1)
            .with_bounds(0.0, 100.0)
            .with_allowed_types(vec![PerturbationType::Constant, PerturbationType::Step]);

        // Admissible: constant within bounds
        let p1 = Perturbation::constant("ok", vec![50.0], None);
        assert!(class.is_admissible(&p1));

        // Not admissible: value out of bounds
        let p2 = Perturbation::constant("too_high", vec![150.0], None);
        assert!(!class.is_admissible(&p2));

        // Not admissible: wrong type
        let p3 = Perturbation::impulse("impulse", 0.0, vec![50.0]);
        assert!(!class.is_admissible(&p3));

        // Not admissible: wrong dimension
        let p4 = Perturbation::constant("2d", vec![50.0, 50.0], None);
        assert!(!class.is_admissible(&p4));
    }

    #[test]
    fn test_vigilance_system_perturbation_class() {
        let system = VigilanceSystem::new_signal_space().with_perturbation_class(
            PerturbationClass::new(1)
                .with_bounds(0.0, 1000.0)
                .with_max_duration(24.0),
        );

        assert_eq!(system.perturbation_dim(), 1);

        // Test admissibility
        let p_ok = Perturbation::constant("dose", vec![500.0], Some(12.0));
        assert!(system.is_perturbation_admissible(&p_ok));

        let p_too_long = Perturbation::constant("dose", vec![500.0], Some(48.0));
        assert!(!system.is_perturbation_admissible(&p_too_long));
    }
}
