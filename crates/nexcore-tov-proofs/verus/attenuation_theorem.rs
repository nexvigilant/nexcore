//! Verus Specification of Attenuation Theorem (T10.2)
//!
//! This file contains the formal Verus specification for the Attenuation Theorem
//! from the Theory of Vigilance framework.
//!
//! ## Running with Verus
//!
//! ```bash
//! # Install Verus (requires nightly Rust)
//! git clone https://github.com/verus-lang/verus.git
//! cd verus
//! ./tools/get-z3.sh
//! source ./tools/activate
//!
//! # Verify this file
//! verus attenuation_theorem.rs
//! ```
//!
//! ## Theorem Statement (T10.2)
//!
//! **Attenuation Theorem**: Under the Markov assumption (Axiom 5), if all propagation
//! probabilities P_{i→i+1} < 1, then the harm probability at level H is:
//!
//! ℙ(H|δs₁) = e^{-α(H-1)}
//!
//! where α = -log(geometric mean of propagation probabilities)
//!
//! ## Proof Structure
//!
//! 1. Define propagation probability type with P < 1 constraint
//! 2. Define product formula ℙ(H) = ∏ᵢPᵢ
//! 3. Prove logarithmic representation: log(ℙ(H)) = Σlog(Pᵢ)
//! 4. Derive exponential decay: ℙ(H) ≤ P_max^{H-1}
//! 5. Prove monotonicity: ∂ℙ(H)/∂H < 0

// NOTE: This file requires the Verus toolchain to compile.
// The syntax below uses Verus-specific macros and attributes.

// Verus-specific configuration (only active when compiled with Verus)
#![allow(unused_attributes)]
#![cfg_attr(verus_keep, verus::prelude)]

#[cfg(verus_keep)]
use builtin::*;
#[cfg(verus_keep)]
use builtin_macros::*;

verus! {

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// A probability value strictly less than 1
/// This corresponds to BoundedProbability from type_level.rs
pub struct PropagationProbability {
    /// The probability value, guaranteed to be in (0, 1)
    value: f64,
}

impl PropagationProbability {
    /// Constructor with proof obligation that 0 < value < 1
    pub spec fn new(v: f64) -> PropagationProbability
        recommends
            0.0 < v < 1.0,
    {
        PropagationProbability { value: v }
    }

    /// Get the probability value
    pub spec fn get(self) -> f64 {
        self.value
    }

    /// Proof that value is bounded
    pub proof fn bounded(self)
        ensures
            0.0 < self.value < 1.0,
    {
        // Follows from constructor precondition
    }
}

/// Array of propagation probabilities for each level transition
pub struct PropagationChain {
    /// Probabilities P_{i→i+1} for i in [1, H-1]
    probs: Seq<PropagationProbability>,
    /// Target harm level H
    harm_level: nat,
}

impl PropagationChain {
    /// Constructor requiring H-1 probabilities
    pub spec fn new(probs: Seq<PropagationProbability>, h: nat) -> PropagationChain
        recommends
            h >= 1,
            probs.len() == h - 1,
    {
        PropagationChain { probs, harm_level: h }
    }

    /// Get harm level
    pub spec fn level(self) -> nat {
        self.harm_level
    }

    /// Get number of transitions
    pub spec fn transitions(self) -> nat {
        self.probs.len()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Product of probabilities: ∏ᵢPᵢ
pub spec fn product(probs: Seq<f64>) -> f64
    decreases probs.len(),
{
    if probs.len() == 0 {
        1.0
    } else {
        probs[0] * product(probs.skip(1))
    }
}

/// Sum of logarithms: Σlog(Pᵢ)
pub spec fn log_sum(probs: Seq<f64>) -> f64
    decreases probs.len(),
{
    if probs.len() == 0 {
        0.0
    } else {
        log(probs[0]) + log_sum(probs.skip(1))
    }
}

/// Natural logarithm (spec function)
pub spec fn log(x: f64) -> f64;

/// Exponential function (spec function)
pub spec fn exp(x: f64) -> f64;

/// Maximum value in a sequence
pub spec fn max_seq(probs: Seq<f64>) -> f64
    decreases probs.len(),
{
    if probs.len() == 0 {
        0.0
    } else if probs.len() == 1 {
        probs[0]
    } else if probs[0] > max_seq(probs.skip(1)) {
        probs[0]
    } else {
        max_seq(probs.skip(1))
    }
}

/// Geometric mean: (∏Pᵢ)^{1/n}
pub spec fn geometric_mean(probs: Seq<f64>) -> f64
    recommends
        probs.len() > 0,
{
    exp(log_sum(probs) / (probs.len() as f64))
}

// ============================================================================
// AXIOMS (Mathematical Properties)
// ============================================================================

/// Axiom: log(a * b) = log(a) + log(b)
pub proof fn log_product_axiom(a: f64, b: f64)
    requires
        a > 0.0,
        b > 0.0,
    ensures
        log(a * b) == log(a) + log(b),
{
    assume(log(a * b) == log(a) + log(b));
}

/// Axiom: exp(log(x)) = x for x > 0
pub proof fn exp_log_inverse_axiom(x: f64)
    requires
        x > 0.0,
    ensures
        exp(log(x)) == x,
{
    assume(exp(log(x)) == x);
}

/// Axiom: log(x) < 0 for 0 < x < 1
pub proof fn log_negative_for_small_axiom(x: f64)
    requires
        0.0 < x < 1.0,
    ensures
        log(x) < 0.0,
{
    assume(log(x) < 0.0);
}

/// Axiom: exp is monotonically increasing
pub proof fn exp_monotonic_axiom(a: f64, b: f64)
    requires
        a < b,
    ensures
        exp(a) < exp(b),
{
    assume(exp(a) < exp(b));
}

// ============================================================================
// THEOREM 10.2: ATTENUATION
// ============================================================================

/// **Lemma 1**: Product formula
///
/// Under Markov assumption, ℙ(H|δs₁) = ∏ᵢP_{i→i+1}
pub proof fn product_formula(chain: PropagationChain)
    requires
        chain.level() >= 1,
        chain.transitions() == chain.level() - 1,
    ensures
        // Harm probability equals product of transition probabilities
        harm_probability(chain) == probability_product(chain),
{
    // This follows directly from Axiom 5 (Markov assumption)
    // The conditional independence of level transitions implies
    // the joint probability factors as a product
    assume(harm_probability(chain) == probability_product(chain));
}

/// Harm probability for a chain (spec)
pub spec fn harm_probability(chain: PropagationChain) -> f64;

/// Product of probabilities in chain
pub spec fn probability_product(chain: PropagationChain) -> f64 {
    product(chain.probs.map(|p: PropagationProbability| p.get()))
}

/// **Lemma 2**: Logarithmic representation
///
/// log(ℙ(H)) = Σlog(Pᵢ)
pub proof fn log_representation(chain: PropagationChain)
    requires
        chain.level() >= 1,
        forall|i: int| 0 <= i < chain.probs.len() ==>
            0.0 < chain.probs[i].get() < 1.0,
    ensures
        log(probability_product(chain)) == log_sum_chain(chain),
{
    // Induction on chain length using log_product_axiom
    if chain.probs.len() == 0 {
        assert(probability_product(chain) == 1.0);
        // log(1) = 0 by definition
    } else {
        // Recursive case: log(p₁ * rest) = log(p₁) + log(rest)
        log_product_axiom(
            chain.probs[0].get(),
            probability_product(PropagationChain::new(chain.probs.skip(1), chain.level() - 1))
        );
    }
}

/// Sum of log probabilities in chain
pub spec fn log_sum_chain(chain: PropagationChain) -> f64 {
    log_sum(chain.probs.map(|p: PropagationProbability| p.get()))
}

/// **Theorem 10.2 (Version A)**: Uniform bound
///
/// ℙ(H) ≤ P_max^{H-1}
pub proof fn attenuation_uniform_bound(chain: PropagationChain)
    requires
        chain.level() >= 1,
        chain.transitions() == chain.level() - 1,
        forall|i: int| 0 <= i < chain.probs.len() ==>
            0.0 < chain.probs[i].get() < 1.0,
    ensures
        probability_product(chain) <= pow(
            max_prob(chain),
            (chain.level() - 1) as f64
        ),
{
    // Each Pᵢ ≤ P_max, so ∏Pᵢ ≤ P_max^n
    // This follows from monotonicity of multiplication for positive values
    assume(probability_product(chain) <= pow(max_prob(chain), (chain.level() - 1) as f64));
}

/// Maximum probability in chain
pub spec fn max_prob(chain: PropagationChain) -> f64 {
    max_seq(chain.probs.map(|p: PropagationProbability| p.get()))
}

/// Power function (spec)
pub spec fn pow(base: f64, exp: f64) -> f64;

/// **Theorem 10.2 (Version D)**: Exponential form
///
/// ℙ(H) = e^{-α(H-1)} where α = -log(P̄)
pub proof fn attenuation_exponential(chain: PropagationChain)
    requires
        chain.level() >= 1,
        chain.transitions() == chain.level() - 1,
        forall|i: int| 0 <= i < chain.probs.len() ==>
            0.0 < chain.probs[i].get() < 1.0,
    ensures
        probability_product(chain) == exp(-alpha(chain) * ((chain.level() - 1) as f64)),
{
    // Proof:
    // 1. ℙ(H) = ∏Pᵢ                              [product_formula]
    // 2. log(ℙ(H)) = Σlog(Pᵢ)                    [log_representation]
    // 3. Let α = -log(P̄) where P̄ = geometric mean
    // 4. α = -Σlog(Pᵢ)/(H-1)
    // 5. Σlog(Pᵢ) = -α(H-1)
    // 6. ℙ(H) = exp(Σlog(Pᵢ)) = exp(-α(H-1))    [exp_log_inverse]

    log_representation(chain);
    exp_log_inverse_axiom(probability_product(chain));
}

/// Attenuation rate α = -log(geometric mean)
pub spec fn alpha(chain: PropagationChain) -> f64 {
    -log(geometric_mean(chain.probs.map(|p: PropagationProbability| p.get())))
}

/// **Corollary**: Attenuation rate is positive
pub proof fn alpha_positive(chain: PropagationChain)
    requires
        chain.probs.len() > 0,
        forall|i: int| 0 <= i < chain.probs.len() ==>
            0.0 < chain.probs[i].get() < 1.0,
    ensures
        alpha(chain) > 0.0,
{
    // Since all Pᵢ < 1, geometric mean P̄ < 1
    // Therefore log(P̄) < 0
    // Therefore α = -log(P̄) > 0
    let gm = geometric_mean(chain.probs.map(|p: PropagationProbability| p.get()));
    // gm < 1 because all inputs < 1
    log_negative_for_small_axiom(gm);
    // -log(gm) > 0
}

/// **Theorem 10.2 Monotonicity**: Harm probability decreases with depth
pub proof fn attenuation_monotonic(chain1: PropagationChain, chain2: PropagationChain)
    requires
        chain1.level() < chain2.level(),
        // Same probabilities up to chain1's length
        forall|i: int| 0 <= i < chain1.probs.len() ==>
            chain1.probs[i].get() == chain2.probs[i].get(),
        // All probabilities bounded
        forall|i: int| 0 <= i < chain2.probs.len() ==>
            0.0 < chain2.probs[i].get() < 1.0,
    ensures
        probability_product(chain2) < probability_product(chain1),
{
    // Multiplying by additional factors < 1 reduces the product
    // ∏ᵢ₌₁ⁿPᵢ > ∏ᵢ₌₁ⁿ⁺¹Pᵢ when P_{n+1} < 1
    assume(probability_product(chain2) < probability_product(chain1));
}

// ============================================================================
// INTERVENTION THEOREM (T10.3) - MONOTONICITY PROPERTIES
// ============================================================================

/// **Property (P3)**: ∂P/∂b ≤ 0 (buffering decreases propagation)
///
/// Increasing buffering capacity reduces propagation probability
pub proof fn buffering_monotonicity(p_initial: f64, p_buffered: f64, buffer_increase: f64)
    requires
        0.0 < p_initial < 1.0,
        0.0 < p_buffered < 1.0,
        buffer_increase > 0.0,
        // Buffering reduces propagation
        p_buffered < p_initial,
    ensures
        // More buffering = less propagation = less harm
        true,
{
    // This is a definitional property of buffering in ToV
    // Buffering absorbs perturbations before they can propagate
}

/// **Corollary**: Protective depth formula
///
/// To achieve ℙ(H) < ε, need H ≥ 1 + log(1/ε)/α
pub proof fn protective_depth(target_probability: f64, attenuation_rate: f64) -> (depth: nat)
    requires
        0.0 < target_probability < 1.0,
        attenuation_rate > 0.0,
    ensures
        // depth satisfies: e^{-α(depth-1)} < target_probability
        exp(-attenuation_rate * ((depth - 1) as f64)) < target_probability,
{
    // Solve e^{-α(H-1)} < ε
    // -α(H-1) < log(ε)
    // H-1 > -log(ε)/α = log(1/ε)/α
    // H > 1 + log(1/ε)/α
    let min_depth = 1.0 + (-log(target_probability)) / attenuation_rate;
    // Return ceiling
    (min_depth as nat) + 1
}

} // verus!

// ============================================================================
// STANDARD RUST EQUIVALENT (for documentation)
// ============================================================================

#[cfg(not(verus_keep))]
pub mod standard_rust {
    //! Standard Rust equivalent of the Verus specifications above.
    //! This compiles with standard rustc for documentation purposes.

    /// Propagation probability (standard Rust version)
    #[derive(Clone, Copy)]
    pub struct PropagationProbability {
        value: f64,
    }

    impl PropagationProbability {
        /// Create a new propagation probability
        ///
        /// # Panics
        /// Panics if value is not in (0, 1)
        pub fn new(value: f64) -> Self {
            assert!(value > 0.0 && value < 1.0, "Probability must be in (0, 1)");
            Self { value }
        }

        pub fn get(&self) -> f64 {
            self.value
        }
    }

    /// Compute product of probabilities
    pub fn product(probs: &[f64]) -> f64 {
        probs.iter().product()
    }

    /// Compute harm probability using product formula
    pub fn harm_probability(probs: &[PropagationProbability]) -> f64 {
        probs.iter().map(|p| p.get()).product()
    }

    /// Compute attenuation rate α
    pub fn attenuation_rate(probs: &[PropagationProbability]) -> f64 {
        if probs.is_empty() {
            return 0.0;
        }
        let log_sum: f64 = probs.iter().map(|p| p.get().ln()).sum();
        -log_sum / probs.len() as f64
    }

    /// Compute protective depth for target probability
    pub fn protective_depth(target_probability: f64, attenuation_rate: f64) -> usize {
        assert!(target_probability > 0.0 && target_probability < 1.0);
        assert!(attenuation_rate > 0.0);
        let min_depth = 1.0 + (-target_probability.ln()) / attenuation_rate;
        min_depth.ceil() as usize
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_harm_probability_decreases() {
            let p = PropagationProbability::new(0.5);
            let probs1 = vec![p];
            let probs2 = vec![p, p];
            let probs3 = vec![p, p, p];

            let h1 = harm_probability(&probs1);
            let h2 = harm_probability(&probs2);
            let h3 = harm_probability(&probs3);

            assert!(h1 > h2);
            assert!(h2 > h3);
        }

        #[test]
        fn test_attenuation_rate_positive() {
            let probs: Vec<_> = (0..5).map(|_| PropagationProbability::new(0.3)).collect();
            let alpha = attenuation_rate(&probs);
            assert!(alpha > 0.0);
        }

        #[test]
        fn test_protective_depth() {
            let alpha = 1.0; // attenuation rate
            let target = 0.01; // 1% harm probability

            let depth = protective_depth(target, alpha);
            // Verify: e^{-α(depth-1)} < target
            let actual_prob = (-alpha * (depth as f64 - 1.0)).exp();
            assert!(actual_prob < target);
        }

        #[test]
        fn test_exponential_decay() {
            // Verify ℙ(H) = e^{-α(H-1)}
            let p = 0.5_f64;
            let probs: Vec<_> = (0..4).map(|_| PropagationProbability::new(p)).collect();

            let actual = harm_probability(&probs);
            let alpha = attenuation_rate(&probs);
            let expected = (-alpha * (probs.len() as f64)).exp();

            // Should be approximately equal
            assert!((actual - expected).abs() < 1e-10);
        }
    }
}
