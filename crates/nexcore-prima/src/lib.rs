// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-prima — Thin Re-export Wrapper
//!
//! This crate re-exports [`prima`] — the canonical Prima language implementation.
//!
//! ## History
//!
//! `nexcore-prima` was the original interpreter (10 modules). The `prima` crate
//! evolved into the full-featured implementation (30+ modules) with bytecode VM,
//! type inference, effect system, and optimization passes.
//!
//! This crate is retained for backward compatibility. New code should depend on
//! `prima` directly.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

// NOTE: grounding.rs exists but impls live in canonical prima crate
// (orphan rules prevent implementing GroundsTo on re-exported foreign types here)

pub use prima::*;
