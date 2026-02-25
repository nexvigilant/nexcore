#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod agentic_loop;
pub mod bridge;
pub mod config;
pub mod context;
pub mod decision;
pub mod errors;
pub mod events;
pub mod executors;
pub mod grounding;
pub mod llm;
pub mod mcp_bridge;
pub mod memory;
pub mod models;
pub mod orchestrator;
pub mod projects;
pub mod runtime;
pub mod sources;
pub mod vigilance;

#[cfg(test)]
mod tests;

pub use crate::context::ContextAssembler;
pub use crate::decision::{AuthorityConfig, DecisionEngine};
pub use crate::errors::{Result, VigilError};
pub use crate::events::EventBus;
pub use crate::mcp_bridge::McpBridge;
pub use crate::memory::MemoryLayer;
pub use crate::orchestrator::Friday;
pub use crate::projects::ProjectRegistry;

/// Alias for backward compatibility
pub type Vigil = Friday;
/// Backward-compatible re-export (deprecated in errors.rs)
#[allow(deprecated)]
pub use crate::errors::FridayError;
