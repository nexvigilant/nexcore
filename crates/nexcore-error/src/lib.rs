//! Zero-dependency error handling for `nexcore` ecosystem.
//!
//! Replaces `thiserror` and `anyhow` with zero external dependencies.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod context;
mod error;

pub use context::Context;
pub use error::NexError;

/// Re-export the derive macro so users can `use nexcore_error::Error;`
#[cfg(feature = "derive")]
pub use nexcore_error_derive::Error;

/// A convenient `Result` type alias using `NexError`.
pub type Result<T, E = NexError> = core::result::Result<T, E>;

/// Creates a new `NexError` from a format string.
#[macro_export]
macro_rules! nexerror {
    ($fmt:literal) => { $crate::NexError::new(format!($fmt)) };
    ($fmt:literal, $($arg:tt)*) => { $crate::NexError::new(format!($fmt, $($arg)*)) };
}

/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($msg:literal) => {
        return core::result::Result::Err($crate::NexError::new(format!($msg)).into())
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
    pub use crate::{Context, NexError, Result, bail, ensure, nexerror};

    #[cfg(feature = "derive")]
    pub use crate::Error;
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

    // --- Derive macro tests ---

    #[cfg(feature = "derive")]
    mod derive_tests {
        use super::*;

        #[derive(Debug, Error)]
        enum TestError {
            #[error("not found")]
            NotFound,

            #[error("parse error: {0}")]
            Parse(String),

            #[error("io failed: {0}")]
            Io(#[from] std::io::Error),

            #[error("named: {msg}")]
            Named { msg: String },

            #[error("debug fmt: {0:?}")]
            DebugFmt(Vec<String>),

            #[error(transparent)]
            Other(#[from] std::fmt::Error),
        }

        #[derive(Debug, Error)]
        #[error("struct error: {msg}")]
        struct StructError {
            msg: String,
        }

        #[test]
        fn test_unit_variant_display() {
            let err = TestError::NotFound;
            assert_eq!(err.to_string(), "not found");
        }

        #[test]
        fn test_unnamed_field_display() {
            let err = TestError::Parse("bad input".into());
            assert_eq!(err.to_string(), "parse error: bad input");
        }

        #[test]
        fn test_named_field_display() {
            let err = TestError::Named { msg: "oops".into() };
            assert_eq!(err.to_string(), "named: oops");
        }

        #[test]
        fn test_debug_format() {
            let err = TestError::DebugFmt(vec!["a".into(), "b".into()]);
            assert_eq!(err.to_string(), "debug fmt: [\"a\", \"b\"]");
        }

        #[test]
        fn test_from_io_error() {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
            let err: TestError = io_err.into();
            assert!(err.to_string().contains("io failed"));
        }

        #[test]
        fn test_source_from_io() {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
            let err: TestError = io_err.into();
            assert!(std::error::Error::source(&err).is_some());
        }

        #[test]
        fn test_transparent_display() {
            let inner = std::fmt::Error;
            let err: TestError = inner.into();
            assert_eq!(err.to_string(), std::fmt::Error.to_string());
        }

        #[test]
        fn test_transparent_source() {
            let inner = std::fmt::Error;
            let err: TestError = inner.into();
            assert!(std::error::Error::source(&err).is_some());
        }

        #[test]
        fn test_struct_error_display() {
            let err = StructError {
                msg: "broken".into(),
            };
            assert_eq!(err.to_string(), "struct error: broken");
        }

        #[test]
        fn test_unit_no_source() {
            let err = TestError::NotFound;
            assert!(std::error::Error::source(&err).is_none());
        }
    }
}
