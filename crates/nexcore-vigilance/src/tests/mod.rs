//! Test modules for nexcore-vigilance
//!
//! ## CTVP Phase 1: Chaos Engineering Tests
//!
//! The `chaos` module provides fault injection testing infrastructure for verifying
//! graceful degradation under adverse conditions.
//!
//! Enable with: `cargo test --features chaos-tests -p nexcore-vigilance`
//!
#[cfg(all(test, feature = "chaos-tests"))]
pub mod chaos;
