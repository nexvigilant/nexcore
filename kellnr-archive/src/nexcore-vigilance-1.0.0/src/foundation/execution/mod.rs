//! # Execution Engine Module
//!
//! DAG-based execution engine with checkpointing.

pub mod engine;

pub use engine::{ExecutionEngine, ExecutionResult, TaskStatus};
