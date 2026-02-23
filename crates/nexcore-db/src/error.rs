//! Error types for the database layer.

/// Database errors
#[derive(Debug, nexcore_error::Error)]
pub enum DbError {
    /// SQLite error
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Row not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Schema version mismatch
    #[error("Schema version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected schema version
        expected: u32,
        /// Found schema version
        found: u32,
    },
}

/// Result type for database operations.
pub type Result<T> = std::result::Result<T, DbError>;
