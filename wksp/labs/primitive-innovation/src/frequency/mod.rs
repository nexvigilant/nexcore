// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # P2: Frequency (nu) Expansion
//!
//! **Problem**: nu is underrepresented as dominant (3.4%, 12 types).
//! The codebase models frequency as a *property* but lacks types *dominated* by
//! periodic/rhythmic behavior.
//!
//! **Goal**: Create 3 T2-C types with Frequency dominant.
//!
//! ## New Types
//!
//! | Type | Tier | Composition | Purpose |
//! |------|------|-------------|---------|
//! | `AdaptivePoller` | T2-C | nu + kappa + partial + N | Dynamic polling rate controller |
//! | `RetryStrategy` | T2-C | nu + irrev + partial + N | Backoff/retry with frequency decay |
//! | `PeriodicMonitor` | T2-C | nu + exists + partial + causality | Heartbeat/liveness checking |

mod adaptive_poller;
mod periodic_monitor;
mod retry_strategy;

pub use adaptive_poller::AdaptivePoller;
pub use periodic_monitor::PeriodicMonitor;
pub use retry_strategy::RetryStrategy;
