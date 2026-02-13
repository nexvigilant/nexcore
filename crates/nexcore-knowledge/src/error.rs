//! Error types for nexcore-knowledge.

use core::fmt;

/// Knowledge module errors.
#[derive(Debug)]
pub enum KnowledgeError {
    /// Article not found.
    ArticleNotFound(String),
    /// Domain not found.
    DomainNotFound(String),
    /// Failed to read article.
    ReadError(String),
    /// Failed to parse article.
    ParseError(String),
    /// IO error.
    Io(std::io::Error),
}

impl fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArticleNotFound(s) => write!(f, "Article not found: {s}"),
            Self::DomainNotFound(s) => write!(f, "Domain not found: {s}"),
            Self::ReadError(s) => write!(f, "Failed to read article: {s}"),
            Self::ParseError(s) => write!(f, "Failed to parse article: {s}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for KnowledgeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KnowledgeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
