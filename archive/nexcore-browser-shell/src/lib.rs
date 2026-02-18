//! # `NexVigilant` Browser Shell
//!
//! Rust-native browser shell built with Tauri + Leptos.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │           LEPTOS FRONTEND (WASM)                        │
//! │  TabBar │ AddressBar │ NavControls │ GuardianAlerts     │
//! ├─────────────────────────────────────────────────────────┤
//! │               TAURI IPC COMMANDS                        │
//! │  browser_* │ guardian_* │ vigil_*                       │
//! ├─────────────────────────────────────────────────────────┤
//! │               NEXCORE CRATES                            │
//! │  nexcore-browser │ nexcore-vigil │ nexcore-vigilance    │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Tier Classification
//!
//! - **T3**: Domain-specific browser shell application
//! - Grounds to: T2-C composites (`BrowserState`, `GuardianState`)
//! - Grounds to: T1 primitives via Tauri IPC (JSON serialization)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod commands;
pub mod components;
pub mod grounding;
pub mod hooks;
pub mod models;
pub mod state;

pub use models::ShellError;
pub use state::BrowserShellState;
