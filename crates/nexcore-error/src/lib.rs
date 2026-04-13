//! Zero-dependency error handling for `nexcore` ecosystem.
//!
//! Replaces `thiserror` and `anyhow` with zero external dependencies.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod context;
pub mod error;

pub use context::Context;
pub use error::NexError;

/// Re-export the derive macro so crates can write `#[derive(nexcore_error::Error)]`.
#[cfg(feature = "derive")]
pub use nexcore_error_derive::Error;

/// A convenient `Result` type alias using `NexError`.
pub type Result<T, E = NexError> = core::result::Result<T, E>;

/// Creates a new `NexError` from a format string.
#[macro_export]
macro_rules! nexerror {
    ($msg:literal) => { $crate::NexError::new($msg) };
    ($fmt:literal, $($arg:tt)*) => { $crate::NexError::new(format!($fmt, $($arg)*)) };
}

/// Early-return with an error if a condition is false (anyhow-compatible).
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal) => {
        if !$cond { return Err($crate::NexError::new($msg).into()); }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !$cond { return Err($crate::NexError::new(format!($fmt, $($arg)*)).into()); }
    };
}

/// Early-return with an error unconditionally (anyhow-compatible).
#[macro_export]
macro_rules! bail {
    ($msg:literal) => { return Err($crate::NexError::new($msg).into()) };
    ($fmt:literal, $($arg:tt)*) => { return Err($crate::NexError::new(format!($fmt, $($arg)*)).into()) };
}

/// Commonly used items for glob import.
pub mod prelude {
    pub use crate::{Context, NexError, Result, bail, ensure, nexerror};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nexerror_msg() {
        let err = nexerror!("test error");
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_nexerror_format() {
        let err = nexerror!("error code: {}", 42);
        assert_eq!(err.to_string(), "error code: 42");
    }

    #[test]
    fn test_from_string() {
        let err: NexError = "string error".into();
        assert_eq!(err.to_string(), "string error");
    }

    #[test]
    fn test_from_str() {
        let err: NexError = NexError::from("str error");
        assert_eq!(err.to_string(), "str error");
    }

    #[test]
    fn test_nexerror_debug() {
        let err = nexerror!("debug test");
        let debug = format!("{err:?}");
        assert!(debug.contains("NexError"));
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_error_context_chaining() {
        let inner = nexerror!("root cause");
        let outer = inner.context("while processing");
        assert_eq!(outer.to_string(), "while processing");

        // Check cause chain via Debug
        let debug = format!("{outer:?}");
        assert!(debug.contains("caused by:"));
        assert!(debug.contains("root cause"));
    }

    #[test]
    fn test_error_source_trait() {
        let inner = nexerror!("inner");
        let outer = inner.context("outer");

        // NexError::source() should return the inner error
        let source = outer.source();
        assert!(source.is_some());
        assert!(
            source
                .map_or(String::new(), |s| s.to_string())
                .contains("inner")
        );
    }

    #[test]
    fn test_from_err_with_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let nex_err = NexError::from_err(io_err, "opening config file");
        assert_eq!(nex_err.to_string(), "opening config file");

        let source = nex_err.source();
        assert!(source.is_some());
    }

    #[test]
    fn test_context_trait_on_result() {
        let result: core::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "no access",
        ));

        let contextualized = result.context("reading secrets");
        assert!(contextualized.is_err());
        let err = contextualized
            .err()
            .map_or_else(|| nexerror!("unreachable"), |e| e);
        assert_eq!(err.to_string(), "reading secrets");
    }

    #[test]
    fn test_with_context_lazy() {
        let result: core::result::Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "slow"));

        let contextualized = result.with_context(|| format!("timeout after {}s", 30));
        assert!(contextualized.is_err());
        let err = contextualized
            .err()
            .map_or_else(|| nexerror!("unreachable"), |e| e);
        assert_eq!(err.to_string(), "timeout after 30s");
    }

    #[test]
    fn test_msg_constructor() {
        let err = NexError::msg("from msg");
        assert_eq!(err.to_string(), "from msg");
    }

    #[test]
    fn test_no_source_when_plain() {
        let err = nexerror!("plain error");
        assert!(err.source().is_none());
    }
}
