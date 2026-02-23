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

impl<T, E> Context<T> for core::result::Result<T, E>
where
    NexError: From<E>,
{
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T> {
        self.map_err(|e| NexError::from(e).context(ctx))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| NexError::from(e).context(f()))
    }
}

impl<T> Context<T> for core::option::Option<T> {
    fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Result<T> {
        self.ok_or_else(|| NexError::msg(ctx))
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.ok_or_else(|| NexError::msg(f()))
    }
}
