// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # nexcore-signal-fence
//!
//! Process-level network signal container — default-deny fence grounded in signal theory.
//!
//! ## Core Thesis
//!
//! > "All traffic is noise until proven otherwise." — A2 (Noise Dominance) applied to networking.
//!
//! Every network connection is an "observation" in signal theory terms. The allowlist
//! defines `FixedBoundary` rules. Default-deny embodies A2. The fence engine runs a
//! continuous scan→evaluate→enforce→audit loop, classifying each connection as signal
//! (allowed) or noise (denied).
//!
//! ## Architecture
//!
//! ```text
//! /proc/net/tcp{,6}  ──► ConnectionScanner ──► ConnectionEvent
//!        │                                          │
//! /proc/<pid>/fd/    ──► ProcessResolver   ──┘      │
//!                                                    ▼
//!                                             ┌─────────────┐
//!                                             │ FenceEngine  │
//!                                             │  ∂ evaluate  │
//!                                             └──────┬──────┘
//!                                                    │
//!                                     ┌──────────────┼──────────────┐
//!                                     ▼              ▼              ▼
//!                               [Allow]         [Deny]         [Alert]
//!                             (pass-through)  (iptables/log)  (Guardian)
//! ```
//!
//! ## Signal Theory Mapping
//!
//! | Signal Theory | Network Fence |
//! |--------------|--------------|
//! | ObservationSpace | Active connections |
//! | Baseline (null) | Default-deny |
//! | FixedBoundary | Allowlist rules |
//! | DetectionOutcome | Allow/Deny/Alert |
//! | A2 (Noise Dominance) | Most traffic unauthorized |
//!
//! ## Dominant Primitive
//!
//! **∂ (Boundary)** — inherited from signal-theory. The entire crate is about
//! drawing boundaries between allowed and denied network traffic.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexcore_signal_fence::engine::FenceEngine;
//! use nexcore_signal_fence::rule::{FenceRule, ProcessMatch, NetworkMatch, FenceVerdict};
//! use nexcore_signal_fence::process::Direction;
//!
//! let mut engine = FenceEngine::default_deny();
//!
//! // Allow nginx on port 443
//! engine.policy.add_rule(FenceRule {
//!     id: "allow-nginx-https".to_string(),
//!     process_match: ProcessMatch::ByName("nginx".to_string()),
//!     network_match: NetworkMatch::ByPort(443),
//!     direction: Direction::Ingress,
//!     verdict: FenceVerdict::Allow,
//!     priority: 10,
//!     description: "Allow nginx HTTPS".to_string(),
//! });
//!
//! // Run one tick
//! let result = engine.tick();
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod audit;
pub mod connection;
pub mod enforcer;
pub mod engine;
pub mod error;
pub mod grounding;
pub mod policy;
pub mod process;
pub mod rule;

/// Prelude re-exports for common usage.
pub mod prelude {
    pub use crate::enforcer::{Enforcer, LogOnlyEnforcer, MockEnforcer};
    pub use crate::engine::{FenceEngine, FenceReport, FenceStats, FenceTickResult};
    pub use crate::error::{FenceError, FenceResult};
    pub use crate::policy::{FenceDecision, FenceMode, FencePolicy};
    pub use crate::process::{ConnectionEvent, Direction, ProcessInfo};
    pub use crate::rule::{FenceRule, FenceVerdict, NetworkMatch, ProcessMatch, RuleSet};
}
