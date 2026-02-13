//! # Domain Types
//!
//! Type-safe primitives that encode domain invariants at the type level.
//!
//! ## Philosophy
//!
//! > "Make illegal states unrepresentable." — Yaron Minsky
//!
//! Instead of validating strings at runtime throughout the codebase, we validate
//! once at construction and guarantee correctness thereafter through the type system.
//!
//! ## Type Hierarchy
//!
//! ```text
//! Strings
//! ├── NonEmptyString      — Guaranteed non-empty
//! ├── BoundedString<N>    — Guaranteed length ≤ N
//! │   ├── ShortId         — BoundedString<64>
//! │   ├── CommandDescription — BoundedString<500>
//! │   └── LongText        — BoundedString<4096>
//! └── SessionId           — Non-empty session identifier
//!
//! Identifiers
//! ├── ToolName            — Valid tool name pattern
//! └── HookEventName       — One of 13 defined events
//!
//! Bounded Values
//! ├── TimeoutSeconds      — Range [1, 3600]
//! └── ExitCode            — 0, 1, or 2 with semantics
//!
//! Paths
//! ├── ValidPath           — Non-empty, valid UTF-8
//! └── ProjectPath         — No traversal beyond root
//! ```
//!
//! ## Usage Patterns
//!
//! ### Construction with Validation
//!
//! ```rust
//! use claude_hooks::types::{ToolName, TimeoutSeconds};
//! use claude_hooks::error::HookResult;
//!
//! fn configure_tool(name: &str, timeout: u16) -> HookResult<(ToolName, TimeoutSeconds)> {
//!     // Validation happens here, once
//!     let tool = ToolName::new(name)?;
//!     let timeout = TimeoutSeconds::new(timeout)?;
//!
//!     // From here on, values are guaranteed valid
//!     Ok((tool, timeout))
//! }
//! # fn main() -> HookResult<()> {
//! #     let _ = configure_tool("Bash", 60)?;
//! #     Ok(())
//! # }
//! ```
//!
//! ### Deserialization with Validation
//!
//! When deserializing from JSON, validation is automatic:
//!
//! ```rust
//! use claude_hooks::types::ToolName;
//!
//! #[derive(serde::Deserialize)]
//! struct Config {
//!     // Serde automatically validates during deserialization
//!     tool: ToolName,
//! }
//!
//! let json = r#"{"tool": "Bash"}"#;
//! let config: Config = serde_json::from_str(json).unwrap();
//! // config.tool is guaranteed valid
//! ```
//!
//! ## Design Rationale
//!
//! | Alternative | Problem | Our Solution |
//! |-------------|---------|--------------|
//! | `String` everywhere | No compile-time guarantees | Newtypes with validation |
//! | Runtime validation | Errors discovered late | Construction-time validation |
//! | Panicking constructors | Unpredictable failures | `Result<T, HookError>` |
//! | Parsing in multiple places | Inconsistent validation | Single validation point |
//!
//! ## Invariants
//!
//! Each type documents and enforces its invariants:
//!
//! - `NonEmptyString`: `len() > 0`
//! - `BoundedString<N>`: `len() <= N`
//! - `ToolName`: Matches standard or MCP tool pattern
//! - `HookEventName`: One of 13 defined variants
//! - `TimeoutSeconds`: `1 <= value <= 3600`
//! - `ProjectPath`: No `..` traversal beyond project root
//!
//! ## Audit Compliance
//!
//! These types support regulatory audit requirements:
//!
//! - **Traceability**: Type name documents intent
//! - **Validation**: Invariants documented and enforced
//! - **Immutability**: No mutation after construction
//! - **Testability**: Property tests prove correctness

mod bounded;
mod identifiers;
mod paths;
mod strings;

#[cfg(test)]
pub mod strategies;

pub use bounded::{ExitCode, TimeoutSeconds};
pub use identifiers::{HookEventName, SessionId, ToolName};
pub use paths::{ProjectPath, ValidPath};
pub use strings::{BoundedString, CommandDescription, LongText, NonEmptyString, ShortId};
