//! # NexVigilant Core — sop-anatomy
//!
//! SOP-Anatomy-Code triple mapping: 18 governance sections through biological
//! anatomy to software code structures. Implements the I.R.O.N.M.A.N. reactor
//! (15 chemistry operations across 7 phases) and the Capability Transfer Protocol.
//!
//! ## Triple Mapping
//!
//! Each SOP section has a governance function. That function has an exact analog
//! in a body system (anatomy). That body system maps to a code structure. The T1
//! primitive is the invariant across all three domains.
//!
//! ```text
//! SOP Section ──[T1 primitive]──> Anatomical System ──[same T1]──> Code Structure
//! ```
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Dominant |
//! |------|------|----------|
//! | `SopSection` | T2-P | Σ (Sum) — 18-variant enum |
//! | `SectionMapping` | T3 | σ (Sequence) + μ (Mapping) |
//! | `ChemOp` | T2-P | Σ (Sum) — 15-variant enum |
//! | `IronmanPhase` | T2-P | σ (Sequence) — 7-step pipeline |
//! | `TransferResult` | T3 | μ (Mapping) + κ (Comparison) |
//!
//! Root primitives: **σ** (Sequence) + **μ** (Mapping)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod audit;
pub mod grounding;
pub mod mapping;
pub mod reactor;
pub mod transfer;
