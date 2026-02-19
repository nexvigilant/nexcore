// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima MCP Server
//!
//! Executes Prima functions as MCP tools. Fills the gap:
//! - compile(prima→schema) ✓ (prima-mcp)
//! - execute(schema→result) ✓ (this crate)
//! - codegen(prima→target) ✓ (prima-codegen)
//!
//! ## Tier: T2-C (σ + μ + → + π)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod codegen;
pub mod executor;
pub mod serialize;
pub mod server;

pub use codegen::{CodegenResult, codegen_tools, generate, list_primitives, list_targets};
pub use executor::Executor;
pub use serialize::{json_to_prima, prima_to_json};
pub use server::Server;
