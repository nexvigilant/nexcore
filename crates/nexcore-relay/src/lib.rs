//! # nexcore-relay — Relay Trait System
//!
//! Behavioral layer for relay theory (Cluster 6: Rust Implementation).
//! Provides the `Relay<I,O>` trait, `RelayOutcome<T>`, `FidelityBound`,
//! `RelayChain`, and `RelayStrategy` — the typed processing surface over
//! the measurement primitives in `nexcore-primitives::relay`.
//!
//! ## Layer Relationship
//!
//! ```text
//! nexcore-relay          ← behavioral (this crate): traits, composition, strategy
//!   └── nexcore-primitives::relay  ← structural: Fidelity, RelayHop, RelayChain (data)
//! ```
//!
//! ## Five-Axiom Enforcement
//!
//! | Axiom | Enforcement Point |
//! |-------|------------------|
//! | A1 Directionality | `Relay<I,O>` signature — unidirectional by type |
//! | A2 Mediation | Every `impl Relay` IS the required intermediary |
//! | A3 Preservation | `RelayOutcome::Forwarded` + `FidelityBound::meets_minimum()` |
//! | A4 Threshold | `RelayOutcome::Filtered` for subthreshold inputs |
//! | A5 Boundedness | Type parameters `I`/`O` encode boundary crossing |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_relay::relay::Relay;
//! use nexcore_relay::outcome::RelayOutcome;
//! use nexcore_relay::strategy::RelayStrategy;
//!
//! struct SignalFilter { threshold: f64 }
//!
//! impl Relay<f64, f64> for SignalFilter {
//!     fn process(&self, input: f64) -> RelayOutcome<f64> {
//!         if input < self.threshold {
//!             RelayOutcome::Filtered
//!         } else {
//!             RelayOutcome::Forwarded(input)
//!         }
//!     }
//!     fn min_fidelity(&self) -> f64 { 0.95 }
//!     fn stage_name(&self) -> &str { "signal_filter" }
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

pub mod chain;
pub mod fidelity;
pub mod outcome;
pub mod relay;
pub mod strategy;

#[cfg(test)]
mod tests;

// Top-level re-exports for ergonomic imports.
pub use chain::RelayChain;
pub use fidelity::{FidelityBound, FidelityMetrics};
pub use outcome::RelayOutcome;
pub use relay::Relay;
pub use strategy::RelayStrategy;
