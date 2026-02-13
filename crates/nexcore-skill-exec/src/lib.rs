//! # NexVigilant Core — Skill Execution Engine
//!
//! Real execution of Claude Code skills via shell scripts, Rust binaries, or library calls.
//!
//! This crate replaces the simulated execution in `nexcore-vigilance::orchestrator::executor`
//! with actual skill invocation.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │          SkillExecutor Trait            │
//! ├─────────────────────────────────────────┤
//! │ async fn execute(&self, skill, params)  │
//! │ fn can_execute(&self, skill) -> bool    │
//! └─────────────────────────────────────────┘
//!              │
//!    ┌─────────┼─────────┐
//!    ▼         ▼         ▼
//! ┌──────┐ ┌──────┐ ┌──────┐
//! │Shell │ │Binary│ │ Lib  │
//! │Exec  │ │ Exec │ │ Exec │
//! └──────┘ └──────┘ └──────┘
//! ```
//!
//! ## Execution Types
//!
//! - **Shell**: Invokes `scripts/*.sh` with JSON input via stdin, captures JSON output
//! - **Binary**: Invokes compiled `scripts/bin/*` or `target/release/*` binaries
//! - **Library**: Dynamically loads Rust crates (future capability)
//!
//! ## Example
//!
//! ```ignore
//! use nexcore_skill_exec::{SkillExecutor, ShellExecutor, ExecutionRequest};
//! use serde_json::json;
//!
//! let executor = ShellExecutor::new();
//! let request = ExecutionRequest::new("skill-dev", json!({"path": "/some/skill"}));
//! let result = executor.execute(request).await?;
//!
//! println!("Status: {:?}", result.status);
//! println!("Output: {}", result.output);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod executor;
pub mod grounding;
mod models;
mod shell;
mod validator;

pub use error::{ExecutionError, Result};
pub use executor::{CompositeExecutor, SkillExecutor};
pub use models::{ExecutionMethod, ExecutionRequest, ExecutionResult, ExecutionStatus, SkillInfo};
pub use shell::ShellExecutor;
pub use validator::ParameterValidator;

/// Prelude for common imports.
pub mod prelude {
    pub use crate::{
        CompositeExecutor, ExecutionError, ExecutionRequest, ExecutionResult, ExecutionStatus,
        ParameterValidator, Result, ShellExecutor, SkillExecutor, SkillInfo,
    };
}
