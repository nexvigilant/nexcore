//! # Lex Primitiva Symbols as Constants
//!
//! This module provides all 16 Lex Primitiva symbols and 10 mathematical constants
//! as `pub const` strings for internal mapping and CLI display.

// ══════════════════════════════════════════════════════════════════════════════
// Lex Primitiva T1 Symbols
// ══════════════════════════════════════════════════════════════════════════════

/// σ (sigma) - Ordered succession (Sequence)
pub const SIGMA_SEQ: &str = "σ";
/// μ (mu) - Transformation A → B (Mapping)
pub const MU_MAP: &str = "μ";
/// ς (varsigma) - Context at a point in time (State)
pub const VARSIGMA_STATE: &str = "ς";
/// ρ (rho) - Self-reference via indirection (Recursion)
pub const RHO_REC: &str = "ρ";
/// ∅ (empty set) - Meaningful absence (Void)
pub const EMPTYSET_VOID: &str = "∅";
/// ∂ (partial) - Delimiters and limits (Boundary)
pub const PARTIAL_BOUND: &str = "∂";
/// ν (nu) - Rate of occurrence (Frequency)
pub const NU_FREQ: &str = "ν";
/// ∃ (exists) - Instantiation of being (Existence)
pub const EXISTS_INST: &str = "∃";
/// π (pi) - Continuity across time (Persistence)
pub const PI_PERSIST: &str = "π";
/// → (arrow) - Cause and consequence (Causality)
pub const ARROW_CAUSAL: &str = "→";
/// κ (kappa) - Predicate matching (Comparison)
pub const KAPPA_COMP: &str = "κ";
/// N - Numerical magnitude (Quantity)
pub const N_QUANT: &str = "N";
/// λ (lambda) - Positional context (Location)
pub const LAMBDA_LOC: &str = "λ";
/// ∝ (proportional) - One-way state transition (Irreversibility)
pub const PROPTORTO_IRREV: &str = "∝";
/// Σ (sigma) - Exclusive disjunction (Sum)
pub const SIGMA_SUM: &str = "Σ";
/// × (times) - Conjunctive combination (Product)
pub const CROSS_PROD: &str = "×";

// ══════════════════════════════════════════════════════════════════════════════
// Mathematical Constants
// ══════════════════════════════════════════════════════════════════════════════

/// 0 - Zero (Absence)
pub const CONST_ZERO: &str = "0";
/// 1 - One (Existence)
pub const CONST_ONE: &str = "1";
/// π - Pi (Cycle)
pub const CONST_PI: &str = "π";
/// e - Euler's Number (Growth)
pub const CONST_E: &str = "e";
/// φ - Golden Ratio (Self-similarity)
pub const CONST_PHI: &str = "φ";
/// ∞ - Infinity (Unbounded)
pub const CONST_INFINITY: &str = "∞";
/// ω - Omega (Transfinite)
pub const CONST_OMEGA: &str = "ω";
/// ln(2) - Natural Log of 2 (Information)
pub const CONST_LN2: &str = "ln(2)";
/// kᵦ - Boltzmann Constant (Entropy)
pub const CONST_KB: &str = "kᵦ";
/// ε - Epsilon (Infinitesimal)
pub const CONST_EPSILON: &str = "ε";

/// Returns all 16 Lex Primitiva symbols as an array.
pub const fn all_symbols() -> [&'static str; 16] {
    [
        SIGMA_SEQ,
        MU_MAP,
        VARSIGMA_STATE,
        RHO_REC,
        EMPTYSET_VOID,
        PARTIAL_BOUND,
        NU_FREQ,
        EXISTS_INST,
        PI_PERSIST,
        ARROW_CAUSAL,
        KAPPA_COMP,
        N_QUANT,
        LAMBDA_LOC,
        PROPTORTO_IRREV,
        SIGMA_SUM,
        CROSS_PROD,
    ]
}

/// Returns all 10 math constants as an array.
pub const fn all_constants() -> [&'static str; 10] {
    [
        CONST_ZERO,
        CONST_ONE,
        CONST_PI,
        CONST_E,
        CONST_PHI,
        CONST_INFINITY,
        CONST_OMEGA,
        CONST_LN2,
        CONST_KB,
        CONST_EPSILON,
    ]
}
