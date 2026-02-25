//! # NexVigilant Core — constants: T1/T2-P Bedrock Types
//!
//! Single source of truth for universal primitives used across the nexcore workspace.
//! Zero domain dependencies. All crates import from here.
//!
//! ## Types
//!
//! | Type | Tier | Codex |
//! |------|------|-------|
//! | [`Confidence`] | T2-P | IX (MEASURE) |
//! | [`Measured<T>`] | T2-C | IX (MEASURE) |
//! | [`Tier`] | T1 | II (CLASSIFY) |
//! | [`Correction<T>`] | T2-C | XI (CORRECT) |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    reason = "Core primitives are intentionally closed and versioned within this workspace"
)]

pub mod bathroom_lock;
pub mod confidence;
pub mod correction;
pub mod grounding;
pub mod interval;
pub mod measured;
pub mod tier;

pub use bathroom_lock::{BathroomLock, LockError, Occupancy, OccupiedGuard};
pub use confidence::Confidence;
pub use correction::Correction;
pub use interval::ConfidenceInterval;
pub use measured::Measured;
pub use tier::Tier;
