//! # STEM: Science, Technology, Engineering, Mathematics as Rust Traits
//!
//! Unified facade for cross-domain T2-P primitives owned by NexVigilant.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                              stem (facade)                              │
//! ├────────────┬───────────┬───────────┬───────────┬───────────┬───────────┤
//! │ stem-core  │ stem-bio  │ stem-chem │ stem-phys │ stem-math │ stem-fin  │
//! │ 8 SCIENCE  │ Endocrine │ 9 CHEMIST │ 8 PHYSICS │ 9 MATHS   │ 9 FINANCE │
//! │ traits     │ System    │ traits    │ traits    │ traits    │ traits    │
//! ├────────────┴───────────┴───────────┴───────────┴───────────┴───────────┤
//! │                        T1 Primitives (12 of 16)                         │
//! │  σ · μ · ρ · ς · ∂ · Σ · π · N · → · κ · ∝ · ×                        │
//! └──────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Taxonomy
//!
//! ```text
//! T1 (Universal)
//! ├── SEQUENCE (σ) ──► Experiment, Transit, Interact, Couple, Prove, Flow
//! ├── MAPPING (μ) ───► Sense, Classify, Codify, Extend, Concentrate,
//! │                    Energize, Transform, Yield, YieldForce, Scale,
//! │                    Membership, Homeomorph, Symmetric, Commute
//! ├── RECURSION (ρ) ─► Infer, Modulate, Regulate, Harmonics, Associate, Compound
//! ├── STATE (ς) ─────► Normalize, Harmonize, Saturate, Identify
//! ├── QUANTITY (N) ──► Appraise
//! ├── CAUSALITY (→) ─► Discount
//! ├── COMPARISON (κ) ► Arbitrage
//! ├── IRREVERSIBILITY (∝) ► Mature
//! ├── PRODUCT (×) ───► Leverage
//! ├── BOUNDARY (∂) ──► Bound, Hedge
//! ├── SUM (Σ) ───────► Superpose, Diversify
//! └── PERSISTENCE (π) ► Preserve, Inertia
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use stem::prelude::*;
//!
//! // Access all STEM primitives through unified prelude
//! let confidence = Confidence::new(0.95);
//! let tier = Tier::T2Primitive;
//! ```
//!
//! ## Complete Trait → T1 Grounding Map
//!
//! | Domain | Trait | T1 Grounding | Cross-Domain Transfer |
//! |--------|-------|--------------|-----------------------|
//! | **Science** | `Sense` | MAPPING (μ) | Environment → Signal |
//! | | `Classify` | MAPPING (μ) | Signal → Category |
//! | | `Infer` | RECURSION (ρ) | Pattern × Data → Prediction |
//! | | `Experiment` | SEQUENCE (σ) | Action → Outcome |
//! | | `Normalize` | STATE (ς) | Prior × Evidence → Posterior |
//! | | `Codify` | MAPPING (μ) | Belief → Representation |
//! | | `Extend` | MAPPING (μ) | Source → Target domain |
//! | **Chemistry** | `Concentrate` | MAPPING (μ) | Substance → Ratio |
//! | | `Harmonize` | STATE (ς) | System → Equilibrium |
//! | | `Energize` | MAPPING (μ) | Energy → Threshold → Rate |
//! | | `Modulate` | RECURSION (ρ) | Catalyst → Rate change |
//! | | `Interact` | SEQUENCE (σ) | Ligand → Affinity |
//! | | `Saturate` | STATE (ς) | Capacity → Fraction |
//! | | `Transform` | MAPPING (μ) | Reactants → Products |
//! | | `Regulate` | RECURSION (ρ) | Inhibitor → Rate decrease |
//! | | `Yield` | MAPPING (μ) | Actual / Theoretical |
//! | **Physics** | `Preserve` | PERSISTENCE (π) | Quantity unchanged across transform |
//! | | `Harmonics` | RECURSION (ρ) | Oscillation around center |
//! | | `YieldForce` | MAPPING (μ) | Force → Acceleration |
//! | | `Superpose` | SUM (Σ) | Sum of parts = whole |
//! | | `Inertia` | PERSISTENCE (π) | Resistance to change |
//! | | `Couple` | SEQUENCE (σ) | Action → Reaction |
//! | | `Scale` | MAPPING (μ) | Proportional transform |
//! | **Math** | `Membership` | MAPPING (μ) | Element ∈ Set |
//! | | `Associate` | RECURSION (ρ) | (a·b)·c = a·(b·c) |
//! | | `Transit` | SEQUENCE (σ) | a→b ∧ b→c ⟹ a→c |
//! | | `Homeomorph` | MAPPING (μ) | Structure-preserving map |
//! | | `Symmetric` | MAPPING (μ) | a~b ⟹ b~a |
//! | | `Bound` | BOUNDARY (∂) | Upper/lower limits |
//! | | `Prove` | SEQUENCE (σ) | Premises → Conclusion |
//! | | `Commute` | MAPPING (μ) | a·b = b·a |
//! | | `Identify` | STATE (ς) | Neutral element |
//! | **Finance** | `Appraise` | QUANTITY (N) | Asset → Monetary value |
//! | | `Flow` | SEQUENCE (σ) | Value moves between accounts |
//! | | `Discount` | CAUSALITY (→) | Future value → Present value |
//! | | `Compound` | RECURSION (ρ) | Growth applied to growth |
//! | | `Hedge` | BOUNDARY (∂) | Position → Bounded risk |
//! | | `Arbitrage` | COMPARISON (κ) | Two prices → Exploit gap |
//! | | `Mature` | IRREVERSIBILITY (∝) | Instrument → Terminal event |
//! | | `Leverage` | PRODUCT (×) | Equity × Multiplier → Exposure |
//! | | `Diversify` | SUM (Σ) | Positions → Reduced risk |
//!
//! **T1 Distribution**: MAPPING (14) · SEQUENCE (6) · RECURSION (6) · STATE (4) · PERSISTENCE (2) · BOUNDARY (2) · SUM (2) · QUANTITY (1) · CAUSALITY (1) · COMPARISON (1) · IRREVERSIBILITY (1) · PRODUCT (1) = 41 traits across 12 T1 primitives
//!
//! ## Three Unfixable Limits (All Domains)
//!
//! 1. **Heisenberg**: Observation alters the observed
//! 2. **Gödel**: No system proves its own consistency
//! 3. **Shannon**: Codification has irreducible loss

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

// ============================================================================
// Derive Macros
// ============================================================================

/// Derive macros for building T2-P newtypes
pub use stem_derive::StemNewtype;

// ============================================================================
// Inline Modules (formerly separate crates)
// ============================================================================

/// Core scientific method primitives (7 traits)
pub mod core;

/// Biology primitives (endocrine system, organism behavior)
pub mod bio;

/// Chemistry primitives (9 traits: reactions, equilibrium, binding)
pub mod chem;

/// Physics primitives (7 traits: conservation, force, oscillation)
pub mod phys;

/// Mathematics primitives (9 traits: sets, proofs, relations, bounds)
pub mod math;

/// Finance primitives (9 traits: valuation, flow, discounting, compounding,
/// hedging, arbitrage, maturity, leverage, diversification)
pub mod finance;

/// Lex Primitiva GroundsTo implementations for finance types
pub mod grounding;

// ============================================================================
// Prelude
// ============================================================================

/// Prelude: import everything commonly needed
///
/// ```rust
/// use stem::prelude::*;
/// ```
pub mod prelude {
    // Core types and traits
    pub use crate::core::{
        Classify, Codify, Confidence, Correction, Experiment, Extend, Infer, Measured, Normalize,
        Science, ScienceError, Sense, Tier,
    };

    // Chemistry types and traits
    pub use crate::chem::{
        Affinity, Balance, Chemistry, ChemistryError, Concentrate, Energize, Fraction, Harmonize,
        Interact, MeasuredRate, MeasuredRatio, Modulate, Rate, Ratio, Regulate, Saturate,
        Transform, Yield,
    };

    // Physics types and traits
    pub use crate::phys::{
        Acceleration, Amplitude, Couple, Force, Frequency, Harmonics, Inertia, Mass, MeasuredForce,
        MeasuredQuantity, Physics, PhysicsError, Preserve, Quantity, Scale, ScaleFactor, Superpose,
        YieldForce,
    };

    // Mathematics types and traits
    pub use crate::math::{
        Associate, Bound, Bounded, Commute, Homeomorph, Identify, Identity, MathError, Mathematics,
        MeasuredBound, Membership, Proof, Prove, Relation, Symmetric, Transit,
    };

    // Finance types and traits
    pub use crate::finance::{
        Appraise, Arbitrage, Compound, Discount, Diversify, Exposure, Finance, FinanceError, Flow,
        Hedge, InterestRate, Leverage, Mature, Maturity, MeasuredPrice, MeasuredReturn, Price,
        Return, Spread, TimeValueOfMoney,
    };
}

// ============================================================================
// Version Info
// ============================================================================

/// STEM version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Number of T2-P traits across all domains
pub const TRAIT_COUNT: usize = 43; // 8 SCIENCE + 9 CHEMISTRY + 8 PHYSICS + 9 MATHS + 9 FINANCE

/// Number of STEM domains
pub const DOMAIN_COUNT: usize = 5; // core, bio, chem, phys, math, finance (bio has no composite trait)

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_imports_core_types() {
        let c = Confidence::new(0.8);
        assert!((c.value() - 0.8).abs() < f64::EPSILON);

        let t = Tier::T2Primitive;
        assert!((t.transfer_multiplier() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn prelude_imports_chemistry() {
        let r = Ratio::new(2.5);
        assert!((r.value() - 2.5).abs() < f64::EPSILON);

        let f = Fraction::new(0.99);
        assert!(f.is_saturated());
    }

    #[test]
    fn prelude_imports_physics() {
        let q = Quantity::new(100.0);
        assert!((q.value() - 100.0).abs() < f64::EPSILON);

        let f = Force::new(10.0);
        assert!((f.magnitude() - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn prelude_imports_mathematics() {
        let b = Bounded::new(5, Some(0), Some(10));
        assert!(b.in_bounds());

        let r = Relation::LessThan;
        assert_eq!(r.invert(), Relation::GreaterThan);
    }

    #[test]
    fn prelude_imports_finance() {
        let p = Price::new(100.0);
        assert!((p.value() - 100.0).abs() < f64::EPSILON);

        let r = InterestRate::new(0.05);
        assert!(r.is_some());

        let s = Spread::from_bid_ask(Price::new(99.0), Price::new(101.0));
        assert!((s.value() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn version_and_counts() {
        assert!(!super::VERSION.is_empty());
        assert_eq!(super::TRAIT_COUNT, 43);
        assert_eq!(super::DOMAIN_COUNT, 5);
    }

    #[test]
    fn cross_domain_confidence_composition() {
        // Same Confidence type used across all domains
        let science_conf = Confidence::new(0.9);
        let chem_measured = MeasuredRatio::new(Ratio::new(2.0), science_conf);
        let phys_measured = MeasuredQuantity::new(Quantity::new(100.0), Confidence::new(0.85));
        let math_measured = MeasuredBound::new(
            Bounded::new(5.0, Some(0.0), Some(10.0)),
            Confidence::new(0.95),
        );

        // All share Confidence from stem-core
        assert!((chem_measured.confidence.value() - 0.9).abs() < f64::EPSILON);
        assert!((phys_measured.confidence.value() - 0.85).abs() < f64::EPSILON);
        assert!((math_measured.confidence.value() - 0.95).abs() < f64::EPSILON);
    }
}
