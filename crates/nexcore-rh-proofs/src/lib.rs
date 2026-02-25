//! # nexcore-rh-proofs
//!
//! Type-level Riemann Hypothesis proof infrastructure via Curry-Howard
//! correspondence.
//!
//! Encodes RH and its equivalences as Rust types.  Provides numerical
//! verification certificates that bridge computational evidence to the
//! proof-theoretic structure.
//!
//! ## Honest Boundaries
//!
//! This crate provides exploration infrastructure, **NOT** a proof of RH.
//!
//! - f64 precision limits verification to zeros up to height ~10^6
//! - Rust lacks dependent types — uninhabited types like `RiemannHypothesis`
//!   encode open conjectures but cannot be inhabited without a formal proof
//! - Numerical certificates are evidence, not proof
//!
//! ## What We CAN Do
//!
//! - Verify PNT constructively (it is an established theorem)
//! - Verify RH-equivalent statements (Robin, Mertens) to finite bounds
//! - Construct numerical certificates of partial zero verification
//! - Encode the logical structure of RH and its consequences as types
//! - Ground all types in the Lex Primitiva T1 primitive system
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_rh_proofs::consequences::verify_pnt;
//! use nexcore_rh_proofs::equivalences::{test_robin, test_mertens};
//!
//! // PNT is proven — we can always construct this witness
//! let pnt = verify_pnt(10_000).unwrap();
//! assert_eq!(pnt.pi_x, 1229);
//!
//! // Robin's inequality: numerical RH evidence at n = 5041
//! let robin = test_robin(5041).unwrap();
//! assert!(robin.satisfies);
//!
//! // Mertens bound: |M(100)| < √100
//! let mertens = test_mertens(100).unwrap();
//! assert!(mertens.satisfies);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![warn(missing_docs)]

pub mod bridge;
pub mod certificates;
pub mod consequences;
pub mod equivalences;
pub mod error;
pub mod grounding;
pub mod propositions;

pub use bridge::{build_certificate_from_zeta, convert_zeros};
pub use certificates::{NumericalCertificate, NumericallyVerified};
pub use consequences::{ChebyshevBoundsWitness, SharpPntWitness};
pub use equivalences::{MertensTest, RobinTest};
pub use error::RhProofError;
pub use propositions::{
    MertensBound, PrimeNumberTheorem, RhImpliesSharpPnt, RhVerifiedToHeight, RiemannHypothesis,
    RobinsInequality, ZeroOnCriticalLine,
};
