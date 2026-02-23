//! Verification errors.

use nexcore_error::Error;

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("SKILL.md not found at {path}")]
    SkillMdNotFound { path: String },

    #[error("Failed to read {path}: {source}")]
    ReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid YAML frontmatter: {message}")]
    InvalidYaml { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },
}
