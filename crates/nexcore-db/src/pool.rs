//! Connection management for the Brain SQLite database.
//!
//! Provides a lightweight connection wrapper that handles initialization,
//! WAL mode, and provides the default database path.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::Result;
use crate::schema;

/// Default database filename within the Brain directory.
const DB_FILENAME: &str = "brain.db";

/// Default Brain directory relative to home.
const BRAIN_DIR: &str = ".claude/brain";

/// Resolve the default database path: `~/.claude/brain/brain.db`
#[must_use]
pub fn default_db_path() -> PathBuf {
    let home = dirs_path();
    home.join(BRAIN_DIR).join(DB_FILENAME)
}

/// Get the home directory path.
fn dirs_path() -> PathBuf {
    // Use $HOME on Linux/macOS
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// A managed database connection with automatic schema initialization.
///
/// Thread-safe via internal `Mutex<Connection>`. For the Brain use case
/// (single CLI process, sequential MCP calls), this is sufficient.
/// For higher concurrency, swap to `r2d2-sqlite` or `deadpool-sqlite`.
#[derive(Debug, Clone)]
pub struct DbPool {
    inner: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl DbPool {
    /// Open (or create) a database at the given path and initialize schema.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or schema fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::error::DbError::Migration(format!("Cannot create dir: {e}"))
                })?;
            }
        }

        let conn = Connection::open(&path)?;
        schema::initialize(&conn)?;

        tracing::info!(path = %path.display(), "Brain database opened");

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            path,
        })
    }

    /// Open an in-memory database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if the schema cannot be initialized.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        schema::initialize(&conn)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            path: PathBuf::from(":memory:"),
        })
    }

    /// Open the default database at `~/.claude/brain/brain.db`.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened.
    pub fn open_default() -> Result<Self> {
        Self::open(default_db_path())
    }

    /// Execute a closure with exclusive access to the connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the lock is poisoned or the closure fails.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self
            .inner
            .lock()
            .map_err(|e| crate::error::DbError::Migration(format!("Lock poisoned: {e}")))?;
        f(&conn)
    }

    /// Get the database file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let pool = DbPool::open_in_memory();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_with_conn() {
        let pool = DbPool::open_in_memory().expect("open");
        let result = pool.with_conn(|conn| {
            let count: i64 =
                conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
            Ok(count)
        });
        assert_eq!(result.expect("query"), 0);
    }

    #[test]
    fn test_default_db_path() {
        let path = default_db_path();
        assert!(path.to_string_lossy().contains("brain.db"));
        assert!(path.to_string_lossy().contains(".claude/brain"));
    }
}
