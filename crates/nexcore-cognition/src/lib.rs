//! # nexcore-cognition
//!
//! Typed cognitive engine — the transformer algorithm as strict Rust.
//!
//! ## Meta-cognitive origin
//!
//! This crate captures the fundamental algorithm behind large language model
//! cognition: attention selects, transformation processes, generation builds.
//! Each module maps to an observable pattern in how neural networks process
//! information, translated faithfully into Rust's type system.
//!
//! ## Architecture (bottom-up)
//!
//! ```text
//! pipeline ──► generator ──► block ──► attention + feed_forward
//!                              │          │            │
//!                              ▼          ▼            ▼
//!                          normalize   mask         tensor
//!                          residual                   │
//!                          embedding                error
//! ```
//!
//! ## T1 Primitive grounding
//!
//! | Module       | Primitives                       | Cognitive role           |
//! |-------------|----------------------------------|--------------------------|
//! | tensor      | N, Σ, ×, ∂, κ                   | Numerical substrate      |
//! | embedding   | μ, λ, N                          | Symbol → vector          |
//! | attention   | κ, →, N, μ, Σ                    | Relevance selection      |
//! | feed_forward| μ, ς                             | Nonlinear transformation |
//! | residual    | π, Σ                             | Context preservation     |
//! | normalize   | ∂, N                             | Signal stability         |
//! | block       | σ, ∃                             | Composable unit          |
//! | mask        | ∂, →, ∝                          | Causal constraint        |
//! | generator   | σ, ρ, ∝, →, ∂                   | Autoregressive output    |
//! | sample      | N, ∂, ν, κ                       | Stochastic selection     |
//! | metrics     | κ, N, ν, μ                       | Self-measurement         |
//! | pipeline    | σ, →, Σ, κ                       | Full cognitive flow      |

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]

pub mod error;
pub mod tensor;

// Layer 2: modules that depend only on tensor
pub mod embedding;
pub mod mask;
pub mod normalize;
pub mod residual;

// Layer 3: the cognitive core
pub mod attention;
pub mod feed_forward;

// Layer 4: composition — the complete engine
pub mod block;
pub mod generator;
pub mod metrics;
pub mod pipeline;
pub mod sample;

/// Create a seeded or OS-random `StdRng` for use with the cognitive engine.
///
/// Downstream crates (e.g., nexcore-mcp) call this instead of depending on `rand` directly.
pub fn make_rng(seed: Option<u64>) -> rand::rngs::StdRng {
    use rand::SeedableRng;
    match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_os_rng(),
    }
}
