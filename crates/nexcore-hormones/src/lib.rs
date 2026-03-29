//! # NexVigilant Core — Hormones - Endocrine System for .claude
//!
//! Persistent state modulators affecting system behavior across sessions.
//!
//! Type definitions and Lex Primitiva grounding live in [`nexcore_hormone_types`]
//! (stem-foundation layer). This crate re-exports all types for backward
//! compatibility with existing bio-molecular consumers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Re-export all types and grounding from the Foundation-layer types crate.
// Downstream consumers that depend on nexcore-hormones continue to work unchanged.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]
pub use nexcore_hormone_types::*;
