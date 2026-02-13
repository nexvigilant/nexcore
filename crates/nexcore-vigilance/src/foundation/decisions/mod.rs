//! # Decision Engine Module
//!
//! Decision tree execution and skill chaining.
//!
//! ## Submodules
//!
//! - **engine** - Decision tree execution engine
//! - **chain** - Skill chaining and condition evaluation

pub mod chain;
pub mod engine;

pub use chain::{ChainExecutor, ConditionChain};
pub use engine::{DecisionEngine, DecisionNode, DecisionResult};
