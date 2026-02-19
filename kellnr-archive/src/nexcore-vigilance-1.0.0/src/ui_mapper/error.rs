//! Error types for CLI-UI mapper

use thiserror::Error;

/// Errors that can occur during CLI-UI mapping
#[derive(Error, Debug)]
pub enum MapperError {
    #[error("Orphan UI component: {component} does not map to any CLI command")]
    OrphanUi { component: String },

    #[error("Ghost field: {field} in UI does not map to any CLI arg/flag")]
    GhostField { field: String },

    #[error(
        "Type mismatch: UI component {component} has type {ui_type:?} but CLI arg has type {cli_type:?}"
    )]
    TypeMismatch {
        component: String,
        ui_type: String,
        cli_type: String,
    },

    #[error("Required field mismatch: {field} is required in CLI but not in UI")]
    RequiredMismatch { field: String },

    #[error("Output field not found: {field} displayed in UI but not in CLI output schema")]
    OutputFieldNotFound { field: String },

    #[error("CLI command not found: {path}")]
    CommandNotFound { path: String },

    #[error("Invalid route: {route}")]
    InvalidRoute { route: String },

    #[error("Parsing error: {message}")]
    ParseError { message: String },

    #[error("Validation failed: {message}")]
    ValidationError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MapperError>;
