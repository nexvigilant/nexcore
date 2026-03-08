//! # nexcore-molt
//!
//! Embeddable Tcl scripting engine for NexCore — domain-agnostic Molt wrapper.
//!
//! Wraps [molt 0.3.1](https://docs.rs/molt) with nexcore conventions:
//! error translation, typed context bridge, serde-based value conversion,
//! and sandbox policy.
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_molt::{Engine, SandboxPolicy};
//!
//! let mut engine = Engine::safe();
//! let result = engine.execute("expr {2 + 3}").unwrap();
//! assert_eq!(result, "5");
//! ```
//!
//! ## Sandbox Policies
//!
//! - `SandboxPolicy::Full` — all 33 standard Tcl commands
//! - `SandboxPolicy::Safe` (default) — 30 safe commands, no `exit`/`source`
//! - `SandboxPolicy::Empty` — bare interpreter
//! - `SandboxPolicy::Allowlist(vec)` — only named commands
//!
//! ## Custom Commands
//!
//! Register Rust functions as Tcl commands:
//!
//! ```rust
//! use nexcore_molt::{Engine, CommandRegistry};
//! use molt::types::{ContextID, MoltResult, Value};
//! use molt::{molt_ok, Interp};
//!
//! fn cmd_greet(_interp: &mut Interp, _ctx: ContextID, args: &[Value]) -> MoltResult {
//!     molt::check_args(1, args, 2, 2, "name")?;
//!     molt_ok!(format!("Hello, {}!", args[1].as_str()))
//! }
//!
//! let mut engine = Engine::safe();
//! let mut reg = CommandRegistry::new();
//! reg.add("greet", cmd_greet);
//! engine.register(&reg);
//!
//! let result = engine.execute("greet World").unwrap();
//! assert_eq!(result, "Hello, World!");
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod bridge;
pub mod engine;
pub mod ensemble;
pub mod error;
mod grounding;
pub mod registry;
pub mod sandbox;
pub mod value;

// Primary re-exports
pub use engine::Engine;
pub use error::MoltError;
pub use registry::{CommandEntry, CommandRegistry};
pub use sandbox::{SAFE_COMMANDS, SandboxPolicy};

// Re-export key molt types so consumers don't need a direct molt dependency
pub use molt::Interp;
pub use molt::check_args;
pub use molt::types::{CommandFunc, ContextID, MoltResult, Value};
pub use molt::{molt_err, molt_ok};
