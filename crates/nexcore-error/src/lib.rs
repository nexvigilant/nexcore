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

/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($msg:literal) => {
        return core::result::Result::Err($crate::NexError::new($msg).into())
    };
    ($err:expr) => {
        return core::result::Result::Err($crate::NexError::from($err).into())
    };
    ($fmt:literal, $($arg:tt)*) => {
        return core::result::Result::Err($crate::NexError::new(format!($fmt, $($arg)*)).into())
    };
}

/// Return early with an error if a condition is not satisfied.
#[macro_export]
macro_rules! ensure {
    ($cond:expr $(,)?) => {
        if !($cond) {
            return core::result::Result::Err($crate::NexError::new(concat!("Condition failed: `", stringify!($cond), "`")).into());
        }
    };
    ($cond:expr, $msg:literal $(,)?) => {
        if !($cond) {
            return core::result::Result::Err($crate::NexError::new($msg).into());
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !($cond) {
            return core::result::Result::Err($crate::NexError::from($err).into());
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !($cond) {
            return core::result::Result::Err($crate::NexError::new(format!($fmt, $($arg)*)).into());
        }
    };
}

/// Commonly used items for glob import.
pub mod prelude {
    pub use crate::{Context, NexError, Result, nexerror, bail, ensure};
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


}
