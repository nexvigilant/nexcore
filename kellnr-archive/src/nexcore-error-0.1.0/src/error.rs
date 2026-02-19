//! Core error type implementation.

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String};

use core::fmt;

/// A type-erased error that can hold any error type.
///
/// Equivalent to `anyhow::Error`.
pub struct NexError {
    inner: Box<dyn fmt::Display + Send + Sync + 'static>,
    #[cfg(feature = "std")]
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl NexError {
    /// Creates a new error from a displayable message.
    #[must_use]
    pub fn msg<M: fmt::Display + Send + Sync + 'static>(message: M) -> Self {
        Self {
            inner: Box::new(message),
            #[cfg(feature = "std")]
            source: None,
        }
    }

    /// Creates a new error from a string.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self::msg(message.into())
    }

    /// Wraps an existing error with additional context.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn context<C: fmt::Display + Send + Sync + 'static>(self, ctx: C) -> Self {
        Self {
            inner: Box::new(ctx),
            source: Some(Box::new(self)),
        }
    }

    /// Returns the wrapped error if it was created with context.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn source(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.source.as_deref()
    }

    /// Creates a new error from any `std::error::Error` with context.
    #[cfg(feature = "std")]
    #[must_use]
    pub fn from_err<E, C>(err: E, ctx: C) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
        C: fmt::Display + Send + Sync + 'static,
    {
        Self {
            inner: Box::new(ctx),
            source: Some(Box::new(err)),
        }
    }
}

impl fmt::Display for NexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Debug for NexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NexError({})", self.inner)?;
        #[cfg(feature = "std")]
        if let Some(ref src) = self.source {
            write!(f, "\n  caused by: {src}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for NexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl From<String> for NexError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for NexError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
