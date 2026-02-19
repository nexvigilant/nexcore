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

mod context;
mod error;

pub use context::Context;
pub use error::NexError;

/// A convenient `Result` type alias using `NexError`.
pub type Result<T, E = NexError> = core::result::Result<T, E>;

/// Creates a new `NexError` from a format string.
#[macro_export]
macro_rules! nexerror {
    ($msg:literal) => { $crate::NexError::new($msg) };
    ($fmt:literal, $($arg:tt)*) => { $crate::NexError::new(format!($fmt, $($arg)*)) };
}

/// Commonly used items for glob import.
pub mod prelude {
    pub use crate::{Context, NexError, Result, nexerror};
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
}
