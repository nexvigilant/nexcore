//! Publishing error types.

use std::fmt;

/// All errors that can occur during the publishing pipeline.
#[derive(Debug)]
pub enum PublishingError {
    /// Failed to read or parse a DOCX file.
    DocxRead(String),
    /// Invalid or missing metadata.
    Metadata(String),
    /// Chapter extraction or ordering failure.
    Chapter(String),
    /// Cover image validation failure.
    Cover(String),
    /// EPUB generation failure.
    EpubWrite(String),
    /// Kindle/KDP compliance failure.
    KindleCompliance(String),
    /// EPUB validation failure.
    Validation(String),
    /// Pipeline orchestration error.
    Pipeline(String),
    /// I/O error wrapper.
    Io(std::io::Error),
    /// ZIP archive error.
    Zip(String),
    /// XML parsing error.
    Xml(String),
}

impl fmt::Display for PublishingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocxRead(msg) => write!(f, "DOCX read error: {msg}"),
            Self::Metadata(msg) => write!(f, "Metadata error: {msg}"),
            Self::Chapter(msg) => write!(f, "Chapter error: {msg}"),
            Self::Cover(msg) => write!(f, "Cover error: {msg}"),
            Self::EpubWrite(msg) => write!(f, "EPUB write error: {msg}"),
            Self::KindleCompliance(msg) => write!(f, "Kindle compliance: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::Pipeline(msg) => write!(f, "Pipeline error: {msg}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Zip(msg) => write!(f, "ZIP error: {msg}"),
            Self::Xml(msg) => write!(f, "XML error: {msg}"),
        }
    }
}

impl std::error::Error for PublishingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PublishingError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Convenience result type for publishing operations.
pub type Result<T> = std::result::Result<T, PublishingError>;
