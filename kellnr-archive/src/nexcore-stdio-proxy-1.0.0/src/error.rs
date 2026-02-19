//! Error types for the stdio proxy.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Void (∅) | Error as absence of valid state |
//! | T1: Mapping (μ) | Error → display string |

/// Tier: T2-C — All proxy error variants.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("child process error: {0}")]
    Child(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("watcher error: {0}")]
    Watcher(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("channel closed: {0}")]
    Channel(String),

    #[error("queue full: capacity {capacity}, pending {pending}")]
    QueueFull { capacity: usize, pending: usize },

    #[error("reload failed: {0}")]
    Reload(String),

    #[error("binary not found: {path}")]
    BinaryNotFound { path: String },

    #[error("binary not executable: {path}")]
    BinaryNotExecutable { path: String },

    #[error("shutdown requested")]
    Shutdown,
}

pub type Result<T> = std::result::Result<T, ProxyError>;
