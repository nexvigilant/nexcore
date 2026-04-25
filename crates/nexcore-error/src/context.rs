//! Context trait for error chaining.

use crate::{NexError, Result};
use core::fmt;

/// Extension trait to add context to errors.
pub trait Context<T> {
    /// Adds context to an error.
    ///
    /// # Errors
    /// Returns the original error with the new context attached.
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T>;

    /// Adds lazy context to an error.
    ///
    /// # Errors
    /// Returns the original error with the new context attached.
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

/// Blanket impl for any `std::error::Error` type (enables `?` with `.context()`)
#[cfg(feature = "std")]
impl<T, E> Context<T> for core::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T> {
        self.map_err(|e| NexError::from_err(e, ctx))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| NexError::from_err(e, f()))
    }
}

/// Option impl: `.context(msg)` converts `None` into a `NexError` with the given message.
impl<T> Context<T> for core::option::Option<T> {
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T> {
        match self {
            Some(t) => Ok(t),
            None => Err(NexError::msg(ctx)),
        }
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(NexError::msg(f())),
        }
    }
}

#[cfg(not(feature = "std"))]
impl<T> Context<T> for Result<T, NexError> {
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T> {
        self.map_err(|_| NexError::msg(ctx))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|_| NexError::msg(f()))
    }
}
