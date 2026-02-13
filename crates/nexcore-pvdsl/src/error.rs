//! # PVDSL Error Types
//!
//! Error types for lexer, parser, and runtime.

use thiserror::Error;

/// PVDSL Error
#[derive(Error, Debug)]
pub enum PvdslError {
    /// Parse error with location
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        /// Line number
        line: usize,
        /// Column number
        column: usize,
        /// Error message
        message: String,
    },

    /// Lexer error
    #[error("Lexer error: {0}")]
    LexerError(String),

    /// Runtime/execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Function not found
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    /// Invalid argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Type error
    #[error("Type error: expected {expected}, got {actual}")]
    TypeError {
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },

    /// Stack underflow
    #[error("Stack underflow")]
    StackUnderflow,

    /// Undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
}

/// PVDSL Result type
pub type PvdslResult<T> = Result<T, PvdslError>;
