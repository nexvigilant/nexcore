//! Error types for nexcore-measure.
//!
//! Tier: T2-C (composed domain errors)

use thiserror::Error;

/// Errors that can occur during measurement operations.
#[derive(Debug, Error)]
pub enum MeasureError {
    /// Empty input where non-empty data is required.
    #[error("empty input: {context}")]
    EmptyInput { context: String },

    /// A value is outside its valid mathematical range.
    #[error("value out of range: {value} not in [{min}, {max}] for {context}")]
    OutOfRange {
        value: f64,
        min: f64,
        max: f64,
        context: String,
    },

    /// Failed to parse a Cargo.toml file.
    #[error("cargo parse error: {path}: {reason}")]
    CargoParse { path: String, reason: String },

    /// Filesystem I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML deserialization error.
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),

    /// WalkDir traversal error.
    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    /// Insufficient data points for statistical computation.
    #[error("insufficient data: need at least {need}, got {got} for {context}")]
    InsufficientData {
        need: usize,
        got: usize,
        context: String,
    },

    /// A crate was not found in the workspace.
    #[error("crate not found: {name}")]
    CrateNotFound { name: String },
}

/// Convenience type alias.
pub type MeasureResult<T> = Result<T, MeasureError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let e = MeasureError::EmptyInput {
            context: "counts".into(),
        };
        assert!(format!("{e}").contains("empty input"));

        let e = MeasureError::OutOfRange {
            value: 1.5,
            min: 0.0,
            max: 1.0,
            context: "probability".into(),
        };
        assert!(format!("{e}").contains("1.5"));

        let e = MeasureError::CrateNotFound { name: "foo".into() };
        assert!(format!("{e}").contains("foo"));
    }

    #[test]
    fn io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let me: MeasureError = io_err.into();
        assert!(format!("{me}").contains("gone"));
    }
}
