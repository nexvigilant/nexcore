//! Error translation between `molt::Exception` and `nexcore_error::NexError`.
//!
//! Molt uses `Exception` for both errors and control flow (return, break, continue).
//! This module translates Molt errors into NexCore's error system while preserving
//! error codes and stack traces from the Tcl interpreter.

use molt::types::{Exception, ResultCode};
use nexcore_error::NexError;

/// Errors originating from the Molt Tcl interpreter.
#[derive(Debug)]
pub enum MoltError {
    /// A Tcl script error with message, error code, and error info.
    Script {
        /// Human-readable error message
        message: String,
        /// Tcl errorCode (e.g., "NONE", "ARITH DIVZERO")
        error_code: String,
        /// Tcl errorInfo (stack trace)
        error_info: String,
    },

    /// A type conversion error (e.g., "expected integer but got 'abc'").
    Type(String),

    /// A value conversion error between JSON and Molt values.
    Value(String),

    /// A sandbox policy violation (command not allowed).
    Sandbox(String),
}

impl core::fmt::Display for MoltError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Script { message, .. } => write!(f, "Molt script error: {message}"),
            Self::Type(msg) => write!(f, "Molt type error: {msg}"),
            Self::Value(msg) => write!(f, "Molt value conversion error: {msg}"),
            Self::Sandbox(msg) => write!(f, "Molt sandbox violation: {msg}"),
        }
    }
}

impl std::error::Error for MoltError {}

impl From<Exception> for MoltError {
    fn from(ex: Exception) -> Self {
        if ex.is_error() {
            Self::Script {
                message: ex.value().to_string(),
                error_code: ex.error_code().to_string(),
                error_info: ex.error_info().to_string(),
            }
        } else {
            // Non-error exceptions (return, break, continue) — treat as control flow error
            let code_label = match ex.code() {
                ResultCode::Return => "return",
                ResultCode::Break => "break",
                ResultCode::Continue => "continue",
                _ => "unknown",
            };
            Self::Script {
                message: format!(
                    "unexpected control flow: {code_label} (value: {})",
                    ex.value()
                ),
                error_code: String::new(),
                error_info: String::new(),
            }
        }
    }
}

// Note: From<MoltError> for NexError is provided by the blanket impl
// `impl<E: Error + Send + Sync + 'static> From<E> for NexError` in nexcore-error.

/// Convenience type alias for Molt operations.
pub type Result<T> = std::result::Result<T, MoltError>;

#[cfg(test)]
mod tests {
    use super::*;
    use molt::types::Exception;

    #[test]
    fn error_exception_converts() {
        let ex = Exception::molt_err(molt::types::Value::from("bad command"));
        let err: MoltError = ex.into();
        match &err {
            MoltError::Script {
                message,
                error_code,
                ..
            } => {
                assert_eq!(message, "bad command");
                assert!(!error_code.is_empty()); // NONE by default
            }
            _ => panic!("expected Script variant"),
        }
        // Display
        assert!(err.to_string().contains("bad command"));
    }

    #[test]
    fn non_error_exception_converts() {
        // ResultCode::Return with a value
        let ex = Exception::molt_return(molt::types::Value::from("42"));
        let err: MoltError = ex.into();
        match &err {
            MoltError::Script { message, .. } => {
                assert!(message.contains("return"));
            }
            _ => panic!("expected Script variant"),
        }
    }

    #[test]
    fn molt_error_to_nexerror() {
        let err = MoltError::Type("expected integer".into());
        let nex: NexError = err.into();
        assert!(nex.to_string().contains("type error"));
    }

    #[test]
    fn sandbox_error_display() {
        let err = MoltError::Sandbox("exit not allowed".into());
        assert!(err.to_string().contains("sandbox violation"));
    }
}
