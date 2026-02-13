//! # Strategic Pipeline Innovation System (SPIS)
//!
//! Consolidated from `nexcore-spis` crate.
//!
//! ## Features
//!
//! - **Pipeline Models**: Data structures for pipeline management
//! - **CostTracker**: GCP cost tracking with STARK sync
//! - **StrategicAnalyzer**: AI-powered capability gap analysis
//!
//! ## Conservation Law Reference
//!
//! SPIS operations must respect:
//! - **CL-6 (Entity Integrity)**: Pipeline executions preserve audit trails
//! - **CL-10 (Causality)**: Cost tracking maintains temporal ordering

#![allow(missing_docs)] // Module being consolidated

mod cost_tracker;
mod pipeline;
mod strategic_analyzer;

pub use cost_tracker::CostTracker;
pub use pipeline::{Pipeline, PipelineComponent, PipelineExecution};
pub use strategic_analyzer::{CapabilityGap, StrategicAnalysis, StrategicAnalyzer};
