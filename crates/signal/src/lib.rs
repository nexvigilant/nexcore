//! # signal — Thin Re-export Wrapper
//!
//! This crate re-exports [`nexcore_signal_pipeline`] — the canonical signal
//! detection pipeline implementation.
//!
//! ## History
//!
//! `signal` was the original pipeline (13 modules). `nexcore-signal-pipeline`
//! evolved into the full-featured version with the additional `relay` module
//! for fidelity-tracked signal transport.
//!
//! This crate is retained for backward compatibility. New code should depend
//! on `nexcore-signal-pipeline` directly.

#![forbid(unsafe_code)]

// NOTE: grounding.rs exists but impls live in canonical nexcore-signal-pipeline crate
// (orphan rules prevent implementing GroundsTo on re-exported foreign types here)

pub use nexcore_signal_pipeline::*;
