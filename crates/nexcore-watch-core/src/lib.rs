#![deny(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
//! NexCore Watch Core — Pure computation for patient safety vigilance on the wrist.
//!
//! ## Architecture
//!
//! This crate contains zero platform dependencies. All Android/NDK/UI
//! concerns live in `nexcore-watch-app`. This separation enables:
//! - Full test suite on host (no emulator needed)
//! - Pure computation: signal detection, Guardian state, P0-P5 alerts
//! - PVOS integration via `WatchPvos` facade
//!
//! ## Modules
//!
//! | Module | Dominant Primitive | Purpose |
//! |--------|--------------------|---------|
//! | `system` | × (Product) | **Unified runtime** — owns all subsystems |
//! | `signal` | κ (Comparison) | 5-metric disproportionality detection |
//! | `guardian` | ς (State) | Homeostasis loop state machine |
//! | `alerts` | ∂ (Boundary) | P0-P5 priority classification |
//! | `pvos_bridge` | μ (Mapping) | PVOS OS facade for watch |
//! | `sync` | σ (Sequence) | Background state sync |
//!
//! ## Entry Point
//!
//! ```rust,no_run
//! use nexcore_watch_core::{WatchSystem, SystemConfig};
//!
//! let system = WatchSystem::boot(SystemConfig::default());
//! assert!(system.is_running());
//! ```
//!
//! ## Tier: T3 (domain-specific, grounded to T1 via full Quindecet)

pub mod alerts;
pub mod guardian;
pub mod pvos_bridge;
pub mod signal;
pub mod sync;
pub mod system;

pub use alerts::{Alert, AlertLevel};
pub use guardian::{GuardianState, GuardianStatus, RiskLevel};
pub use pvos_bridge::WatchPvos;
pub use signal::SignalResult;
pub use sync::SyncManager;
pub use system::{SystemConfig, SystemMetrics, SystemState, TickResult, WatchSystem};
