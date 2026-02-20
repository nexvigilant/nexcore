//! # NexVigilant Core — Primitives
//!
//! Universal computational primitives for the NexVigilant platform,
//! extracted from the vigilance monolith.

pub mod chemistry;
pub mod dynamics;
pub mod grounding;
pub mod measurement;
pub mod quantum;
pub mod relay;
pub mod spatial_bridge;
pub mod transfer;

pub use measurement::{Confidence, Measured};
pub use relay::{Fidelity, RelayChain, RelayHop};
