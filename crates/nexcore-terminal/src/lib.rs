//! NexVigilant Terminal — multi-tenant, AI-augmented terminal sessions.
//!
//! Three modes in one surface:
//! - **Shell**: Raw PTY commands in a sandboxed process
//! - **Regulatory**: MCP tool dispatch for pharmacovigilance queries
//! - **AI**: Natural language routed to Claude (or other backends) with tool_use
//!
//! ## Architecture
//!
//! Orchestration-layer crate consumed by `nexcore-api` for the WebSocket
//! terminal endpoint. Depends on `vr-core` for tenant isolation.
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary) + ς(State) + σ(Sequence) + μ(Mapping) + →(Causality)`

#![warn(missing_docs)]
pub mod ai;
pub mod artifacts;
pub mod audit;
pub mod chi_monitor;
pub mod config;
pub mod conversation;
pub mod formatter;
pub mod health;
pub mod keybindings;
pub mod layout;
pub mod metering;
pub mod microgram;
pub mod preferences;
pub mod protocol;
pub mod pty;
pub mod registry;
pub mod relay;
pub mod router;
pub mod sandbox;
pub mod session;
pub mod station;
