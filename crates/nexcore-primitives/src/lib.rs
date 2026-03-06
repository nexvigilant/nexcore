//! # NexVigilant Core — Primitives
//!
//! Universal computational primitives for the NexVigilant platform,
//! extracted from the vigilance monolith.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_mut)]
#![warn(missing_docs)]
pub mod chemistry;
pub mod dynamics;
pub mod entropy;
pub mod grounding;
pub mod measurement;
pub mod quantum;
pub mod relay;
pub mod spatial_bridge;
pub mod transfer;

pub use measurement::{Confidence, Measured};
pub use relay::{Fidelity, RelayChain, RelayHop, RelayVerification};
pub use transfer::{CircuitBreaker, DecayFunction, FeedbackLoop, Homeostasis, TopologicalAddress};
