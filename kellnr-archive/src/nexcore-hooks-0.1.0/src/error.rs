//! # Error Types
//!
//! Comprehensive error handling for the claude-hooks library.
//!
//! ## Design Philosophy
//!
//! All errors in this library are represented as variants of [`HookError`].
//! This provides:
//!
//! - **Exhaustive matching**: Handle all error cases explicitly
//! - **Context preservation**: Errors carry diagnostic information
//! - **Chain compatibility**: Works with `?` operator and error chains
//!
//! ## Error Categories
//!
//! | Category | Variants | Recovery |
//! |----------|----------|----------|
//! | Parsing | `ParseInput`, `Parse` | Fix input format |
//! | I/O | `Io`, `StdinRead` | Retry or report |
//! | Validation | `ValidationFailed`, `InvalidPattern` | Fix input data |
//! | Access | `IndexOutOfBounds`, `EmptyCollection`, `MissingField` | Check data |
//! | Decision | `Block` | Communicate to Claude |
//! | System | `Timeout`, `InvalidConfig` | Fix configuration |
//!
//! ## Exit Code Mapping
//!
//! For CLI tools, map errors to appropriate exit codes:
//!
//! | Error Type | Exit Code | Claude Behavior |
//! |------------|-----------|-----------------|
//! | Success | 0 | Continues normally |
//! | Non-blocking | 1 | Shows in verbose mode |
//! | Blocking | 2 | Stops and shows stderr |
//!
//! ```rust
//! use claude_hooks::error::HookError;
//! use claude_hooks::types::ExitCode;
//!
//! fn to_exit_code(err: &HookError) -> ExitCode {
//!     match err {
//!         HookError::Block(_) => ExitCode::BlockingError,
//!         HookError::ValidationFailed(_) => ExitCode::BlockingError,
//!         _ => ExitCode::NonBlockingError,
//!     }
//! }
//! ```
//!
//! ## Usage Patterns
//!
//! ### Propagation with `?`
//!
//! ```rust
//! use claude_hooks::error::{HookError, HookResult};
//!
//! fn process_input(json: &str) -> HookResult<String> {
//!     let value: serde_json::Value = serde_json::from_str(json)?;
//!
//!     let name = value.get("name")
//!         .ok_or(HookError::MissingField("name"))?;
//!
//!     Ok(name.to_string())
//! }
//! # fn main() -> HookResult<()> {
//! #     let _ = process_input(r#"{"name": "test"}"#)?;
//! #     Ok(())
//! # }
//! ```
//!
//! ### Specific Error Handling
//!
//! ```rust
//! use claude_hooks::error::HookError;
//!
//! fn handle_result(result: Result<(), HookError>) {
//!     match result {
//!         Ok(()) => println!("Success"),
//!         Err(HookError::ValidationFailed(msg)) => {
//!             eprintln!("Invalid input: {}", msg);
//!         }
//!         Err(HookError::Io(e)) => {
//!             eprintln!("I/O error: {}", e);
//!         }
//!         Err(e) => {
//!             eprintln!("Unexpected error: {}", e);
//!         }
//!     }
//! }
//! ```

use thiserror::Error;

/// Errors that can occur during hook execution.
///
/// # Design Principle
///
/// This enum is exhaustive: every failure path in the library maps to
/// exactly one variant. No `panic!()` calls exist; all failures return
/// `HookError`.
///
/// # Exhaustive Matching
///
/// Always match exhaustively without wildcards to catch new variants:
///
/// ```rust
/// use claude_hooks::error::HookError;
///
/// fn exit_code_for(err: &HookError) -> i32 {
///     match err {
///         HookError::ParseInput(_) => 1,
///         HookError::Parse(_) => 1,
///         HookError::StdinRead(_) => 1,
///         HookError::Io(_) => 1,
///         HookError::ValidationFailed(_) => 2,
///         HookError::UnknownEvent(_) => 1,
///         HookError::UnknownTool(_) => 1,
///         HookError::MissingField(_) => 2,
///         HookError::Block(_) => 2,
///         HookError::IndexOutOfBounds { .. } => 1,
///         HookError::EmptyCollection(_) => 1,
///         HookError::InvalidConfig(_) => 1,
///         HookError::InvalidPattern(_) => 1,
///         HookError::Timeout(_) => 1,
///         // No wildcard — compiler catches new variants
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum HookError {
    // =========================================================================
    // Parsing Errors
    // =========================================================================
    /// Failed to parse JSON input from stdin.
    #[error("failed to parse hook input: {0}")]
    ParseInput(#[from] serde_json::Error),

    /// Generic parse error with custom message.
    #[error("parse error: {0}")]
    Parse(String),

    // =========================================================================
    // I/O Errors
    // =========================================================================
    /// Failed to read from stdin.
    #[error("failed to read from stdin: {0}")]
    StdinRead(std::io::Error),

    /// I/O error for file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // =========================================================================
    // Domain Errors
    // =========================================================================
    /// Hook validation failed (use exit code 2).
    #[error("{0}")]
    ValidationFailed(String),

    /// Unknown hook event type.
    #[error("unknown hook event: {0}")]
    UnknownEvent(String),

    /// Unknown tool name.
    #[error("unknown tool: {0}")]
    UnknownTool(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    /// Hook explicitly blocks an action.
    #[error("blocked: {0}")]
    Block(String),

    // =========================================================================
    // Collection Access Errors (Phase 2: Zero-Panic)
    // =========================================================================
    /// Index out of bounds for collection access.
    #[error("index {index} out of bounds for length {len}")]
    IndexOutOfBounds {
        /// The index that was accessed.
        index: usize,
        /// The length of the collection.
        len: usize,
    },

    /// Attempted operation on empty collection.
    #[error("empty collection: {0}")]
    EmptyCollection(&'static str),

    // =========================================================================
    // Configuration Errors
    // =========================================================================
    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Regex or glob pattern compilation failed.
    #[error("invalid pattern: {0}")]
    InvalidPattern(String),

    /// Operation timed out.
    #[error("operation timed out after {0}ms")]
    Timeout(u64),
}

/// Result type alias for hook operations.
///
/// All fallible operations in this library return `HookResult<T>`.
///
/// # Examples
///
/// ```rust
/// use claude_hooks::error::{HookError, HookResult};
///
/// fn validate_name(name: &str) -> HookResult<()> {
///     if name.is_empty() {
///         return Err(HookError::ValidationFailed("name cannot be empty".into()));
///     }
///     Ok(())
/// }
/// ```
pub type HookResult<T> = Result<T, HookError>;
