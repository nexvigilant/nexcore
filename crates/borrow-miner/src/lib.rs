//! Borrow Miner — PV signal detection educational game logic.
//!
//! Pure game mechanics: ore types, scoring, challenges, signal detection,
//! and achievements. No UI dependencies.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
pub const MAX_COMBO: u32 = 10;

pub mod achievements;
pub mod challenges;
pub mod grounding;
pub mod signals;
pub mod types;
