// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Spliceosome error types.
//!
//! ## Primitive Grounding: boundary(d) + existence(E)
//!
//! Errors represent boundary violations (d) in the splicing process
//! and existence failures (E) when expected structures are missing.

use std::fmt;

/// Errors that can occur during spliceosome operations.
#[derive(Debug)]
pub enum SpliceosomeError {
    /// Task specification was empty or unparseable
    EmptyTaskSpec,
    /// Template not found for the given task category
    TemplateNotFound(String),
    /// Configuration file could not be loaded
    ConfigLoad(String),
    /// IO error during file operations
    Io(std::io::Error),
    /// Serialization/deserialization error
    Serde(serde_json::Error),
}

impl fmt::Display for SpliceosomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTaskSpec => write!(f, "Task specification is empty"),
            Self::TemplateNotFound(cat) => write!(f, "No EJC template for category: {cat}"),
            Self::ConfigLoad(msg) => write!(f, "Config load error: {msg}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Serde(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl std::error::Error for SpliceosomeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Serde(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SpliceosomeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for SpliceosomeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

/// Result type for spliceosome operations.
pub type Result<T> = std::result::Result<T, SpliceosomeError>;
