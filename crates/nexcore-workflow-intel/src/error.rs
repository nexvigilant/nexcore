//! Error types for workflow intelligence.

use std::fmt;

/// Errors that can occur during workflow analysis.
#[derive(Debug)]
pub enum WorkflowError {
    /// Database access failure.
    Db(String),
    /// No data available for the requested analysis.
    NoData(String),
    /// Invalid parameter.
    InvalidParam(String),
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Db(msg) => write!(f, "database error: {msg}"),
            Self::NoData(msg) => write!(f, "no data: {msg}"),
            Self::InvalidParam(msg) => write!(f, "invalid parameter: {msg}"),
        }
    }
}

impl std::error::Error for WorkflowError {}

impl From<rusqlite::Error> for WorkflowError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e.to_string())
    }
}

/// Result alias for workflow operations.
pub type Result<T> = std::result::Result<T, WorkflowError>;
