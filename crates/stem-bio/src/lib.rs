//! # stem-bio: Biological System Primitives
//!
//! Implements `stem-core` SCIENCE traits for biological systems.
//!
//! ## Modules
//!
//! - `endocrine` - Hormone-based behavioral modulation
//! - `immune` (planned) - Pattern recognition and threat response
//! - `neural` (planned) - Signal propagation and learning
//!
//! ## Tier Classification
//!
//! | Type | Tier | Grounding |
//! |------|------|-----------|
//! | `HormoneLevel` | T2-P | Quantity (N) |
//! | `EndocrineState` | T2-C | State (ς) + Persistence (π) |
//! | `Stimulus` | T2-P | Mapping (μ): Event → State change |
//! | `BehavioralModifiers` | T2-C | Mapping (μ): State → Parameters |

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
pub mod endocrine;
pub mod grounding;

// Re-export core types
pub use endocrine::{BehaviorModulation, EndocrineSystem, HormoneSignal};
