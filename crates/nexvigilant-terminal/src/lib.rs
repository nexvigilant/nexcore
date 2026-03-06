// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # NexVigilant Terminal — Desktop Agent Cloud Shell
//!
//! Tauri v2 desktop application wrapping three nexcore subsystems into
//! a unified agent observability terminal.
//!
//! ## Architecture
//!
//! ```text
//! ┌─ NexVigilant Terminal (Tauri v2) ─────────────────────────┐
//! │                                                            │
//! │  ┌─ Frontend (HTML/JS) ──────────────────────────────────┐│
//! │  │  Terminal (xterm.js) │ Agent Dashboard │ Command Palette│
//! │  └────────────────────┬───────────────────────────────────┘│
//! │                       │ Tauri IPC Commands                 │
//! │  ┌────────────────────┴───────────────────────────────────┐│
//! │  │  commands::terminal  → nexcore-terminal (sessions/PTY) ││
//! │  │  commands::cloud     → nexcloud (CloudSupervisor)      ││
//! │  │  commands::shell     → nexcore-shell (apps/login/AI)   ││
//! │  └────────────────────────────────────────────────────────┘│
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary: Tauri IPC gate) + ς(State: session lifecycle) +
//!  μ(Mapping: command routing) + σ(Sequence: boot→login→terminal) +
//!  ν(Frequency: agent event stream)`

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod commands;
