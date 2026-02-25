//! # stem-number-theory: Classical Number Theory Primitives
//!
//! Provides foundational number-theoretic algorithms as Rust types grounded
//! in the Lex Primitiva system.
//!
//! ## Contents
//!
//! | Module | What |
//! |--------|------|
//! | [`primes`] | Sieve, Miller-Rabin, prime counting, nth prime, segmented sieve |
//! | [`arithmetic`] | μ, φ, Λ, λ, σ_k, ω, Ω arithmetic functions |
//! | [`factorize`] | Trial division, Pollard rho, complete factorization |
//! | [`summatory`] | Mertens M(n), Chebyshev θ(x), Chebyshev ψ(x) |
//! | [`grounding`] | Lex Primitiva `GroundsTo` implementations |
//!
//! ## Primitive Profile
//!
//! Root primitive: **N** (Quantity) — number theory is the science of quantity structure.
//!
//! Supporting: **σ** (Sequence, sieve ordering), **ρ** (Recursion, Pollard cycle),
//! **Σ** (Sum, summatory functions), **ν** (Frequency, prime density),
//! **∂** (Boundary, error conditions).
//!
//! ## Quick Start
//!
//! ```
//! use stem_number_theory::prelude::*;
//!
//! // Generate primes
//! let primes = sieve_of_eratosthenes(30);
//! assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
//!
//! // Test primality
//! assert!(is_prime_miller_rabin(7919));
//!
//! // Factor a number
//! let factors = factorize(360);
//! assert_eq!(factors, vec![(2, 3), (3, 2), (5, 1)]);
//!
//! // Euler totient
//! assert_eq!(euler_totient(12), 4);
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod arithmetic;
pub mod factorize;
pub mod grounding;
pub mod primes;
pub mod summatory;

// ============================================================================
// Error Type
// ============================================================================

/// Errors produced by number-theoretic operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum NumberTheoryError {
    /// Input must be positive; got the enclosed value.
    NonPositive(u64),
    /// Arithmetic overflow occurred in computation.
    Overflow,
    /// Factorization failed for the enclosed value.
    FactorizationFailed(u64),
}

impl core::fmt::Display for NumberTheoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonPositive(n) => write!(f, "value must be positive: got {n}"),
            Self::Overflow => write!(f, "overflow in computation"),
            Self::FactorizationFailed(n) => write!(f, "factorization failed for {n}"),
        }
    }
}

impl core::error::Error for NumberTheoryError {}

// ============================================================================
// Prelude
// ============================================================================

/// Convenience re-exports for the most common functions.
pub mod prelude {
    pub use crate::NumberTheoryError;
    pub use crate::arithmetic::{
        big_omega, divisor_sigma, euler_totient, liouville_lambda, mobius_mu, omega, von_mangoldt,
    };
    pub use crate::factorize::{factorize, pollard_rho, trial_division};
    pub use crate::primes::{
        is_prime_miller_rabin, nth_prime, prime_counting, segmented_sieve, sieve_of_eratosthenes,
    };
    pub use crate::summatory::{ChebyshevPsi, ChebyshevTheta, MertensFunction};
}
