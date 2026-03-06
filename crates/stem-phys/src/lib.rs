//! # stem-phys: Physics Primitives as Rust Traits
//!
//! Implements cross-domain T2-P primitives derived from physics.
//!
//! ## The PHYSICS Composite (T2-C)
//!
//! ```text
//! P - PRESERVE   : Quantity unchanged across transform  (T1: STATE ς + PERSISTENCE π)
//! H - HARMONICS  : Periodic oscillation around center   (T1: RECURSION ρ + FREQUENCY f)
//! Y - YIELD_FORCE: Cause → proportional effect          (T1: CAUSALITY → + QUANTITY N)
//! S - SUPERPOSE  : Sum of parts equals whole            (T1: SUM Σ)
//! I - INERTIA    : Resistance to state change           (T1: STATE ς + PERSISTENCE π)
//! C - COUPLE     : Action produces equal reaction       (T1: SEQUENCE σ + CAUSALITY →)
//! S - SCALE      : Proportional dimension transform     (T1: MAPPING μ)
//! D - DETECT     : Stimulus → measurement signal        (T1: MAPPING μ)
//! ```
//!
//! ## Cross-Domain Transfer
//!
//! | Physics | PV Signals | Economics | Software |
//! |---------|------------|-----------|----------|
//! | Conservation | Case count | Budget balance | Data integrity |
//! | Oscillation | Seasonal | Business cycles | Retry backoff |
//! | Force | Dose-response | Price elasticity | Load → latency |
//!
//! ## Three Unfixable Limits
//!
//! 1. **Heisenberg**: Measuring position alters momentum
//! 2. **Gödel**: Physics cannot fully model itself
//! 3. **Shannon**: Energy measurement has quantization loss

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
pub mod grounding;

use serde::{Deserialize, Serialize};
use stem_core::Confidence;

// ============================================================================
// Core Types (T2-P)
// ============================================================================

/// Conserved quantity (T2-P)
///
/// Grounded in T1 Persistence (π) and State (ς): invariant across transformations
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantity(f64);

impl Quantity {
    /// Create new quantity
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero quantity
    pub const ZERO: Self = Self(0.0);

    /// Check conservation (within tolerance)
    #[must_use]
    pub fn conserved_with(&self, other: &Self, tolerance: f64) -> bool {
        (self.0 - other.0).abs() < tolerance
    }
}

impl Default for Quantity {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Quantity {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Quantity {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

/// Mass (resistance to acceleration) (T2-P)
///
/// Grounded in T1 State (ς): intrinsic property
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Mass(f64);

impl Mass {
    /// Create new mass (must be positive)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Unit mass
    pub const UNIT: Self = Self(1.0);
}

impl Default for Mass {
    fn default() -> Self {
        Self::UNIT
    }
}

/// Force (cause of acceleration) (T2-P)
///
/// Grounded in T1 Causality (→): cause → effect
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Force(f64);

impl Force {
    /// Create new force
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero force
    pub const ZERO: Self = Self(0.0);

    /// Magnitude (absolute value)
    #[must_use]
    pub fn magnitude(&self) -> f64 {
        self.0.abs()
    }
}

impl Default for Force {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Neg for Force {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl std::ops::Add for Force {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

/// Acceleration (rate of velocity change) (T2-P)
///
/// Grounded in T1 Mapping (μ): force / mass ratio
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Acceleration(f64);

impl Acceleration {
    /// Create new acceleration
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero acceleration
    pub const ZERO: Self = Self(0.0);

    /// Calculate from F=ma
    #[must_use]
    pub fn from_force_and_mass(force: Force, mass: Mass) -> Self {
        if mass.value() > 0.0 {
            Self(force.value() / mass.value())
        } else {
            Self::ZERO
        }
    }
}

impl Default for Acceleration {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Frequency (cycles per unit time) (T2-P)
///
/// Grounded in T1 Frequency (f): repetition rate
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Frequency(f64);

impl Frequency {
    /// Create new frequency (must be positive)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Period (1/frequency)
    #[must_use]
    pub fn period(&self) -> f64 {
        if self.0 > 0.0 {
            1.0 / self.0
        } else {
            f64::INFINITY
        }
    }

    /// Unit frequency (1 Hz)
    pub const UNIT: Self = Self(1.0);
}

impl Default for Frequency {
    fn default() -> Self {
        Self::UNIT
    }
}

/// Amplitude (oscillation magnitude) (T2-P)
///
/// Grounded in T1 Quantity (N): displacement from center
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Amplitude(f64);

impl Amplitude {
    /// Create new amplitude (must be positive)
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero amplitude
    pub const ZERO: Self = Self(0.0);
}

impl Default for Amplitude {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Amplitude {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

/// Scale factor (T2-P)
///
/// Grounded in T1 Mapping (μ): proportional transformation
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    /// Create new scale factor
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Identity scale (no change)
    pub const IDENTITY: Self = Self(1.0);

    /// Apply scale to a value
    #[must_use]
    pub fn apply(&self, value: f64) -> f64 {
        value * self.0
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ============================================================================
// PHYSICS Traits (T2-P)
// ============================================================================

/// T2-P: Conservation law - quantity unchanged across transform
///
/// Grounded in T1 Persistence (π): quantity endures through time/transformation
///
/// # Cross-Domain Transfer
/// - PV: Case count conservation across datasets
/// - Economics: Budget/accounting balance
/// - Software: Data integrity, checksum
pub trait Preserve {
    /// The conserved quantity type
    type Conserved;

    /// Get the conserved quantity
    fn conserved(&self) -> Self::Conserved;

    /// Check if quantity is preserved (within tolerance)
    fn is_preserved(&self, before: &Self::Conserved, tolerance: f64) -> bool;
}

/// T2-P: Harmonic oscillation around equilibrium
///
/// Grounded in T1 Recursion (ρ) and Frequency (f): periodic self-repetition
///
/// # Cross-Domain Transfer
/// - PV: Seasonal reporting patterns
/// - Economics: Business cycles
/// - Software: Retry with exponential backoff
pub trait Harmonics {
    /// Oscillation state type
    type State;

    /// Apply one oscillation step
    fn oscillate(&mut self, frequency: Frequency);

    /// Get current phase (0.0 to 1.0)
    fn phase(&self) -> f64;

    /// Get current amplitude
    fn amplitude(&self) -> Amplitude;

    /// Check if at equilibrium (amplitude near zero)
    fn at_equilibrium(&self, tolerance: f64) -> bool {
        self.amplitude().value() < tolerance
    }
}

/// T2-P: Force yields proportional acceleration (F=ma)
///
/// Grounded in T1 Causality (→): cause → effect
///
/// # Cross-Domain Transfer
/// - PV: Dose-response relationship
/// - Economics: Price elasticity
/// - Software: Load → latency relationship
pub trait YieldForce {
    /// Apply force and get resulting acceleration
    fn apply_force(&self, force: Force) -> Acceleration;

    /// Get intrinsic mass (resistance)
    fn mass(&self) -> Mass;
}

/// T2-P: Detect physical stimulus and produce measurement signal.
///
/// Grounded in T1 MAPPING (μ): stimulus → measurement.
///
/// This is the physics-specific signal *input* trait, completing the
/// asymmetry where Physics had output traits (Preserve, YieldForce)
/// but no explicit signal sensing mechanism.
///
/// # Sense ↔ Detect Relationship
///
/// `stem_core::Sense` is the abstract observation trait.
/// `Detect` is its physics-specific incarnation for physical stimuli.
///
/// # Cross-Domain Transfer
/// - PV: Case report intake → structured ICSR
/// - Economics: Market movement → trading signal
/// - Software: Log event → alert
pub trait Detect {
    /// The physical stimulus being measured
    type Stimulus;
    /// The measurement signal produced
    type Measurement;

    /// Detect stimulus and produce measurement signal.
    ///
    /// Measurement confidence decreases with stimulus magnitude
    /// near the detection limit (Heisenberg applies).
    fn detect(&self, stimulus: &Self::Stimulus) -> Self::Measurement;

    /// Detection limit — minimum stimulus that produces a signal.
    ///
    /// Below this threshold, the measurement is noise.
    fn detection_limit(&self) -> Self::Stimulus;
}

/// T2-P: Superposition - sum of parts equals whole
///
/// Grounded in T1 Sum (Σ): additive composition of independent parts
///
/// # Cross-Domain Transfer
/// - PV: Combined signal from multiple sources
/// - Economics: Portfolio value = sum of assets
/// - Software: Event aggregation
pub trait Superpose: Sized {
    /// Combine with another instance (linear superposition)
    fn superpose(&self, other: &Self) -> Self;

    /// Check if superposition is valid (associative)
    fn superposition_valid(&self, a: &Self, b: &Self) -> bool;
}

/// T2-P: Inertia - resistance to state change
///
/// Grounded in T1 Persistence (π): persistence tendency
///
/// # Cross-Domain Transfer
/// - PV: Reporting behavior lag
/// - Economics: Market momentum
/// - Software: Cache persistence, stickiness
pub trait Inertia {
    /// Get inertial mass (resistance to change)
    fn inertial_mass(&self) -> Mass;

    /// Force required to change state by given amount
    fn resistance(&self, change: f64) -> Force;
}

/// T2-P: Coupling - action produces equal opposite reaction
///
/// Grounded in T1 Sequence (σ) and Causality (→): bidirectional causation
///
/// # Cross-Domain Transfer
/// - PV: Drug-drug interaction
/// - Economics: Supply-demand response
/// - Software: Request-response pairing
pub trait Couple {
    /// Action type
    type Action;
    /// Reaction type (often same as Action)
    type Reaction;

    /// Get reaction to given action
    fn react(&self, action: &Self::Action) -> Self::Reaction;

    /// Check if action-reaction are equal and opposite
    fn is_balanced(&self, action: &Self::Action, reaction: &Self::Reaction) -> bool;
}

/// T2-P: Scale transformation across dimensions
///
/// Grounded in T1 Mapping (μ): proportional change
///
/// # Cross-Domain Transfer
/// - PV: Population adjustment
/// - Economics: Currency conversion
/// - Software: Unit normalization
pub trait Scale: Sized {
    /// Apply scale transformation
    fn scale(&self, factor: ScaleFactor) -> Self;

    /// Get scale factor between self and other
    fn scale_factor_to(&self, other: &Self) -> Option<ScaleFactor>;
}

// ============================================================================
// Physics Composite Trait (T2-C)
// ============================================================================

/// T2-C: The complete physics methodology as composite trait.
///
/// Combines all eight T2-P primitives into a coherent system.
///
/// `Detect` completes the signal I/O asymmetry — Physics now has both
/// signal input (Detect) and output (YieldForce, Couple) pathways.
///
/// # Gödel Acknowledgment
///
/// A physics system modeling itself encounters incompleteness.
pub trait Physics:
    Preserve + Harmonics + YieldForce + Detect + Superpose + Inertia + Couple + Scale
{
    /// Execute one physics simulation step
    fn step(&mut self, dt: f64)
    where
        Self: Sized;
}

// ============================================================================
// Measured Physics Types
// ============================================================================

/// A physics measurement with confidence (Codex IX: MEASURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredQuantity {
    /// The measured quantity
    pub value: Quantity,
    /// Confidence in measurement
    pub confidence: Confidence,
}

impl MeasuredQuantity {
    /// Create new measured quantity
    pub fn new(value: Quantity, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

/// A force measurement with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredForce {
    /// The measured force
    pub value: Force,
    /// Confidence in measurement
    pub confidence: Confidence,
}

impl MeasuredForce {
    /// Create new measured force
    pub fn new(value: Force, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors in physics operations
#[derive(Debug, nexcore_error::Error)]
pub enum PhysicsError {
    /// Conservation violated
    #[error("conservation violated: expected {expected}, got {actual}")]
    ConservationViolated { expected: f64, actual: f64 },

    /// Invalid mass (zero or negative)
    #[error("invalid mass: {0}")]
    InvalidMass(f64),

    /// Scale factor undefined
    #[error("scale factor undefined between incompatible values")]
    ScaleUndefined,

    /// Coupling imbalance
    #[error("action-reaction imbalance: {0}")]
    CouplingImbalance(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_conservation_check() {
        let q1 = Quantity::new(100.0);
        let q2 = Quantity::new(100.001);
        assert!(q1.conserved_with(&q2, 0.01));
        assert!(!q1.conserved_with(&Quantity::new(101.0), 0.01));
    }

    #[test]
    fn quantity_arithmetic() {
        let a = Quantity::new(10.0);
        let b = Quantity::new(5.0);
        assert!(((a + b).value() - 15.0).abs() < f64::EPSILON);
        assert!(((a - b).value() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn force_negation() {
        let f = Force::new(10.0);
        let neg_f = -f;
        assert!((neg_f.value() - (-10.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn acceleration_from_fma() {
        let f = Force::new(10.0);
        let m = Mass::new(2.0);
        let a = Acceleration::from_force_and_mass(f, m);
        assert!((a.value() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn frequency_period_inverse() {
        let f = Frequency::new(2.0);
        assert!((f.period() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn scale_factor_apply() {
        let s = ScaleFactor::new(2.0);
        assert!((s.apply(5.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn amplitude_superposition() {
        let a1 = Amplitude::new(3.0);
        let a2 = Amplitude::new(4.0);
        assert!(((a1 + a2).value() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mass_clamps_negative() {
        let m = Mass::new(-5.0);
        assert!((m.value() - 0.0).abs() < f64::EPSILON);
    }
}
