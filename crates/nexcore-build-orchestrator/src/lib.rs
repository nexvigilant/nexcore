//! # NexVigilant Core — build-orchestrator
//!
//! Full-stack build orchestrator with CI/CD pipeline management, live dashboard, and CLI.
//!
//! ## Architecture
//!
//! Core library (always available):
//! - `pipeline/` — stage definitions, state machines, executor
//! - `workspace/` — crate discovery, change detection
//! - `history/` — JSON persistence for pipeline runs
//! - `metrics/` — timing, summaries, aggregates
//!
//! Web dashboard (behind `ssr` feature):
//! - `components/` — Leptos SSR components
//! - `routes/` — page routes
//! - `server/` — Axum REST + SSE endpoints
//!
//! ## Primitive Foundation (T3)
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | σ Sequence | Pipeline stages execute in order |
//! | μ Mapping | Build targets → configurations |
//! | ς State | RunStatus FSM: Pending → Running → Success/Failed |
//! | ρ Recursion | Dependency graph traversal |
//! | ∂ Boundary | Timeouts, resource limits |
//! | → Causality | Triggers cause pipeline execution |
//! | κ Comparison | Compare results, version checks |
//! | N Quantity | Timing, counts, metrics |
//! | λ Location | Source paths, artifact locations |
//! | π Persistence | Build history (JSON files) |
//! | Σ Sum | Result variants (Success \| Failure \| Cancelled) |
//! | ∃ Existence | Check if artifacts exist |

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

// Core modules (always available)
pub mod cli;
pub mod error;
pub mod grounding;
pub mod history;
pub mod metrics;
pub mod pipeline;
pub mod types;
pub mod workspace;

// SSR modules (feature-gated)
#[cfg(feature = "ssr")]
pub mod components;
#[cfg(feature = "ssr")]
pub mod routes;
#[cfg(feature = "ssr")]
pub mod server;

// Re-exports for convenience
pub use error::{BuildOrcError, BuildOrcResult};
pub use pipeline::definition::PipelineDefinition;
pub use pipeline::executor::{dry_run, execute_pipeline};
pub use pipeline::stage::{PipelineStage, StageConfig};
pub use pipeline::state::{PipelineRunState, RunStatus, StageRunState};
pub use types::{BuildDuration, LogChunk, PipelineId, StageId};
