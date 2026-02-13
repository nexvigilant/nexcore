//! Error types for the signal fence.
//!
//! Tier: T2-P (∂ Boundary + ∅ Void — error boundary with absence detection)

use std::fmt;

/// All errors produced by the signal fence.
#[derive(Debug)]
pub enum FenceError {
    /// Failed to read /proc filesystem entries.
    ProcRead(String),
    /// Failed to parse a /proc entry.
    ProcParse {
        file: String,
        line: usize,
        detail: String,
    },
    /// Process resolution failed for the given inode.
    ProcessResolution { inode: u64, detail: String },
    /// Rule validation error.
    InvalidRule { rule_id: String, reason: String },
    /// CIDR parse error.
    CidrParse(String),
    /// Enforcement backend error.
    Enforcement(String),
    /// IO error from filesystem operations.
    Io(std::io::Error),
}

impl fmt::Display for FenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcRead(path) => write!(f, "failed to read proc entry: {path}"),
            Self::ProcParse { file, line, detail } => {
                write!(f, "failed to parse {file}:{line}: {detail}")
            }
            Self::ProcessResolution { inode, detail } => {
                write!(f, "process resolution failed for inode {inode}: {detail}")
            }
            Self::InvalidRule { rule_id, reason } => {
                write!(f, "invalid rule '{rule_id}': {reason}")
            }
            Self::CidrParse(s) => write!(f, "CIDR parse error: {s}"),
            Self::Enforcement(detail) => write!(f, "enforcement error: {detail}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for FenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FenceError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Result alias for fence operations.
pub type FenceResult<T> = Result<T, FenceError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_display_proc_read() {
        let err = FenceError::ProcRead("/proc/net/tcp".to_string());
        assert!(err.to_string().contains("/proc/net/tcp"));
    }

    #[test]
    fn test_display_proc_parse() {
        let err = FenceError::ProcParse {
            file: "/proc/net/tcp".to_string(),
            line: 3,
            detail: "bad hex".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("tcp"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn test_display_process_resolution() {
        let err = FenceError::ProcessResolution {
            inode: 12345,
            detail: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("12345"));
    }

    #[test]
    fn test_display_invalid_rule() {
        let err = FenceError::InvalidRule {
            rule_id: "r1".to_string(),
            reason: "empty process match".to_string(),
        };
        assert!(err.to_string().contains("r1"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let fence_err = FenceError::from(io_err);
        assert!(matches!(fence_err, FenceError::Io(_)));
        assert!(fence_err.source().is_some());
    }

    #[test]
    fn test_non_io_has_no_source() {
        let err = FenceError::CidrParse("bad".to_string());
        assert!(err.source().is_none());
    }
}
