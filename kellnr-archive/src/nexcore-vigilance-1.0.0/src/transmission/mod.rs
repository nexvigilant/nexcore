//! # nexcore Transmission
//!
//! Workflow orchestration types for multi-engine transmission systems.
//!
//! This crate provides pure domain types for workflow execution:
//! - Workflow execution options, results, and state
//! - Engine configuration and registry types
//! - Metrics types (counters, gauges, histograms)
//! - Structured error types
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::transmission::workflow::{
//!     WorkflowExecutionOptions,
//!     WorkflowState,
//!     WorkflowStatus,
//! };
//!
//! // Create execution options
//! let options = WorkflowExecutionOptions::default();
//! assert!(!options.request_id.is_empty());
//!
//! // Create workflow state
//! let mut state = WorkflowState::new(
//!     "exec-123".to_string(),
//!     "daily-workflow".to_string(),
//!     5,
//!     options.request_id.clone(),
//!     options.correlation_id.clone(),
//!     serde_json::json!({}),
//! );
//!
//! assert_eq!(state.status, WorkflowStatus::Running);
//! state.complete();
//! assert_eq!(state.status, WorkflowStatus::Completed);
//! ```

#![forbid(unsafe_code)]

pub mod engine;
pub mod error;
pub mod metrics;
pub mod workflow;

pub use engine::*;
pub use error::{ErrorDetails, ErrorResponse, TransmissionError, TransmissionResult};
pub use metrics::*;
pub use workflow::*;
