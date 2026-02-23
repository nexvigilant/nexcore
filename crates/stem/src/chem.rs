//! # stem-chem: Chemistry Primitives as Rust Traits
//!
//! Implements cross-domain T2-P primitives derived from chemistry.
//!
//! ## The CHEMISTRY Composite (T2-C)
//!
//! ```text
//! C - CONCENTRATE : Substance → Ratio          (T1: MAPPING μ + QUANTITY N)
//! H - HARMONIZE   : System → Equilibrium       (T1: STATE varsigma)
//! E - ENERGIZE    : Input → Activation → Rate  (T1: MAPPING μ + BOUNDARY ∂)
//! M - MODULATE    : Catalyst → Rate Change     (T1: CAUSALITY →)
//! I - INTERACT    : Ligand → Affinity          (T1: SEQUENCE σ)
//! S - SATURATE    : Capacity → Fraction        (T1: STATE varsigma)
//! T - TRANSFORM   : Reactants → Products       (T1: MAPPING μ)
//! R - REGULATE    : Inhibitor → Rate Decrease  (T1: RECURSION ρ)
//! Y - YIELD       : Actual / Theoretical       (T1: MAPPING μ)
//! ```
//!
//! ## Cross-Domain Transfer
//!
//! | Chemistry | PV Signals | Economics | Software |
//! |-----------|------------|-----------|----------|
//! | Concentration | Case density | Market share | Request rate |
//! | Equilibrium | Baseline | Supply-demand | Load balance |
//! | Activation | Signal threshold | Startup cost | Trigger threshold |
//!
//! ## Three Unfixable Limits
//!
//! 1. **Heisenberg**: Measuring concentration alters the system
//! 2. **Gödel**: Chemistry cannot fully model itself
//! 3. **Shannon**: Yield measurement has irreducible loss

use crate::core::Confidence;
use serde::{Deserialize, Serialize};

// ============================================================================
// Core Types (T2-P)
// ============================================================================

/// Ratio of substance to volume (T2-P)
///
/// Grounded in T1 Quantity (N) and Mapping (μ): quantity → ratio
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Ratio(f64);

impl Ratio {
    /// Create new ratio, clamping to non-negative
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for Ratio {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Fraction between 0.0 and 1.0 (T2-P)
///
/// Grounded in T1 Quantity (N): actual/maximum ratio
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Fraction(f64);

impl Fraction {
    /// Create new fraction, clamping to [0.0, 1.0]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if saturated (≥ 0.99)
    #[must_use]
    pub fn is_saturated(&self) -> bool {
        self.0 >= 0.99
    }
}

impl Default for Fraction {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Rate of change (T2-P)
///
/// Grounded in T1 Mapping (μ): time → change
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Rate(f64);

impl Rate {
    /// Create new rate
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Zero rate (no change)
    pub const ZERO: Self = Self(0.0);
}

impl Default for Rate {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Binding strength (T2-P)
///
/// Grounded in T1 Mapping (μ): interaction → strength
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Affinity(f64);

impl Affinity {
    /// Create new affinity
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get raw value
    #[must_use]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// No binding
    pub const NONE: Self = Self(0.0);

    /// Perfect binding
    pub const PERFECT: Self = Self(1.0);
}

impl Default for Affinity {
    fn default() -> Self {
        Self::NONE
    }
}

/// Equilibrium state (T2-P)
///
/// Grounded in T1 State (ς): forward rate = reverse rate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    /// Forward reaction rate
    pub forward: Rate,
    /// Reverse reaction rate
    pub reverse: Rate,
    /// Equilibrium constant K = forward/reverse
    pub constant: f64,
}

impl Balance {
    /// Create new balance state
    #[must_use]
    pub fn new(forward: Rate, reverse: Rate) -> Self {
        let constant = if reverse.value() > 0.0 {
            forward.value() / reverse.value()
        } else {
            f64::INFINITY
        };
        Self {
            forward,
            reverse,
            constant,
        }
    }

    /// Check if at equilibrium (rates equal within tolerance)
    #[must_use]
    pub fn is_equilibrium(&self, tolerance: f64) -> bool {
        (self.forward.value() - self.reverse.value()).abs() < tolerance
    }

    /// Products favored (K > 1)
    #[must_use]
    pub fn products_favored(&self) -> bool {
        self.constant > 1.0
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::new(Rate::new(1.0), Rate::new(1.0))
    }
}

// ============================================================================
// CHEMISTRY Traits (T2-P)
// ============================================================================

/// T2-P: Measure concentration (amount per volume)
///
/// Grounded in T1 Mapping (μ): substance → ratio
///
/// # Cross-Domain Transfer
/// - PV: Case density per population
/// - Economics: Market concentration
/// - Software: Request rate per server
pub trait Concentrate {
    /// The substance being measured
    type Substance;

    /// Measure concentration of substance
    fn concentration(&self, substance: &Self::Substance) -> Ratio;
}

/// T2-P: Reach equilibrium state
///
/// Grounded in T1 State (ς): stable balance point
///
/// # Cross-Domain Transfer
/// - PV: Baseline reporting rate
/// - Economics: Supply-demand equilibrium
/// - Software: Load balancing steady state
pub trait Harmonize {
    /// The system state type
    type System;

    /// Calculate equilibrium balance
    fn equilibrium(&self, system: &Self::System) -> Balance;
}

/// T2-P: Apply activation energy to initiate process
///
/// Grounded in T1 Mapping (μ) and Boundary (∂): energy → threshold → rate
///
/// # Cross-Domain Transfer
/// - PV: Signal threshold (PRR ≥ 2.0)
/// - Economics: Startup activation cost
/// - Software: Trigger threshold for alerts
pub trait Energize {
    /// Energy input type
    type Energy;

    /// Activate with given energy, return resulting rate
    fn activate(&self, energy: Self::Energy) -> Rate;

    /// Activation energy threshold
    fn activation_threshold(&self) -> Self::Energy;
}

/// T2-P: Alter rate without being consumed
///
/// Grounded in T1 Causality (→): apply, restore, repeat
///
/// # Cross-Domain Transfer
/// - PV: Confounding factors
/// - Economics: Market makers
/// - Software: Middleware, caching
pub trait Modulate {
    /// Catalyst type
    type Catalyst;

    /// Apply catalyst to modulate rate
    fn catalyze(&mut self, catalyst: &Self::Catalyst);

    /// Current rate multiplier from catalysis
    fn rate_multiplier(&self) -> f64;
}

/// T2-P: Selective binding with affinity
///
/// Grounded in T1 Sequence (σ): approach → bind → hold
///
/// # Cross-Domain Transfer
/// - PV: Drug-receptor binding
/// - Economics: Contract execution
/// - Software: API coupling, dependency injection
pub trait Interact {
    /// Ligand type that binds
    type Ligand;

    /// Bind ligand and return affinity
    fn bind(&self, ligand: &Self::Ligand) -> Affinity;

    /// Check if binding site is occupied
    fn is_bound(&self) -> bool;
}

/// T2-P: Approach capacity limit
///
/// Grounded in T1 State (ς): maximum capacity reached
///
/// # Cross-Domain Transfer
/// - PV: Case processing capacity (Vmax)
/// - Economics: Market saturation
/// - Software: Queue/buffer limits
pub trait Saturate {
    /// Current saturation fraction
    fn saturation(&self) -> Fraction;

    /// Maximum capacity
    fn capacity(&self) -> f64;

    /// Check if saturated
    fn is_saturated(&self) -> bool {
        self.saturation().is_saturated()
    }
}

/// T2-P: Convert reactants to products
///
/// Grounded in T1 Mapping (μ): A + B → C + D
///
/// # Cross-Domain Transfer
/// - PV: Raw data → standardized format
/// - Economics: Input → output transformation
/// - Software: ETL pipelines
pub trait Transform {
    /// Input reactants type
    type Reactants;
    /// Output products type
    type Products;

    /// Transform reactants into products
    fn react(&self, reactants: Self::Reactants) -> Self::Products;

    /// Check if transformation is possible
    fn can_react(&self, reactants: &Self::Reactants) -> bool;
}

/// T2-P: Reduce activity via interference
///
/// Grounded in T1 Recursion (ρ): negative feedback loop
///
/// # Cross-Domain Transfer
/// - PV: Signal suppression factors
/// - Economics: Circuit breakers
/// - Software: Rate limiting, backpressure
pub trait Regulate {
    /// Inhibitor type
    type Inhibitor;

    /// Apply inhibitor to reduce activity
    fn inhibit(&mut self, inhibitor: &Self::Inhibitor);

    /// Current inhibition level (0.0 = none, 1.0 = complete)
    fn inhibition_level(&self) -> Fraction;
}

/// T2-P: Measure efficiency (actual / theoretical)
///
/// Grounded in T1 Mapping (μ): actual → expected ratio
///
/// # Cross-Domain Transfer
/// - PV: Detection efficiency
/// - Economics: Conversion rate
/// - Software: Throughput vs capacity
pub trait Yield {
    /// Calculate efficiency
    fn efficiency(&self) -> Fraction;

    /// Theoretical maximum output
    fn theoretical_max(&self) -> f64;

    /// Actual output
    fn actual_output(&self) -> f64;
}

// ============================================================================
// Chemistry Composite Trait (T2-C)
// ============================================================================

/// T2-C: The complete chemistry methodology as composite trait
///
/// Combines all nine T2-P primitives into a coherent system.
///
/// # Gödel Acknowledgment
///
/// A chemistry system modeling itself encounters incompleteness.
pub trait Chemistry:
    Concentrate + Harmonize + Energize + Modulate + Interact + Saturate + Transform + Regulate + Yield
{
    /// Execute one reaction cycle
    ///
    /// Returns the yield efficiency of this cycle
    fn cycle(&mut self) -> Fraction
    where
        Self: Sized;
}

// ============================================================================
// Measured Chemistry Types
// ============================================================================

/// A chemistry measurement with confidence (Codex IX: MEASURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredRatio {
    /// The measured ratio
    pub value: Ratio,
    /// Confidence in measurement
    pub confidence: Confidence,
}

impl MeasuredRatio {
    /// Create new measured ratio
    pub fn new(value: Ratio, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

/// A chemistry rate with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredRate {
    /// The measured rate
    pub value: Rate,
    /// Confidence in measurement
    pub confidence: Confidence,
}

impl MeasuredRate {
    /// Create new measured rate
    pub fn new(value: Rate, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors in chemistry operations
#[derive(Debug, nexcore_error::Error)]
pub enum ChemistryError {
    /// Concentration measurement failed
    #[error("concentration measurement failed: {0}")]
    ConcentrationFailed(String),

    /// Equilibrium not reached
    #[error("equilibrium not reached: {0}")]
    EquilibriumFailed(String),

    /// Activation energy insufficient
    #[error("activation energy insufficient: needed {needed}, got {got}")]
    InsufficientEnergy { needed: f64, got: f64 },

    /// Binding failed
    #[error("binding failed: {0}")]
    BindingFailed(String),

    /// Saturation exceeded
    #[error("saturation exceeded capacity")]
    SaturationExceeded,

    /// Transformation failed
    #[error("transformation failed: {0}")]
    TransformFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_clamps_negative() {
        let r = Ratio::new(-5.0);
        assert!((r.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fraction_clamps_to_range() {
        assert!((Fraction::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Fraction::new(-0.5).value() - 0.0).abs() < f64::EPSILON);
        assert!((Fraction::new(0.7).value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn fraction_detects_saturation() {
        assert!(Fraction::new(0.99).is_saturated());
        assert!(Fraction::new(1.0).is_saturated());
        assert!(!Fraction::new(0.5).is_saturated());
    }

    #[test]
    fn balance_equilibrium_detection() {
        let balanced = Balance::new(Rate::new(1.0), Rate::new(1.0));
        assert!(balanced.is_equilibrium(0.01));
        assert!(!balanced.products_favored()); // K = 1

        let products = Balance::new(Rate::new(2.0), Rate::new(1.0));
        assert!(products.products_favored()); // K = 2
    }

    #[test]
    fn affinity_bounds() {
        assert!((Affinity::NONE.value() - 0.0).abs() < f64::EPSILON);
        assert!((Affinity::PERFECT.value() - 1.0).abs() < f64::EPSILON);
        assert!((Affinity::new(0.5).value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn rate_zero_constant() {
        assert!((Rate::ZERO.value() - 0.0).abs() < f64::EPSILON);
        assert!((Rate::new(5.0).value() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn measured_ratio_preserves_confidence() {
        let m = MeasuredRatio::new(Ratio::new(2.5), Confidence::new(0.9));
        assert!((m.value.value() - 2.5).abs() < f64::EPSILON);
        assert!((m.confidence.value() - 0.9).abs() < f64::EPSILON);
    }
}
