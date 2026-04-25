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

/// Conversion from `std::io::Error`.
#[cfg(feature = "std")]
impl From<std::io::Error> for NexError {
    fn from(err: std::io::Error) -> Self {
        Self {
            inner: Box::new(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `Box<dyn std::error::Error + Send + Sync>`.
#[cfg(feature = "std")]
impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for NexError {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self {
            inner: Box::new(err.to_string()),
            source: Some(err),
        }
    }
}

/// Conversion from `serde_json::Error`.
#[cfg(feature = "json")]
impl From<serde_json::Error> for NexError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            inner: Box::new(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `rusqlite::Error`.
#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for NexError {
    fn from(err: rusqlite::Error) -> Self {
        Self {
            inner: Box::new(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `reqwest::Error`.
#[cfg(feature = "http")]
impl From<reqwest::Error> for NexError {
    fn from(err: reqwest::Error) -> Self {
        Self {
            inner: Box::new(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `rmcp::service::ServerInitializeError`.
///
/// MCP server bootstrap errors map to a stringified context — the underlying
/// rmcp error type is held in `source` for chained debug output.
#[cfg(feature = "mcp")]
impl From<rmcp::service::ServerInitializeError> for NexError {
    fn from(err: rmcp::service::ServerInitializeError) -> Self {
        Self {
            inner: Box::new(format!("rmcp serve: {err}")),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `tokio::task::JoinError`.
///
/// `service.waiting().await?` and other tokio task joins routinely produce
/// these — short-circuit at the boundary instead of repeating .map_err glue.
#[cfg(feature = "tokio_runtime")]
impl From<tokio::task::JoinError> for NexError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self {
            inner: Box::new(format!("tokio join: {err}")),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `tokio::time::error::Elapsed`.
///
/// `tokio::time::timeout` returns this on deadline expiry. `nexcore-mcp-stdio`
/// hits this on read-response paths.
#[cfg(feature = "tokio_runtime")]
impl From<tokio::time::error::Elapsed> for NexError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self {
            inner: Box::new(format!("tokio timeout: {err}")),
            source: Some(Box::new(err)),
        }
    }
}

/// Conversion from `tracing_subscriber::filter::ParseError`.
///
/// EnvFilter directive parsing produces this; ubiquitous in MCP server
/// `main.rs` files via `.add_directive("name=info".parse()?)`.
#[cfg(feature = "tracing_filter")]
impl From<tracing_subscriber::filter::ParseError> for NexError {
    fn from(err: tracing_subscriber::filter::ParseError) -> Self {
        Self {
            inner: Box::new(format!("tracing filter parse: {err}")),
            source: Some(Box::new(err)),
        }
    }
}
