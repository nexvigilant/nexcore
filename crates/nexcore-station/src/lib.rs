#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # NexVigilant Station
//!
//! PV tool resolution gate for NexCore. Resolves domain queries to
//! confidence-scored tool configs using Observatory-derived quality metrics.
//!
//! ## Architecture
//!
//! - [`types`]: Core request/response types and trust tiers
//! - [`confidence`]: Weighted confidence computation
//! - [`feed`]: Observatory data feed trait and stub implementation
//! - [`client`]: Station client — bridges registry configs with Observatory quality
//! - [`error`]: Station-specific error types
//! - [`config`]: WebMCP Hub config types (StationConfig, StationTool, PvVertical)
//! - [`registry`]: Typed config store mapping domains to StationConfigs
//! - [`builder`]: Fluent builder for constructing configs

pub mod builder;
pub mod client;
pub mod confidence;
pub mod config;
pub mod disclaimer;
pub mod error;
pub mod feed;
pub mod registry;
pub mod telemetry;
pub mod types;
pub mod verticals;

pub use builder::StationBuilder;
pub use client::StationClient;
pub use confidence::{DAILY_DECAY_RATE, apply_staleness_decay, compute_confidence, is_stale};
pub use config::{AccessTier, ExecutionType, PvVertical, StationConfig, StationTool};
pub use error::StationError;
pub use feed::{DomainCoverage, ObservatoryFeed, QualityMetrics, StubObservatoryFeed};
pub use registry::StationRegistry;
pub use telemetry::{StationTelemetryEvent, new_trace_id};
pub use types::{GapInfo, ResolutionRequest, ResolutionResponse, TrustTier};
